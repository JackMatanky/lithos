# Phase 6: Database Layer Implementation - Detailed Plan

**Date:** 2026-02-02
**Author:** Dev Agent
**Status:** READY FOR EXECUTION
**Scope:** Implement zero-copy database operations using redb + rkyv
**Duration:** 22-29 hours (estimated)

---

## 0. Prerequisites & Context

### Completed Work (Phases 3-5)

- ✅ Phase 3: Domain migration to `lithos-core` with co-located errors/events/ports
- ✅ Phase 4: CQRS command/query stubs created in all contexts
- ✅ Phase 5: CLI converted to sync-first, architecture tests created

### Current State

- Database stub exists at `lithos-core/src/db.rs` with API signatures
- Command/Query implementations exist with `todo!()` placeholders
- Port traits defined in each context (`note/ports.rs`, etc.)
- 457 tests passing, zero clippy warnings

### Key Constraints from Proposal

1. **Use Port Traits**: Implement against existing `Command` and `Query` traits in `*ports.rs`
2. **Concrete Database Type**: Use `Database` struct, not trait-based abstraction
3. **Zero-Copy Performance**: Target 5-10x improvement for reads
4. **Sync-First**: No async in core database layer
5. **Co-located Implementation**: CQRS implementations stay in `command.rs` and `query.rs`

---

## 1. Architecture Review & Research Phase (2-3 hours)

### 1.1 Study Official Documentation

**Objective**: Understand rkyv 0.8 and redb 3.1 APIs comprehensively.
**Tasks**:

1. Read rkyv 0.8 official docs:
   - https://docs.rs/rkyv/0.8.14/rkyv/
   - https://rkyv.org/
   - Focus on: `Archive` trait, `access()` vs `access_unchecked()`, format control features
   - Understand rancor error handling (new in 0.8)
2. Read redb 3.1 official docs:
   - https://docs.rs/redb/latest/redb/
   - https://github.com/cberner/redb/blob/master/docs/design.md
   - Focus on: `Value` trait, `AccessGuard` lifetimes, `TableDefinition`, `MultimapTable`
3. Review existing code:
   - Study current domain types for serialization requirements
   - Identify which types need `Archive` derives
   - Map port trait methods to database operations

**Reference Materials**:
- `docs/refs/crates/redb.md` - Starting point, not comprehensive
- `docs/refs/crates/rkyv.md` - Starting point, not comprehensive
- Official documentation MUST be consulted for accuracy

**Deliverable**:
- Architecture decision notes on:
  - rkyv format features (endianness, alignment, pointer width)
  - redb table schema design
  - Lifetime management strategy for `AccessGuard`
  - Validation strategy (when to use `access()` vs `access_unchecked()`)

**Acceptance Criteria**:
- [ ] Understand rkyv 0.8 error handling with `rancor`
- [ ] Understand redb `Value` trait and orphan rules workaround
- [ ] Clear strategy for lifetime management documented
- [ ] Format control decisions documented (PERMANENT for on-disk format)

---

## 2. Domain Type Preparation (3-4 hours)

### 2.1 Add rkyv Derives to Domain Types

**Objective**: Prepare domain aggregates for serialization.
**Tasks**:

1. **Add rkyv derives to Note aggregate** (`lithos-core/src/note/aggregate.rs`):

   ```rust
   use rkyv::{Archive, Serialize, Deserialize};

   #[derive(Archive, Serialize, Deserialize, Debug, Clone)]
   #[rkyv(
       compare(PartialEq),  // Allow comparison with archived form
       derive(Debug),        // Debug for archived type
   )]
   pub struct Note {
       // existing fields
   }
   ```

2. **Add derives to all value objects**:
   - `note/frontmatter.rs` - `Frontmatter`
   - `note/link.rs` - `Link`, `Style`
   - `note/tag.rs` - `Tag`
   - `note/task.rs` - `Task`
   - `note/structure.rs` - `Heading`, `Section`
3. **Repeat for other contexts**:
   - Config: `Config`, `Global`, `Vault`
   - Schema: `Schema`, `Property`, `PropertySpec`
   - Template: `Template`, `Variable`, `Composition`
