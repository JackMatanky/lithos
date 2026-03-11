# Schema Refactor: Detailed Migration Plan

**Status**: Ready to Execute
**Created**: 2026-03-06
**Estimated Duration**: 46 hours (~6 days)

---

## Overview

This document provides **step-by-step migration instructions** for refactoring the schema module. Each step is designed to be:
- **Atomic**: Can be completed in one sitting
- **Testable**: Can verify correctness before proceeding
- **Reversible**: Can rollback if issues arise

---

## Pre-Flight Checklist

Before starting ANY phase:

- [ ] All existing tests pass (`mise run test`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] Code formatted (`mise run fmt`)
- [ ] Working tree clean (`git status`)
- [ ] Create feature branch: `git checkout -b refactor/schema-file-centric`
- [ ] Set up tracking: Copy this file to `MIGRATION_PROGRESS.md` and check off tasks

---

## Phase 0: Planning ✅ COMPLETE

- [x] Identify architectural issues
- [x] Define target architecture
- [x] Choose module structure (flat + loader)
- [x] Document validation boundaries
- [x] Create refactor plan (`SCHEMA_REFACTOR_PLAN.md`)
- [x] Create decision log (`SCHEMA_REFACTOR_DECISIONS.md`)
- [x] Create migration plan (this document)

**Deliverable**: Planning documents approved

---

## Phase 1: Infrastructure (Raw File Storage + Blake3)

**Goal**: Add raw file caching without breaking existing code.

**Duration**: 8 hours

**Git strategy**: Work in `phase-1-raw-storage` branch, merge to feature branch when complete.

---

### Step 1.1: Add Dependencies (30 min)

**Task**: Add `blake3` and `zstd` crates to `Cargo.toml`.

**Commands**:
```bash
cd lithos-core
cargo add blake3
cargo add zstd
```

**Verification**:
```bash
cargo check
```

**Git**:
```bash
git add Cargo.toml Cargo.lock
git commit -m "build: add blake3 and zstd dependencies"
```

---

### Step 1.2: Create `Blake3Hash` Type (1 hour)

**Task**: Create newtype wrapper for Blake3 hash.

**File**: `lithos-core/src/schema/hash.rs`

```rust
//! Blake3 hash types for content addressing.

use std::fmt;
use rkyv::{Archive, Deserialize, Serialize};

/// Blake3 hash (32 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
pub struct Blake3Hash([u8; 32]);

impl Blake3Hash {
    /// Compute Blake3 hash of bytes.
    #[inline]
    pub fn compute(bytes: &[u8]) -> Self {
        let hash = blake3::hash(bytes);
        Self(*hash.as_bytes())
    }

    /// Get hash as byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::str::FromStr for Blake3Hash {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(Self(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_compute() {
        let data = b"hello world";
        let hash1 = Blake3Hash::compute(data);
        let hash2 = Blake3Hash::compute(data);
        assert_eq!(hash1, hash2, "Same input produces same hash");
    }

    #[test]
    fn test_blake3_display_parse() {
        let hash = Blake3Hash::compute(b"test");
        let hex_str = hash.to_string();
        let parsed: Blake3Hash = hex_str.parse().unwrap();
        assert_eq!(hash, parsed, "Display/parse roundtrip works");
    }
}
```

**Update**: `lithos-core/src/schema/mod.rs`
```rust
pub mod hash;
```

**Verification**:
```bash
cargo test --package lithos-core --lib schema::hash
```

**Git**:
```bash
git add lithos-core/src/schema/hash.rs lithos-core/src/schema/mod.rs
git commit -m "feat(schema): add Blake3Hash type with rkyv support"
```

---

### Step 1.3: Create `RingBuffer<T, N>` (1.5 hours)

**Task**: Create fixed-size ring buffer for versioned storage.

**File**: `lithos-core/src/schema/ring_buffer.rs`

