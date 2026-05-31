# Merge Strategy — Worktree Merge: split-hash-rs

## Overview
Merge the 4 committed changes from `.worktrees/04-split-hash-rs` into `main`. Main has had **zero commits** since divergence point `4897d835`, so this is a clean linear fast-forward with no conflicts.

## Overlapping Edits Analysis

| File | Worktree Changes | Main Changes Since Divergence | Conflict? |
|------|-----------------|-------------------------------|-----------|
| `lithos-core/src/support/hash.rs` | DELETED (split into 2 files) | None | No |
| `lithos-core/src/support/content_hash.rs` | CREATED (432 lines) | N/A | No |
| `lithos-core/src/support/hash_index.rs` | CREATED (559 lines) | N/A | No |
| `lithos-core/src/support/mod.rs` | Facade redirect + re-exports | None | No |
| 6 consumer files under `lithos-core/src/schema/` | Import path updates (from `hash` → `content_hash`/`hash_index`) | None | No |
| 2 consumer files under `lithos-core/src/config/` | Import path updates | None | No |
| `AGENTS.md` | Minor update (2 lines) | None | No |
| 4 scratch files under `.scratch/` | New/updated | None | No |

**Conclusion**: Zero overlapping edit regions. Merge is purely additive from the worktree side.

## Merge Sequence

1. **Fetch + check**: Ensure main is up-to-date (`git fetch origin main`)
2. **Merge**: From worktree, `git checkout main && git merge --no-ff .worktrees/04-split-hash-rs`
   - `--no-ff` preserves feature branch visibility
3. **Validate**: `mise run verify` (fmt, clippy, all tests, ADR validation)
4. **Cleanup**: Remove worktree reference

## Required Migrations
- None. The refactor is internal to `support/` with re-exports preserving all public API paths.
- All consumers already updated (verified by 1391 passing tests).

## Manual Interventions
- None expected. No conflict resolution needed.

## Validation Procedures

| Step | Command | Expected |
|------|---------|----------|
| fmt | `cargo fmt --all --check` | Clean |
| clippy | `cargo clippy -p lithos-core --all-targets -- -D warnings` | Clean |
| lib+unit tests | `cargo test -p lithos-core` | 1391 passed |
| doc tests | `cargo test -p lithos-core --doc` | 152 passed |
| all workspace | `cargo test --workspace` | All passed |
| ADR validation | `mise run adr:validate` | Passed |

## Rollback Procedures

| Failure Point | Rollback Command |
|---------------|-----------------|
| Merge conflict | `git merge --abort` |
| Test failure after merge | `git reset --hard ORIG_HEAD` |
| Post-merge regression | `git reset --hard ORIG_HEAD && git branch -D worktree-merge/04-split-hash-rs` |