4. **Handle UUID serialization**:
   ```rust
   // UUID is [u8; 16] internally - already rkyv-compatible
   // But verify with test
   ```
5. **Handle String/Vec serialization**:
   - rkyv handles `String` → `ArchivedString` automatically
   - rkyv handles `Vec<T>` → `ArchivedVec<T>` automatically
     **Testing**:

```rust
#[cfg(test)]
mod serialization_tests {
    use super::*;
    use rkyv::{to_bytes, access};

    #[test]
    fn note_roundtrip() {
        let note = Note::new(/* ... */);
        let bytes = to_bytes::<rkyv::rancor::Error>(&note).unwrap();
        let archived = access::<ArchivedNote, rkyv::rancor::Error>(&bytes).unwrap();
        assert_eq!(archived.id, note.id);
    }
}
```

**Acceptance Criteria**:

- [ ] All domain aggregates have `Archive`, `Serialize`, `Deserialize` derives
- [ ] Roundtrip tests pass for each aggregate
- [ ] No compilation errors
- [ ] Existing 241 unit tests still pass

---

## 3. Database Core Implementation (6-8 hours)

### 3.1 Implement redb Value Wrapper for rkyv

**Objective**: Bridge redb's `Value` trait with rkyv serialization.
**Challenge**: Orphan rules prevent implementing `redb::Value` for external types.
**Solution**: Create newtype wrapper (per `redb.md` guidance).

```rust
// lithos-core/src/db.rs
use redb::{Value, TypeName};
use rkyv::{Archive, Serialize, Deserialize};
use std::marker::PhantomData;
/// Newtype wrapper for rkyv-serialized values stored in redb
///
/// This wrapper allows us to implement redb::Value for any rkyv-serializable type
/// without violating orphan rules.
pub struct RkyvValue<T> {
    _phantom: PhantomData<T>,
}
impl<T> Value for RkyvValue<T>
where
    T: Archive,
{
    type SelfType<'a> = &'a [u8] where Self: 'a;
    type AsBytes<'a> = &'a [u8] where Self: 'a;
    fn fixed_width() -> Option<usize> {
        None // rkyv produces variable-width output
    }
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> {
        // Return raw bytes - validation happens in get_archived()
        data
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        value
    }
    fn type_name() -> TypeName {
        TypeName::new(&format!("RkyvValue<{}>", std::any::type_name::<T>()))
    }
}
```

### 3.2 Implement Database Struct with Configuration

**Objective**: Create main `Database` type with builder pattern.

```rust
use redb::{Database as RedbDatabase, Builder, Durability};
use std::path::Path;
pub struct Database {
    inner: RedbDatabase,
}
impl Database {
    /// Create a new database with default configuration
    pub fn create(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let inner = RedbDatabase::create(path.as_ref())
            .map_err(|e| DbError::Open(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Open existing database or create if not exists
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        Self::create(path)
    }

    /// Create with custom configuration (for advanced use)
    pub fn builder() -> DatabaseBuilder {
        DatabaseBuilder::new()
    }
}
/// Builder for Database with configuration options
pub struct DatabaseBuilder {
    builder: Builder,
}
impl DatabaseBuilder {
    pub fn new() -> Self {
        Self {
            builder: Builder::new(),
        }
    }

    /// Set cache size in bytes (per ADR 0002)
    pub fn cache_size(mut self, bytes: usize) -> Self {
        self.builder.set_cache_size(bytes);
        self
    }

    /// Set page size in bytes (per ADR 0002)
    pub fn page_size(mut self, bytes: usize) -> Self {
        self.builder.set_page_size(bytes);
        self
    }

    pub fn create(self, path: impl AsRef<Path>) -> Result<Database, DbError> {
        let inner = self.builder.create(path.as_ref())
            .map_err(|e| DbError::Open(e.to_string()))?;
        Ok(Database { inner })
    }
}
```

### 3.3 Implement Table Definitions

**Objective**: Define compile-time table schemas.

