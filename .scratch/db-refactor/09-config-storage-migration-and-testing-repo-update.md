---
title: 09-config-storage-migration-and-testing-repo-update
category: enhancement
label: needs-triage
status: open
date_created: 2026-05-10
---

## Type

AFK

## Labels

- needs-triage

## What to build

Migrate Config persistence to the new storage seam with `repository.rs`, `storage/read.rs`, `storage/write.rs`, and `storage/tables.rs`. Update Config `testing.rs` in-memory Repository Adapter to match the new Repository Interface and behavior.

This slice is complete when Config read/write and batch behavior are preserved end-to-end in both redb-backed and in-memory test flows.

## TDD Implementation Plan

### Phase 1: Infrastructure & Traits
1. **Repository Error**: Verify `ConfigRepositoryError` in `config/error.rs` (exists). Implement `From<InMemoryDbError> for ConfigRepositoryError` to support the testing seam.
2. **Define Traits**: Create `config/repository.rs` following the `schema`/`vault` pattern:
   * `ReadRepository` and `WriteRepository` traits.
   * Unified `Repository` marker trait extending both, with a blanket impl for all `T: ReadRepository + WriteRepository`.
3. **Migrate Tables**: Create `config/storage/tables.rs` using `db` typed wrappers (`Table`, `PathTable`) instead of raw `redb::TableDefinition`.

### Phase 2: In-Memory Implementation (`config/storage/testing.rs`)
Implement `InMemoryRepository` following **Structure A** (submodules) per `unit-naming.md`. Move and refactor from `config/testing.rs`.

#### Vertical 2.1: Global & Vault Config
* **RED**: Test `lookup::global_roundtrip` and `lookup::vault_roundtrip` in `storage/testing.rs`.
* **GREEN**: Implement `get_global`, `save_global`, `get_vault`, `save_vault` using `db::testing` primitives (`read_lock`, `write_lock`, `harness`).

#### Vertical 2.2: Versioning & Merged Config
* **RED**: Test `update::save_config_allocates_version` (atomic increments) and `lookup::config_retrieval`.
* **GREEN**: Implement `get_config`, `save_config`, and `get_active_version`.

#### Vertical 2.3: Instrumentation & Injection
* **RED**: Test `counters::increments_on_ops` and `injection::returns_error_on_injected_failure`.
* **GREEN**: Wire `harness.fail_at(FailurePoint::BeforeRead/Write)` into all methods. Ensure all lock acquisitions are counted.

### Phase 3: Redb Implementation (`config/storage/`)
Implement a single `RedbRepository` struct split across submodules.

#### Vertical 3.1: Repository Shell (`storage/mod.rs`)
* Define `RedbRepository { pub(crate) store: Arc<Store> }`.
* Implement the unified `Repository` blanket impl.

#### Vertical 3.2: Read Operations (`storage/read.rs`)
* **RED**: Integration test for `get_global` and `get_vault` using `Store::open_temp()`.
* **GREEN**: `impl ReadRepository for RedbRepository` using `Store::read` transactions and `rkyv` deserialization.

#### Vertical 3.3: Write Operations (`storage/write.rs`)
* **RED**: Integration test for `save_config` (atomic version allocation) and `save_vault_path_mapping`.
* **GREEN**: `impl WriteRepository for RedbRepository` using `Store::write` or `Store::read_write_unit_of_work`.

### Phase 4: Integration & Cleanup
1. **Update Builder**: Refactor `config/builder.rs` to accept `impl Repository`.
2. **Module Export**: Update `config/mod.rs` to export the new `storage` submodule layout.
3. **Cleanup**: Delete legacy `config/storage.rs` and `config/testing.rs`.
4. **Verification**: Run `mise run verify` to ensure all tests pass and standards are met.

## Agent Brief (v1 - 2026-05-12)

**Category:** enhancement
**Summary:** Migrate Config persistence to the segregated storage seam.

**Current behavior:**
Config persistence uses the legacy v1 repository and storage pattern.

**Desired behavior:**
1. Define `ConfigReadRepository` and `ConfigWriteRepository` traits in `config/repository.rs`.
2. Define `ConfigRepository` as a marker trait extending both.
3. Implement `ConfigRedbRepository` split across `config/storage/read.rs` and `config/storage/write.rs`.
4. Update `testing.rs` in-memory adapter to implement the new segregated traits.
5. Adopt the shared `db::testing` seam in Config's in-memory adapter:
   - Use `read_lock` / `write_lock` helpers
   - Embed `InMemoryHarness` for counters/failure injection
   - Map `InMemoryDbError` directly into Config storage errors

**Key interfaces:**
- `ConfigReadRepository` / `ConfigWriteRepository`
- `ConfigRedbRepository`
- `ConfigRepository` (marker)

**Acceptance criteria:**
- [ ] `ConfigReadRepository` and `ConfigWriteRepository` defined in `config/repository.rs`.
- [ ] `ConfigRedbRepository` implemented split across `read.rs` and `write.rs`.
- [ ] Config `testing.rs` in-memory Repository Adapter updated and passing tests.
- [ ] Existing Config behavior tests pass with new storage seam.
- [ ] Config in-memory adapter uses `db::testing::{read_lock, write_lock, InMemoryHarness}`.
- [ ] Config in-memory adapter supports failure injection (`BeforeRead`/`BeforeWrite`) and has integration tests for both paths.
- [ ] Config in-memory adapter follows naming/structure conventions from `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`.

**Revision Note (2026-05-12):**
Plan established following the **Segregated Unified Repository** pattern (ADR 016).

## Acceptance criteria

- [ ] Config Repository Adapter uses the new storage module layout and DB seam.
- [ ] Config `testing.rs` in-memory Repository Adapter is updated to the new interface and passes tests.
- [ ] Existing Config behavior tests pass, with added coverage for changed batch/error semantics where needed.
- [ ] Cross-context adapter adoption complete for Config:
  - [ ] lock helpers use `db::testing` primitives
  - [ ] harness counters wired and verified
  - [ ] failure injection wired and verified
  - [ ] direct `InMemoryDbError` mapping in place

## Cross-context guidance reference

- This issue must apply the shared adapter guidance established by Issue 06
  (DB seam foundation) and keep adapter behavior local to Config context.

## Blocked by

- `06-db-testing-seam-and-in-memory-alignment.md`
