# Worktree Merge Plan - 01 EventId

## Goal

Safely merge changes from `feat/eventid-core-type-redb-contract` and `main` from divergence point `a11bd95490e989cc6e5146effd2479290ab2200d`, preserving all changes from both lines of development.

## Scope

- Planning-only phase artifacts in `.scratch/event-sourcing-foundation/01-worktree-merge/`
- Analyze divergence, overlap, conflict risk, merge order, migration/manual steps
- Define validation and rollback procedures

## Constraints

- Work only in dedicated worktree: `.worktrees/feat/eventid-core-type-redb-contract`
- Do not execute merge until user approval
- Preserve both branches' changes since divergence

## Phases

### Phase 1 - Divergence and change-set inventory
- [x] Identify divergence commit
- [x] Enumerate feature branch commits since divergence
- [x] Enumerate main branch commits since divergence
- [x] Enumerate file-level change sets on both sides

### Phase 2 - Overlap/conflict/risk analysis
- [x] Compute overlapping changed files
- [x] Identify likely merge conflicts
- [x] Analyze semantic coupling and migration/manual interventions
- [x] Assess validation needs

### Phase 3 - Merge strategy design
- [x] Define recommended merge sequence
- [x] Define pre-merge safety checks
- [x] Define post-merge validation gates
- [x] Define rollback procedure

### Phase 4 - Approval gate
- [x] Present findings and strategy for approval
- [ ] Wait for approval before merge execution

### Phase 5 - Execution (post-approval only)
- [ ] Stage/commit planning artifacts
- [ ] Execute approved merge strategy
- [ ] Validate merged state
- [ ] Stage/commit merge-related changes

## Decision Log

- Preferred integration direction: merge feature branch into `main` via non-fast-forward merge commit to preserve history.
- If execution occurs from feature worktree, use explicit refs and verify target branch/worktree before each destructive operation.

## Errors Encountered

- None during planning analysis.
