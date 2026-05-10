---
title: 03-schema-batch-semantics-in-read-write
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

Add Schema batch behavior inside `schema/storage/read.rs` and `schema/storage/write.rs` (no extra batch files unless unwieldy). Deliver complete batch read and batch write flows for Schema operations that touch multiple tables in one transaction when needed.

This slice must prove atomicity and performance-oriented transaction locality for bulk operations.

## Acceptance criteria

- [ ] Batch write behavior is implemented through single write transactions and validated for atomic commit/rollback.
- [ ] Batch read behavior is implemented through single read transactions for bulk lookup/query flows.
- [ ] Tests cover multi-table batch cases and verify no partial write leaks on failure.

## Blocked by

- `02-schema-tracer-bullet-read-write.md`
