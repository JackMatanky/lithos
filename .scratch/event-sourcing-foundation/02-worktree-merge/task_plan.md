# Worktree Merge Plan - Issue 02

## Goal

Merge `feat/eventtable-wrapper-typed-table` back into `main` while preserving all changes made on both branches since divergence, with explicit conflict handling, validation, and rollback procedures.

## Scope

- Analyze divergence from merge-base onward for:
  - current issue branch (`feat/eventtable-wrapper-typed-table`)
  - `main`
- Include committed and uncommitted worktree changes for issue 02.
- Produce merge strategy only (no merge execution until approval).

## Phase Plan

| Phase | Description | Status |
|---|---|---|
| 1 | Gather branch divergence and changed-file inventories | complete |
| 2 | Identify overlaps/conflict risk and impact surfaces | complete |
| 3 | Draft merge sequence, validation, rollback | complete |
| 4 | Present for approval before merge | in_progress |
| 5 | (After approval) Commit planning artifacts | pending |
| 6 | (After approval) Execute merge strategy | pending |
| 7 | (After approval) Validate merged state and commit merge changes | pending |

## Constraints

- All actions must run from `.worktrees/feat/eventtable-wrapper-typed-table`.
- No merge actions before explicit approval.
- Planning artifacts stored only under `.scratch/event-sourcing-foundation/02-worktree-merge`.

## Errors Encountered

| Error | Attempt | Resolution |
|---|---:|---|
| None so far | 1 | N/A |
