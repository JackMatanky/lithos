---
title: 04-complete-schema-adapter-migration
category: enhancement
label: decomposed
status: decomposed
date_created: 2026-05-10
date_updated: 2026-05-13
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

- [ ] Runtime schema call paths use only the segregated v2 seam (`schema/repository.rs` + `schema/storage_v2/*`) for reads and writes.
- [ ] Legacy schema storage seam call paths are removed from runtime schema orchestration.
- [ ] Multi-table invariants for Schema projections are preserved under atomic write semantics.
- [ ] Existing Schema integration/unit tests pass, with additional tests where behavior coverage was missing.
- [ ] Transitional module/component names are renamed to intended canonical names after legacy removal is verified.

## Decomposition (Updated 2026-05-13)

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

6. **`04f-builder-discovery-seam-migration.md`** - Builder/Discovery seam migration
   - Move runtime discovery/builder orchestration off legacy `schema::storage::Repository`
   - Preserve discovery behavior and cached-state read semantics
   - Depends on: 04e

7. **`04g-schema-processor-write-path-migration.md`** - Processor write-path cutover
   - Replace legacy save-many write orchestration with v2 write seam
   - Preserve atomic save/delete/raw-view/topology behavior in processor flows
   - Depends on: 04f

8. **`04h-batch-read-compat-migration.md`** - Batch read compatibility migration
   - Remove runtime dependency on legacy `with_batch_schema_reader` coupling
   - Preserve mixed hit/miss behavior and discovery efficiency
   - Depends on: 04g

9. **`04i-runtime-cutover-and-legacy-rename-cleanup.md`** - Runtime cutover + canonical renaming
   - Verify all legacy runtime schema storage call paths are removed
   - Rename transitional modules/components to intended canonical names
   - Depends on: 04h

10. **`04j-epic-closeout-docs-and-verification.md`** - Epic closeout and verification
    - Reconcile parent acceptance/progress/docs with delivered migration
    - Run full verification gates and mark epic complete
    - Depends on: 04i

### Progress Tracking

- [x] 04a - Property Bank migration
- [x] 04b - Schema index operations
- [x] 04c - Raw view operations
- [x] 04d - Topology operations
- [x] 04e - Remaining schema operations
- [ ] 04f - Builder/Discovery seam migration
- [ ] 04g - Schema processor write-path migration
- [ ] 04h - Batch read compatibility migration
- [ ] 04i - Runtime cutover and legacy rename cleanup
- [ ] 04j - Epic closeout docs and verification

Once all sub-issues are complete, this epic can be marked as `completed`.

## Blocked by

- ✅ `03-schema-batch-semantics-in-read-write.md` (Completed 2026-05-12)
