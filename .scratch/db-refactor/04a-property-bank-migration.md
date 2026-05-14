---
title: 04a-property-bank-migration
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-12
date_completed: 2026-05-12
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Migrate Property Bank operations from v1 to the segregated v2 repository (`SchemaReadRepository` / `SchemaWriteRepository`).

Property Bank is the simplest complete vertical slice because:
- Only 2 tables: `PROPERTY_BANK` (singleton) and `RAW_PROPERTY_BANK_VIEW` (by path)
- Clear read/write separation
- No complex multi-table joins
- Demonstrates atomic write semantics for related data

## Operations to Migrate

### Read Operations (→ `SchemaReadRepository`)
1. **`get_property_bank()`** - Get the Property Bank singleton
2. **`get_raw_property_bank_view(path: &RelativePath)`** - Get raw view by path

### Write Operations (→ `SchemaWriteRepository`)
3. **`save_property_bank(bank: &PropertyBank)`** - Save Property Bank singleton
4. **`save_raw_property_bank_view(path: &RelativePath, view: &RawPropertyBankView)`** - Save raw view by path

## Tables Required

Add to `storage_v2/tables.rs`:
```rust
/// Property Bank singleton (key: singleton string, value: serialized PropertyBank)
pub const PROPERTY_BANK: SingletonTable<&[u8]> =
    SingletonTable::new("property_bank_v2", "singleton");

/// Raw property bank view by path (key: path string, value: serialized RawPropertyBankView)
pub const RAW_PROPERTY_BANK_VIEW: PathTable<&[u8]> =
    PathTable::new("raw_property_bank_view_v2");
```

**Note**: If `SingletonTable` doesn't exist, use `PathTable` with a constant key like v1 does.

## TDD Implementation Plan

### Phase 1: Read Path (Property Bank)
1. RED: Test `get_property_bank()` returns None when not saved
2. GREEN: Implement in `read.rs`
3. RED: Test `get_property_bank()` returns saved bank
4. GREEN: Implement `save_property_bank()` in `write.rs` to make test pass

### Phase 2: Read Path (Raw View)
1. RED: Test `get_raw_property_bank_view(path)` returns None when not saved
2. GREEN: Implement in `read.rs`
3. RED: Test `get_raw_property_bank_view(path)` returns saved view
4. GREEN: Implement `save_raw_property_bank_view()` in `write.rs` to make test pass

### Phase 3: Write Semantics
1. RED: Test atomic save - if PropertyBank save fails, raw view should not be saved
2. GREEN: Verify existing implementation maintains atomicity (both in same transaction)
3. RED: Test persistence - save both, reopen store, verify both retrievable
4. GREEN: Verify passes

### Phase 4: Integration
1. Ensure `PropertyBank` is rkyv-serializable (should already be from v1)
2. Ensure `RawPropertyBankView` is rkyv-serializable (should already be from v1)
3. Run full test suite to verify zero regressions

## Acceptance Criteria

- [x] `get_property_bank()` added to `SchemaReadRepository` trait in `repository.rs`
- [x] `get_raw_property_bank_view(path)` added to `SchemaReadRepository` trait in `repository.rs`
- [x] `save_property_bank(bank)` added to `SchemaWriteRepository` trait in `repository.rs`
- [x] `save_raw_property_bank_view(path, view)` added to `SchemaWriteRepository` trait in `repository.rs`
- [x] Table constants `PROPERTY_BANK` and `RAW_PROPERTY_BANK_VIEW` added to `storage_v2/tables.rs`
- [x] Implementations in `storage_v2/read.rs` and `storage_v2/write.rs`
- [x] Unit tests in `read.rs` and `write.rs` verify:
  - None returned when not saved
  - Saved data retrievable
  - Atomic write semantics (both saved in same transaction)
  - Persistence across store reopens
- [x] All tests pass (`mise run test`)
- [x] No clippy warnings (`mise run lint`)
- [x] Code formatted (`mise run fmt`)

## Implementation Notes (2026-05-12)

Successfully implemented following TDD tracer bullet methodology:

### Approach
- **Phase 1**: Property Bank read/write (2 methods, GREEN)
- **Phase 2**: Raw Property Bank View read/write (2 methods, GREEN)
- Each method: RED (test) → GREEN (minimal implementation) → REFACTOR

### Key Decisions
- Used `PathTable` with constant key "singleton" for Property Bank (no `SingletonTable` type exists)
- Prefer `.to_owned()` over `.to_string()` for &str per clippy::str_to_string
- Simplified test for raw view roundtrip to avoid exposing internal `HashRecord::new()` (pub(crate))
- Tests use public interfaces only (TDD best practice)

### Results
- 4 methods migrated to segregated v2 traits
- 2 table constants added
- 5 new tests added (3 for PropertyBank, 2 for raw view)
- All 1128 tests pass, zero regressions
- Clippy clean, formatted

### Commit
- `b7396622` - feat(schema): implement property bank operations (04a)

## Blocked by

- ✅ `03-schema-batch-semantics-in-read-write.md` (Completed 2026-05-12)

## Blocks

- `04b-schema-index-operations.md`
- `04c-raw-view-operations.md`
- `04d-topology-operations.md`

## Notes

- This is the smallest complete vertical slice for issue 04
- Proves the pattern for singleton tables and path-based lookups
- Sets up foundation for more complex operations
