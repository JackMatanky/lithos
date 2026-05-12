# PRD: Refactor DB Module for Maintainability and Type Safety

## Problem Statement

The current `db/` module (reader.rs: 1511+ lines, writer.rs: 1287 lines) has become rigid and hard to maintain due to:

1. **Type coupling**: `TableDefinition<&str, &[u8]>` hardcoded everywhere forces UUID→String conversions that allocate unnecessarily
2. **Method explosion**: Every operation has `_by_uuid` variants, leading to ~30+ methods with duplicated logic
3. **Unclear abstraction level**: The module tries to be a thin redb wrapper, a domain-agnostic persistence layer, and hidden behind Repository traits simultaneously
4. **Poor readability**: Over-modularization in some areas (helper functions) and under-modularization in others (reader/writer split doesn't map to meaningful concepts)
5. **Leaky abstractions**: rkyv trait bounds appear in every public method signature, exposing implementation details

The module was originally designed for "Option A" (hide redb/rkyv details completely) but this proved incompatible with Rust's type system, leaving it in an awkward middle ground that provides neither the flexibility of exposing redb directly nor the simplicity of complete abstraction.

## Solution

Redesign the db module as a **library of composable building blocks** rather than a unified interface:

1. **Expose redb more directly** to storage adapters while providing safety-enforcing helpers
2. **Use native UUID keys** via `UuidV7DbType` trait, eliminating allocation-heavy conversions
3. **Provide newtype wrappers** (`UuidTable`, `UuidMultimap`, `PathTable`) for common table patterns
4. **Centralize rkyv safety patterns** in `pub(crate)` helpers that enforce alignment and validation
5. **Move transaction logic** to storage adapters where it's clearer and more testable
6. **Create storage/ submodules** in each context to enforce domain/infrastructure boundaries

This shifts the db module from "shallow with complex interface" to "deep with simple interface" — small public API (Store + table wrappers + helpers) that hides significant complexity (rkyv alignment, validation, transaction scoping, AccessGuard lifetime management).

## User Stories

As a **developer implementing a Repository adapter**, I want to:

1. Use native domain ID types (e.g., `SchemaId`) as database keys without string conversion, so that I avoid unnecessary allocations
2. See clear transaction boundaries in storage adapter code, so that I understand when data is committed
3. Access rkyv serialization helpers that enforce safety invariants, so that I don't accidentally create alignment or validation bugs
4. Write storage adapter code that is structured and navigable for contexts with 4-10 tables, so that it remains maintainable even when total lines are larger
5. Use table definition wrappers that encode common patterns (UUID keys, multimap relationships), so that I don't repeat boilerplate
6. Test storage adapters in isolation with different Store instances, so that tests are fast and independent
7. See explicit error handling at the redb level, so that I can distinguish database errors from serialization errors
8. Understand the full data flow from domain type → bytes → redb, so that performance optimization is tractable
9. Batch multiple writes in a single transaction using `store.write()` closure, so that bulk operations are efficient
10. Have automatic transaction commit/rollback based on Result, so that I don't forget to commit or handle errors
11. Work with multiple tables within one transaction, so that related data changes are atomic

As a **developer maintaining the db module**, I want to:

9. Have safety-critical rkyv patterns centralized in one place (`db/rkyv.rs`), so that updates to rkyv don't require changing 30+ method signatures
10. Provide building blocks that eliminate repetition across storage adapters, so that the module earns its keep through leverage
11. Keep the public API surface small (<10 exported types), so that the module is navigable
12. Have clear documentation on alignment requirements and validation boundaries, so that contributors don't introduce unsafe patterns
13. Avoid leaking rkyv trait bounds into public interfaces, so that domain code never sees Archive/Serialize/Deserialize details

As a **developer working on domain logic**, I want to:

14. Define Repository traits that express domain operations (get, save, list, delete), so that business logic is decoupled from persistence
15. Never see redb or rkyv types in domain code, so that the domain stays pure
16. Have storage adapters live in a separate `storage/` submodule, so that the domain/infrastructure boundary is enforced by module visibility
17. Test domain logic with `FakeStorage` implementations that use HashMap, so that tests don't touch the filesystem
18. Understand that the database is a rebuildable projection, so that data integrity is guaranteed by file-backed source truth

As a **developer optimizing performance**, I want to:

19. See where allocations happen (serialization, deserialization, key conversion), so that I can profile hotspots
20. Use zero-copy reads via closure-based patterns for hot paths, so that LSP queries are fast
21. Use owned deserialization for cold paths where convenience matters, so that code is ergonomic
22. Batch write operations within a single transaction, so that bulk operations are efficient
23. Use native UUID keys that implement `redb::Key` directly, so that key comparisons don't allocate

As a **developer adding a new context** (e.g., template):

27. Create a `repository.rs` file defining the Repository trait for the context, so that the port is clearly separated
28. Copy the pattern from `schema/storage/read.rs` + `schema/storage/write.rs` + `schema/storage/tables.rs`, so that storage adapters are consistent
29. Define table constants using `UuidTable` wrappers, so that type safety is enforced
30. Implement the context's Repository trait using db building blocks with operation-focused internal modules, so that complexity stays localized by concern instead of accumulating in one file
31. Use `impl_redb_uuid!` macro on the context's ID type, so that it works as a redb key
32. Write integration tests for the storage adapter using a temporary database, so that persistence logic is verified
33. Implement batch operations (`save_many`, `delete_many`) using `store.write()` for efficiency, so that bulk operations perform well

## Implementation Decisions

### Module Structure

The db module will be restructured as:

```
lithos-core/src/db/
├── mod.rs           # Store + public API exports
├── error.rs         # DbError + DbErrorKind + Result type alias
├── table.rs         # UuidTable, UuidMultimap, PathTable, Table wrappers
├── read.rs          # ReadTx newtype + read helpers
├── write.rs         # WriteTx newtype + write helpers
├── rkyv.rs          # pub(crate) serialization helpers (serialize, deserialize, with_archived)
├── uuid.rs          # UuidV7DbType trait + impl_redb_uuid macro (keep existing)
└── retry.rs         # Keep existing retry logic
```

**Remove**: reader.rs (1511+ lines), writer.rs (1287 lines) — their logic moves to storage adapters.

### Core Types

**Store** — Database handle with transaction methods:
```rust
pub struct Store {
    inner: redb::Database,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self>;

    /// Execute read-only operations within a transaction.
    pub fn read<R>(&self, f: impl FnOnce(&ReadTx) -> Result<R>) -> Result<R>;

    /// Execute read-write operations within a transaction (auto-commits on Ok).
    pub fn write<R>(&self, f: impl FnOnce(&mut WriteTx) -> Result<R>) -> Result<R>;
}
```

**Key insight**: `Store::write()` automatically commits the transaction on `Ok(_)` and rolls back on `Err(_)`. This pattern enables:
- **Batch writes**: Multiple operations within one closure = one transaction
- **Atomic updates**: Related changes (e.g., schema + properties) commit together
- **Automatic rollback**: Any error aborts the entire transaction

**ReadTx / WriteTx** — Transaction newtypes that scope lifetimes:
```rust
pub struct ReadTx {
    inner: redb::ReadTransaction,
}

pub struct WriteTx {
    inner: redb::WriteTransaction,
}
```

These are intentionally thin — storage adapters access `.inner` directly. May gain helper methods during implementation if duplication emerges.

**Table Wrappers** — Newtype wrappers for common patterns:
```rust
pub struct UuidTable<K: UuidV7DbType, V: Value> { definition: TableDefinition<K, V> }
pub struct UuidMultimap<K: UuidV7DbType, V: Value> { definition: MultimapTableDefinition<K, V> }
pub struct PathTable<V: Value> { definition: TableDefinition<&'static str, V> }
pub struct Table<K: Key, V: Value> { definition: TableDefinition<K, V> }
```

Each provides:
- `const fn new(name: &'static str) -> Self`
- `const fn definition(&self) -> TableDefinition<K, V>` (for tx.open_table())

May gain helper methods (e.g., `get_rkyv()`, `put_rkyv()`) if duplication across storage adapters emerges during implementation.

**rkyv Helpers (pub(crate))** — Safety-enforcing functions:
```rust
pub(crate) fn serialize<T>(value: &T) -> Result<AlignedVec<16>>;
pub(crate) fn deserialize<T>(bytes: &[u8]) -> Result<T>;
pub(crate) fn with_archived<T, F, R>(bytes: &[u8], f: F) -> Result<R>;
```

These enforce:
- AlignedVec<16> for serialization (not Vec<u8>)
- Validation with bytecheck (never access_unchecked)
- Two-phase pattern for iteration (collect bytes, then deserialize)
- Alignment fast path (direct access if already aligned, copy if not)

### Storage Adapter Pattern

Each context gets a `storage/` submodule (or `storage_v2/` during transition):
```
lithos-core/src/schema/
├── mod.rs              # Domain types (re-exports Repository trait)
├── types.rs            # Domain entities (Schema, Property, etc.)
├── identifier.rs       # SchemaId (with impl_redb_uuid! macro)
├── validation.rs       # Business logic validation
├── error.rs            # SchemaError
├── repository.rs       # Repository trait definitions (Split into Read/Write)
└── storage/            # (storage_v2 during transition; renamed to storage after migration)
     ├── mod.rs          # RedbRepository struct + re-exports
     ├── read.rs         # Redb-backed read implementation
     ├── write.rs        # Redb-backed write implementation
     ├── tables.rs       # Context table definitions
     ├── memory.rs       # InMemoryStorage (uses HashMap)
     └── fake.rs         # FakeStorage (test fake)
```

**Repository traits** (in `schema/repository.rs`):
The single unified trait is split into segregated interfaces to improve clarity and allow components to request only the access level they need.

```rust
/// Read-side port for schema persistence.
pub trait ReadRepository {
    fn get(&self, id: SchemaId) -> Result<Option<Schema>, SchemaError>;
    fn list(&self) -> Result<Vec<Schema>, SchemaError>;
    fn count(&self) -> Result<usize, SchemaError>;

    // Context-specific read queries
    fn find_by_name(&self, name: &str) -> Result<Option<Schema>, SchemaError>;
    fn list_properties(&self, schema_id: SchemaId) -> Result<Vec<Property>, SchemaError>;
}

/// Write-side port for schema persistence.
pub trait WriteRepository {
    fn save(&mut self, schema: Schema) -> Result<(), SchemaError>;
    fn save_many(&mut self, schemas: Vec<Schema>) -> Result<(), SchemaError>;
    fn delete(&mut self, id: SchemaId) -> Result<bool, SchemaError>;
}

/// Full repository port for schema persistence (hexagonal architecture).
///
/// Extends both read and write capabilities.
pub trait Repository: ReadRepository + WriteRepository {}

impl<T> Repository for T where T: ReadRepository + WriteRepository {}
```

**Implementation Split**:
To maintain large storage adapters, the implementation of `RedbRepository` is split across `read.rs` and `write.rs`. This is achieved by defining the struct in `mod.rs` (with internal fields as `pub(crate)` or accessible within the module tree) and implementing the segregated traits in their respective files.

**RedbStorage** pattern (realistically several hundred lines for 4-10 tables; organized into operation-focused modules):
```rust
const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas");
const PROPERTIES: UuidTable<PropertyId, &[u8]> = UuidTable::new("properties");
const SCHEMA_PROPERTIES: UuidMultimap<SchemaId, PropertyId> =
    UuidMultimap::new("schema_properties");

pub struct RedbStorage {
    store: Store,
}
```

```rust
// storage/read.rs
impl ReadRepository for RedbStorage {
    fn get(&self, id: SchemaId) -> Result<Option<Schema>, SchemaError> {
        self.store
            .read(|tx| {
                let table = tx.inner.open_table(SCHEMAS.definition())?;

                match table.get(&id)? {
                    Some(guard) => {
                        let bytes = guard.value();
                        let schema = db::rkyv::deserialize(bytes)?;
                        Ok(Some(schema))
                    }
                    None => Ok(None),
                }
            })
            .map_err(SchemaError::from)
    }

    fn list(&self) -> Result<Vec<Schema>, SchemaError> {
        self.store
            .read(|tx| {
                let table = tx.inner.open_table(SCHEMAS.definition())?;

                let mut all_bytes = Vec::new();
                for entry in table.iter()? {
                    let (_key, guard) = entry?;
                    all_bytes.push(guard.value().to_vec());
                }

                let mut schemas = Vec::new();
                for bytes in all_bytes {
                    schemas.push(db::rkyv::deserialize(&bytes)?);
                }

                Ok(schemas)
            })
            .map_err(SchemaError::from)
    }

    fn count(&self) -> Result<usize, SchemaError> {
        self.store
            .read(|tx| {
                let table = tx.inner.open_table(SCHEMAS.definition())?;
                Ok(table.len()? as usize)
            })
            .map_err(SchemaError::from)
    }

    fn find_by_name(&self, name: &str) -> Result<Option<Schema>, SchemaError> {
        todo!()
    }

    fn list_properties(&self, schema_id: SchemaId) -> Result<Vec<Property>, SchemaError> {
        todo!()
    }
}

// storage/write.rs
impl WriteRepository for RedbStorage {
    fn save(&mut self, schema: Schema) -> Result<(), SchemaError> {
        self.store
            .write(|tx| {
                let mut table = tx.inner.open_table(SCHEMAS.definition())?;
                let bytes = db::rkyv::serialize(&schema)?;
                table.insert(&schema.id, bytes.as_slice())?;
                Ok(())
            })
            .map_err(SchemaError::from)
    }

    fn save_many(&mut self, schemas: Vec<Schema>) -> Result<(), SchemaError> {
        self.store
            .write(|tx| {
                let mut table = tx.inner.open_table(SCHEMAS.definition())?;

                for schema in schemas {
                    let bytes = db::rkyv::serialize(&schema)?;
                    table.insert(&schema.id, bytes.as_slice())?;
                }

                Ok(())
            })
            .map_err(SchemaError::from)
    }

    fn delete(&mut self, id: SchemaId) -> Result<bool, SchemaError> {
        self.store
            .write(|tx| {
                let mut table = tx.inner.open_table(SCHEMAS.definition())?;
                Ok(table.remove(&id)?.is_some())
            })
            .map_err(SchemaError::from)
    }
}
```

Batch behavior is implemented directly within these segregated modules. Only split further if either file becomes unwieldy.

This keeps the Repository adapter deep at the interface while improving locality in implementation.

### Safety Patterns from Research

Based on research documented in `docs/refs/crates/rkyv/persistent-storage.md` and `docs/refs/crates/redb/rkyv-integration.md`:

1. **Always use AlignedVec<16>** for serialization buffers (never Vec<u8>)
2. **Always validate with rkyv::access()** when reading from disk (never access_unchecked)
3. **Two-phase iteration pattern** to avoid AccessGuard + rkyv lifetime conflicts
4. **Alignment fast path** with fallback to AlignedVec for unaligned data
5. **Closure-based zero-copy** for hot paths (eliminates return type self-referential issues)
6. **Format stability** — alignment feature flags (`aligned` vs `unaligned`) are part of the format contract

### Error Handling

`DbError` will be redesigned to avoid uniform string wrapping of redb errors. The interface should preserve machine-meaningful categories while still supporting context mapping.

Target direction:

```rust
pub enum DbError {
    Open(redb::DatabaseError),
    Transaction(redb::TransactionError),
    Table(redb::TableError),
    Commit(redb::CommitError),
    Storage(redb::StorageError),
    Serialization(SerializationError),
    Deserialization(DeserializationError),
    Corruption(CorruptionError),
}
```

Where redb variants are transparent wrappers of redb error types, preserving source error metadata and enabling:

- classification (`is_transient`, `is_corruption`, `is_validation`)
- richer logging without string parsing
- precise adapter-level mapping to context errors

To avoid redb-specific matching leaking upward, add a simple stable classifier:

```rust
pub enum DbErrorKind {
    Open,
    Transaction,
    Table,
    Commit,
    Storage,
    Serialization,
    Deserialization,
    Corruption,
}

impl DbError {
    pub fn kind(&self) -> DbErrorKind;
    pub fn is_transient(&self) -> bool;
}
```

Keep this intentionally small: no deep error taxonomy, no backend-agnostic hierarchy beyond `DbErrorKind`.

Key decision: avoid collapsing redb errors into generic `DbError::Database(String)` where possible.

Follow-up option (not in first implementation): introduce internal normalized error structs only if a second backend requires cross-backend stability.

Storage adapters convert `DbError` to context-specific errors (e.g., `SchemaError`) via explicit mapping rules rather than ad-hoc string conversion.

### Migration Strategy

1. **Keep old reader.rs/writer.rs** during refactor (don't delete immediately)
2. **Refactor one context end-to-end** (schema) as proof of concept
3. **Validate with existing tests** — schema tests should pass with new storage adapter
4. **Extract common patterns** if duplication emerges across contexts
5. **Refactor note and template** using validated pattern
6. **Delete old reader.rs/writer.rs** once all contexts migrated
7. **Update CONTEXT.md** to reflect new architecture

### ADR Considerations

No existing ADRs contradict this refactor. This implements the "Unified Repository traits" pattern documented in AGENTS.md while fixing the implementation issues that made the original db module rigid.

Should create ADR documenting:
- Why we expose redb directly to storage adapters (vs complete abstraction)
- Why rkyv helpers are pub(crate) (vs public API)
- Why storage adapters live in context-specific submodules (vs centralized)

## Testing Decisions

### What Makes a Good Test

- **Test external behavior, not implementation details**: Repository trait methods (get, save, list, delete) are the interface. Internal details (which tables, how serialization works) should be opaque.
- **Use temporary databases for integration tests**: Each test gets its own temp file, cleaned up via `tempfile::TempDir` drop.
- **Test safety patterns explicitly**: Alignment, validation, AccessGuard lifetimes should have dedicated tests in db module.
- **Prefer Repository trait tests over storage adapter tests**: If domain logic can be tested via FakeStorage, do that. Only test RedbStorage for persistence-specific concerns.

### Modules to Test

**db module (unit tests)**:
- `rkyv::serialize()` / `rkyv::deserialize()` roundtrip
- `rkyv::with_archived()` zero-copy access
- Alignment handling (aligned fast path, unaligned slow path)
- Error conversion (redb errors → DbError variants)
- Store transaction scoping (closures enforce commit/rollback)

**Prior art**: Existing tests in `db/reader.rs` (lines 1029-1487) and `db/writer.rs` (lines 892-1287) provide patterns. Key difference: new tests should focus on helpers, not full CRUD operations.

**Storage adapters (integration tests)**:
- RedbStorage implements Repository trait correctly (all methods)
- Single writes commit successfully
- Batch operations (`save_many`) commit atomically within one transaction
- Batch reads perform efficiently (no N+1 transaction overhead)
- Multiple tables within one transaction work correctly (schema + properties)
- Two-phase iteration pattern doesn't corrupt data during `list()`
- Transaction rollback on error (partially written batch is aborted)
- Error distinguishes between DbError and domain errors (e.g., SchemaError)

**Prior art**: Existing schema/note/template storage tests (if any). New tests should live next to the relevant adapter module (`schema/storage/read.rs` or `schema/storage/write.rs`) with `#[cfg(test)]` modules, not in detached files.

**What NOT to test**:
- Don't test redb itself (table operations, transactions) — trust the library
- Don't test rkyv serialization — trust the library
- Don't test implementation details like which helper functions are called
- Don't test FakeStorage exhaustively — it's just a test double

### Testing the Two-Phase Pattern

Critical test: verify that the AccessGuard + rkyv lifetime footgun is avoided:

```rust
#[test]
fn list_many_items_does_not_corrupt() {
    let (_temp, mut storage) = temp_storage();

    // Insert 100 items
    for i in 0..100 {
        let schema = Schema { id: SchemaId::new(), name: format!("schema_{i}") };
        storage.save(schema).unwrap();
    }

    // List all (exercises two-phase iteration)
    let schemas = storage.list().unwrap();
    assert_eq!(schemas.len(), 100);

    // Verify data integrity
    for (i, schema) in schemas.iter().enumerate() {
        assert!(schema.name.starts_with("schema_"));
    }
}
```

## Out of Scope

- **Schema migration**: Database format changes (adding fields, removing fields) are out of scope. This refactor maintains format compatibility.
- **Caching layer**: No caching of deserialized objects (e.g., moka). Storage adapters always hit redb.
- **Async support**: redb is sync-only; no tokio integration planned.
- **Alternative serialization**: Only rkyv is supported. No serde, bincode, etc.
- **Query DSL**: No abstraction over redb queries. Storage adapters use redb APIs directly.
- **Multiple databases**: Single Store per process. No sharding, replication, or multi-database support.
- **InMemoryStorage / FakeStorage refactor**: These are out of scope; focus is on RedbStorage pattern.

## Further Notes

### Why This Refactor Now

The current db module has become a choke point for development. Adding new contexts (template) requires duplicating 30+ methods. Performance optimization is blocked by unclear boundaries (where does allocation happen?). The UUID→String conversion workaround revealed that the abstraction level is wrong.

This refactor unblocks:
- Native UUID keys (eliminating allocations in hot paths)
- Per-context optimization (e.g., note versioning, schema caching)
- Performance profiling (clear boundaries between serialization, DB, domain logic)
- Testing (operation-focused storage modules are easier to reason about than monolithic files)

### Alignment Feature Flag Decision

The codebase currently uses rkyv's default `aligned` feature (primitives have natural alignment). This is the right choice for performance. If we ever need `unaligned` (e.g., for cross-platform compatibility), that's a **breaking format change** requiring database rebuild.

Document in code: "Changing rkyv alignment features breaks on-disk format. See docs/refs/crates/rkyv/persistent-storage.md."

### Relationship to Clean Architecture

This refactor implements hexagonal architecture more idiomatically in Rust:
- **Domain** (note, schema, template) = pure logic, depends on Repository trait
- **Port** = Repository trait (defined per-context)
- **Adapter** = RedbStorage, InMemoryStorage, FakeStorage (implement Repository)
- **Infrastructure** = db module (library of tools for adapters)

The key insight: **db module is not the adapter** — it's infrastructure that adapters use. This is why exposing redb directly (with safety helpers) is the right level of abstraction.

### Success Criteria

This refactor succeeds if:
1. **All existing tests pass** with new storage adapters
2. **Storage adapter complexity is split into operation-focused modules** (no monolithic adapter choke point, even when total lines are large)
3. **Native UUID keys work** (no string allocations)
4. **rkyv details are isolated** to storage adapters (not in domain code)
5. **Batch operations are efficient** (single transaction for multiple writes)
6. **Repository traits are clearly defined** in separate `repository.rs` files per context
7. **DbError preserves redb error categories** without uniform string wrapping
8. **Future contributors can understand the code** by reading operation-focused adapter modules and table definition modules
