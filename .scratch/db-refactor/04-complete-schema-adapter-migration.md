---
title: 04-complete-schema-adapter-migration
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-10
date_updated: 2026-05-14
date_completed: 2026-05-14
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
3. Implement the operations in the active storage seam (`schema/storage/read.rs` / `schema/storage/write.rs`).
4. Ensure all operations use the new `db::Store` transaction model and rkyv helpers.

**Key interfaces:**
- `ReadRepository` / `WriteRepository`
- `RedbRepository`

**Acceptance criteria:**
- [x] Schema read and write operations are fully served by the active seam (`schema/storage/read.rs` and `schema/storage/write.rs`).
- [x] Property Bank operations migrated to segregated traits.
- [x] Inheritance/Topology projection data migrated to segregated traits.
- [x] Multi-table invariants for Schema projections are preserved under atomic write semantics.
- [x] Existing Schema integration/unit tests pass.

**Revision Note (2026-05-12):**
This plan is updated to align with ADR 016. All migrated methods must be placed in the appropriate `Read` or `Write` segregated trait.

## Acceptance criteria

- [x] Runtime schema call paths use only the segregated seam (`schema/repository.rs` + `schema/storage/*`) for reads and writes.
- [x] Legacy schema storage seam call paths are removed from runtime schema orchestration.
- [x] Multi-table invariants for Schema projections are preserved under atomic write semantics.
- [x] Existing Schema integration/unit tests pass, with additional tests where behavior coverage was missing.
- [x] Transitional module/component names are renamed to intended canonical names after legacy removal is verified.

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

- [x] 04a - Property Bank migration (Completed 2026-05-13)
- [x] 04b - Schema index operations (Completed 2026-05-13)
- [x] 04c - Raw view operations (Completed 2026-05-13)
- [x] 04d - Topology operations (Completed 2026-05-13)
- [x] 04e - Remaining schema operations (Completed 2026-05-13)
- [x] 04f - Builder/Discovery seam migration (Completed 2026-05-14)
- [x] 04g - Schema processor write-path migration (Completed 2026-05-14)
- [x] 04h - Batch read compatibility migration (Completed 2026-05-14)
- [x] 04i - Runtime cutover and legacy rename cleanup (Completed 2026-05-14)
- [x] 04j - Epic closeout docs and verification (Completed 2026-05-14)

**All sub-issues complete. This epic is ready to be marked as `completed`.**

## Blocked by

- ✅ `03-schema-batch-semantics-in-read-write.md` (Completed 2026-05-12)

## Implementation Summary

### Final Architecture

**Schema Storage Seam (Post-Migration):**
```
schema/
├── repository.rs          # Trait definitions (ReadRepository, WriteRepository, Repository)
└── storage/
    ├── mod.rs            # RedbRepository struct
    ├── read.rs           # ReadRepository implementation (private)
    ├── write.rs          # WriteRepository implementation (private)
    ├── tables.rs         # Table definitions and constants (public)
    └── testing.rs        # InMemoryRepository test double
```

**Segregated Traits (ADR 016 Compliance):**
- `ReadRepository` - 18 read-only methods
- `WriteRepository` - 6 write methods with atomicity
- `Repository` - Unified trait (blanket impl for T: ReadRepository + WriteRepository)

**Implementations:**
- `RedbRepository` - Production storage using `redb`
- `InMemoryRepository` - Test double for pure unit tests

### Migration Outcomes

**Operations Migrated:** 27 operations across 8 tables
- 4 Property Bank operations (2 tables: `PROPERTY_BANK`, `RAW_PROPERTY_BANK_VIEW`)
- 6 Schema index operations (3 tables: `SCHEMA_ID_BY_NAME`, `SCHEMA_ID_BY_PATH`, `SCHEMAS`)
- 3 Raw view operations (2 tables: `RAW_SCHEMA_VIEWS`, `SCHEMA_ID_BY_PATH`)
- 2 Topology operations (1 table: `SCHEMA_TOPOLOGICAL_GRAPH`)
- 4 Remaining schema operations (coordinated across all tables)
- 8 Builder/Discovery operations (runtime cutover)

**Tables in Active Seam:**
1. `SCHEMAS` - Schema aggregates by ID
2. `RAW_SCHEMA_VIEWS` - Staleness detection views
3. `PROPERTY_BANK` - Singleton property bank
4. `RAW_PROPERTY_BANK_VIEW` - Property bank staleness detection
5. `SCHEMA_BASE_PROPERTIES` - Cached base properties per schema
6. `SCHEMA_TOPOLOGICAL_GRAPH` - Inheritance graph singleton
7. `SCHEMA_ID_BY_NAME` - Name-to-ID index
8. `SCHEMA_ID_BY_PATH` - Path-to-ID index

### Key Invariants Preserved

**Multi-Table Atomicity:**
- Schema save updates both `SCHEMAS` and `SCHEMA_ID_BY_NAME` atomically
- Schema delete removes aggregate + all indexes + raw view in single transaction
- Raw view save updates both view storage and path index atomically