```rust
use redb::TableDefinition;
// Standard tables (key-value)
const NOTES_TABLE: TableDefinition<&str, RkyvValue<Note>> =
    TableDefinition::new("notes");
const SCHEMAS_TABLE: TableDefinition<&str, RkyvValue<Schema>> =
    TableDefinition::new("schemas");
const TEMPLATES_TABLE: TableDefinition<&str, RkyvValue<Template>> =
    TableDefinition::new("templates");
const CONFIG_TABLE: TableDefinition<&str, RkyvValue<Config>> =
    TableDefinition::new("config");
// Multimap tables (1:N indexes)
use redb::MultimapTableDefinition;
const TAGS_TO_NOTES: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("tags_to_notes");
const BACKLINKS: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("backlinks");
const PATH_TO_NOTE_ID: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("path_to_note_id");
```

**Note**: Table names are string constants for now. Could be enhanced with type-safe table registry later.

### 3.4 Implement Serialization Helpers

**Objective**: Centralize rkyv serialization/deserialization logic.

```rust
/// Serialize a value to bytes using rkyv
fn serialize_value<T>(value: &T) -> Result<Vec<u8>, DbError>
where
    T: for<'a> Serialize<
        rkyv::rancor::Strategy<
            rkyv::ser::Serializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::BoxedError,
            >,
            rkyv::rancor::BoxedError,
        >,
    >,
{
    rkyv::to_bytes::<rkyv::rancor::Error>(value)
        .map(|bytes| bytes.into_vec())
        .map_err(|e| DbError::Serialization(e.to_string()))
}
/// Deserialize bytes to a value using rkyv
fn deserialize_value<T>(bytes: &[u8]) -> Result<T, DbError>
where
    T: Archive,
    T::Archived: for<'a> Deserialize<T, rkyv::rancor::Strategy<
        rkyv::de::Pool,
        rkyv::rancor::BoxedError,
    >>,
{
    let archived = rkyv::access::<T, rkyv::rancor::Error>(bytes)
        .map_err(|e| DbError::Deserialization(e.to_string()))?;

    rkyv::deserialize::<T, rkyv::rancor::Error>(archived)
        .map_err(|e| DbError::Deserialization(e.to_string()))
}
```

**Note**: The exact trait bounds will need to be refined based on rkyv 0.8 actual API after consulting official docs.
**Acceptance Criteria**:

- [ ] `Database` struct compiles
- [ ] Can create database file
- [ ] Table definitions work
- [ ] Serialization helpers work for domain types
- [ ] Basic integration test passes

---

## 4. Zero-Copy Read Implementation (5-6 hours)

### 4.1 Challenge: Lifetime Management

**Problem**: From proposal example, `ArchivedGuard` needs to hold both transaction and `AccessGuard`, but their lifetimes conflict.
**Research Required**:

1. Check if redb `ReadTransaction` can be stored and kept alive
2. Investigate if `AccessGuard` can outlive transaction with `'static` lifetime hack
3. Consider alternative: closure-based API
   **Option A: Try to Return Guard** (attempt first):

```rust
pub struct ArchivedGuard<'a, V> {
    _txn: ReadTransaction,  // Owns transaction
    guard: AccessGuard<'a>,  // Borrows from transaction
    _phantom: PhantomData<V>,
}
```

**Option B: Closure-Based API** (fallback if Option A fails):

```rust
pub fn with_archived<K, V, F, R>(
    &self,
    table: TableDefinition<K, RkyvValue<V>>,
    key: K::SelfType<'_>,
    f: F,
) -> Result<Option<R>, DbError>
where
    K: redb::Key,
    V: Archive,
    F: FnOnce(&rkyv::Archived<V>) -> R,
{
    let txn = self.inner.begin_read()?;
    let table = txn.open_table(table)?;

    if let Some(guard) = table.get(key)? {
        let bytes = guard.value();

        // Validate and access archived data
        let archived = rkyv::access::<V, rkyv::rancor::Error>(bytes)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;

        Ok(Some(f(archived)))
    } else {
        Ok(None)
    }
}
```

**Decision Point**: Try Option A first (research official docs). If lifetimes are impossible, use Option B.

### 4.2 Implement get() - Full Deserialization

**Objective**: Implement full deserialization for mutation paths.

