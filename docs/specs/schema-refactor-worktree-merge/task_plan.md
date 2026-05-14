# Task Plan: Safe Worktree Reconciliation (schema-refactor -> main)

## Goal
Produce a low-risk, repeatable merge plan that integrates `schema-refactor` work into `main` while preserving all work from both worktrees and avoiding destructive history edits.

## Current Phase
Phase 2

## Phases

### Phase 1: Requirements & Discovery
- [x] Confirm both worktrees and tracked branches
- [x] Measure branch divergence and identify risk level
- [x] Capture branch topology and current state in findings
- **Status:** complete

### Phase 2: Define Safety Rails
- [x] Define non-destructive merge policy (no reset/rewrite)
- [x] Define backup checkpoints before any integration attempt
- [x] Define isolated rehearsal environment
- **Status:** complete

### Phase 3: Build Integration Procedure
- [ ] Create a dedicated integration worktree off `main`
- [ ] Rehearse merge with preserved branches untouched
- [ ] Resolve conflicts in isolated branch with incremental verification
- [ ] Capture conflict-resolution decisions for repeatability
- **Status:** in_progress

### Phase 4: Validate and Promote
- [ ] Run full quality gates (`mise run verify`) on integration branch
- [ ] Diff-check integrated result against source branch intent
- [ ] Merge integration branch into `main` via non-fast-forward merge
- **Status:** pending

### Phase 5: Post-merge Safeguards
- [ ] Keep reconciliation artifacts (notes, tags) until stabilization
- [ ] Run smoke checks in both original worktree paths
- [ ] Retire temporary branches/worktree only after confidence window
- **Status:** pending

## Key Questions
1. Should integration be a full merge commit or selective commit migration by topic?
2. What conflict domains (schema vs note/parser vs docs/tooling) should be split into separate reconciliation passes?

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Use an isolated integration worktree | Prevents accidental mutation of either active worktree while reconciling heavy divergence |
| Preserve both source branches untouched during rehearsal | Guarantees recoverability if merge strategy needs iteration |
| Use checkpoint tags plus a git bundle backup before integration | Creates local, immutable recovery points independent of reflog expiration |
| Prefer a rehearsal merge before any promotion to `main` | Exposes conflict scope early and allows process refinement |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| None so far | 1 | N/A |

## Implementation Runbook
1. **Freeze and snapshot**
   - Ensure both worktrees are clean before integration.
   - Create immutable checkpoints:
     - `git tag pre-merge-main-<date> main`
     - `git tag pre-merge-schema-refactor-<date> schema-refactor`
   - Create belt-and-suspenders backup: `git bundle create ../lithos-pre-merge-<date>.bundle --all`
2. **Create reconciliation sandbox (third worktree)**
   - `git worktree add ../lithos-reconcile main`
   - In sandbox: `git switch -c reconcile/schema-refactor-into-main`
3. **Sync latest refs safely**
   - `git fetch --all --prune`
   - Fast-forward local `main` only (no rebases on shared branches).
4. **Pre-merge analysis (in sandbox)**
   - `git rev-list --left-right --count main...schema-refactor`
   - `git diff --name-status main...schema-refactor`
   - Partition conflict domains: `lithos-core/src/schema/**`, `lithos-core/src/note/**`, docs, tooling.
5. **Rehearsal merge**
   - `git merge --no-ff schema-refactor`
   - Resolve conflicts by domain; commit after each coherent conflict batch when possible.
   - Enable reuse for repeated attempts: `git config rerere.enabled true` (local repo setting only if acceptable).
6. **Verification loop**
   - Run targeted checks during conflict resolution:
     - `mise run fmt`
     - `mise run lint`
     - `mise run test:unit:schema`
     - `mise run test:unit:note`
   - Final gate: `mise run verify`
7. **Promotion**
   - From `main` worktree, merge only the validated reconcile branch:
     - `git merge --no-ff reconcile/schema-refactor-into-main`
   - Keep source branches for rollback confidence window.
8. **Rollback strategy (non-destructive)**
   - If promotion is bad: `git revert -m 1 <merge_commit_sha>` on `main`.
   - Do not reset or force-push shared branches.

## Notes
- Divergence is high (`main`: 183 unique commits, `schema-refactor`: 337 unique commits), so one-shot blind merge is high risk.
- This plan optimizes for safety, auditability, and repeatability over shortest path.
