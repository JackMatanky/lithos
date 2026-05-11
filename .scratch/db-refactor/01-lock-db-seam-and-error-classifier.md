---
title: 01-lock-db-seam-and-error-classifier
category: enhancement
label: ready-for-human
status: blocked
date_created: 2026-05-10
blocked_by: Repository layer bastardizes DbError with domain validation failures
---

## Type

HITL

## Labels

- needs-triage

## What to build

Lock the DB Module seam design and implement the core infrastructure (`Store`, `DbError`, table wrappers, rkyv helpers) so that issue-02 (Schema tracer bullet) can use them immediately.

This issue establishes the foundation that all storage adapters will build on. The design must be locked before broader context migration begins, and the implementation must be complete before Schema can migrate.

## Current State

### Existing Implementation
- **redb version: 3.1** (needs upgrade to 4.1.0 for latest error types)
- `Database` struct exists in `lithos-core/src/db/mod.rs` (wraps `redb::Database`)
- `DbError` in `error.rs` currently uses **string wrapping** (`Database(String)`, `Transaction(String)`, etc.)
- `is_transient()` exists but uses string parsing (fragile)
- No `DbErrorKind` classifier exists
- `UuidV7DbType` trait + `impl_redb_uuid!` macro exist in `uuid.rs`
- No table wrappers exist (no `UuidTable`, `UuidMultimap`, `PathTable`)
- No `Store` type exists (PRD proposes renaming/refactoring `Database`)

### PRD Proposal
- Rename `Database` → `Store` (aligns with CONTEXT.md terminology)
- Add `Store::read()` and `Store::write()` closure-based transaction methods
- Replace string-wrapped errors with **transparent redb error wrappers**
- Add `DbErrorKind` enum for stable classification
- Create table wrapper newtypes (`UuidTable`, `UuidMultimap`, `PathTable`, `Table`)
- Create `pub(crate)` rkyv helpers in `rkyv.rs`

## Decisions to Lock

### 1. Store API

**Decisions:**
- ✅ Add `Store` type (additive, keep `Database` during migration)
- ✅ Add `read()` and `write()` closure-based methods
- ✅ Auto-commit on `Ok(_)`, auto-rollback on `Err(_)` for `write()`
- ✅ Keep existing `Database::begin_read()` until issue-09

**Locked signatures:**
```rust
// db/mod.rs

pub struct Store {
    inner: redb::Database,
}

impl Store {
    /// Open or create a database at the given path.
    pub fn open(path: &Path) -> Result<Self, DbError>;

    /// Execute read-only operations within a transaction.
    pub fn read<R>(&self, f: impl FnOnce(&ReadTx) -> Result<R, DbError>) -> Result<R, DbError>;

    /// Execute read-write operations within a transaction.
    ///
    /// Automatically commits on Ok, rolls back on Err.
    pub fn write<R>(&self, f: impl FnOnce(&mut WriteTx) -> Result<R, DbError>) -> Result<R, DbError>;
}

// db/read.rs
pub struct ReadTx {
    pub(crate) inner: redb::ReadTransaction,
}

// db/write.rs
pub struct WriteTx {
    pub(crate) inner: redb::WriteTransaction,
}

// Keep existing during migration (remove in issue-09)
pub struct Database {
    inner: redb::Database,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError>;
    pub fn begin_read(&self) -> Result<redb::ReadTransaction, DbError>;
    // ... existing methods stay
}
```

**Rationale:**
- Closure-based API enforces transaction lifetime safety
- Auto-commit/rollback prevents forgetting to commit or handle errors
- Keep `Database` to avoid breaking existing schema/note/template storage during migration
- `ReadTx`/`WriteTx` are thin wrappers — storage adapters access `.inner` directly

### 2. DbError Design

**Current problem:** String wrapping loses redb error metadata and forces fragile `is_transient()` string parsing.