```rust
//! Fixed-size ring buffer for versioned file storage.

use rkyv::{Archive, Deserialize, Serialize};

/// Fixed-size ring buffer (compile-time size, zero allocation).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RingBuffer<T, const N: usize> {
    items: [Option<T>; N],
    head: u8,  // Next write position
    len: u8,   // Current count (0..=N)
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Create empty ring buffer.
    #[inline]
    pub const fn new() -> Self {
        Self {
            items: [const { None }; N],
            head: 0,
            len: 0,
        }
    }

    /// Push item (evicts oldest if full).
    #[inline]
    pub fn push(&mut self, item: T) {
        self.items[self.head as usize] = Some(item);
        self.head = (self.head + 1) % (N as u8);
        if self.len < N as u8 {
            self.len += 1;
        }
    }

    /// Get most recent item.
    #[inline]
    pub fn current(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let idx = (self.head + (N as u8) - 1) % (N as u8);
        self.items[idx as usize].as_ref()
    }

    /// Get item at index (0 = oldest, len-1 = newest).
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len as usize {
            return None;
        }
        let offset = (self.head + (N as u8) - self.len + index as u8) % (N as u8);
        self.items[offset as usize].as_ref()
    }

    /// Number of items.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Check if empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterate over items (oldest to newest).
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len()).filter_map(move |i| self.get(i))
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_push() {
        let mut buf = RingBuffer::<i32, 3>::new();
        assert_eq!(buf.len(), 0);

        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.current(), Some(&3));
    }

    #[test]
    fn test_ring_buffer_eviction() {
        let mut buf = RingBuffer::<i32, 3>::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);  // Evicts 1

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.get(0), Some(&2));  // Oldest
        assert_eq!(buf.get(1), Some(&3));
        assert_eq!(buf.get(2), Some(&4));  // Newest
        assert_eq!(buf.current(), Some(&4));
    }
}
```

**Update**: `lithos-core/src/schema/mod.rs`
```rust
pub mod ring_buffer;
```

**Verification**:
```bash
cargo test --package lithos-core --lib schema::ring_buffer
```

**Git**:
```bash
git add lithos-core/src/schema/ring_buffer.rs lithos-core/src/schema/mod.rs
git commit -m "feat(schema): add RingBuffer<T, N> for versioned storage"
```

---

### Step 1.4: Add Zstd Compression for rkyv (1 hour)

**Task**: Create rkyv `with` attribute for transparent compression.

**File**: `lithos-core/src/schema/compression.rs`

```rust
//! Zstd compression for rkyv fields.

use rkyv::{
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Deserialize, Fallible, Serialize,
};

/// Zstd compression wrapper for rkyv (level 3).
#[derive(Debug)]
pub struct ZstdCompressed;

const COMPRESSION_LEVEL: i32 = 3;

impl<T> ArchiveWith<T> for ZstdCompressed
where
    T: AsRef<str>,
{
    type Archived = rkyv::string::ArchivedString;
    type Resolver = rkyv::string::StringResolver;

    #[inline]
    unsafe fn resolve_with(
        field: &T,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        let compressed = zstd::encode_all(field.as_ref().as_bytes(), COMPRESSION_LEVEL)
            .expect("zstd compression failed");
        let compressed_str = String::from_utf8(compressed)
            .expect("zstd output not UTF-8");

        rkyv::string::ArchivedString::resolve_from_str(
            &compressed_str,
            pos,
            resolver,
            out,
        );
    }
}

impl<T, S> SerializeWith<T, S> for ZstdCompressed
where
    T: AsRef<str>,
    S: Fallible + ?Sized,
{
    #[inline]
    fn serialize_with(
        field: &T,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        let compressed = zstd::encode_all(field.as_ref().as_bytes(), COMPRESSION_LEVEL)
            .map_err(|_| rkyv::rancor::Error::new("zstd compression failed"))?;

        // Store as base64 to ensure valid UTF-8
        let compressed_str = base64::encode(&compressed);

        ArchivedString::serialize_from_str(&compressed_str, serializer)
    }
}

impl<T, D> DeserializeWith<rkyv::string::ArchivedString, T, D> for ZstdCompressed
where
    T: From<String>,
    D: Fallible + ?Sized,
{
    #[inline]
    fn deserialize_with(
        field: &rkyv::string::ArchivedString,
        deserializer: &mut D,
    ) -> Result<T, D::Error> {
        let compressed_str: String = field.deserialize(deserializer)?;
        let compressed_bytes = base64::decode(&compressed_str)
            .map_err(|_| rkyv::rancor::Error::new("base64 decode failed"))?;

        let decompressed = zstd::decode_all(&compressed_bytes[..])
            .map_err(|_| rkyv::rancor::Error::new("zstd decompression failed"))?;

        let original = String::from_utf8(decompressed)
            .map_err(|_| rkyv::rancor::Error::new("decompressed data not UTF-8"))?;

        Ok(T::from(original))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zstd_roundtrip() {
        let original = "hello world".repeat(100);
        let compressed = zstd::encode_all(original.as_bytes(), COMPRESSION_LEVEL).unwrap();
        let decompressed = zstd::decode_all(&compressed[..]).unwrap();
        let result = String::from_utf8(decompressed).unwrap();

        assert_eq!(original, result);
        assert!(compressed.len() < original.len(), "Compression reduces size");
    }
}
```

