---
title: 04-complete-schema-adapter-migration
category: enhancement
label: decomposed
status: decomposed
date_created: 2026-05-10
date_updated: 2026-05-12
---

## Type

EPIC (Decomposed into Sub-Issues)

## Labels

- decomposed

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

## Decomposition (2026-05-12)

This issue was too large to implement as a single unit (27 operations across 8 tables). It has been decomposed into focused vertical slices following TDD tracer bullet principles:

### Sub-Issues (In Recommended Order)

1. **`04a-property-bank-migration.md`** - Property Bank operations (4 methods, 2 tables)
   - Smallest complete slice
   - Proves singleton pattern
   - No dependencies beyond 03

2. **`04b-schema-index-operations.md`** - Schema index and lookup operations (6 methods, 1 new table)
   - Name and path lookups
   - Updates `save_schema()` to maintain indexes
   - Depends on: 04a (pattern proof)

3. **`04c-raw-view-operations.md`** - Raw view operations (3 methods, uses existing tables)
   - Staleness detection support
   - Cross-table lookups
   - Coordinates with 04b on `SCHEMA_ID_BY_PATH` usage

4. **`04d-topology-operations.md`** - Inheritance graph operations (2 methods, 1 table)
   - Singleton pattern (like 04a)
   - Simple, self-contained
   - Depends on: 04a (pattern proof)

5. **`04e-remaining-schema-operations.md`** - Remaining schema operations (4 methods)
   - List, find, delete operations
   - Coordinates all tables for atomic delete
   - Depends on: 04b, 04c (needs indexes and views)

### Progress Tracking

- [ ] 04a - Property Bank migration
- [ ] 04b - Schema index operations
- [ ] 04c - Raw view operations
- [ ] 04d - Topology operations
- [ ] 04e - Remaining schema operations

Once all sub-issues are complete, this epic can be marked as `completed`.

## Blocked by

- ✅ `03-schema-batch-semantics-in-read-write.md` (Completed 2026-05-12)