```rust
impl Database {
    /// Get a value by key with full deserialization (cold path)
    pub fn get<K, V>(
        &self,
        table: TableDefinition<K, RkyvValue<V>>,
        key: K::SelfType<'_>,
    ) -> Result<Option<V>, DbError>
    where
        K: redb::Key,
        V: Archive,
        V::Archived: for<'a> Deserialize<V, rkyv::rancor::Strategy<
            rkyv::de::Pool,
            rkyv::rancor::BoxedError,
        >>,
    {
        let txn = self.inner.begin_read()?;
        let table = txn.open_table(table)?;

        match table.get(key)? {
            Some(guard) => {
                let bytes = guard.value();
                let value = deserialize_value::<V>(bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn get_returns_none_for_missing_key() {
        let temp = NamedTempFile::new().unwrap();
        let db = Database::open(temp.path()).unwrap();

        let result: Option<Note> = db.get(NOTES_TABLE, "missing").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn get_roundtrip() {
        let temp = NamedTempFile::new().unwrap();
        let db = Database::open(temp.path()).unwrap();

        let note = Note::new(Uuid::now_v7(), "test.md".to_string()).unwrap();
        let note_id = note.id.to_string();

        // Will implement put() next
        db.put(NOTES_TABLE, note_id.as_str(), &note).unwrap();

        let retrieved = db.get(NOTES_TABLE, note_id.as_str()).unwrap().unwrap();
        assert_eq!(retrieved.id, note.id);
    }
}
```

**Acceptance Criteria**:

- [ ] `get()` compiles and works
- [ ] Returns `None` for missing keys
- [ ] Roundtrip test passes
- [ ] No memory leaks (check with Valgrind if needed)

---

## 5. Write Operations Implementation (4-5 hours)

### 5.1 Implement put() - Standard Write

**Objective**: Basic write operation with rkyv serialization.

```rust
impl Database {
    /// Insert or update a value (convenience method)
    pub fn put<K, V>(
        &self,
        table: TableDefinition<K, RkyvValue<V>>,
        key: K::SelfType<'_>,
        value: &V,
    ) -> Result<(), DbError>
    where
        K: redb::Key,
        V: for<'a> Serialize<
            rkyv::rancor::Strategy<
                rkyv::ser::Serializer<
                    rkyv::util::AlignedVec,
                    rkyv::ser::allocator::ArenaHandle<'a>,
                    rkyv::rancor::BoxedError,
                >,
                rkyv::rancor::BoxedError,
            >,
        >,
    {
        let bytes = serialize_value(value)?;

        let txn = self.inner.begin_write()?;
        {
            let mut tbl = txn.open_table(table)?;
            tbl.insert(key, bytes.as_slice())?;
        }
        txn.commit()?;

        Ok(())
    }

    /// Delete a value by key
    pub fn delete<K, V>(
        &self,
        table: TableDefinition<K, RkyvValue<V>>,
        key: K::SelfType<'_>,
    ) -> Result<bool, DbError>
    where
        K: redb::Key,
        V: Archive,
    {
        let txn = self.inner.begin_write()?;
        {
            let mut tbl = txn.open_table(table)?;
            let existed = tbl.remove(key)?.is_some();
            txn.commit()?;
            Ok(existed)
        }
    }
}
```

### 5.2 Implement put_reserve() - Zero-Copy Write (Deferred)

**Status**: DEFERRED to future optimization phase.
**Reason**: Complex API, requires careful buffer management. MVP can use `put()` which allocates a temp buffer but is simpler and still fast enough.
**Future Implementation**:

```rust
// Phase 7 or later
pub fn put_reserve<K, V, F>(
    &self,
    table: TableDefinition<K, RkyvValue<V>>,
    key: K::SelfType<'_>,
    value_size: u32,
    write_fn: F,
) -> Result<(), DbError>
where
    K: redb::Key,
    V: Archive,
    F: FnOnce(&mut [u8]) -> Result<(), DbError>,
{
    // Use table.insert_reserve()
    // Write directly to DB page
    // Per redb.md: insert_reserve takes u32 size
    todo!("Implement zero-copy write in future phase")
}
```

**Testing**:

