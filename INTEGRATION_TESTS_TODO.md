# Integration Tests Migration TODO

## Status

All 730 unit tests are passing. Integration tests need migration from CQRS to Repository pattern.

## Required Changes

### 1. Common Test Module (`lithos-core/tests/common/mod.rs`)

- [x] Update imports: `db_command`, `db_query`, `ports` → `storage::Repository`
- [x] Update `QueryExt` → `RepositoryExt` with generic `impl<R: Repository>`
- [x] Remove `CommandExt` (Repository already has save methods)
- [ ] Update `SchemaBuilder::build()` to return `Schema` instead of `StoredSchema`
- [ ] Update `StoredProperty` → `Property` throughout builders
- [ ] Update assertion helpers to use `Schema` instead of `StoredSchema`

### 2. Test Files Needing Migration

All files need:
- Replace `db_command::Command::new(&db)` with `RedbRepository::new(Arc::new(db))`
- Replace `db_query::Query::new(&db)` with same Repository instance
- Update `StoredSchema` → `Schema` throughout
- Update `StoredProperty` → `Property` throughout
- Update method calls to match new Repository API

#### Files:
1. `schema_cqrs.rs` - ~1770 lines, CQRS pattern tests
2. `schema_ingestion.rs` - Pipeline tests
3. `schema_inheritance.rs` - Inheritance tests
4. `schema_raw_file_storage.rs` - Raw file storage tests
5. `schema_staleness.rs` - Staleness detection tests

### 3. Migration Pattern

Old CQRS pattern:
```rust
let db = Database::open(&path)?;
let command = db_command::Command::new(&db);
let query = db_query::Query::new(&db);

command.save_property_bank(&bank)?;
let loaded = query.get_property_bank()?;
```

New Repository pattern:
```rust
use std::sync::Arc;
use lithos_core::schema::storage::{Repository, RedbRepository};

let db = Arc::new(Database::open(&path)?);
let repo = RedbRepository::new(db);

repo.save_property_bank(&bank)?;
let loaded = repo.get_property_bank()?;
```

## Recommendation

Update integration tests incrementally:
1. Start with `common/mod.rs` builders
2. Migrate `schema_cqrs.rs` (most comprehensive)
3. Migrate remaining files

Estimated time: 3-4 hours for full migration.