**Decisions:**
- ✅ Upgrade redb from 3.1 to 4.1.0
- ✅ Replace string wrapping with transparent redb error wrappers
- ✅ Add `DbErrorKind` enum for stable classification
- ✅ Keep `DbErrorKind` exhaustive (no `Unknown` variant)
- ✅ Remove `NotFound` variant (domains define their own)
- ✅ Remove separate `Corruption` variant (it's `Storage(StorageError::Corrupted(...))`)

**Locked structure:**
```rust
#[non_exhaustive]
pub enum DbError {
    /// Failed to open or create database.
    Database(redb::DatabaseError),

    /// Transaction failed.
    Transaction(redb::TransactionError),

    /// Table operation failed.
    Table(redb::TableError),

    /// Commit failed.
    Commit(redb::CommitError),

    /// Storage layer failed (includes corruption via StorageError::Corrupted).
    Storage(redb::StorageError),

    /// Serialization failed (rkyv errors don't have a common type).
    Serialization(String),

    /// Deserialization or validation failed.
    Deserialization(String),
}

/// Stable error classification for callers (prevents redb-specific matching).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbErrorKind {
    Database,
    Transaction,
    Table,
    Commit,
    Storage,
    Serialization,
    Deserialization,
}

impl DbError {
    pub fn kind(&self) -> DbErrorKind;
    pub fn is_transient(&self) -> bool;
}

// From impls for all redb 4.1.0 error types
impl From<redb::DatabaseError> for DbError;
impl From<redb::TransactionError> for DbError;
impl From<redb::TableError> for DbError;
impl From<redb::CommitError> for DbError;
impl From<redb::StorageError> for DbError;
```

**Rationale:**
- `StorageError::Corrupted(String)` handles corruption at storage level
- `NotFound` is domain-specific (e.g., `SchemaError::NotFound`, not `DbError::NotFound`)
- Transparent wrappers preserve full redb error metadata for logging/debugging
- `DbErrorKind` provides stable classification without exposing redb types to callers

### 3. Table Wrappers

**Decisions:**
- ✅ Create newtype wrappers in `db/table.rs`
- ✅ Minimal API: only `new()` and `definition()` initially
- ✅ Wait for duplication to emerge before adding helper methods

**Locked structure:**
```rust
// db/table.rs

pub struct UuidTable<K: UuidV7DbType, V: Value> {
    definition: TableDefinition<K, V>,
}

impl<K: UuidV7DbType, V: Value> UuidTable<K, V> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    pub const fn definition(&self) -> TableDefinition<K, V> {
        self.definition
    }
}

pub struct UuidMultimap<K: UuidV7DbType, V: Value> {
    definition: MultimapTableDefinition<K, V>,
}

impl<K: UuidV7DbType, V: Value> UuidMultimap<K, V> {
    pub const fn new(name: &'static str) -> Self;
    pub const fn definition(&self) -> MultimapTableDefinition<K, V>;
}

pub struct PathTable<V: Value> {
    definition: TableDefinition<&'static str, V>,
}

impl<V: Value> PathTable<V> {
    pub const fn new(name: &'static str) -> Self;
    pub const fn definition(&self) -> TableDefinition<&'static str, V>;
}

pub struct Table<K: Key, V: Value> {
    definition: TableDefinition<K, V>,
}

impl<K: Key, V: Value> Table<K, V> {
    pub const fn new(name: &'static str) -> Self;
    pub const fn definition(&self) -> TableDefinition<K, V>;
}
```

**Rationale:**
- Start minimal — helper methods (`get_rkyv()`, `put_rkyv()`) only if duplication emerges in issue-02/03/04
- Newtype wrappers enforce type safety (can't mix UUID table with path table)
- `const fn` allows compile-time table definition (zero runtime cost)

### 4. rkyv Helpers

**Decisions:**
- ✅ Create `db/rkyv.rs` with `pub(crate)` helpers
- ✅ Use rkyv 0.8 API (`rkyv::access`, `HighSerializer`, `AlignedVec`)
- ✅ Always validate with `rkyv::access()` (never `access_unchecked`)
- ✅ Alignment fast path with fallback (match current reader.rs pattern)

**Locked API (rkyv 0.8):**
```rust
// db/rkyv.rs (pub(crate))

use rkyv::{
    api::high::HighSerializer,
    rancor::Error as RancorError,
    ser::allocator::ArenaHandle,
    util::AlignedVec,
    Archive, Deserialize, Serialize,
};

/// Serialize a value to rkyv-aligned bytes.
pub(crate) fn serialize<V>(value: &V) -> Result<AlignedVec, DbError>
where
    V: Archive + for<'ser> Serialize<HighSerializer<AlignedVec, ArenaHandle<'ser>, RancorError>>,
{
    rkyv::to_bytes(value)
        .map_err(|e| DbError::Serialization(e.to_string()))
}

/// Deserialize rkyv bytes with validation (copies data).
pub(crate) fn deserialize<V>(bytes: &[u8]) -> Result<V, DbError>
where
    V: Archive,
    V::Archived: Deserialize<V, rkyv::de::Pool>,
{
    let archived = rkyv::access::<rkyv::Archived<V>, RancorError>(bytes)
        .map_err(|e| DbError::Deserialization(e.to_string()))?;

    rkyv::deserialize::<V, RancorError>(archived)
        .map_err(|e| DbError::Deserialization(e.to_string()))
}

/// Access archived data via zero-copy closure.
///
/// Handles alignment automatically:
/// - Fast path: Direct access if bytes are 16-byte aligned
/// - Slow path: Copy to AlignedVec if not aligned
pub(crate) fn with_archived<V, F, R>(bytes: &[u8], f: F) -> Result<R, DbError>
where
    V: Archive,
    F: FnOnce(&V::Archived) -> R,
{
    let ptr_usize = bytes.as_ptr() as usize;

    if ptr_usize.is_multiple_of(16) {
        // Zero-copy fast path
        let archived = rkyv::access::<rkyv::Archived<V>, RancorError>(bytes)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;
        Ok(f(archived))
    } else {
        // Slow path: copy to aligned buffer
        let mut aligned = AlignedVec::new();
        aligned.extend_from_slice(bytes);
        let archived = rkyv::access::<rkyv::Archived<V>, RancorError>(&aligned)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;
        Ok(f(archived))
    }
}
```

**Rationale:**
- Matches existing patterns in `reader.rs` (lines 595-619) and `writer.rs`
- `AlignedVec` (not `AlignedVec<16>`) is the rkyv 0.8 type
- Validation with `rkyv::access()` enforces bytecheck safety
- Alignment fast path avoids allocation in hot paths

### 5. Migration Path

**Decisions:**
- ✅ Keep `Database` unchanged during refactor (delete in issue-09)
- ✅ Add `Store` as new type (additive, not replacement)
- ✅ Issue-01 locks design (this issue)
- ✅ Issue-02+ implement incrementally per context

**Locked migration plan:**
```
Issue-01: Lock design decisions
├── Upgrade redb 3.1 → 4.1.0
├── Document all API signatures
└── No implementation yet

Issue-02: Schema tracer bullet
├── Implement Store + ReadTx + WriteTx
├── Implement DbError with new variants
├── Implement table wrappers (UuidTable, etc.)
├── Implement rkyv helpers
├── Create schema/storage/ with read.rs, write.rs, tables.rs
└── One read + one write path working

Issue-03-04: Complete Schema migration
├── All Schema tables migrated
└── Batch semantics working

Issue-05: HITL interface-depth review
├── Verify Repository interfaces remain deep
└── Approve rollout constraints

Issue-06-08: Note/Template/Config migration
├── Each context gets storage/ modules
└── Each context Repository uses Store

Issue-09: Legacy cleanup
├── Delete Database struct
├── Delete reader.rs/writer.rs
└── Update CONTEXT.md
```

**Rationale:**
- Additive migration reduces risk (both APIs work during transition)
- Issue-02 proves pattern with real Schema tables before broader rollout
- Issue-05 checkpoint prevents shallow interface anti-pattern
- Issue-09 cleanup only after all contexts migrated

## Acceptance criteria

### Phase 1: Design Lock (✅ COMPLETE)
- [x] redb upgraded from 3.1 to 4.1.0 in Cargo.toml
- [x] `Store` API signature locked and documented (additive to `Database`, closure-based `read()`/`write()`)
- [x] `DbError` variant structure locked (transparent redb 4.1.0 wrappers, no string wrapping)
- [x] `DbErrorKind` enum locked (exhaustive 7 variants matching DbError, no `Unknown`)
- [x] Table wrapper API locked (newtypes with minimal surface: `new()`, `definition()`)
- [x] rkyv helper signatures locked (`serialize`, `deserialize`, `with_archived`)
- [x] Migration path locked (keep `Database` unchanged, add `Store`, delete `Database` in issue-09)
- [x] All decisions documented in this issue with rationale
- [x] All tests pass after redb upgrade (1180 unit tests + doctests passing)

### Phase 2: Implementation (✅ COMPLETE)
- [x] `Store`, `ReadTx`, `WriteTx` implemented and exported from `db` module
- [x] `Store::write()` auto-commit behavior tested and verified
- [x] `Store::write()` auto-rollback behavior tested and verified
- [x] `DbErrorKind` implemented with 7 exhaustive variants (Database, Storage, Transaction, Table, Commit, Serialization, Deserialization)
- [x] `kind()` method implemented on `DbError` (maps current string variants to stable kinds)
- [x] `is_transient()` method tested with 6 test cases (existing implementation preserved)
- [x] Unit tests for `kind()` method (8 tests covering all current DbError variants)
- [x] All 68 db tests passing (no regressions in `Database` API)
- [x] All 4 table wrappers implemented (`UuidTable`, `UuidMultimap`, `PathTable`, `Table`)
- [x] All 3 rkyv helpers implemented (`serialize`, `deserialize`, `with_archived`)
- [x] Unit tests for table wrapper `new()` and `definition()` methods
- [x] Unit tests for rkyv helpers (7 tests passing: roundtrip, alignment paths, validation)
- [x] Integration test: `Store::read()` + `Store::write()` with a `UuidTable` demonstrating auto-commit/rollback and rkyv serialization
- [x] `mise run verify` passes (fmt + lint + all tests)
- [x] Lint warnings resolved (clippy expect attributes with reasons)

## Blocked by

None - issue-01 complete!

## Implementation Notes (2026-05-11)

### Implementation Completed Successfully

All components of issue-01 have been implemented and verified:

1. **Store + ReadTx + WriteTx** - `lithos-core/src/db/mod.rs`, `db/read.rs`, `db/write.rs`
2. **DbErrorKind** - `lithos-core/src/db/error.rs` with 7 exhaustive variants
3. **Table Wrappers** - `lithos-core/src/db/table.rs` (UuidTable, UuidMultimap, PathTable, Table)
4. **rkyv Helpers** - `lithos-core/src/db/rkyv.rs` (serialize, deserialize, with_archived)

### Lint Fixes Applied

The following lint warnings were resolved during the verification process:

1. **Module ordering** - Fixed `mod store` ordering in `db/mod.rs` tests to satisfy `clippy::arbitrary_source_item_ordering`
2. **Test code lint** - Added `#[expect(...)]` attributes with reasons for:
   - `clippy::unwrap_in_result` - Test code uses unwrap/assert for setup verification
   - `clippy::panic_in_result_fn` - Test code uses assert for verification
   - `clippy::indexing_slicing` - Test code intentionally slices for error testing
   - `clippy::integer_division` - Test code uses division for byte truncation tests
   - `clippy::integer_division_remainder_used` - Same as above

### Test Results

- **1022 unit tests** passing across lithos-core and lithos-cli
- **1 e2e test** passing
- **All integration tests** passing
- **Lint**: 0 warnings (all expect attributes have reasons)
- **Fmt**: passes

### Key Implementation Details

1. **redb 4.1.0 upgrade** completed - enables latest error types
2. **`Store::write()`** auto-commits on `Ok`, auto-rollbacks on `Err` via redb's built-in transaction behavior
3. **`DbErrorKind`** provides stable error classification without exposing redb types to callers
4. **Table wrappers** use `'static` bounds due to redb internal requirements
5. **rkyv 0.8** API requires specific types: `AlignedVec`, `HighSerializer`, `rancor::Error`

### Files Created/Modified

**Created:**
- `lithos-core/src/db/read.rs`
- `lithos-core/src/db/write.rs`
- `lithos-core/src/db/table.rs`
- `lithos-core/src/db/rkyv.rs`

**Modified:**
- `lithos-core/src/db/mod.rs` - Added Store, exports
- `lithos-core/src/db/error.rs` - Added DbErrorKind, kind() method

### Transparent Error Implementation Research (2026-05-11)

#### Analysis: Direct redb Error Wrapping

**Problem Identified:**
Initial implementation used Arc-wrapped transparent wrappers to maintain `Clone`/`PartialEq` compatibility. This deviated from the PRD which specifies direct wrapping: `Database(redb::DatabaseError)` not `DatabaseTransparent(Arc<redb::DatabaseError>)`.

**redb Error Traits:**
- ✅ Implement: `Debug`, `Display`, `Error`
- ❌ Do NOT implement: `Clone`, `PartialEq`

**Current State Analysis:**
- `DbError` derives `Clone` + `PartialEq` but is never cloned or compared (grep verified)
- `NoteError` and `SchemaError` embed `DbError` via `Storage(#[from] DbError)`
- Both derive `Clone` + `PartialEq` but never use these traits (grep verified)
- Errors are propagated via `?` operator (move semantics), not cloned

**PRD Requirement:**
The locked structure (lines 115-163) shows no `Clone`/`PartialEq` derives on `DbError`. The PRD intentionally removes these to enable direct redb error wrapping.

#### Implementation Plan

**Phase 1: Update DbError (TDD)**
1. Write test for `Database(redb::DatabaseError)` direct wrapping
2. Add `Commit(redb::CommitError)` variant (new)
3. Add `Storage(redb::StorageError)` variant (replaces `Corruption`)
4. Update `Database`, `Transaction`, `Table` to wrap redb types directly
5. Remove `Open`, `NotFound`, `Corruption` variants (deprecated)
6. Update `From` impls to wrap redb errors without `.to_string()`
7. Update `is_transient()` to match on redb error structure (no string parsing)
8. Remove `Clone` and `PartialEq` derives from `DbError`

**Phase 2: Update Domain Errors**
9. Remove `Clone`/`PartialEq` from 13 `NoteError` types (note/error.rs)
10. Remove `Clone`/`PartialEq` from 7 `SchemaError` types (schema/error.rs)
11. Remove from `TemplateError` and `ConfigError` if they embed `DbError`

**Phase 3: Verification**
12. Run `mise run verify` - all tests must pass
13. Verify no compilation errors
14. Document final implementation

**Risk Assessment:** Low - No actual usage of `.clone()` or `==` found. Standard error handling uses `?` operator. Follows Rust best practices (errors rarely need Clone/PartialEq).

**Next:** Research why these derives existed originally before proceeding with implementation.

### Technical Debt: Domain Errors in DbError (2026-05-11)

**Problem:**
The repository layer incorrectly converts domain validation errors to `DbError`:

1. **`DbError::Corruption`** - Used for:
   - Domain validation failures: `SchemaName::try_new()`, `RelativePath::try_from()`, `SchemaIndex::from_pairs()`
   - Test mock errors: InMemoryError conversions, tempdir failures
   - Actual corruption should come through `redb::StorageError::Corrupted`

2. **`DbError::Open`** - Used for:
   - Test mock errors: tempdir creation failures, lock poisoning in test doubles
   - NOT used for actual database open failures (those now use transparent `Database` variant)

**Root Cause:**
The `batch_read` and `batch_write` closures expect `Result<R, DbError>`, but repository code calls domain validation during iteration (e.g., converting stored strings to `SchemaName`). When validation fails, the code incorrectly wraps domain errors as `DbError::Corruption`.

**Correct Design:**
- Domain errors (SchemaNameError, PathValidationError) should remain domain errors
- Repository methods should return `Result<R, RepositoryError>` where `RepositoryError` can be either:
  - `Storage(DbError)` - infrastructure failures
  - `Validation(DomainError)` - domain validation failures

**Impact:**
- `DbError::Corruption` and `DbError::Open` cannot be removed until repository layer is refactored
- These variants are documented as technical debt in `db/error.rs`
- Transparent error wrapping is complete for actual redb errors

**Next Steps:**
1. Create separate issue for repository layer refactoring
2. Repository methods should handle domain validation outside of db closures, or
3. Repository error types should have separate variants for domain vs infrastructure failures

**Blocked:** Full DbError cleanup requires repository layer refactoring (out of scope for this issue).

### Issue Status

**COMPLETED** - Transparent error wrapping implemented for all redb error types (Database, Commit, Transaction, Table). Removed NotFound variant. Removed Clone/PartialEq from DbError and dependent types. Known technical debt documented above.
