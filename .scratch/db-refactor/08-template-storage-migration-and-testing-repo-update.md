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

**Key interfaces:**
- `TemplateReadRepository` / `TemplateWriteRepository`
- `TemplateRedbRepository`
- `TemplateRepository` (marker)

**Acceptance criteria:**
- [ ] `TemplateReadRepository` and `TemplateWriteRepository` defined in `template/repository.rs`.
- [ ] `TemplateRedbRepository` implemented split across `read.rs` and `write.rs`.
- [ ] Template `testing.rs` in-memory Repository Adapter updated and passing tests.
- [ ] Existing Template behavior tests pass with new storage seam.

**Revision Note (2026-05-12):**
Plan established following the **Segregated Unified Repository** pattern (ADR 016).

## Acceptance criteria

- [ ] Template Repository Adapter uses the new storage module layout and DB seam.
- [ ] Template `testing.rs` in-memory Repository Adapter is updated to the new interface and passes tests.
- [ ] Existing Template behavior tests pass, with added coverage for changed batch/error semantics where needed.

## Blocked by

- `06-db-testing-seam-and-in-memory-alignment.md`
