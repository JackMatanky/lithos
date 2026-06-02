# Strategy: Base Processor Worktree Merge

## Recommended Sequence
1. **Merge Discovery to Main**: Merge `05-move-discovery-module-boundary` (`6b6bf0a2`) into `main`. This establishes the latest infrastructure baseline.
2. **Merge Feature to Main**: Merge `base-processor-init-and-fast-paths` (`2c02e9db`) into `main`. This adds the schema processor logic.
3. **Validate**: Run full verification suite on the merged `main`.

## Merge Details

### Step 1: Discovery Merge
- **Source**: `6b6bf0a2a2e23ffceaef1b7f56d86698f46fb29d`
- **Target**: `main`
- **Method**: `git merge --no-ff`
- **Expected Conflicts**: None (fast-forward-like topology relative to main tip).

### Step 2: Feature Merge
- **Source**: `2c02e9db5ff1853db4bd645633989c244367d385`
- **Target**: `main`
- **Method**: `git merge --no-ff`
- **Expected Conflicts**: None (orthogonal file sets).

## Manual Interventions
- None identified. The codebases are decoupled at the module level.

## Validation Procedure
1. `mise run verify` (runs fmt, lint, tests, and adr validation).
2. `gitnexus_detect_changes()` to verify combined impact.

## Rollback Procedure
1. `git reset --hard 7939a92ea17884dc717e91fdbfde110a9eeb1a7a` (return to current main tip).
2. `git checkout -b fix/merge-failure` to investigate if needed.
