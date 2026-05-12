---
title: 08-config-storage-migration-and-testing-repo-update
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

**Key interfaces:**
- `ConfigReadRepository` / `ConfigWriteRepository`
- `ConfigRedbRepository`
- `ConfigRepository` (marker)

**Acceptance criteria:**
- [ ] `ConfigReadRepository` and `ConfigWriteRepository` defined in `config/repository.rs`.
- [ ] `ConfigRedbRepository` implemented split across `read.rs` and `write.rs`.
- [ ] Config `testing.rs` in-memory Repository Adapter updated and passing tests.
- [ ] Existing Config behavior tests pass with new storage seam.

**Revision Note (2026-05-12):**
Plan established following the **Segregated Unified Repository** pattern (ADR 016).

## Acceptance criteria

- [ ] Config Repository Adapter uses the new storage module layout and DB seam.
- [ ] Config `testing.rs` in-memory Repository Adapter is updated to the new interface and passes tests.
- [ ] Existing Config behavior tests pass, with added coverage for changed batch/error semantics where needed.

## Blocked by

- `05-cross-context-interface-depth-review.md`
