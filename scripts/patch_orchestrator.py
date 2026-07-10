import re

with open('.github/workflows/ermete-forge-orchestrator.yml', 'r') as f:
    content = f.read()

# 1. Insert orchestrator-brain job right after `jobs:`
brain_job = """  orchestrator-brain:
    name: 🧠 Orchestrator Brain
    runs-on: ubuntu-latest
    outputs:
      custom_packages: ${{ steps.brain.outputs.custom_packages }}
      cachyos_addons: ${{ steps.brain.outputs.cachyos_addons }}
      upstream_core: ${{ steps.brain.outputs.upstream_core }}
      upstream_desktop: ${{ steps.brain.outputs.upstream_desktop }}
      upstream_media: ${{ steps.brain.outputs.upstream_media }}
      upstream_cli: ${{ steps.brain.outputs.upstream_cli }}
    steps:
      - uses: actions/checkout@v4
      - name: Compute Dynamic Matrix
        id: brain
        run: ./scripts/dynamic-matrix.sh

"""
content = re.sub(r'^(jobs:\n)', r'\1' + brain_job, content, flags=re.MULTILINE)

# 2. Patch custom-packages
content = re.sub(
    r'  custom-packages:\n    needs: build-builder\n    name: 📦 Custom Packages\n    runs-on: ubuntu-latest\n    strategy:\n      fail-fast: false\n      matrix:\n        package: \[.*?\]',
    r'''  custom-packages:
    needs: [build-builder, orchestrator-brain]
    if: needs.orchestrator-brain.outputs.custom_packages != '[]' && needs.orchestrator-brain.outputs.custom_packages != ''
    name: 📦 Custom Packages
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        package: ${{ fromJson(needs.orchestrator-brain.outputs.custom_packages) }}''',
    content
)

# 3. Patch custom-packages Check Idempotency steps
content = re.sub(
    r'      - name: Calculate Hash\n        id: hash\n        run: echo "hash=\$\{\{ hashFiles.*?>> \$GITHUB_OUTPUT\n      - name: Check Idempotency\n        id: check\n        run: \|\n          dnf install -y skopeo\n          IMAGE=.*?\n          IMAGE_LOWER=.*?\n          if skopeo inspect.*?\n            echo "skip=true".*?\n          else\n            echo "skip=false".*?\n          fi',
    r'''      - name: Check Idempotency (Content Hash)
        id: check
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          bash scripts/check_idempotency.sh --package ${{ matrix.package }} --registry ${{ env.REGISTRY }} --owner ${{ github.repository_owner }} --image-name ermete-forge-${{ matrix.package }} > idemp.out
          source ./idemp.out
          echo "hash=$CONTENT_HASH" >> $GITHUB_OUTPUT
          if [ "$CACHE_HIT" = "true" ]; then echo "skip=true" >> $GITHUB_OUTPUT; else echo "skip=false" >> $GITHUB_OUTPUT; fi''',
    content, flags=re.DOTALL
)

# 4. Patch cachyos-addons
content = re.sub(
    r'  cachyos-addons:\n    needs: build-builder\n    name: ⚙️ Build CachyOS Addons\n    runs-on: ubuntu-latest\n    strategy:\n      fail-fast: false\n      matrix:\n        package: \[.*?\]',
    r'''  cachyos-addons:
    needs: [build-builder, orchestrator-brain]
    if: needs.orchestrator-brain.outputs.cachyos_addons != '[]' && needs.orchestrator-brain.outputs.cachyos_addons != ''
    name: ⚙️ Build CachyOS Addons
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        package: ${{ fromJson(needs.orchestrator-brain.outputs.cachyos_addons) }}''',
    content
)

# 5. Patch upstream-core
content = re.sub(
    r'  upstream-core:\n    needs: build-builder\n    name: 📦 Upstream Core\n\n    runs-on: ubuntu-latest\n    strategy:\n      fail-fast: false\n      matrix:\n        package: \[.*?\]',
    r'''  upstream-core:
    needs: [build-builder, orchestrator-brain]
    if: needs.orchestrator-brain.outputs.upstream_core != '[]' && needs.orchestrator-brain.outputs.upstream_core != ''
    name: 📦 Upstream Core

    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        package: ${{ fromJson(needs.orchestrator-brain.outputs.upstream_core) }}''',
    content
)

# 6. Patch upstream-desktop
content = re.sub(
    r'  upstream-desktop:\n    needs: build-builder\n    name: 📦 Upstream Desktop\n\n    runs-on: ubuntu-latest\n    strategy:\n      fail-fast: false\n      matrix:\n        package: \[.*?\]',
    r'''  upstream-desktop:
    needs: [build-builder, orchestrator-brain]
    if: needs.orchestrator-brain.outputs.upstream_desktop != '[]' && needs.orchestrator-brain.outputs.upstream_desktop != ''
    name: 📦 Upstream Desktop

    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        package: ${{ fromJson(needs.orchestrator-brain.outputs.upstream_desktop) }}''',
    content
)

# 7. Patch upstream-media
content = re.sub(
    r'  upstream-media:\n    needs: build-builder\n    name: 📦 Upstream Media\n\n    runs-on: ubuntu-latest\n    strategy:\n      fail-fast: false\n      matrix:\n        package: \[.*?\]',
    r'''  upstream-media:
    needs: [build-builder, orchestrator-brain]
    if: needs.orchestrator-brain.outputs.upstream_media != '[]' && needs.orchestrator-brain.outputs.upstream_media != ''
    name: 📦 Upstream Media

    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        package: ${{ fromJson(needs.orchestrator-brain.outputs.upstream_media) }}''',
    content
)

# 8. Patch upstream-cli
content = re.sub(
    r'  upstream-cli:\n    needs: build-builder\n    name: 📦 Upstream Cli\n\n    runs-on: ubuntu-latest\n    strategy:\n      fail-fast: false\n      matrix:\n        package: \[.*?\]',
    r'''  upstream-cli:
    needs: [build-builder, orchestrator-brain]
    if: needs.orchestrator-brain.outputs.upstream_cli != '[]' && needs.orchestrator-brain.outputs.upstream_cli != ''
    name: 📦 Upstream Cli

    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        package: ${{ fromJson(needs.orchestrator-brain.outputs.upstream_cli) }}''',
    content
)


with open('.github/workflows/ermete-forge-orchestrator.yml', 'w') as f:
    f.write(content)
