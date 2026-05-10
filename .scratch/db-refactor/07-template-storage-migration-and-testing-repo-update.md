---
title: 07-template-storage-migration-and-testing-repo-update
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

## Acceptance criteria

- [ ] Template Repository Adapter uses the new storage module layout and DB seam.
- [ ] Template `testing.rs` in-memory Repository Adapter is updated to the new interface and passes tests.
- [ ] Existing Template behavior tests pass, with added coverage for changed batch/error semantics where needed.

## Blocked by

- `05-cross-context-interface-depth-review.md`