```rust
#[test]
fn put_and_get_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    let db = Database::open(temp.path()).unwrap();

    let note = Note::new(Uuid::now_v7(), "test.md".to_string()).unwrap();
    let id_str = note.id.to_string();

    db.put(NOTES_TABLE, id_str.as_str(), &note).unwrap();
    let retrieved = db.get(NOTES_TABLE, id_str.as_str()).unwrap().unwrap();

    assert_eq!(retrieved.id, note.id);
    assert_eq!(retrieved.path, note.path);
}
#[test]
fn delete_existing_returns_true() {
    let temp = NamedTempFile::new().unwrap();
    let db = Database::open(temp.path()).unwrap();

    let note = Note::new(Uuid::now_v7(), "test.md".to_string()).unwrap();
    let id_str = note.id.to_string();

    db.put(NOTES_TABLE, id_str.as_str(), &note).unwrap();
    let deleted = db.delete(NOTES_TABLE, id_str.as_str()).unwrap();

    assert!(deleted);
    assert!(db.get::<_, Note>(NOTES_TABLE, id_str.as_str()).unwrap().is_none());
}
```

**Acceptance Criteria**:

- [ ] `put()` works correctly
- [ ] `delete()` works correctly
- [ ] Roundtrip tests pass
- [ ] Concurrent read/write test passes

---

## 6. Multimap Support for Indexes (4-5 hours)

### 6.1 Implement Multimap Insert

**Objective**: Support 1:N relationships (tags→notes, backlinks).
**Note**: From `redb.md`: `MultimapTable` requires both K and V to implement `Key` trait (not `Value`).

```rust
impl Database {
    /// Insert a value into a multimap (1:N relationship)
    pub fn multimap_insert<K, V>(
        &self,
        table: MultimapTableDefinition<K, V>,
        key: K::SelfType<'_>,
        value: V::SelfType<'_>,
    ) -> Result<(), DbError>
    where
        K: redb::Key,
        V: redb::Key,
    {
        let txn = self.inner.begin_write()?;
        {
            let mut tbl = txn.open_multimap_table(table)?;
            tbl.insert(key, value)?;
        }
        txn.commit()?;
        Ok(())
    }

    /// Remove a specific value from a multimap
    pub fn multimap_remove<K, V>(
        &self,
        table: MultimapTableDefinition<K, V>,
        key: K::SelfType<'_>,
        value: V::SelfType<'_>,
    ) -> Result<bool, DbError>
    where
        K: redb::Key,
        V: redb::Key,
    {
        let txn = self.inner.begin_write()?;
        {
            let mut tbl = txn.open_multimap_table(table)?;
            let removed = tbl.remove(key, value)?;
            txn.commit()?;
            Ok(removed)
        }
    }
}
```

### 6.2 Implement Multimap Query

**Objective**: Query all values for a key.
**Challenge**: Similar lifetime issue as `get_archived()`. Use closure-based API.

```rust
impl Database {
    /// Get all values for a key in a multimap
    ///
    /// Uses closure to avoid lifetime issues with iterator
    pub fn multimap_get<K, V, F, R>(
        &self,
        table: MultimapTableDefinition<K, V>,
        key: K::SelfType<'_>,
        f: F,
    ) -> Result<R, DbError>
    where
        K: redb::Key,
        V: redb::Key + 'static,
        F: FnOnce(Vec<V::SelfType<'static>>) -> R,
    {
        let txn = self.inner.begin_read()?;
        let tbl = txn.open_multimap_table(table)?;

        let mut values = Vec::new();
        if let Some(iter) = tbl.get(key)? {
            for result in iter {
                let value_guard = result?;
                let value = value_guard.value();
                // Need to copy/clone value here to escape transaction lifetime
                // For &str keys, this means allocation
                values.push(value);
            }
        }

        Ok(f(values))
    }
}
```

**Alternative**: If `&str` → `String` allocation is acceptable, collect into `Vec<String>`:

```rust
pub fn multimap_get_strings(
    &self,
    table: MultimapTableDefinition<&str, &str>,
    key: &str,
) -> Result<Vec<String>, DbError> {
    self.multimap_get(table, key, |values| {
        values.into_iter().map(|s| s.to_string()).collect()
    })
}
```

**Testing**:

