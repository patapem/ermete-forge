import sys
import re

def process_workflow(file_path):
    with open(file_path, 'r') as f:
        content = f.read()

    # The goal is to completely rewrite the orchestrator.
    # It's safer to extract the Upstream matrices and Kernel/NVIDIA, and drop the rest, then reconstruct.
    
    # Extract jobs preamble
    preamble = content.split('jobs:\n')[0] + 'jobs:\n'
    
    # Extract upstream jobs
    upstream_pattern = re.compile(r'(  upstream-[a-z]+:\n(?:.+(?:\n|$))*)')
    # wait, this pattern is too greedy. Let's extract them manually by splitting on "  upstream-"
    # Actually, we can just copy the `upstream-*` from the original file, and `build-kernel`, `build-nvidia`.
    
    jobs = {}
    current_job_name = None
    current_job_content = []
    
    for line in content.split('\n'):
        if line.startswith('jobs:'):
            continue
        
        match = re.match(r'^  ([a-zA-Z0-9_-]+):', line)
        if match:
            if current_job_name:
                jobs[current_job_name] = '\n'.join(current_job_content)
            current_job_name = match.group(1)
            current_job_content = [line]
        elif current_job_name:
            current_job_content.append(line)
            
    if current_job_name:
        jobs[current_job_name] = '\n'.join(current_job_content)
        
    new_jobs = []
    
    # 1. Custom Packages Matrix
    custom_matrix = """  custom-packages:
    name: 📦 Custom Packages
    runs-on: self-hosted
    strategy:
      fail-fast: false
      matrix:
        package: [starship, bat, selinux, ananicy, base-config, ags-config, niri-session, ide-bootstrap, system-services, nix-support, system-config, system-tweaks, matugen, bibata]
    permissions:
      contents: read
      packages: write
    container:
      image: registry.fedoraproject.org/fedora:43
      options: --privileged
    steps:
      - uses: actions/checkout@v4
      - name: Calculate Hash
        id: hash
        run: echo "hash=${{ hashFiles(format('specs/ermete-{0}/**', matrix.package), 'config/rpmmacros') }}" >> $GITHUB_OUTPUT
      - name: Check Idempotency
        id: check
        run: |
          dnf install -y skopeo
          IMAGE="docker://${{ env.REGISTRY }}/${{ github.repository_owner }}/ermete-forge-${{ matrix.package }}:${{ steps.hash.outputs.hash }}"
          IMAGE_LOWER=$(echo "$IMAGE" | tr '[:upper:]' '[:lower:]')
          if skopeo inspect "$IMAGE_LOWER" >/dev/null 2>&1; then
            echo "skip=true" >> $GITHUB_OUTPUT
          else
            echo "skip=false" >> $GITHUB_OUTPUT
          fi
      - name: Cache RPMs
        id: cache
        if: steps.check.outputs.skip != 'true'
        uses: actions/cache@v4
        with:
          path: /github/home/rpmbuild/RPMS/
          key: rpm-${{ matrix.package }}-${{ steps.hash.outputs.hash }}
      - name: Install Dependencies and Build
        if: steps.cache.outputs.cache-hit != 'true' && steps.check.outputs.skip != 'true'
        run: |
          sed -i "/tsflags=nodocs/d" /etc/dnf/dnf.conf
          dnf install -y rpm-build rpmdevtools gcc gcc-c++ cargo rust cmake mold tar xz curl pkgconf-pkg-config zlib-devel openssl-devel checkpolicy policycoreutils spdlog-devel systemd-devel nlohmann-json-devel fmt-devel
          cp config/rpmmacros ~/.rpmmacros
          rpmdev-setuptree
          
          if [ -d "specs/ermete-${{ matrix.package }}/SOURCES" ]; then 
            cp -a specs/ermete-${{ matrix.package }}/SOURCES/* ~/rpmbuild/SOURCES/ || true
          fi
          
          if grep -q "Source0:.*http" specs/ermete-${{ matrix.package }}/*.spec 2>/dev/null; then
             spectool -g -R specs/ermete-${{ matrix.package }}/*.spec
          fi
          
          dnf builddep -y specs/ermete-${{ matrix.package }}/*.spec || true
          rpmbuild -bb --nocheck specs/ermete-${{ matrix.package }}/*.spec
          
          mkdir -p /github/home/RPMS
          cp ~/rpmbuild/RPMS/*/*.rpm /github/home/RPMS/
      - name: Publish Micro-Container OCI
        if: steps.check.outputs.skip != 'true'
        run: |
          dnf install -y buildah
          export STORAGE_DRIVER=vfs
          export BUILDAH_ISOLATION=chroot
          buildah login -u ${{ github.actor }} -p ${{ secrets.GITHUB_TOKEN }} ${{ env.REGISTRY }}
          ctr=$(buildah from scratch)
          buildah copy $ctr /github/home/RPMS/*.rpm /
          IMAGE_LOWER=$(echo "${{ env.REGISTRY }}/${{ github.repository_owner }}/ermete-forge-${{ matrix.package }}" | tr '[:upper:]' '[:lower:]')
          buildah commit $ctr $IMAGE_LOWER:latest
          buildah tag $IMAGE_LOWER:latest $IMAGE_LOWER:${{ steps.hash.outputs.hash }}
          buildah push $IMAGE_LOWER:latest
          buildah push $IMAGE_LOWER:${{ steps.hash.outputs.hash }}"""
    new_jobs.append(custom_matrix)
    
    # 2. AGS Ecosystem
    ags_ecosystem = """
  ags-ecosystem:
    name: 🖥️ Build AGS Ecosystem
    runs-on: self-hosted
    permissions:
      contents: read
      packages: write
    container:
      image: registry.fedoraproject.org/fedora:43
      options: --privileged
    steps:
      - uses: actions/checkout@v4
      - name: Build and Publish AGS Components
        run: |
          sed -i "/tsflags=nodocs/d" /etc/dnf/dnf.conf
          dnf install -y rpm-build rpmdevtools meson ninja-build gcc gcc-c++ vala pkgconf-pkg-config buildah skopeo
          cp config/rpmmacros ~/.rpmmacros
          rpmdev-setuptree
          export STORAGE_DRIVER=vfs
          export BUILDAH_ISOLATION=chroot
          buildah login -u ${{ github.actor }} -p ${{ secrets.GITHUB_TOKEN }} ${{ env.REGISTRY }}
          
          for pkg in appmenu-glib-translator astal-io astal astal-libs astal-gjs astal-gtk4 astal-lua aylurs-gtk-shell aylurs-gtk-shell2 hyprpanel; do
            echo "========================================"
            echo "Building $pkg..."
            echo "========================================"
            
            # Compute Hash
            hash=$(sha256sum specs/ermete-astal/$pkg/*.spec config/rpmmacros | sha256sum | awk '{print $1}')
            IMAGE="docker://${{ env.REGISTRY }}/${{ github.repository_owner }}/ermete-forge-$pkg:$hash"
            IMAGE_LOWER=$(echo "$IMAGE" | tr '[:upper:]' '[:lower:]')
            
            if skopeo inspect "$IMAGE_LOWER" >/dev/null 2>&1; then
              echo "Immagine trovata per $pkg, skip."
            else
              if grep -q "Source0:.*http" specs/ermete-astal/$pkg/*.spec 2>/dev/null; then
                 spectool -g -R specs/ermete-astal/$pkg/*.spec
              fi
              dnf builddep -y specs/ermete-astal/$pkg/*.spec || true
              rpmbuild -bb --nocheck specs/ermete-astal/$pkg/*.spec
              
              # Installa nel sistema locale per i pacchetti successivi!
              dnf install -y ~/rpmbuild/RPMS/*/*.rpm
              
              # Publish
              ctr=$(buildah from scratch)
              buildah copy $ctr ~/rpmbuild/RPMS/*/*.rpm /
              IMAGE_LOWER_LATEST=$(echo "${{ env.REGISTRY }}/${{ github.repository_owner }}/ermete-forge-$pkg" | tr '[:upper:]' '[:lower:]')
              buildah commit $ctr $IMAGE_LOWER_LATEST:latest
              buildah tag $IMAGE_LOWER_LATEST:latest $IMAGE_LOWER_LATEST:$hash
              buildah push $IMAGE_LOWER_LATEST:latest
              buildah push $IMAGE_LOWER_LATEST:$hash
              
              # Pulizia RPMS per il prossimo
              rm -rf ~/rpmbuild/RPMS/*
            fi
          done"""
    new_jobs.append(ags_ecosystem)
    
    # 3. CachyOS Addons Matrix
    cachyos_matrix = """
  cachyos-addons:
    name: ⚙️ Build CachyOS Addons
    runs-on: self-hosted
    strategy:
      fail-fast: false
      matrix:
        package: [bore-sysctl, scx-scheds, scx-tools]
    permissions:
      contents: read
      packages: write
    container:
      image: registry.fedoraproject.org/fedora:43
      options: --privileged
    steps:
      - name: Fetch and Package
        run: |
          dnf install -y dnf5 curl buildah
          curl -Lo /etc/yum.repos.d/bieszczaders.repo "https://copr.fedorainfracloud.org/coprs/bieszczaders/kernel-cachyos-addons/repo/fedora-43/bieszczaders-kernel-cachyos-addons-fedora-43.repo"
          mkdir -p /github/home/RPMS
          dnf5 download -y --destdir=/github/home/RPMS ${{ matrix.package }}
          
          export STORAGE_DRIVER=vfs BUILDAH_ISOLATION=chroot
          buildah login -u ${{ github.actor }} -p ${{ secrets.GITHUB_TOKEN }} ${{ env.REGISTRY }}
          ctr=$(buildah from scratch)
          buildah copy $ctr /github/home/RPMS/*.rpm /
          IMAGE=$(echo "${{ env.REGISTRY }}/${{ github.repository_owner }}/ermete-forge-${{ matrix.package }}:latest" | tr '[:upper:]' '[:lower:]')
          buildah commit $ctr $IMAGE && buildah push $IMAGE"""
    new_jobs.append(cachyos_matrix)

    # 4. Include existing heavy jobs and upstreams
    for j in ['build-kernel', 'build-nvidia', 'upstream-core', 'upstream-desktop', 'upstream-media', 'upstream-cli']:
        if j in jobs:
            new_jobs.append("\n" + jobs[j])
            
    final_content = preamble + '\n'.join(new_jobs) + '\n'
    
    with open(file_path, 'w') as f:
        f.write(final_content)

if __name__ == '__main__':
    process_workflow('.github/workflows/ermete-forge-orchestrator.yml')
