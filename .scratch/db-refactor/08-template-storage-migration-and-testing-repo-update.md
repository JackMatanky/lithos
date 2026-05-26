---
title: 08-template-storage-migration-and-testing-repo-update
category: enhancement
label: ready-for-agent
status: closed
date_created: 2026-05-10
date_completed: 2026-05-26
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Migrate Template persistence to the new storage seam with `repository.rs`, `storage/read.rs`, `storage/write.rs`, and `storage/tables.rs`. Update Template `testing.rs` in-memory Repository Adapter to match the new Repository Interface and behavior.

This slice is complete when Template read/write and batch behavior are preserved end-to-end in both redb-backed and in-memory test flows.

## TDD Implementation Plan (v1 - 2026-05-26)

Migrate `Template` storage layer following the **Segregated Unified Repository** pattern (ADR 016).

### Phase 1: Foundation (Infrastructure) - COMPLETED
1. **Error Taxonomy**: Add `TemplateRepositoryError` to `lithos-core/src/template/error.rs` to handle storage-specific failures and domain validation at the persistence boundary.
2. **Trait Definition**: Create `lithos-core/src/template/repository.rs` defining `ReadRepository` and `WriteRepository` traits.

### Phase 2: In-Memory Adapter (Tracer Bullet) - COMPLETED
1. **New Adapter**: Create `lithos-core/src/template/storage/testing.rs` implementing `InMemoryRepository`.
2. **DB Seam**: Wire `InMemoryHarness` into the repository for operation counting and failure injection.
3. **TDD Cycles**:
   - **RED**: Write a test for `save_template` and `find_template_by_id`.
   - **GREEN**: Implement basic `HashMap` storage with `RwLock` and `db::testing` lock helpers.
   - **RED/GREEN**: Add tests for failure injection (`BeforeRead`/`BeforeWrite`) using the harness.

### Phase 3: Redb Implementation - COMPLETED
1. **Table Definitions**: Extract table definitions into `lithos-core/src/template/storage/tables.rs`.
2. **Segregated Implementation**: Implement `TemplateRedbRepository` split across `read.rs` and `write.rs`, using typed table wrappers.
3. **Verification**: Verify the `redb` implementation against the same behavior tests used for the in-memory double.

### Phase 4: Migration & Cleanup - COMPLETED
1. **Catalog Update**: Update `TemplateCatalog` to depend on the new repository traits.
2. **Module Reorganization**: Update `template/mod.rs` and remove legacy `ports.rs` and `adapter/` directory.

---

## Acceptance criteria

- [x] Template Repository Adapter uses the new storage module layout and DB seam.
- [x] Template `testing.rs` in-memory Repository Adapter is updated to the new interface and passes tests.
- [x] Existing Template behavior tests pass, with added coverage for changed batch/error semantics where needed.
- [x] Cross-context adapter adoption complete for Template:
  - [x] lock helpers use `db::testing` primitives
  - [x] harness counters wired and verified
  - [x] failure injection wired and verified
  - [x] direct `InMemoryDbError` mapping in place

## Cross-context guidance reference

- This issue must apply the shared adapter guidance established by Issue 06
  (DB seam foundation) and keep adapter behavior local to Template context.

## Blocked by

- `06-db-testing-seam-and-in-memory-alignment.md`