```rust
#[test]
fn multimap_insert_and_query() {
    let temp = NamedTempFile::new().unwrap();
    let db = Database::open(temp.path()).unwrap();

    // Insert multiple notes for same tag
    db.multimap_insert(TAGS_TO_NOTES, "rust", "note-1").unwrap();
    db.multimap_insert(TAGS_TO_NOTES, "rust", "note-2").unwrap();
    db.multimap_insert(TAGS_TO_NOTES, "rust", "note-3").unwrap();

    // Query all notes with "rust" tag
    let note_ids = db.multimap_get_strings(TAGS_TO_NOTES, "rust").unwrap();

    assert_eq!(note_ids.len(), 3);
    assert!(note_ids.contains(&"note-1".to_string()));
    assert!(note_ids.contains(&"note-2".to_string()));
    assert!(note_ids.contains(&"note-3".to_string()));
}
#[test]
fn multimap_remove_works() {
    let temp = NamedTempFile::new().unwrap();
    let db = Database::open(temp.path()).unwrap();

    db.multimap_insert(TAGS_TO_NOTES, "rust", "note-1").unwrap();
    db.multimap_insert(TAGS_TO_NOTES, "rust", "note-2").unwrap();

    let removed = db.multimap_remove(TAGS_TO_NOTES, "rust", "note-1").unwrap();
    assert!(removed);

    let note_ids = db.multimap_get_strings(TAGS_TO_NOTES, "rust").unwrap();
    assert_eq!(note_ids.len(), 1);
    assert!(note_ids.contains(&"note-2".to_string()));
}
```

**Acceptance Criteria**:

- [ ] Can insert multiple values for same key
- [ ] Can query all values for a key
- [ ] Can remove specific value from multimap
- [ ] Tests pass

---

## 7. Batch Operations Implementation (2-3 hours)

### 7.1 Implement batch_write()

**Objective**: Bulk writes with single fsync per ADR 0002.
**From `redb.md`**: Use `Durability::None` for batch, then final commit with `Durability::Immediate`.

```rust
impl Database {
    /// Execute multiple writes in a single transaction with deferred fsync
    ///
    /// Per ADR 0002: Use Durability::None for performance, then final fsync
    pub fn batch_write<F>(&self, f: F) -> Result<(), DbError>
    where
        F: FnOnce(&mut WriteTransaction) -> Result<(), DbError>,
    {
        let mut txn = self.inner.begin_write()?;
        txn.set_durability(Durability::None)?;

        f(&mut txn)?;

        txn.commit()?;

        // Final fsync for durability
        let mut final_txn = self.inner.begin_write()?;
        final_txn.set_durability(Durability::Immediate)?;
        final_txn.commit()?;

        Ok(())
    }
}
```

**Usage Pattern**:

```rust
// In Note CQRS implementation
pub fn bulk_index_notes(db: &Database, notes: &[Note]) -> Result<(), NoteError> {
    db.batch_write(|txn| {
        let mut table = txn.open_table(NOTES_TABLE)?;

        for note in notes {
            let id_str = note.id.to_string();
            let bytes = serialize_value(note)?;
            table.insert(id_str.as_str(), bytes.as_slice())?;
        }

        Ok(())
    })?;

    Ok(())
}
```

**Testing**:

```rust
#[test]
fn batch_write_performance() {
    let temp = NamedTempFile::new().unwrap();
    let db = Database::open(temp.path()).unwrap();

    // Create 1000 test notes
    let notes: Vec<Note> = (0..1000)
        .map(|i| Note::new(Uuid::now_v7(), format!("note-{}.md", i)).unwrap())
        .collect();

    let start = std::time::Instant::now();

    db.batch_write(|txn| {
        let mut table = txn.open_table(NOTES_TABLE)?;

        for note in &notes {
            let id_str = note.id.to_string();
            let bytes = serialize_value(note)?;
            table.insert(id_str.as_str(), bytes.as_slice())?;
        }

        Ok(())
    }).unwrap();

    let duration = start.elapsed();
    println!("Batch insert of 1000 notes: {:?}", duration);

    // Should be < 2 seconds per proposal metrics
    assert!(duration.as_secs() < 2);

    // Verify all inserted
    for note in &notes {
        let id_str = note.id.to_string();
        let retrieved = db.get::<_, Note>(NOTES_TABLE, id_str.as_str()).unwrap();
        assert!(retrieved.is_some());
    }
}
```

