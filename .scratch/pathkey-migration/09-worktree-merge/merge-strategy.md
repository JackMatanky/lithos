# Merge Strategy: pathkey-09-relativepath

## Recommended Sequence
1. **Switch to main worktree**: Perform all operations from the repository root.
2. **Merge Branch**: Execute `git merge pathkey-09-relativepath`.
3. **Verify**: Run the full quality gate.

## Manual Interventions
- None expected.

## Validation Procedure
1. `mise run verify` (runs fmt, lint, tests, and ADR validation).
2. Check `git status` for any leftover artifacts.

## Rollback Procedure
If the merge or verification fails:
1. `git merge --abort` (if in the middle of a conflict).
2. `git reset --hard 63d6c7d9` (to return to the last known good state of `main`).
