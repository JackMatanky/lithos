# Task Plan: Safe Merge of `db-refactor-segregated-traits` into `main`

## Goal
Merge completed schema migration work from `db-refactor-segregated-traits` into `main` without losing ongoing `fs-inode-architecture` work on `main`.

## Scope
- In scope: merge strategy, safety checks, conflict-resolution approach, verification steps, cleanup steps.
- Out of scope: new feature edits unrelated to merge.

## Current State Snapshot
- Main worktree: `/Users/jack/Documents/41_personal/lithos` on `main`.
- DB refactor worktree: `/Users/jack/Documents/41_personal/lithos/.worktrees/db-refactor-segregated-traits` on `db-refactor-segregated-traits`.
- Divergence count: `main...db-refactor-segregated-traits` = `34` (main-only) / `37` (branch-only).
- Both worktrees currently clean.

## Phases

### Phase 1 - Pre-merge Safety Baseline
Status: complete

1. Confirm both worktrees are clean (`git status --short --branch`).
2. Capture divergence and candidate conflicts (`git log --left-right --cherry-pick main...db-refactor-segregated-traits`).
3. Create safety branch from current main, for rollback convenience:
   - `git checkout main`
   - `git branch backup/main-before-db-merge-<date>`

### Phase 2 - Controlled Merge Execution
Status: complete

1. In main worktree, merge with explicit commit:
   - `git merge --no-ff db-refactor-segregated-traits`
2. If conflicts occur, resolve by ownership boundary:
   - Prefer schema branch changes in `lithos-core/src/schema/**` and related schema issue docs.
   - Preserve main changes for fs-inode streams (`lithos-core/src/fs/**`, fs issue docs).
   - For shared files (`AGENTS.md`, docs, tests/common), do line-level/manual reconciliation.
3. Stage conflict resolutions and complete merge commit.

### Phase 3 - Verification Gate
Status: complete

1. Run project quality gates from main worktree:
   - `mise run fmt`
   - `mise run lint`
   - `mise run test`
2. If failures occur, fix only merge-integration regressions (no unrelated refactors).

### Phase 4 - Post-merge Validation
Status: complete

1. Validate critical changed surfaces:
   - Schema storage tests
   - FS inode path tests
   - Any shared test harness touched by both lines of work
2. Confirm no loss of fs commits:
   - `git log --oneline --left-right --cherry-pick origin/main...main`
   - Spot-check known fs commits remain reachable.

### Phase 5 - Cleanup
Status: pending

1. Keep db-refactor branch/worktree until merge is validated and optionally pushed.
2. After user confirmation, remove worktree branch if desired:
   - `git worktree remove .worktrees/db-refactor-segregated-traits`
   - `git branch -d db-refactor-segregated-traits`

## Conflict Hotspots (Validated by merge-tree)
- **Both edited (manual 3-way required):**
  - `lithos-core/src/config/error.rs`
  - `lithos-core/src/schema/error.rs`
  - `lithos-core/src/schema/mod.rs`
  - `lithos-core/src/schema/schema_processor.rs`
- **Delete/modify conflict:**
  - `lithos-core/src/schema/repository.rs` (main removes legacy file, db branch modifies/new seam file)
- **Delete/keep conflicts:**
  - `docs/refs/rust/patterns/typestate-branching-enums.md`
  - `docs/refs/rust/state-machine-pattern.md`
  - `docs/refs/rust/typestate-pattern-research.md`
  - `lithos-core/src/schema/storage.rs`
  - `lithos-core/src/schema/testing.rs`
- **Likely follow-on semantic reconciliation in tests:**
  - `lithos-core/tests/common/mod.rs`
  - `lithos-core/tests/property_bank_processor.rs`
  - `lithos-core/tests/schema_loader.rs`
  - `lithos-core/tests/schema_storage.rs`

## Risk Controls
- Never rebase either branch before merge.
- Use backup branch before merge.
- Use merge commit (`--no-ff`) to preserve history.
- Resolve conflicts manually in shared files; avoid blanket `ours/theirs` at repo level.
- Resolve in this order to minimize cascading compile errors:
  1) `schema/mod.rs` and seam module wiring
  2) repository + storage file moves/deletes
  3) schema processor/error surfaces
  4) tests/common harness and integration tests
  5) docs conflict cleanup

## Detailed Resolution Policy
- `lithos-core/src/schema/mod.rs`: keep final module graph from db-refactor seam (`repository` + `storage` split), then re-apply any main-side exports needed by fs-inode work.
- `lithos-core/src/schema/repository.rs`: MUST follow db-refactor worktree completely.
- `lithos-core/src/schema/storage.rs` and `lithos-core/src/schema/testing.rs`: accept db-refactor deletions if replaced by `storage/*` and test support moved accordingly.
- `lithos-core/src/config/error.rs` + `lithos-core/src/schema/error.rs`: preserve both semantic refactors; no regression to removed CQRS variants.
- Test harness files: prefer API shape expected by current seam (`Store`, `ReadRepository`/`WriteRepository`) while preserving fs-side behavioral assertions.

## User-Directed Hard Rules (Authoritative)
1. `lithos-core/src/schema/repository.rs` must come entirely from `db-refactor-segregated-traits`.
2. `lithos-core/tests/**` should primarily follow db-refactor branch; only keep main-side edits that were made after worktree divergence and are still required.
3. Typestate docs must remain in their moved location under `docs/refs/rust/state_machine_typestate/` after merge:
   - `docs/refs/rust/patterns/typestate-branching-enums.md`
   - `docs/refs/rust/state-machine-pattern.md`
   - `docs/refs/rust/typestate-pattern-research.md`
4. `lithos-core/src/schema/testing.rs` is intentionally replaced by `lithos-core/src/schema/storage/testing.rs`; keep db-refactor structure.
5. Treat main-branch additions as mistaken and exclude them from final merge outcome:
   - `lithos-core/src/schema/repository.rs`
   - `lithos-core/src/schema/storage_v2/core.rs`
   - `lithos-core/src/schema/storage_v2/mod.rs`
   - `lithos-core/src/schema/storage_v2/tables.rs`

## Concrete Merge Resolution Commands (during conflict)
- For files that must match db-refactor exactly:
  - `git checkout --theirs lithos-core/src/schema/repository.rs`
- For mistaken main-side files to exclude from final outcome:
  - `git rm lithos-core/src/schema/storage_v2/core.rs`
  - `git rm lithos-core/src/schema/storage_v2/mod.rs`
  - `git rm lithos-core/src/schema/storage_v2/tables.rs`
- For typestate docs relocation verification (post-merge):
  - ensure old paths above are absent
  - ensure files exist under `docs/refs/rust/state_machine_typestate/`

## Success Criteria
- Merge commit created on `main`.
- Both schema and fs lines of work are present.
- `fmt`, `lint`, and `test` pass.
- No unintended file deletions from either workstream.

## Errors Encountered
| Error | Attempt | Resolution |
|---|---:|---|
| Conventional commit hook rejected `merge:` type | 1 | Retried with allowed type: `refactor(schema): ...` |
