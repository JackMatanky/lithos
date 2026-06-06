# Merge Strategy: BaseSchemaProcessor Redesign

## Recommended Sequence
1.  **Preparation:**
    *   Delete stale root planning files (`findings.md`, `progress.md`, `task_plan.md`) from the feature branch.
    *   Commit all changes in `feat/base-schema/05-stale-refs`.
2.  **Merge:**
    *   Switch to `main`.
    *   Merge `feat/base-schema/05-stale-refs` into `main`.
    *   Resolve conflict in `AGENTS.md` by accepting `main` version (GitNexus stats will be updated later).
3.  **Validation:**
    *   Run `mise run verify` to ensure all tests pass and clippy is happy.
4.  **Completion:**
    *   Push `main`.
    *   Delete worktree and feature branch.

## Manual Interventions
- Resolution of `AGENTS.md` conflict.

## Validation Plan
- [ ] `mise run fmt`
- [ ] `mise run lint`
- [ ] `mise run test`
- [ ] `npx gitnexus analyze` (to refresh stats)

## Rollback Procedure
- If merge fails: `git merge --abort`
- If validation fails: `git reset --hard HEAD~1` (on `main`)
