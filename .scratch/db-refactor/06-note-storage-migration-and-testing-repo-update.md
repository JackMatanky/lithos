---
title: 06-note-storage-migration-and-testing-repo-update
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

## Acceptance criteria

- [ ] Note Repository Adapter uses the new storage module layout and DB seam.
- [ ] Note `testing.rs` in-memory Repository Adapter is updated to the new interface and passes tests.
- [ ] Existing Note behavior tests pass, with added coverage for changed batch/error semantics where needed.

## Blocked by

- `05-cross-context-interface-depth-review.md`