**Acceptance Criteria**:

- [ ] Batch write works
- [ ] Performance test shows < 2s for 1000 notes
- [ ] Data is durable after final fsync
- [ ] Rollback works on error

---

## 8. Connect to Port Trait Implementations (6-8 hours)

### 8.1 Implement Note Command Trait

**Objective**: Implement `note::ports::Command` trait for database.
**Current State** (from Phase 4):

- `note/command.rs` has `NoteCommand<'db>` struct
- Methods are stubbed with `todo!()`
- Trait is defined in `note/ports.rs`
  **Strategy**: Implement trait by delegating to `Database` methods.
  **File**: `lithos-core/src/note/command.rs`

```rust
// Replace todo!() implementations
impl<'db> NoteCommand<'db> {
    pub fn create(&self, path: String) -> Result<Note, NoteError> {
        // Create new note aggregate
        let note = Note::new(Uuid::now_v7(), path)?;

        // Save to database
        let id_str = note.id.to_string();
        self.db.put(NOTES_TABLE, id_str.as_str(), &note)
            .map_err(|e| NoteError::Storage(e.to_string()))?;

        // Update path index
        self.db.multimap_insert(PATH_TO_NOTE_ID, note.path.as_str(), id_str.as_str())
            .map_err(|e| NoteError::Storage(e.to_string()))?;

        Ok(note)
    }

    pub fn delete(&self, id: Uuid) -> Result<(), NoteError> {
        let id_str = id.to_string();

        // Get note first to clean up indexes
        if let Some(note) = self.db.get::<_, Note>(NOTES_TABLE, id_str.as_str())
            .map_err(|e| NoteError::Storage(e.to_string()))?
        {
            // Remove from path index
            self.db.multimap_remove(PATH_TO_NOTE_ID, note.path.as_str(), id_str.as_str())
                .map_err(|e| NoteError::Storage(e.to_string()))?;

            // Remove from tag indexes
            for tag in &note.tags {
                self.db.multimap_remove(TAGS_TO_NOTES, tag.as_str(), id_str.as_str())
                    .map_err(|e| NoteError::Storage(e.to_string()))?;
            }

            // Delete note
            self.db.delete(NOTES_TABLE, id_str.as_str())
                .map_err(|e| NoteError::Storage(e.to_string()))?;
        }

        Ok(())
    }

    pub fn update(&self, note: Note) -> Result<Note, NoteError> {
        let id_str = note.id.to_string();

        // Get old note to update indexes
        let old_note = self.db.get::<_, Note>(NOTES_TABLE, id_str.as_str())
            .map_err(|e| NoteError::Storage(e.to_string()))?;

        if let Some(old) = old_note {
            // Update path index if changed
            if old.path != note.path {
                self.db.multimap_remove(PATH_TO_NOTE_ID, old.path.as_str(), id_str.as_str())
                    .map_err(|e| NoteError::Storage(e.to_string()))?;
                self.db.multimap_insert(PATH_TO_NOTE_ID, note.path.as_str(), id_str.as_str())
                    .map_err(|e| NoteError::Storage(e.to_string()))?;
            }

            // Update tag indexes (delta)
            let old_tags: std::collections::HashSet<_> = old.tags.iter().collect();
            let new_tags: std::collections::HashSet<_> = note.tags.iter().collect();

            for tag in old_tags.difference(&new_tags) {
                self.db.multimap_remove(TAGS_TO_NOTES, tag.as_str(), id_str.as_str())
                    .map_err(|e| NoteError::Storage(e.to_string()))?;
            }

            for tag in new_tags.difference(&old_tags) {
                self.db.multimap_insert(TAGS_TO_NOTES, tag.as_str(), id_str.as_str())
                    .map_err(|e| NoteError::Storage(e.to_string()))?;
            }
        }

        // Update note
        self.db.put(NOTES_TABLE, id_str.as_str(), &note)
            .map_err(|e| NoteError::Storage(e.to_string()))?;

        Ok(note)
    }
}
// Implement the Command trait from ports.rs
impl<'db> super::ports::Command for NoteCommand<'db> {
    fn create(&self, path: String) -> Result<Note, NoteError> {
        self.create(path)
    }

    fn delete(&self, id: Uuid) -> Result<(), NoteError> {
        self.delete(id)
    }

    fn update(&self, note: Note) -> Result<Note, NoteError> {
        self.update(note)
    }
}
```

