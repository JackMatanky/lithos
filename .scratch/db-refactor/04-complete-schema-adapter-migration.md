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

## Agent Brief (v1 - 2026-05-12)

**Category:** enhancement
**Summary:** Complete migration of all Schema storage operations to the segregated RedbRepository.

**Current behavior:**
The tracer bullets (02, 03) cover basic Schema and Raw View operations. Other Schema operations (Property Bank, inheritance trees) still use legacy v1 paths or are not yet implemented in v2.

**Desired behavior:**
1. Identify all remaining operations in the legacy `SchemaRepository` (v1).
2. Add these operations to the new segregated traits (`SchemaReadRepository` or `SchemaWriteRepository`).
3. Implement the operations in `storage_v2/read.rs` or `storage_v2/write.rs`.
4. Ensure all operations use the new `db::Store` transaction model and rkyv helpers.

**Key interfaces:**
- `SchemaReadRepository` / `SchemaWriteRepository`
- `SchemaRedbRepository`

**Acceptance criteria:**
- [ ] Schema read and write operations are fully served by `schema/storage_v2/read.rs` and `schema/storage_v2/write.rs`.
- [ ] Property Bank operations migrated to segregated traits.
- [ ] Inheritance/Topology projection data migrated to segregated traits.
- [ ] Multi-table invariants for Schema projections are preserved under atomic write semantics.
- [ ] Existing Schema integration/unit tests pass.

**Revision Note (2026-05-12):**
This plan is updated to align with ADR 016. All migrated methods must be placed in the appropriate `Read` or `Write` segregated trait.

## Acceptance criteria

- [ ] Schema read and write operations are fully served by `schema/storage/read.rs`, `schema/storage/write.rs`, and `schema/storage/tables.rs`.
- [ ] Multi-table invariants for Schema projections are preserved under atomic write semantics.
- [ ] Existing Schema integration/unit tests pass, with additional tests where behavior coverage was missing.

## Blocked by

- `03-schema-batch-semantics-in-read-write.md`
