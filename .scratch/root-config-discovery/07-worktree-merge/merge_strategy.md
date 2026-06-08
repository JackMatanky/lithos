# Merge Strategy: Feature -> Main

## Overview
Merge the Phase 2 Environment Config Discovery changes into the `main` branch.

## Recommended Merge Sequence
1. **Sync `main`**: Ensure local `main` is up to date with remote.
2. **Merge `main` into `feature`**: Perform a merge in the worktree to resolve the trivial conflict in `lithos-core/src/db/core.rs`.
3. **Validate `feature`**: Run all tests in the worktree after merge.
4. **Fast-forward `main`**: Merge the feature branch into `main`.
5. **Cleanup**: Remove the worktree.

## Manual Interventions
- **Conflict Resolution**: `lithos-core/src/db/core.rs`.
  - Prefer the `feature` branch's `TODO` comment as it adds specific issue tracking (#09).
  - Ensure the `expect` reason reflects the latest status.

## Validation Procedures
- [ ] `mise run fmt`
- [ ] `mise run lint`
- [ ] `mise run test:unit`
- [ ] Verify discovery tests specifically (99 tests).

## Rollback Procedures
- If merge fails or tests break: `git merge --abort`.
- If merged state is broken: `git reset --hard HEAD~1` on `main`.

## Verification Evidence
- Full test suite passed on feature branch (1573 tests total, 99 discovery).
- Clippy and Fmt clean.
