---
title: 02-schema-tracer-bullet-read-write
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

Implement a Schema tracer-bullet vertical slice using the new seam: one complete Schema read path and one complete Schema write path through `schema/repository.rs`, `schema/storage/read.rs`, `schema/storage/write.rs`, and `schema/storage/tables.rs`, backed by `db::Store` and DB helpers.

This slice must run end-to-end with tests and demonstrate that the new seam is workable before broader Schema migration.

## Acceptance criteria

- [ ] Schema Repository seam is explicit in `schema/repository.rs` and used by the tracer-bullet read/write operations.
- [ ] One read path and one write path execute through new storage modules and pass tests with redb-backed persistence.
- [ ] Tests validate behavior at the Interface level (not helper internals) and confirm basic rollback/commit semantics.

## Blocked by

- `01-lock-db-seam-and-error-classifier.md`
