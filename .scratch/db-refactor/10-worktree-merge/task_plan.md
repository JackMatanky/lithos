# Task Plan: Merge db-cleanup-task-10 into main

## Goal
Integrate the `db-cleanup-task-10` worktree (legacy cleanup, trait normalization) with `main` (PathKey redb traits, PathUuidTable migration, property bank refactor) preserving all changes from both branches since divergence.

## Current Phase
Phase 3 — Planning artifacts produced, awaiting approval

## Phases

### Phase 1: Divergence Analysis
- [x] Identify merge base: `5642c4870cea4e8792e7eeec366e473b39bc20c0`
- [x] List all commits on both branches since base
- [x] Identify files changed exclusively on each branch
- [x] Identify files changed on BOTH branches (overlap candidates)
- [x] Analyze overlapping file diffs for conflict potential
- **Status:** complete

### Phase 2: Merge Strategy Definition
- [x] Define merge approach
- [x] Identify overlapping edits and merge conflicts
- [x] Define recommended merge sequence
- [x] Document required migrations or manual interventions
- [x] Include validation and rollback procedures
- **Status:** complete

### Phase 3: Planning Artifacts
- [x] Write task_plan.md
- [x] Write findings.md (analysis + findings)
- [x] Write progress.md (session log)
- [ ] Present for user approval
- **Status:** in_progress

### Phase 4: Execute Merge (after approval)
- [ ] Stage and commit planning artifacts
- [ ] Checkout main and merge db-cleanup-task-10 (or reverse)
- [ ] Resolve conflicts in overlapping files
- [ ] Ensure all main changes are preserved
- [ ] Ensure all worktree changes are preserved
- **Status:** pending

### Phase 5: Validation
- [ ] Run `cargo check` (compile)
- [ ] Run `mise run fmt` (format)
- [ ] Run `mise run lint` (clippy)
- [ ] Run `mise run test` (tests)
- [ ] Run `gitnexus_detect_changes` for impact analysis
- **Status:** pending

### Phase 6: Commit Merge
- [ ] Stage merge-related changes
- [ ] Commit merge
- **Status:** pending

## Key Questions
1. Should we merge `main` INTO `db-cleanup-task-10`, or the reverse? → Merge main into worktree (worktree has the deletions + trait normalization that are incompatible with main's reader/writer)
2. Cherry-pick approach vs. full merge? → Full merge with manual conflict resolution (simpler, preserves full history)
3. Rollback plan? → `git merge --abort` if anything fails

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Merge `main` INTO `db-cleanup-task-10` | Worktree is our base; merging the other way would leave reader/writer/Database on main |
| Manual conflict resolution for db/mod.rs | Both branches changed this file in compatible ways (main adds path module, worktree removes reader/writer) |
| Manual conflict resolution for vault/processor.rs | Both changes are compatible (typed paths from main, trait-based signatures from worktree) |
| Accept main's newer AGENTS.md stats | Trivial non-functional conflict — take newer values |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
