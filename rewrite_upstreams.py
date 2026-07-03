import json

files_map = {
    "core": "config/upstream_core.txt",
    "desktop": "config/upstream_desktop.txt",
    "cli": "config/upstream_cli.txt",
    "media": "config/upstream_media.txt"
}

template_body = """
    runs-on: self-hosted
    strategy:
      fail-fast: false
      matrix:
        package: __PACKAGE_ARRAY__
    permissions:
      contents: read
      packages: write
    env:
      REGISTRY: ghcr.io
      FEDORA_VERSION: 43
    container:
      image: registry.fedoraproject.org/fedora:43
      options: --privileged
    steps:
      - uses: actions/checkout@v4
      
      - name: Calculate Hash
        id: hash
        run: echo "hash=${{ hashFiles('config/rpmmacros') }}" >> $GITHUB_OUTPUT
        
      - name: Cache RPMs
        id: cache
        uses: actions/cache@v4
        with:
          path: /github/home/rpmbuild/RPMS/
          key: rpm-rolling-${{ matrix.package }}-${{ steps.hash.outputs.hash }}-${{ github.run_id }}
          restore-keys: |
            rpm-rolling-${{ matrix.package }}-${{ steps.hash.outputs.hash }}-
            rpm-rolling-${{ matrix.package }}-
            
      - name: Check Idempotency (Upstream Version Match)
        id: check_idempotency
        run: |
          dnf install -y dnf5 skopeo jq
          IMAGE_LOWER=$(echo "${{ env.REGISTRY }}/${{ github.repository_owner }}/ermete-forge-rolling-${{ matrix.package }}:latest" | tr '[:upper:]' '[:lower:]')
          
          # Trova versione upstream su Fedora
          UPSTREAM_INFO=$(dnf5 info --repo=updates --repo=fedora ${{ matrix.package }})
          UPSTREAM_VER=$(echo "$UPSTREAM_INFO" | awk '/^Version/ {print $3}' | head -n 1)
          UPSTREAM_REL=$(echo "$UPSTREAM_INFO" | awk '/^Release/ {print $3}' | head -n 1)
          UPSTREAM_FULL="${UPSTREAM_VER}-${UPSTREAM_REL}"
          
          # Trova versione remota su GHCR
          REMOTE_FULL=$(skopeo inspect docker://$IMAGE_LOWER | jq -r '.Labels."org.opencontainers.image.version" // "none"' || echo "none")
          
          echo "Upstream: $UPSTREAM_FULL"
          echo "Remote: $REMOTE_FULL"
          
          if [ "$UPSTREAM_FULL" == "$REMOTE_FULL" ] && [ "$REMOTE_FULL" != "none" ]; then
            echo "Match esatto. Salto la compilazione."
            echo "skip=true" >> $GITHUB_OUTPUT
          else
            echo "Nuova versione rilevata. Procedo con la compilazione."
            echo "skip=false" >> $GITHUB_OUTPUT
            echo "version=$UPSTREAM_FULL" >> $GITHUB_OUTPUT
          fi
          
      - name: Fetch, Patch and Build ${{ matrix.package }}
        if: steps.check_idempotency.outputs.skip != 'true'
        run: |
          dnf install -y rpm-build dnf-plugins-core rpmdevtools
          cp config/rpmmacros ~/.rpmmacros
          rpmdev-setuptree
          cd ~/rpmbuild/SRPMS
          
          echo "--- Downloading Source RPM per ${{ matrix.package }} ---"
          dnf download --source ${{ matrix.package }}
          dnf builddep -y *.src.rpm          
          
          echo "--- Estrazione SRPM per applicare Ponytail Ultra ---"
          rpm -ivh *.src.rpm
          
          echo "--- Iniezione Dinamica Ponytail Ultra ---"
          for spec in ~/rpmbuild/SPECS/*.spec; do
            # Disattiva pacchetti di debug globalmente
            if ! grep -q "debug_package %nil" "$spec"; then
              awk '/^Name:/ { print "%global debug_package %nil"; print $0; next } 1' "$spec" > "$spec.tmp" && mv "$spec.tmp" "$spec"
            fi
          done
          
          echo "--- Ricompilazione estrema con macro Ermete e Ponytail ---"
          if ! rpmbuild -bb --nocheck ~/rpmbuild/SPECS/*.spec; then
            if ls ~/rpmbuild/SRPMS/*.buildreqs.nosrc.rpm >/dev/null 2>&1; then
              echo "--- Installazione Dipendenze Dinamiche (%generate_buildrequires) ---"
              dnf install -y ~/rpmbuild/SRPMS/*.buildreqs.nosrc.rpm
              echo "--- Secondo tentativo di Ricompilazione Estrema ---"
              rpmbuild -bb --nocheck ~/rpmbuild/SPECS/*.spec
            else
              exit 1
            fi
          fi
          
          mkdir -p /github/home/RPMS
          cp ~/rpmbuild/RPMS/*/*.rpm /github/home/RPMS/
          
      - name: Publish Micro-Container OCI
        if: steps.check_idempotency.outputs.skip != 'true'
        run: |
          dnf install -y buildah
          export STORAGE_DRIVER=vfs
          export BUILDAH_ISOLATION=chroot
          buildah login -u ${{ github.actor }} -p ${{ secrets.GITHUB_TOKEN }} ${{ env.REGISTRY }}
          ctr=$(buildah from scratch)
          buildah config --label org.opencontainers.image.version="${{ steps.check_idempotency.outputs.version }}" $ctr
          buildah copy $ctr /github/home/RPMS/*.rpm /
          IMAGE_LOWER=$(echo "${{ env.REGISTRY }}/${{ github.repository_owner }}/ermete-forge-rolling-${{ matrix.package }}:latest" | tr '[:upper:]' '[:lower:]')
          buildah commit $ctr $IMAGE_LOWER
          buildah push $IMAGE_LOWER
"""

for category, list_file in files_map.items():
    pkgs = []
    with open(list_file, 'r') as f:
        for line in f:
            if not line.strip().startswith('#') and line.strip():
                pkgs.append(line.strip())
    
    pkg_array = json.dumps(pkgs)
    
    workflow_content = f"""name: Ermete Upstream {category.capitalize()}

on:
  push:
    branches: [ main ]
    paths:
      - 'config/upstream_{category}.txt'
      - 'config/rpmmacros'
      - '.github/workflows/build-upstream-{category}.yml'
  workflow_dispatch:
  schedule:
    - cron: '0 4 * * *'

jobs:
  build:
""" + template_body.replace("__PACKAGE_ARRAY__", pkg_array)

    with open(f".github/workflows/build-upstream-{category}.yml", "w") as f:
        f.write(workflow_content)

print("Rewritten 4 workflow files!")
