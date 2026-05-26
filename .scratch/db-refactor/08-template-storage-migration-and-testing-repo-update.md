---
title: 08-template-storage-migration-and-testing-repo-update
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

Migrate Template persistence to the new storage seam with `repository.rs`, `storage/read.rs`, `storage/write.rs`, and `storage/tables.rs`. Update Template `testing.rs` in-memory Repository Adapter to match the new Repository Interface and behavior.

This slice is complete when Template read/write and batch behavior are preserved end-to-end in both redb-backed and in-memory test flows.

## Agent Brief (v1 - 2026-05-12)

**Category:** enhancement
**Summary:** Migrate Template persistence to the segregated storage seam.

**Current behavior:**
Template persistence uses the legacy v1 repository and storage pattern.

**Desired behavior:**
1. Define `TemplateReadRepository` and `TemplateWriteRepository` traits in `template/repository.rs`.
2. Define `TemplateRepository` as a marker trait extending both.
3. Implement `TemplateRedbRepository` split across `template/storage/read.rs` and `template/storage/write.rs`.
4. Update `testing.rs` in-memory adapter to implement the new segregated traits.
5. Adopt the shared `db::testing` seam in Template's in-memory adapter:
   - Use `read_lock` / `write_lock` helpers
   - Embed `InMemoryHarness` for counters/failure injection
   - Map `InMemoryDbError` directly into Template storage errors

**Key interfaces:**
- `TemplateReadRepository` / `TemplateWriteRepository`
- `TemplateRedbRepository`
- `TemplateRepository` (marker)

**Acceptance criteria:**
- [ ] `TemplateReadRepository` and `TemplateWriteRepository` defined in `template/repository.rs`.
- [ ] `TemplateRedbRepository` implemented split across `read.rs` and `write.rs`.
- [ ] Template `testing.rs` in-memory Repository Adapter updated and passing tests.
- [ ] Existing Template behavior tests pass with new storage seam.
- [ ] Template in-memory adapter uses `db::testing::{read_lock, write_lock, InMemoryHarness}`.
- [ ] Template in-memory adapter supports failure injection (`BeforeRead`/`BeforeWrite`) and has integration tests for both paths.
- [ ] Template in-memory adapter follows naming/structure conventions from `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`.

**Revision Note (2026-05-12):**
Plan established following the **Segregated Unified Repository** pattern (ADR 016).

## TDD Implementation Plan (v1 - 2026-05-26)

Migrate `Template` storage layer following the **Segregated Unified Repository** pattern (ADR 016).

### Phase 1: Foundation (Infrastructure)
1. **Error Taxonomy**: Add `TemplateRepositoryError` to `lithos-core/src/template/error.rs` to handle storage-specific failures and domain validation at the persistence boundary.
2. **Trait Definition**: Create `lithos-core/src/template/repository.rs` defining `ReadRepository` and `WriteRepository` traits.

### Phase 2: In-Memory Adapter (Tracer Bullet)
1. **New Adapter**: Create `lithos-core/src/template/storage/testing.rs` implementing `InMemoryRepository`.
2. **DB Seam**: Wire `InMemoryHarness` into the repository for operation counting and failure injection.
3. **TDD Cycles**:
   - **RED**: Write a test for `save_template` and `find_template_by_id`.
   - **GREEN**: Implement basic `HashMap` storage with `RwLock` and `db::testing` lock helpers.
   - **RED/GREEN**: Add tests for failure injection (`BeforeRead`/`BeforeWrite`) using the harness.

### Phase 3: Redb Implementation
1. **Table Definitions**: Extract table definitions into `lithos-core/src/template/storage/tables.rs`.
2. **Segregated Implementation**: Implement `TemplateRedbRepository` split across `read.rs` and `write.rs`, using typed table wrappers.
3. **Verification**: Verify the `redb` implementation against the same behavior tests used for the in-memory double.

### Phase 4: Migration & Cleanup
1. **Catalog Update**: Update `TemplateCatalog` to depend on the new repository traits.
2. **Module Reorganization**: Update `template/mod.rs` and remove legacy `ports.rs` and `adapter/` directory.

---

## Acceptance criteria

- [ ] Template Repository Adapter uses the new storage module layout and DB seam.
- [ ] Template `testing.rs` in-memory Repository Adapter is updated to the new interface and passes tests.
- [ ] Existing Template behavior tests pass, with added coverage for changed batch/error semantics where needed.
- [ ] Cross-context adapter adoption complete for Template:
  - [ ] lock helpers use `db::testing` primitives
  - [ ] harness counters wired and verified
  - [ ] failure injection wired and verified
  - [ ] direct `InMemoryDbError` mapping in place

## Cross-context guidance reference

- This issue must apply the shared adapter guidance established by Issue 06
  (DB seam foundation) and keep adapter behavior local to Template context.

## Blocked by

- `06-db-testing-seam-and-in-memory-alignment.md`
