import re

with open('.github/workflows/ermete-forge-orchestrator.yml', 'r') as f:
    content = f.read()

# Replace idempotency blocks
pattern_idemp = r"""\s+- name: Check Idempotency \(Upstream Version Match\).*?if \[ "\$UPSTREAM_FULL" == "\$REMOTE_FULL" \].*?fi"""
replacement_idemp = """
      - name: Check Idempotency (Content Hash)
        id: cache
        run: |
          python3 scripts/idempotency_checker.py --package ${{ matrix.package }} --registry ${{ env.REGISTRY }} --owner ${{ github.repository_owner }} > idemp.out
          source idemp.out
          if [ "$CACHE_HIT" = "true" ]; then
            echo "cache-hit=true" >> $GITHUB_OUTPUT
          else
            echo "cache-hit=false" >> $GITHUB_OUTPUT
          fi
          echo "content_hash=$CONTENT_HASH" >> $GITHUB_OUTPUT"""

content = re.sub(pattern_idemp, replacement_idemp, content, flags=re.DOTALL)

# Replace Publish blocks
pattern_pub = r"""\s+- name: Publish Micro-Container OCI
\s+if: steps\.check_idempotency\.outputs\.skip != 'true'
\s+run: \|
\s+dnf install -y buildah
\s+export STORAGE_DRIVER=vfs
\s+export BUILDAH_ISOLATION=chroot
\s+buildah login -u \$\{\{ github\.actor \}\} -p \$\{\{ secrets\.GITHUB_TOKEN \}\} \$\{\{ env\.REGISTRY \}\}
\s+ctr=\$\(buildah from scratch\)
\s+buildah config --label org\.opencontainers\.image\.version="\$\{\{ steps\.check_idempotency\.outputs\.version \}\}" \$ctr
\s+buildah copy \$ctr /github/home/RPMS/\*\.rpm /
\s+IMAGE_LOWER=\$\(echo "\$\{\{ env\.REGISTRY \}\}/\$\{\{ github\.repository_owner \}\}/ermete-forge-rolling-\$\{\{ matrix\.package \}\}:latest" \| tr '\[:upper:\]' '\[:lower:\]'\)
\s+buildah commit \$ctr \$IMAGE_LOWER
\s+buildah push \$IMAGE_LOWER"""

replacement_pub = """
      - name: Publish Micro-Container OCI
        if: steps.cache.outputs.cache-hit != 'true'
        run: |
          dnf install -y buildah
          export STORAGE_DRIVER=vfs
          export BUILDAH_ISOLATION=chroot
          buildah login -u ${{ github.actor }} -p ${{ secrets.GITHUB_TOKEN }} ${{ env.REGISTRY }}
          ctr=$(buildah from scratch)
          buildah copy $ctr /github/home/RPMS/*.rpm /
          IMAGE_LOWER=$(echo "${{ env.REGISTRY }}/${{ github.repository_owner }}/ermete-forge-rolling-${{ matrix.package }}" | tr '[:upper:]' '[:lower:]')
          buildah commit $ctr $IMAGE_LOWER:latest
          buildah tag $IMAGE_LOWER:latest $IMAGE_LOWER:${{ steps.cache.outputs.content_hash }}
          buildah push $IMAGE_LOWER:latest
          buildah push $IMAGE_LOWER:${{ steps.cache.outputs.content_hash }}"""

content = re.sub(pattern_pub, replacement_pub, content)

# Fix Fetch if:
content = content.replace("if: steps.check_idempotency.outputs.skip != 'true'", "if: steps.cache.outputs.cache-hit != 'true'")

with open('.github/workflows/ermete-forge-orchestrator.yml', 'w') as f:
    f.write(content)