**Update**: `lithos-core/src/schema/mod.rs`
```rust
pub mod compression;
```

**Note**: Compression implementation is complex - may need adjustment based on rkyv version.

**Verification**:
```bash
cargo test --package lithos-core --lib schema::compression
```

**Git**:
```bash
git add lithos-core/src/schema/compression.rs lithos-core/src/schema/mod.rs
git commit -m "feat(schema): add zstd compression for rkyv fields"
```

---

### Step 1.5-1.11: Raw File Types + Database Tables (4 hours)

**Due to length, these steps are summarized. See `SCHEMA_REFACTOR_PLAN.md` Phase 1 for full details.**

**Tasks**:
- Create `RawFileVersion`, `RawSchemaFile`, `RawPropertyBankFile` types
- Add `RAW_SCHEMA_FILES` and `RAW_PROPERTY_BANK_FILE` tables to `db_tables.rs`
- Update `Ingestor` to compute hashes
- Update `Command` adapter to save raw files
- Add comprehensive unit tests

**Estimated time**: 4 hours

**Git**: Commit after each type/table addition

---

### Phase 1 Completion Checklist

- [ ] All new types compile without errors
- [ ] All unit tests pass (`cargo test --package lithos-core --lib schema`)
- [ ] Blake3 hashing works correctly
- [ ] Ring buffer eviction works correctly
- [ ] Zstd compression reduces file size
- [ ] Raw files are saved alongside resolved schemas
- [ ] DB size increase is reasonable (<10 MB for 1000 schemas)

**Git**:
```bash
git checkout refactor/schema-file-centric
git merge phase-1-raw-storage
git push origin refactor/schema-file-centric
```

---

## Phase 2: Two-Tier Staleness Detection (6 hours)

**Goal**: Use timestamp fast path + hash slow path for accurate change detection.

**Git strategy**: Work in `phase-2-staleness` branch.

### Step 2.1-2.8: Staleness Detection (6 hours)

**Key tasks**:
1. Add `source_file_hash: Blake3Hash` to `StoredMetadata`
2. Add `created_at: Option<SystemTime>` to `StoredMetadata`
3. Update `partition_by_staleness()` with two-tier approach
4. Implement `diff_raw_files()` helper
5. Add integration tests

**See**: `SCHEMA_REFACTOR_PLAN.md` Phase 2 for detailed tasks.

---

## Phase 3: Event System (6 hours)

**Goal**: Add fine-grained events for observability.

**Git strategy**: Work in `phase-3-events` branch.

