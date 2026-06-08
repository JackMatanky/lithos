# Task Plan: Merge `base-schema-07-integration-regression-suite` into `main`

## Goal
Safely merge the dedicated worktree changes back into the main branch, preserving all developments and resolving conflicts with a documented strategy and validation.

## Phases

### Phase 1: Divergence Analysis
- [ ] Determine merge base between worktree branch and main
- [ ] List all changes in worktree branch since divergence
- [ ] List all changes in main branch since divergence
- [ ] Identify overlapping file edits

### Phase 2: Impact & Semantic Analysis
- [ ] Use GitNexus to identify if changes in main affect symbols modified/added in worktree
- [ ] Use GitNexus to identify if changes in worktree affect symbols modified in main
- [ ] Apply `rust-best-practices` to evaluate the combined state

### Phase 3: Merge Strategy Documentation
- [ ] Define the merge sequence (e.g., main -> worktree first)
- [ ] Document conflict resolution approach for specific files
- [ ] List any required migrations or manual interventions
- [ ] Define validation steps (test suites, lints) and rollback plan

### Phase 4: Execution (Post-Approval)
- [ ] Perform the merge sequence
- [ ] Resolve conflicts as planned
- [ ] Run full validation suite (`mise run verify`)
- [ ] Commit merged state and planning artifacts

## Decisions
- Merge strategy: Merge `main` into worktree branch first to resolve root-level planning file conflicts (preferring `main`) and verify against the evolved `main` codebase before final merge.

## Status
- **Current Phase:** Phase 3: Merge Strategy Documentation (Complete)
- **Progress:** 75%
