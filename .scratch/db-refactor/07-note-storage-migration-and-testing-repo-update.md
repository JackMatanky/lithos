---
title: 07-note-storage-migration-and-testing-repo-update
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

Migrate Note persistence to the new storage seam with `repository.rs`, `storage/read.rs`, `storage/write.rs`, and `storage/tables.rs`. Update Note `testing.rs` in-memory Repository Adapter to match the new Repository Interface and behavior.

This slice is complete when Note read/write and batch behavior are preserved end-to-end in both redb-backed and in-memory test flows.

## Agent Brief (v1 - 2026-05-12)

**Category:** enhancement
**Summary:** Migrate Note persistence to the segregated storage seam.

**Current behavior:**
Note persistence uses the legacy v1 repository and storage pattern.

**Desired behavior:**
1. Define `NoteReadRepository` and `NoteWriteRepository` traits in `note/repository.rs`.
2. Define `NoteRepository` as a marker trait extending both.
3. Implement `NoteRedbRepository` split across `note/storage/read.rs` and `note/storage/write.rs`.
4. Update `testing.rs` in-memory adapter to implement the new segregated traits.

**Key interfaces:**
- `NoteReadRepository` / `NoteWriteRepository`
- `NoteRedbRepository`
- `NoteRepository` (marker)

**Acceptance criteria:**
- [ ] `NoteReadRepository` and `NoteWriteRepository` defined in `note/repository.rs`.
- [ ] `NoteRedbRepository` implemented split across `read.rs` and `write.rs`.
- [ ] Note `testing.rs` in-memory Repository Adapter updated and passing tests.
- [ ] Existing Note behavior tests pass with new storage seam.

**Revision Note (2026-05-12):**
Plan established following the **Segregated Unified Repository** pattern (ADR 016).

## Acceptance criteria

- [ ] Note Repository Adapter uses the new storage module layout and DB seam.
- [ ] Note `testing.rs` in-memory Repository Adapter is updated to the new interface and passes tests.
- [ ] Existing Note behavior tests pass, with added coverage for changed batch/error semantics where needed.

## Blocked by

- `06-db-testing-seam-and-in-memory-alignment.md`
