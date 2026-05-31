# Merge Strategy - Worktree Merge: refactor-propertybank-metadata

## Overview
This document defines the procedure for merging the `refactor-propertybank-metadata` feature branch into `main`.

## Merge Sequence
1. **Rebase**: Rebase `refactor-propertybank-metadata` onto `main` to ensure a linear history and resolve any conflicts early.
2. **Validate**: Run the project's verification suite (`mise run verify`) on the rebased branch.
3. **Merge**: Execute a non-fast-forward merge (`git merge --no-ff`) into `main` to preserve feature branch visibility.
4. **Final Check**: Run verification one last time on the merged `main`.

## Manual Interventions
- `AGENTS.md`: If conflicts occur due to GitNexus stat updates, prioritize the latest version (usually from `main` or regenerated via `npx gitnexus analyze`).

## Validation Procedures
- `cargo test property_bank` (targeted test suite)
- `mise run verify` (full quality gate)

## Rollback Procedures
- In case of failure during rebase: `git rebase --abort`.
- In case of failure after merge: `git reset --hard ORIG_HEAD` on `main`.
