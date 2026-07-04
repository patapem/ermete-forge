import re

with open('.github/workflows/ermete-forge-orchestrator.yml', 'r') as f:
    lines = f.readlines()

for i in range(len(lines)):
    # Fix the actions/cache steps that were broken by my sed
    if "if: steps.cache.outputs.cache_hit != 'true' && steps.check.outputs.skip != 'true'" in lines[i]:
        lines[i] = lines[i].replace("cache_hit", "cache-hit")
    if "if: steps.cache.outputs.cache_hit != 'true'" in lines[i]:
        # Only change it back to cache-hit if it's right after an actions/cache step.
        # But wait, in upstream-* it's checking my custom output which is now cache_hit.
        pass

# Wait, let's do a smarter approach using python.
