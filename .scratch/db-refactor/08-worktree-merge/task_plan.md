# Task Plan: Merge `template-storage-refactor` into `main`

## Goal
Safely merge the `template-storage-refactor` branch into `main`, ensuring all improvements to Template persistence and testing are preserved without losing any work on `main`.

## Worktree Context
- **Base (Main)**: `/Users/jack/Documents/41_personal/lithos` [branch: `main`]
- **Feature**: `/Users/jack/Documents/41_personal/lithos/.worktrees/template-storage-refactor` [branch: `template-storage-refactor`]

## Phases

### Phase 1: Pre-merge Verification (Isolation)
- [x] Run full test suite in `template-storage-refactor` worktree.
- [x] Run clippy in `template-storage-refactor` worktree.
- [x] Ensure `main` is clean and up to date.

### Phase 2: Merge Execution
- [x] Switch to `main` worktree.
- [x] Merge `template-storage-refactor` into `main` using `--no-ff` to preserve history if requested, or standard merge.
- [x] Resolve any conflicts (none expected based on diff, but `Cargo.toml` or `db/mod.rs` might have minor drift).

### Phase 3: Post-merge Verification (Integrated)
- [x] Run `mise run verify` (fmt, lint, test, adr) on `main`.
- [x] Verify Template storage tests specifically.

### Phase 4: Cleanup
- [x] Remove the `template-storage-refactor` worktree.
- [x] Delete the `template-storage-refactor` branch.
- [x] Final check of `.scratch/db-refactor/` documentation.

## Decisions
- **Merge Strategy**: Standard `git merge` from `main`.
- **Worktree Removal**: `git worktree remove` after successful verification.

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| | | |