**Transaction Boundaries:**
- Each repository method manages its own transaction
- Batch operations group multiple ops into single transaction
- Write failures trigger automatic rollback (no partial state visible)

### Test Coverage

**Verification Status:**
- ✅ All existing integration tests pass (36 tests)
- ✅ All unit tests pass (1147 tests)
- ✅ E2E test passes (1 test)
- ✅ Total: 1184 tests, 0 failures

**Test Organization:**
- Unit tests use `InMemoryRepository` for pure computation testing
- Integration tests use `RedbRepository` for durability verification
- Discovery/processor tests verify end-to-end orchestration

### Notable Tradeoffs

**1. Segregated vs Unified Interface:**
- **Decision:** Provide both segregated traits and unified `Repository` trait
- **Rationale:** Capability-based access control for read-only consumers + convenience for orchestration
- **Impact:** More flexibility, slight API surface increase

**2. Transaction Granularity:**
- **Decision:** Per-method transactions (not user-controlled)
- **Rationale:** Simplifies API, prevents transaction leakage
- **Impact:** Can't batch arbitrary operations across traits

**3. Batch Read Semantics:**
- **Decision:** Preserve legacy `find_raw_schema_views_by_paths` ordering
- **Rationale:** Discovery code relies on index alignment (see 04h)
- **Impact:** Maintains `Vec<Option<T>>` pattern for partial hits

**4. Helper Visibility:**
- **Decision:** Keep `path_key` as `pub(super)` not public
- **Rationale:** Implementation detail, not part of public API
- **Impact:** Centralized but not exposed to external consumers

### Documentation Quality

**Coverage:** 100% of public API documented
- 3 module docs with architecture and design rationale
- 2 struct docs with performance characteristics
- 24 trait methods with error documentation
- 10 table constants with purpose and usage
- 8 helper functions with behavior documentation
- 5 usage examples demonstrating common patterns

**Standards:** Full compliance with Rust Best Practices Chapter 8
- Module docs explain purpose, exports, invariants
- Item docs explain what, how, parameters, returns, errors
- Examples show practical usage patterns
- All intra-doc links functional
- Performance and thread safety documented

### Verification Evidence

```bash
# All verification gates passed (2026-05-14)
mise run fmt      # ✅ PASS
mise run lint     # ✅ PASS (0 warnings in lithos-core)
mise run test     # ✅ PASS (1184 tests)
cargo doc         # ✅ PASS (0 warnings in schema storage seam)
```

### Files Modified (Epic-Wide)

**Core Implementation:**
- `lithos-core/src/schema/repository.rs` - Trait definitions
- `lithos-core/src/schema/storage/mod.rs` - RedbRepository
- `lithos-core/src/schema/storage/read.rs` - Read implementation
- `lithos-core/src/schema/storage/write.rs` - Write implementation
- `lithos-core/src/schema/storage/tables.rs` - Table definitions
- `lithos-core/src/schema/storage/testing.rs` - Test utilities

**Runtime Integration:**
- `lithos-core/src/schema/builder.rs` - Builder seam migration
- `lithos-core/src/schema/discovery.rs` - Discovery seam migration
- `lithos-core/src/schema/property_bank_processor.rs` - Processor write migration
- `lithos-core/src/schema/schema_processor.rs` - Processor write migration

**Tests:**
- `lithos-core/src/schema/storage/read.rs` - Unit tests
- `lithos-core/src/schema/storage/write.rs` - Unit tests
- `tests/integration/schema_storage.rs` - Integration tests
- `tests/integration/schema_loader.rs` - End-to-end tests

**Total:** ~3000 lines of implementation + ~1500 lines of tests + ~400 lines of documentation

### Lessons Learned

**What Worked Well:**
1. **TDD tracer bullet decomposition** - Breaking 27-operation epic into 10 focused slices
2. **Vertical slice ordering** - Each sub-issue proved a pattern for later slices
3. **Segregated trait design** - Clear separation enabled independent read/write evolution
4. **Test-first approach** - Caught edge cases early (e.g., partial batch hits, lock poisoning)
5. **Documentation audit** - Systematic Chapter 8 checklist found all gaps

**Challenges:**
1. **Batch semantics alignment** - Discovery relied on `Vec<Option<T>>` ordering (not obvious from signature)
2. **Cross-table coordination** - Delete operations needed careful sequencing across 4+ tables
3. **Transaction boundary documentation** - Clarifying "single transaction" vs "per-method transaction"
4. **Helper visibility** - Balancing reusability vs API surface complexity

**Recommendations for Future Migrations:**
1. Start with smallest complete vertical slice (proves pattern)
2. Document transaction boundaries explicitly in module docs
3. Add integration tests for cross-table invariants early
4. Use GitNexus for impact analysis before refactoring
5. Apply documentation audit systematically at closeout
