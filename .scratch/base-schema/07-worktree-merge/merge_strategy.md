# Merge Strategy: `base-schema-07-integration-regression-suite` -> `main`

## Merge Sequence
1. **Pull Main into Worktree:** Merge `main` into the worktree branch `base-schema-07-integration-regression-suite` first. This allows us to resolve any conflicts in the isolated worktree environment and run the full test suite before touching the `main` branch.
2. **Merge Worktree into Main:** Once the worktree branch is verified and conflicts are resolved, perform a fast-forward or clean merge back into `main`.

## Conflict Resolution
- **`lithos-core/tests/`:** No overlaps expected. Take both sets of files.
- **`.scratch/base-schema/`:** No overlaps expected.
- **Root Planning Files (`task_plan.md`, `findings.md`, `progress.md`):**
  - **Action:** DISCARD worktree versions, PRESERVE `main` versions.
  - **Reasoning:** The `main` branch's versions represent the current active state of the broader project. The worktree versions are likely remnants from the session start and are superseded by the work in `main`.
  - **Exception:** If the worktree versions contain specific notes not found elsewhere, they should be manually reconciled into `.scratch/base-schema/07-integration-and-regression-suite.md` Implementation Notes before the merge. (Currently, the Implementation Notes are already up-to-date).

## Required Migrations / Manual Interventions
- None identified. The schema logic and repository traits are stable.

## Validation Procedure
1. **Merge `main` -> `worktree` branch.**
2. **Resolve root file conflicts** by selecting `main`'s versions.
3. **Run `mise run verify`** (includes fmt, lint, tests).
4. **Specifically verify** the new integration tests:
   - `cargo test --test base_processor`
   - `cargo test --test schema_storage`
5. **If all pass**, the state is ready for final merge into `main`.

## Rollback Plan
- If `mise run verify` fails and resolution is non-trivial:
  - `git merge --abort`
  - Re-analyze impact and specific failures.
- If errors are discovered after merging to `main`:
  - `git revert -m 1 <merge_commit_hash>`
