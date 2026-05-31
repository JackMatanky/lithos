# Task Plan - Worktree Merge: issue-17-structured-file-format -> main

## Goal
Design a safe, reversible merge strategy from `issue/17-structured-file-format` into `main` that preserves all divergence-era changes on both sides.

## Scope
- Analyze divergence from merge-base onward.
- Identify overlapping edits and conflict risk.
- Produce merge sequence, migrations/manual interventions, validation, and rollback.
- Planning artifacts only in this phase (no merge execution).

## Phases
- [x] Phase 1: Preconditions and issue log update
  - Verified active worktree is `.worktrees/issue-17-structured-file-format`.
  - Updated issue file with implementation notes and committed.
- [x] Phase 2: Divergence and impact analysis
  - Calculated merge-base and branch/main commit deltas.
  - Compared changed files on both sides.
  - Ran GitNexus process/impact checks for touched areas.
- [x] Phase 3: Merge strategy design
  - Defined recommended merge order.
  - Documented conflict handling, manual interventions, validation, and rollback.
- [ ] Phase 4: Approval gate
  - Present findings and strategy for approval before any merge action.

## Divergence Snapshot
- Merge base: `a11bd95490e989cc6e5146effd2479290ab2200d`
- Commit delta (`main...HEAD`): `7` main-only, `4` branch-only

## Status
- Current phase: Phase 4 (awaiting approval)
- Planning-only constraint: satisfied
