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

### Phase 1: Audit & Dead Code Identification
*Identify lingering legacy patterns across the workspace.*

1.  **Grep Audit**: Search for legacy symbols and patterns:
    *   `Database::batch_read` / `Database::batch_write`
    *   `SchemaReadRepository` (and other prefixed traits) in comments or strings (should be renamed to `ReadRepository`)
    *   `storage_legacy.rs` files
2.  **Impact Analysis**: Use `gitnexus_impact` on the `Database` struct to identify remaining consumers in `lithos-core`.

### Phase 2: Legacy `Database` Cleanup
*Transition remaining calls to the `Store` architecture.*

1.  **Vertical Slice 2.1: Test Suite Migration**:
    *   **RED**: Identify a test suite still using `Database::open_temp()`.
    *   **GREEN**: Migrate to `Store::open_temp()`. Update repository initialization to use `Arc<Store>`.
    *   **VERIFY**: Test passes.
2.  **Vertical Slice 2.2: Processor Migration**:
    *   **RED**: Identify any processor/builder still holding a `Database` reference.
    *   **GREEN**: Inject `impl Repository` or `Arc<Store>` instead.
    *   **VERIFY**: `cargo check`.
3.  **Vertical Slice 2.3: Delete `Database` logic**:
    *   **RED**: Remove `Database` and its associated methods from `src/db/core.rs`.
    *   **GREEN**: Fix any resulting compiler errors by completing the cutover.
    *   **VERIFY**: `cargo build`.

### Phase 3: Interface Normalization & Deep Module Review
*Ensure consistent naming and encapsulation.*

1.  **Generic Trait Verification**:
    *   **CHECK**: Ensure `ReadRepository`, `WriteRepository`, and `Repository` are the only exported trait names for storage in all contexts.
2.  **Encapsulation Audit**:
    *   **RED**: Identify any public exposure of `RedbRepository` or `InMemoryRepository` outside their respective context `storage` modules.
    *   **GREEN**: Mark structs as `pub(crate)` and expose only via traits.
    *   **VERIFY**: `cargo check`.

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

- [ ] Legacy `Database` struct and batch methods are removed from `lithos-core/src/db/`.
- [ ] All context-specific repositories use the standardized `ReadRepository` / `WriteRepository` trait names.
- [ ] `RedbRepository` implementations are encapsulated (`pub(crate)`) and accessed only via traits.
- [ ] `mise run verify` passes with no regressions.

## Blocked by

- ✅ `07-note-storage-migration-and-testing-repo-update.md` (Completed)
- ✅ `08-template-storage-migration-and-testing-repo-update.md` (Completed)
- ✅ `09-config-storage-migration-and-testing-repo-update.md` (Completed)
