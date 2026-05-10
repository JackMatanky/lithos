---
title: 04-complete-schema-adapter-migration
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

Migrate the full Schema Repository Adapter surface to the new storage seam across all Schema projection tables and indexes, including Schema, Property Bank, Raw Views, and inheritance/topology projection data.

This slice is complete when legacy Schema adapter call paths are replaced and Schema behavior is preserved end-to-end.

## Acceptance criteria

- [ ] Schema read and write operations are fully served by `schema/storage/read.rs`, `schema/storage/write.rs`, and `schema/storage/tables.rs`.
- [ ] Multi-table invariants for Schema projections are preserved under atomic write semantics.
- [ ] Existing Schema integration/unit tests pass, with additional tests where behavior coverage was missing.

## Blocked by

- `03-schema-batch-semantics-in-read-write.md`
