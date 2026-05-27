---
title: 10-cross-context-verification-and-legacy-cleanup
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-10
last_updated: 2026-05-27
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Run cross-context verification for Schema, Note, Template, and Config after migration. Remove legacy DB adapter call paths (specifically the `Database` struct and its associated batch methods), ensure in-memory testing adapters are strictly aligned with new Repository seams, and complete project-wide quality-gate verification.

This slice is complete when the migrated architecture (`Store` + `ReadRepository` / `WriteRepository`) is the only active path and all context tests verify expected behavior.

## TDD Implementation Plan (v2 - 2026-05-27)

Following the **Segregated Unified Repository** pattern (ADR 016) and project unit testing standards.

> **Note**: All Phase 1–3 items are complete. Only Phase 4 (Quality Gate) remains.

### Phase 1: Audit & Dead Code Identification
*Identify lingering legacy patterns across the workspace.*

1.  **Grep Audit**: Search for legacy symbols and patterns:
    *   `Database::batch_read` / `Database::batch_write` — ✅ Not found (already removed)
    *   Prefixed traits (e.g. `SchemaReadRepository`) — ✅ All renamed to `ReadRepository`/`WriteRepository`
    *   `storage_legacy.rs` files — ✅ Not found
2.  **Impact Analysis**: `gitnexus_impact` on `Database` struct — ✅ Confirmed zero consumers (only `DbError::Database` error variant references remain, which are unrelated).

### Phase 2: Legacy `Database` Cleanup
*Transition remaining calls to the `Store` architecture.*

1.  **Vertical Slice 2.1: Test Suite Migration**:
    *   **RED**: Identify a test suite still using `Database::open_temp()`.
    *   **GREEN**: Migrate to `Store::open_temp()`. Update repository initialization to use `Arc<Store>`.
    *   **VERIFY**: Test passes. ✅ Done (benchmarks migrated, all tests use `Store::open_temp()`)
2.  **Vertical Slice 2.2: Processor Migration**:
    *   **RED**: Identify any processor/builder still holding a `Database` reference.
    *   **GREEN**: Inject `impl Repository` or `Arc<Store>` instead.
    *   **VERIFY**: `cargo check`. ✅ Done (all processors accept `Arc<Store>` or `impl Repository`)
3.  **Vertical Slice 2.3: Delete `Database` logic**:
    *   **RED**: Remove `Database` and its associated methods from `src/db/core.rs`.
    *   **GREEN**: Fix any resulting compiler errors by completing the cutover.
    *   **VERIFY**: `cargo build`. ✅ Done (`Database` struct removed from `core.rs`, `reader.rs`/`writer.rs` deleted, `cargo check` passes)

### Phase 3: Interface Normalization & Deep Module Review
*Ensure consistent naming and encapsulation.*

1.  **Generic Trait Verification**:
    *   **CHECK**: Ensure `ReadRepository`, `WriteRepository`, and `Repository` are the only exported trait names for storage in all contexts.
    *   ✅ Schema: `ReadRepository`/`WriteRepository`/`Repository`
    *   ✅ Note: `ReadRepository`/`WriteRepository`/`Repository`
    *   ✅ Template: `ReadRepository`/`WriteRepository`/`Repository`
    *   ✅ Config: `ReadRepository`/`WriteRepository`/`Repository`
    *   ✅ Vault: `ReadRepository`/`WriteRepository`/`Repository`
2.  **Encapsulation Audit**:
    *   **Decision (2026-05-27)**: `RedbRepository` visibility stays `pub` — these are the production implementations and need to be instantiated by the application layer. `InMemoryRepository` remains `pub(crate)` for internal testing only.

### Phase 4: Final Quality Gate (Definition of Done)
*Execute full verification suite.*

1.  **Full Verification**: Run `mise run verify` (alias `v`).
2.  **DoD Checklist**:
    *   [ ] All tests pass (`mise run test`)
    *   [ ] Code formatted (`mise run fmt`)
    *   [ ] No clippy warnings (`mise run lint`)
    *   [ ] All public APIs have doc comments
    *   [ ] No `unwrap()`/`panic!` in production code
    *   [ ] Context boundaries respected (no cross-imports)

## Acceptance criteria

- [x] Legacy `Database` struct and batch methods are removed from `lithos-core/src/db/`.
- [x] All context-specific repositories use the standardized `ReadRepository` / `WriteRepository` trait names.
- [x] `RedbRepository` implementations are `pub` (accessible for app-level instantiation); `InMemoryRepository` is `pub(crate)` for internal testing.
- [ ] `mise run verify` passes with no regressions.

## Implementation Notes

### Work Done (Commit `39a7f93f`)
- Deleted `lithos-core/src/db/reader.rs` and `lithos-core/src/db/writer.rs`.
- Migrated benchmarks (`db_storage.rs`, `db_key_handling.rs`, `string_construction.rs`) to `Store`/`Table` APIs.
- Removed `Database` struct from `lithos-core/src/db/core.rs`.
- `pub use core::Store` — only `Store` exported (not `Database`).
- Normalized trait names to `ReadRepository`/`WriteRepository`/`Repository` in all 5 contexts.

### Work Done (Commit `a49d0089`)
- Corrected access modifiers (`RedbRepository` → `pub`, `InMemoryRepository` → `pub(crate)`).
- Fixed `dead_code` warnings.

### Known Caveats
- `cargo check` passes on branch `db-cleanup-task-10`.
- The main branch has additional commits (`PathKey` redb `Value` trait impl, `PathUuidTable`/`UuidPathTable` wrappers, typed `FilePath`/`DirPath` in `vault/processor.rs`) that are **not** required by Issue 10 scope. These should be cherry-picked separately if needed (see below).

### Cherry-Pick Guide (Main → Worktree)
If the `PathUuidTable`/`UuidPathTable` migration is desired later, the minimal files to bring from `main`:
1. `lithos-core/src/db/path.rs` (new file — PathKey redb::Value)
2. `lithos-core/src/db/mod.rs` (add `pub mod path;` + export `PathUuidTable`/`UuidPathTable` — adapt to skip reader/writer)
3. `lithos-core/src/vault/processor.rs` (typed `FilePath`/`DirPath` changes)
4. `lithos-core/src/db/table.rs` (add `PathUuidTable`/`UuidPathTable` structs)
5. Supporting changes in vault/note storage files

DO NOT full-merge `main` — it has `reader.rs`/`writer.rs` and `Database` export that conflict with this branch's cleanup.

## Blocked by

- ✅ `07-note-storage-migration-and-testing-repo-update.md` (Completed)
- ✅ `08-template-storage-migration-and-testing-repo-update.md` (Completed)
- ✅ `09-config-storage-migration-and-testing-repo-update.md` (Completed)
