# Task Plan — Worktree Merge: split-hash-rs

## Goal
Merge `.worktrees/04-split-hash-rs` into `main`, preserving all module-split changes, test improvements, and doc updates without behavioral regression.

## Divergence
- **Common ancestor**: `4897d835`
- **Main changes since divergence**: **NONE**
- **Worktree commits** (4):
  1. `7de0a41a` — docs(scratch): triage + agent brief
  2. `076bbb72` — refactor: split support::hash into content_hash + hash_index
  3. `f55922be` — test: normalize test naming + fill coverage gaps
  4. `cd288ba8` — docs: improve doc comments per rust best practices

## Phases
- [ ] Phase 1: Analysis & Divergence Check
- [ ] Phase 2: Merge Strategy Design
- [ ] Phase 3: Execution & Validation

## Decisions
| Decision | Rationale | Status |
|----------|-----------|--------|
| Merge via rebase (linear history) | No main changes → clean fast-forward possible; rebase keeps linear | pending |
| `git merge --no-ff` | Preserves feature branch visibility in history | pending |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| | | |