### 8.2 Implement Note Query Trait

**File**: `lithos-core/src/note/query.rs`

```rust
impl<'db> NoteQuery<'db> {
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError> {
        let id_str = id.to_string();
        self.db.get(NOTES_TABLE, id_str.as_str())
            .map_err(|e| NoteError::Storage(e.to_string()))
    }

    pub fn find_by_path(&self, path: &str) -> Result<Option<Note>, NoteError> {
        // Use path index to find ID
        let note_id = self.db.multimap_get_strings(PATH_TO_NOTE_ID, path)
            .map_err(|e| N            .map_err(|e| NoteError::Storage(e.to_string()))?;

        if let Some(id_str) = note_id.first() {
            self.db.get(NOTES_TABLE, id_str.as_str())
                .map_err(|e| NoteError::Storage(e.to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn list(&self) -> Result<Vec<Note>, NoteError> {
        // Range scan implementation needed in Database first
        // For MVP, could use multimap of "all_notes" -> id
        todo!("Implement full table scan in Database")
    }
}

// Implement the Query trait from ports.rs
impl<'db> super::ports::Query for NoteQuery<'db> {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError> {
        self.find_by_id(id)
    }

    fn find_by_path(&self, path: &str) -> Result<Option<Note>, NoteError> {
        self.find_by_path(path)
    }

    fn list(&self) -> Result<Vec<Note>, NoteError> {
        self.list()
    }
}
```

### 8.3 Implement Config/Schema/Template CQRS

**Objective**: Similar implementation for other contexts.

**Config**:

- `save_global` → `db.put(CONFIG_TABLE, "global", &config)`
- `save_vault` → `db.put(CONFIG_TABLE, "vault", &config)`
- `load_global` → `db.get(CONFIG_TABLE, "global")`

**Schema**:

- `save` → `db.put(SCHEMAS_TABLE, schema.name, &schema)`
- `delete` → `db.delete(SCHEMAS_TABLE, schema.name)`
- Indexes: `name_to_id` (Schema is identified by name, but has UUID)

**Template**:

- Similar to Note (id-based, with name index)

**Acceptance Criteria**:

- [ ] All `todo!()` macros replaced with implementation
- [ ] Port traits fully implemented
- [ ] CRUD operations work end-to-end
- [ ] Indexes are maintained consistently

---

## 9. Performance Testing & Benchmarks (2-3 hours)

### 9.1 Create Benchmarks

**Objective**: Validate zero-copy performance gains.

**File**: `lithos-core/benches/zero_copy_bench.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lithos_core::db::Database;
use lithos_core::note::aggregate::Note;

fn bench_read(c: &mut Criterion) {
    // Setup DB with 1000 notes
    // ...

    c.bench_function("zero_copy_read", |b| {
        b.iter(|| {
            // Use with_archived() or closure API
            db.with_archived(NOTES_TABLE, "note-1", |archived| {
                black_box(archived.id);
            }).unwrap();
        })
    });

    c.bench_function("full_deserialize", |b| {
        b.iter(|| {
            // Use get() which deserializes
            db.get(NOTES_TABLE, "note-1").unwrap();
        })
    });
}
```

### 9.2 Validate Metrics

**Targets**:

- Read: 5-10x improvement (zero-copy vs deserialize)
- Write: < 2s for 1000 notes (batch)

**Deliverable**: Benchmark report in `docs/benchmarks/phase6-db.md`

---

## 10. Final Verification (1 hour)

### 10.1 Run All Tests

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo bench
```

### 10.2 Architecture Check

Verify no leaks:

- Domain types don't expose rkyv types publicly (unless intended)
- Database internals don't leak into domain logic
- Transaction lifetimes are contained

**Completion Criteria**:

- All Phase 6 tasks complete
- CI Green
- Benchmarks prove performance
