# Loader Refactor TODO

## Current State

The ingestor has been successfully refactored to:
- Use `property_bank()` → `Option<RawPropertyBank>` with embedded metadata
- Use `all_schemas()` → `Vec<RawSchema>` with embedded metadata
- Implement parse-don't-validate via `validated()` pattern

## Remaining Work for Loader

The loader.rs still needs major refactoring:

### 1. Remove Tuple Types
- [x] Update type aliases to remove `RawSchemaWithTimes` and `SchemaWithTimes`
- [ ] Refactor all methods to work with `RawSchema` directly (metadata embedded)

### 2. CQRS → Repository Pattern
- [ ] Replace `query: db_query::Query` with `repository: R`
- [ ] Replace `command: db_command::Command` (remove field)
- [ ] Update all `self.query.*` calls to `self.repository.*`
- [ ] Update all `self.command.*` calls to `self.repository.*`

### 3. Update Method Signatures
- [ ] `partition_by_staleness`: Accept `&[RawSchema]` instead of tuples
- [ ] `apply_incremental_resolution`: Accept `Vec<SchemaWithId>` instead of tuples
- [ ] `persist_schemas`: Accept `Vec<SchemaWithId>` instead of tuples
- [ ] All methods: Extract metadata from `raw.metadata` instead of tuple fields

### 4. StoredSchema → Schema Migration
- [ ] Change return type from `Vec<StoredSchema>` to `Vec<Schema>`
- [ ] Update extender.rs to use `Schema` instead of `StoredSchema`
- [ ] Update resolver.rs to use `Schema` instead of `StoredSchema`

### 5. Raw File Content Storage
Currently the loader needs raw file content for version history but:
- Ingestor no longer returns it (only in metadata.content_hash)
- Need to decide: store full content in metadata? Or re-read when needed?

## Approach

The loader refactor should be done in phases:
1. Fix compilation with minimal changes (partially done)
2. Migrate partition_by_staleness to work with RawSchema directly
3. Migrate to Repository pattern
4. Migrate StoredSchema → Schema
5. Delete old CQRS files
