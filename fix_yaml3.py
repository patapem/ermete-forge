import re
import sys

with open('.github/workflows/ermete-forge-orchestrator.yml', 'r') as f:
    content = f.read()

# Fix 1: Restore cache-hit for lines 68 and 417 which use actions/cache@v4
# Line 68: if: steps.cache.outputs.cache_hit != 'true' && steps.check.outputs.skip != 'true'
content = content.replace(
    "if: steps.cache.outputs.cache_hit != 'true' && steps.check.outputs.skip != 'true'",
    "if: steps.cache.outputs['cache-hit'] != 'true' && steps.check.outputs.skip != 'true'"
)

# Line 417: if: steps.cache.outputs.cache_hit != 'true' (in build-nvidia)
# Wait, I need to make sure I don't replace the one in upstream-* which I want to keep as cache_hit.
# The one in build-nvidia is preceded by actions/cache. Let's do a regex that finds the one after actions/cache.
# Actually, I can just replace `steps.cache.outputs.cache_hit != 'true'` globally with `steps.check_idempotency.outputs.cache_hit != 'true'` for the upstream jobs.

content = content.replace(
    """      - name: Check Idempotency (Content Hash)
        id: cache""",
    """      - name: Check Idempotency (Content Hash)
        id: check_idempotency"""
)

# Now, everywhere in upstream-* it was doing `if: steps.cache.outputs.cache_hit != 'true'`
# We want it to be `steps.check_idempotency.outputs.cache_hit != 'true'`
content = content.replace(
    "if: steps.cache.outputs.cache_hit != 'true'\n        run: |\n          sed -i",
    "if: steps.check_idempotency.outputs.cache_hit != 'true'\n        run: |\n          sed -i"
)

content = content.replace(
    "if: steps.cache.outputs.cache_hit != 'true'\n        run: |\n          dnf install -y buildah",
    "if: steps.check_idempotency.outputs.cache_hit != 'true'\n        run: |\n          dnf install -y buildah"
)

content = content.replace(
    "${{ steps.cache.outputs.content_hash }}",
    "${{ steps.check_idempotency.outputs.content_hash }}"
)

# And now restore the build-nvidia one (and any other actions/cache ones)
# Since we renamed the upstream ones to check_idempotency, the remaining `steps.cache.outputs.cache_hit` MUST be actions/cache.
content = content.replace(
    "steps.cache.outputs.cache_hit",
    "steps.cache.outputs['cache-hit']"
)

with open('.github/workflows/ermete-forge-orchestrator.yml', 'w') as f:
    f.write(content)
