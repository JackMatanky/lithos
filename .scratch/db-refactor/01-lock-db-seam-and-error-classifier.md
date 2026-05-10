---
title: 01-lock-db-seam-and-error-classifier
category: enhancement
label: ready-for-human
status: in-progress
date_created: 2026-05-10
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

### Phase 2: Implementation
- [ ] `Store`, `ReadTx`, `WriteTx` implemented and exported from `db` module
- [ ] `DbError` implemented with all 7 variants + `From` impls for redb types
- [ ] `DbErrorKind` implemented with `kind()` and `is_transient()` methods
- [ ] All 4 table wrappers implemented (`UuidTable`, `UuidMultimap`, `PathTable`, `Table`)
- [ ] All 3 rkyv helpers implemented (`serialize`, `deserialize`, `with_archived`)
- [ ] Unit tests for `DbError::is_transient()` covering all variants
- [ ] Unit tests for table wrapper `new()` and `definition()` methods
- [ ] Unit tests for rkyv helpers (roundtrip, alignment paths, validation)
- [ ] Integration test: `Store::read()` + `Store::write()` with a `UuidTable` demonstrating auto-commit/rollback
- [ ] All existing tests still pass (no regressions in `Database` API)
- [ ] `mise run verify` passes (fmt + lint + all tests)

## Blocked by

None - can start immediately

## Implementation Scope

**This issue (01) implements:**

### Phase 1: Design Lock + redb Upgrade (✅ DONE)
- [x] Upgrade redb from 3.1 to 4.1.0 in `Cargo.toml`
- [x] Lock all API signatures with rationale
- [x] All tests passing after redb upgrade

### Phase 2: DB Module Infrastructure (IN PROGRESS)
- [ ] Implement `Store` in `db/mod.rs`
- [ ] Implement `ReadTx` in `db/read.rs`
- [ ] Implement `WriteTx` in `db/write.rs`
- [ ] Implement `DbError` with new variants in `db/error.rs`
- [ ] Implement `DbErrorKind` classifier in `db/error.rs`
- [ ] Implement table wrappers in `db/table.rs` (`UuidTable`, `UuidMultimap`, `PathTable`, `Table`)
- [ ] Implement rkyv helpers in `db/rkyv.rs` (`serialize`, `deserialize`, `with_archived`)
- [ ] Unit tests for each module
- [ ] Integration test demonstrating `Store::read()` and `Store::write()` with table wrappers
- [ ] All tests passing (including existing Database tests)

**Issue-02 implements:**
- Schema tracer bullet using the db infrastructure from issue-01
- `schema/repository.rs` trait definition
- `schema/storage/` modules (read.rs, write.rs, tables.rs)
- One read path + one write path end-to-end

## Notes

**Phase 1 (design lock) is complete.** All API signatures are locked with rationale. redb 4.1.0 upgrade is done and tests pass.

**Phase 2 (implementation) is next.** We need to implement the db module infrastructure before issue-02 can start the Schema tracer bullet.

### Files to Create
- `lithos-core/src/db/read.rs` — `ReadTx` newtype
- `lithos-core/src/db/write.rs` — `WriteTx` newtype
- `lithos-core/src/db/table.rs` — Table wrapper newtypes
- `lithos-core/src/db/rkyv.rs` — `pub(crate)` rkyv helpers

### Files to Modify
- `lithos-core/src/db/mod.rs` — Add `Store` type, export new types
- `lithos-core/src/db/error.rs` — Replace string wrapping with transparent wrappers, add `DbErrorKind`

### Implementation Order
1. Start with `DbError` + `DbErrorKind` (foundation for all other types)
2. Implement table wrappers (needed by `Store`)
3. Implement rkyv helpers (needed by integration test)
4. Implement `Store`, `ReadTx`, `WriteTx` (uses everything above)
5. Write integration test proving the pattern works
6. Verify no regressions in existing `Database` API