### Step 3.1-3.9: Event Types + Handlers (6 hours)

**Key tasks**:
1. Define `SchemaEvent` and `PropertyBankEvent` enums
2. Create `SchemaEventHandler` trait
3. Implement `LoggingHandler`, `MetricsHandler`, `ReactiveHandler`
4. Update `Loader::load()` to emit events
5. Add event testing utilities

**See**: `SCHEMA_REFACTOR_PLAN.md` Phase 3 for detailed tasks.

---

## Phase 4: Incremental Property Resolution (4 hours)

**Goal**: PropertyBank changes trigger property-level re-resolution.

**Git strategy**: Work in `phase-4-incremental` branch.

**See**: `SCHEMA_REFACTOR_PLAN.md` Phase 4 for detailed tasks.

---

## Phase 5: Type-Driven Validation (4 hours)

**Goal**: Raw validation enforces syntax + basic correctness.

**Git strategy**: Work in `phase-5-validation` branch.

### Key Tasks

1. **Add security limits to `RawSchema::validate()`**:
   - File size limit (checked in `Ingestor`)
   - Property count limit
   - Nesting depth limit
   - String length limits
   - Regex safety checks

2. **Move semantic validation to resolution**:
   - Property ref existence → `RefExpander`
   - Schema ref existence → `Extender`
   - Circular inheritance → `Extender`
   - Depth limits → `Resolver`

**See**: `SCHEMA_REFACTOR_RESEARCH.md` section 1 for validation checklist.

---

## Phase 6: Flatten Structure + Remove Wrappers (6 hours)

**Goal**: Simplify module structure, remove boilerplate.

**Git strategy**: Work in `phase-6-flatten` branch.

⚠️ **CRITICAL**: This phase has BREAKING CHANGES. Coordinate with CLI team.

---

### Step 6.1: Flatten `adapter/` Folder (1 hour)

**Commands**:
```bash
cd lithos-core/src/schema

# Move files to schema root
git mv adapter/stored.rs stored.rs
git mv adapter/query.rs db_query.rs
git mv adapter/command.rs db_command.rs
git mv adapter/ingestor.rs ingestor.rs

# Delete adapter folder
rmdir adapter
```

**Update imports in moved files**:
```rust
// In db_query.rs, db_command.rs, ingestor.rs
// Change: use super::super::*
// To:     use super::*
```

**Update imports across codebase**:
```bash
# Find all imports of adapter module
rg "schema::adapter::" --type rust

# Update each file:
# schema::adapter::Query → schema::db_query::Query
# schema::adapter::Command → schema::db_command::Command
# schema::adapter::stored::* → schema::stored::*
```

**Verification**:
```bash
cargo check
cargo test --package lithos-core --lib schema
```

**Git**:
```bash
git add .
git commit -m "refactor(schema): flatten adapter/ folder to root"
```

---

### Step 6.2: Extract Table Definitions (30 min)

**Create**: `lithos-core/src/schema/db_tables.rs`

**Move** table definitions from `db_query.rs`:
```rust
//! Database table definitions for schema storage.

use redb::{MultimapTableDefinition, TableDefinition};

pub(crate) const SCHEMA_BY_ID: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_by_id");
pub(crate) const SCHEMA_ID_BY_NAME: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_id_by_name");
// ... all other tables
```

**Update** `db_query.rs` and `db_command.rs`:
```rust
use super::db_tables::*;
```

**Git**:
```bash
git add lithos-core/src/schema/db_tables.rs
git commit -m "refactor(schema): extract table definitions to db_tables.rs"
```

---

### Step 6.3: Remove Generic Wrappers (2 hours)

**Delete files**:
```bash
git rm lithos-core/src/schema/query.rs      # 810 lines
git rm lithos-core/src/schema/command.rs    # 394 lines
```

**Update** `lithos-core/src/schema/mod.rs`:
```rust
// Remove these lines:
// pub mod query;
// pub mod command;
// pub type Query<Q> = query::Query<Q>;
// pub type Command<C> = command::Command<C>;

// Add these lines:
pub mod db_query;
pub mod db_command;
```

**Update** `lithos-core/src/schema/error.rs` - ensure `From` impls exist:
```rust
impl From<DbError> for SchemaQueryError {
    fn from(error: DbError) -> Self {
        Self::Storage(error)
    }
}

impl From<DbError> for SchemaCommandError {
    fn from(error: DbError) -> Self {
        Self::Storage(error)
    }
}
```

**Find and update all usages**:
```bash
# Find all imports of Query/Command wrappers
rg "schema::Query|schema::Command" --type rust lithos-core/src

# Update each occurrence:
# schema::Query<adapter::Query> → schema::db_query::Query
# schema::Command<adapter::Command> → schema::db_command::Command
```

**Key files to update**:
- `lithos-core/src/application/schema.rs`
- Any integration tests

**Verification**:
```bash
cargo check
cargo test --package lithos-core
```

**Git**:
```bash
git add .
git commit -m "refactor(schema): remove generic Query/Command wrappers (saves 1204 lines)"
```

---

### Step 6.4: Move Orchestration to Loader (2.5 hours)

**Create**: `lithos-core/src/schema/loader.rs`

**Copy content** from `application/schema.rs` and adapt:
```rust
//! Schema loader — orchestrates file → raw → resolved → DB pipeline.

use super::{
    db_query::Query,
    db_command::Command,
    // ... other imports
};

/// Schema loader.
pub struct Loader<'db> {
    query: Query<'db>,     // Concrete type (not generic)
    command: Command<'db>, // Concrete type (not generic)
}

impl Loader<'_> {
    /// Load schemas from filesystem into database.
    pub fn load(&self, ingestor: &Ingestor<'_>) -> Result<Vec<StoredSchema>, LoadError> {
        // Same implementation as SchemaService::load()
    }
}

/// Loader errors.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("ingestion error: {0}")]
    Ingestion(#[from] SchemaIngestionError),

    #[error("domain error: {0}")]
    Domain(#[from] SchemaError),

    #[error("query error: {0}")]
    Query(#[from] SchemaQueryError),

    #[error("command error: {0}")]
    Command(#[from] SchemaCommandError),
}
```

**Update** `lithos-core/src/schema/mod.rs`:
```rust
pub mod loader;
```

**Delete** `lithos-core/src/application/schema.rs`:
```bash
git rm lithos-core/src/application/schema.rs
```

**Update** `lithos-core/src/application/mod.rs`:
```rust
// Remove: pub mod schema;
```

**Update all usages** (likely in CLI):
```rust
// Before:
use lithos_core::application::schema::SchemaService;
let service = SchemaService::new(...);
service.load(...)?;

// After:
use lithos_core::schema::{loader::Loader, db_query, db_command};
let loader = Loader::new(
    db_query::Query::new(&db),
    db_command::Command::new(&db),
);
loader.load(...)?;
```

**Verification**:
```bash
cargo check
cargo test --package lithos-core
cargo test --package lithos-cli  # If CLI tests exist
```

**Git**:
```bash
git add .
git commit -m "refactor(schema): move orchestration to schema/loader.rs"
```

---

### Phase 6 Completion Checklist

- [ ] No `adapter/` folder exists
- [ ] `query.rs` and `command.rs` wrappers deleted (1204 lines removed)
- [ ] `schema/loader.rs` exists with orchestration logic
- [ ] `application/schema.rs` deleted
- [ ] All imports updated across codebase
- [ ] Error conversion works via `From` trait
- [ ] GAT methods still available in `ports::QueryPort`
- [ ] All tests pass (`mise run test`)
- [ ] No clippy warnings (`mise run lint`)

**Git**:
```bash
git checkout refactor/schema-file-centric
git merge phase-6-flatten
```

---

## Phase 7: Remove Aggregate Layer (6 hours)

**Goal**: Delete `Schema` aggregate, use `StoredSchema` as read model.

⚠️ **BREAKING CHANGE** - Coordinate with CLI team before starting.

**Git strategy**: Work in `phase-7-remove-aggregate` branch.

**See**: `SCHEMA_REFACTOR_PLAN.md` Phase 7 for detailed tasks.

---

## Phase 8: Documentation (4 hours)

**Goal**: Document new architecture, remove dead code.

**Git strategy**: Work in `phase-8-docs` branch.

**Tasks**:
1. Update `AGENTS.md` with new architecture rules
2. Update `_bmad-output/project-context.md`
3. Create ADR: "Schema as Read Model"
4. Update rustdoc comments
5. Remove dead code
6. Add architecture diagrams
7. Update `README.md`

**See**: `SCHEMA_REFACTOR_PLAN.md` Phase 8 for detailed tasks.

---

## Final Merge & Deployment

### Pre-Merge Checklist

- [ ] All phases complete
- [ ] All tests pass (`mise run test`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] Code formatted (`mise run fmt`)
- [ ] ADR validated (`mise run adr:validate`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated with breaking changes
- [ ] Migration guide for users (if public API changed)

### Merge to Main

```bash
git checkout main
git merge refactor/schema-file-centric
git push origin main
```

### Post-Merge Verification

```bash
# Full CI verification
mise run ci

# Benchmarks (ensure no regressions)
cargo bench --package lithos-core --bench schema

# Integration tests
mise run test:integration
```

---

## Rollback Procedures

### Rollback Phase 1-5 (Non-Breaking)
```bash
git revert <commit-range>
git push origin main
```

### Rollback Phase 6-8 (Breaking)
```bash
# Create revert branch
git checkout -b revert/schema-refactor main
git revert <phase-6-commit>..<phase-8-commit>

# Restore old files from before refactor
git checkout <pre-refactor-commit> -- lithos-core/src/schema/adapter
git checkout <pre-refactor-commit> -- lithos-core/src/schema/query.rs
git checkout <pre-refactor-commit> -- lithos-core/src/schema/command.rs
git checkout <pre-refactor-commit> -- lithos-core/src/application/schema.rs

# Fix imports
# ... manual fixes needed ...

# Test
cargo test

# Push
git push origin revert/schema-refactor
```

---

## Tracking Progress

**Use this checklist** to track daily progress:

### Daily Standup Template
```markdown
## Date: YYYY-MM-DD

**Phase**: [Current phase]
**Time spent today**: [hours]
**Completed**:
- [ ] Task 1
- [ ] Task 2

**Blocked by**: [Issues/decisions needed]
**Next**: [Tomorrow's tasks]
```

### Weekly Summary Template
```markdown
## Week of: YYYY-MM-DD

**Phases completed**: [List]
**Total time**: [hours]
**Blockers resolved**: [List]
**Tests added**: [count]
**Lines changed**: +[additions] -[deletions]
**Next week goal**: [Phase target]
```

---

## Emergency Contacts

**If you get stuck**:
1. Review `SCHEMA_REFACTOR_PLAN.md` for context
2. Check `SCHEMA_REFACTOR_DECISIONS.md` for rationale
3. Review `SCHEMA_REFACTOR_RESEARCH.md` for technical details
4. Ask on team chat (include phase number + step number)

---

## Success Criteria

**Phase 1-5**: No breaking changes, all tests pass
**Phase 6-7**: Breaking changes documented, migration guide provided
**Phase 8**: Documentation complete, ready for review

**Final success**:
- ✅ 1204+ lines of boilerplate removed
- ✅ Zero-copy performance preserved (GATs)
- ✅ Flat, cohesive module structure
- ✅ File-centric source of truth with versioning
- ✅ Event-driven pipeline for observability
- ✅ All tests pass, no regressions
