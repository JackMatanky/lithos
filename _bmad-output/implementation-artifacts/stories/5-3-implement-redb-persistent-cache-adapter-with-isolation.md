# Story 5.3: Implement Redb Persistent Cache Adapter with Table Isolation

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a DevOps engineer requiring persistence,
I want a robust `Redb` adapter implementing the `CacheReader` and `CacheWriter` traits with rkyv serialization and table isolation,
So that data persists across application restarts and multiple cache consumers can coexist without conflicts.

## Original Epic Acceptance Criteria

**Given** the `redb` and `rkyv` dependencies are added
**When** I implement the `RedbCache<K, V>` struct in `spi/cache/redb.rs`
**Then** it wraps a Redb database with configuration:

- `db: Arc<redb::Database>` - shared database instance
- `table_name: String` - isolated table for this cache instance

**And** constructor `new(db_path: PathBuf, table_name: &str)` creates the database and opens the table

**Given** multiple cache consumers need isolation
**When** I implement table management
**Then** each `RedbCache` instance operates on a dedicated Redb table (e.g., "schemas", "config", "queries")
**And** tables are created lazily on first access if they don't exist
**And** multiple `RedbCache` instances can coexist in the same database file without interference
**And** documentation provides naming conventions for table names

**Given** persistence requires metadata tracking
**When** I implement the storage schema
**Then** values are stored as `CachedEntry<V>` struct containing:

- `value: V` - the actual cached data
- `timestamp: u64` - Unix timestamp (seconds since epoch) of last write
- `metadata: HashMap<String, String>` - extensible key-value pairs for consumer-specific data (e.g., file hash, version)

**And** `CachedEntry<V>` derives `rkyv::Archive`, `rkyv::Serialize`, `rkyv::Deserialize`
**And** the entire struct is rkyv-serialized for zero-copy deserialization per ADR 006

**Given** the trait must be implemented
**When** I implement `CacheReader<K, V>` and `CacheWriter<K, V>` for `RedbCache<K, V>`
**Then** all trait methods satisfy the async trait bounds
**And** `CacheReader::get()` deserializes the `CachedEntry<V>` and returns `Some(entry.value)` on hit
**And** `CacheReader::has()` checks for entry existence without full deserialization
**And** `CacheWriter::clear()` removes all entries from the isolated table
**And** `CacheWriter::delete()` removes the entry and returns true if it existed
**And** `CacheWriter::invalidate()` delegates to `delete()` for semantic clarity
**And** `CacheWriter::put()` wraps the value in `CachedEntry` with current timestamp and empty metadata, then serializes
**And** all operations use Redb read/write transactions

**Given** serialization errors must be handled
**When** rkyv serialization or deserialization fails
**Then** errors are logged via `tracing::error!` with full context
**And** mapped to `CacheError::SerializationError` with the value type name included

**Given** I/O errors must be handled
**When** Redb transactions fail (disk full, permission denied)
**Then** errors are mapped to `CacheError::IoError` or `CacheError::BackendError` as appropriate

**Given** observability is required
**When** I instrument all methods
**Then** database transactions are wrapped in `tracing` spans:

- Span name: `"redb_transaction"`
- Attributes: `table_name`, `operation`, `key` (if serializable)

**And** successful operations emit events with `cache_layer = "disk"`

**Given** consumers need access to metadata
**When** I provide utility methods
**Then** `get_with_metadata(&self, key: &K) -> Result<Option<(V, HashMap<String, String>)>, CacheError>` returns value and metadata
**And** `put_with_metadata(&self, key: K, value: V, metadata: HashMap<String, String>)` stores custom metadata

## TDD Acceptance Criteria (Quality Gates)

**Given** I need a persistent disk cache
**When** I run `mise run test:unit:core redb_cache`
**Then** all tests pass with all public components validated
**And** data survives cache instance drops and recreations (persistence check)
**And** multiple tables in the same DB file do not leak data between instances
**And** `rkyv` zero-copy deserialization is verified for complex types
**And** metadata is correctly preserved and retrievable

**Given** observability is critical for disk operations
**When** I run tests with a tracing subscriber
**Then** all Redb transactions emit correct `tracing` spans and events
**And** spans include `table_name` and `operation` attributes

**Given** the implementation must be robust
**When** I simulate I/O or serialization failures
**Then** errors are correctly mapped to `CacheError` variants with context
**And** no data corruption occurs on failed writes

**Given** I need documentation-driven examples
**When** I run `mise run test:unit:core --doc`
**Then** all doc tests demonstrate table isolation and metadata usage
**And** examples demonstrate proper database path handling

## TDD Tasks / Subtasks

### Phase 1: Test Infrastructure and Scaffolding
- [x] Task 1: Initialize implementation file and verify module linkage
  - [x] Subtask 1.1: Create empty file at `crates/adapters/src/spi/cache/redb.rs`
  - [x] Subtask 1.2: Add `pub(crate) mod redb;` to `crates/adapters/src/spi/cache/mod.rs`
  - [x] Subtask 1.3: Write a unit test in `redb.rs` under `#[cfg(test)]` that fails to import `RedbCache`
  - [x] Subtask 1.4: Write failing test that fails to find `CachedEntry` type
  - [x] Subtask 1.5: Run `mise run test:unit:core redb` and verify failures (RED)
  - [x] Subtask 1.6: Run `mise run lint` and ensure environment is clean

### Phase 2: Schema & Serialization
- [x] Task 2: Implement CachedEntry and rkyv integration
  - [x] Subtask 2.1: Write failing test for `CachedEntry<V>` struct requiring `Archive`, `Serialize`, `Deserialize`
  - [x] Subtask 2.2: Implement `CachedEntry<V>` with `value`, `timestamp`, `metadata` fields; ensure `V` is constrained by `rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>`.
  - [x] Subtask 2.3: Define `rkyv` compatible `HashMap` or replacement for metadata to ensure serialization works out-of-the-box.
  - [x] Subtask 2.4: Apply `rkyv` macros and verify serialization
  - [x] Subtask 2.5: Write failing test for `SerializationError` mapping
  - [x] Subtask 2.6: Implement error mapping for failed `rkyv` operations
  - [x] Subtask 2.7: Run `mise run test:unit:core redb_serialization` and verify pass (GREEN)
  - [x] Subtask 2.8: Run `mise run lint` and fix all warnings/errors

### Phase 3: Database & Table Management
- [x] Task 3: Implement Redb initialization and table isolation
  - [x] Subtask 3.1: Write failing test for `RedbCache::new(db_path, table_name)`
  - [x] Subtask 3.2: Implement `RedbCache` struct wrapping `Arc<redb::Database>`
  - [x] Subtask 3.3: Write failing test verifying lazy table creation
  - [x] Subtask 3.4: Implement lazy table opening within operations using a `redb::TableDefinition<&[u8], &[u8]>` where keys and values are stored as serialized bytes.
  - [x] Subtask 3.5: Implement a helper to serialize the generic key `K` into a byte-stable representation (e.g., via `rkyv` or `ToString`) for Redb lookups.
  - [x] Subtask 3.6: Verify multiple instances share the same `redb::Database` but different tables
  - [x] Subtask 3.7: Write failing test for `IoError` mapping during DB open
  - [x] Subtask 3.8: Implement I/O error mapping
  - [x] Subtask 3.9: Run `mise run test:unit:core redb_init` and verify pass (GREEN)
  - [x] Subtask 3.10: Run `mise run lint` and fix all warnings/errors

### Phase 4: Cache Trait Implementation
- [x] Task 4: Implement core Cache operations with persistence
  - [x] Subtask 4.1: Write failing test for `put` then `get` across instance drops
  - [x] Subtask 4.2: Implement `put` using Redb write transaction
  - [x] Subtask 4.3: Implement `get` using a Redb **read-only** transaction and `rkyv::access` for zero-copy deserialization of the `CachedEntry`.
  - [x] Subtask 4.4: Write failing test for `has` returning existence status
  - [x] Subtask 4.5: Implement `has` checking if key exists in table
  - [x] Subtask 4.6: Write failing test for `delete` returning existence status
  - [x] Subtask 4.7: Implement `delete` operation
  - [x] Subtask 4.8: Write failing test for `clear` removing all entries
  - [x] Subtask 4.9: Implement `clear` using Redb transaction to delete all rows in table
  - [x] Subtask 4.10: Write failing test for `invalidate` functionality
  - [x] Subtask 4.11: Implement `invalidate`
  - [x] Subtask 4.12: Run `mise run test:unit:core redb_trait` and verify pass (GREEN)
  - [x] Subtask 4.13: Run `mise run lint` and fix all warnings/errors

### Phase 5: Metadata & Extended Operations
- [x] Task 5: Implement metadata tracking and retrieval
  - [x] Subtask 5.1: Write failing test for `put_with_metadata`
  - [x] Subtask 5.2: Implement `put_with_metadata` storing custom HashMap
  - [x] Subtask 5.3: Write failing test for `get_with_metadata`
  - [x] Subtask 5.4: Implement `get_with_metadata` returning both value and metadata
  - [x] Subtask 5.5: Write failing test verifying timestamp updates on every `put`
  - [x] Subtask 5.6: Ensure current Unix timestamp (via `SystemTime::now().duration_since(UNIX_EPOCH)`) is stored in `CachedEntry` on every write.
  - [x] Subtask 5.7: Run `mise run test:unit:core redb_metadata` and verify pass (GREEN)
  - [x] Subtask 5.8: Run `mise run lint` and fix all warnings/errors

### Phase 6: Observability & Tracing
- [x] Task 6: Implement tracing spans for Redb transactions
  - [x] Subtask 6.1: Write failing test expecting `"redb_transaction"` span for operations
  - [x] Subtask 6.2: Add `#[tracing::instrument]` to all methods with required attributes
  - [x] Subtask 6.3: Write failing test expecting `cache_layer = "disk"` events
  - [x] Subtask 6.4: Add events to `get`, `put`, and `delete`
  - [x] Subtask 6.5: Run `mise run test:unit:core redb_tracing` and verify pass (GREEN)
  - [x] Subtask 6.6: Run `mise run lint` and fix all warnings/errors

### Phase 7: Documentation & Doc Testing
- [x] Task 7: Implement module documentation and executable examples
  - [x] Subtask 7.1: Write failing doc test for `RedbCache` with table isolation
  - [x] Subtask 7.2: Add working doc test to module-level documentation
  - [x] Subtask 7.3: Write failing doc test for metadata operations
  - [x] Subtask 7.4: Add metadata example to `RedbCache` docs
  - [x] Subtask 7.5: Run `mise run test:unit:core --doc` and verify all pass (GREEN)
  - [x] Subtask 7.6: Run `mise run lint` and fix all warnings/errors

### Phase 8: Final Quality Gate
- [x] Task 8: Comprehensive project verification
  - [x] Subtask 8.1: Run `mise run test:coverage` and verify `RedbCache` logic is fully exercised
  - [x] Subtask 8.2: Run `mise run fmt` and verify formatting compliance
  - [x] Subtask 8.3: Run `mise run lint` one final time
  - [x] Subtask 8.4: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [x] Subtask 8.5: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [x] Subtask 8.6: Stage and commit all files created, deleted, or modified during the story implementation with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

### Phase 10: CQRS Refactor (Architectural Integrity)
- [x] Task 10: Implement split traits for RedbCache
  - [x] Subtask 10.1: Update `redb.rs` to implement `CacheReader` and `CacheWriter` separately
  - [x] Subtask 10.2: Update doc tests to use split trait imports
  - [x] Subtask 10.3: Verify all tests pass with split traits

## Dev Notes

### Architecture Compliance
- **Hexagonal Architecture**: `RedbCache` is an Adapter in the adapters layer.
- **ACID Persistence**: Redb provides atomic, consistent, isolated, durable transactions.
- **Zero-Copy Performance**: `rkyv` integration ensures that reading from disk doesn't require expensive deserialization steps.
- **Table Isolation**: Uses Redb tables to allow multiple independent cache instances in one DB file.

### Technical Requirements
- **Lazy Tables**: Tables should not be created until the first operation is performed.
- **Metadata Support**: Every entry stores a timestamp and a flexible metadata map.
- **Error Propagation**: Strict mapping of Redb transaction errors and rkyv serialization errors to `CacheError`.

### Library Dependencies
- **redb**: Persistent KV store.
- **rkyv**: Zero-copy serialization (required: `Archive`, `Serialize`, `Deserialize` derives).
- **tracing**: Instrumentation for all DB transactions.
- **async-trait**: For trait implementation.

### File Structure Requirements
- **Location**: `crates/adapters/src/spi/cache/redb.rs`
- **Module Visibility**: `pub(crate)` mod in `cache/mod.rs`.

### Project Structure Notes
- **Alignment**: Consistent with ADR 006 (Redb + rkyv foundation).
- **Conflicts**: None detected. Complements Story 5.2 (Moka).

### TDD Methodology
- **RED-GREEN-REFACTOR**: Strict adherence.
- **Co-located Tests**: Unit tests live in `redb.rs` under `#[cfg(test)]`.
- **Mise Orchestration**: Use `mise run test:unit:core` for verification.

### References
- [Source: project-context.md#Hexagonal-Boundary-Enforcement]
- [Source: project-context.md#Async-Resource-Safety]
- [Source: ADR 006: Storage - Redb + rkyv]
- [Source: ADR 013 (Caching - Superseded): Caching Strategy]
- [Source: Story 5.1: Define Cache Trait and Error Hierarchy]

## Dev Agent Record

### Agent Model Used
gemini-3-flash-preview (2026-01-26)

### Completion Notes List
- Applied TDD-optimized methodology with 51+ atomic subtasks.
- Preserved original Epic ACs.
- Integrated mandatory linting workflows and mise orchestration.
- Ensured co-located tests per Rust project standards.
- Provided detailed Redb table isolation logic.
- **Transaction Orchestration**: Implemented `run_blocking_read` and `run_blocking_write` helper methods to centralize `spawn_blocking` logic, transaction lifecycle management, and automatic commits.
- **Serialization Helpers**: Consolidated `rkyv` serialization and deserialization patterns into static associated functions, improving readability and maintainability of core trait operations.
- **Lint Compliance Mastery**:
  - Resolved contradictory lint requirements between `semicolon_outside_block` and `semicolon_if_nothing_returned` by replacing block-scoped tests with explicit `drop()` calls.
  - Successfully navigated `clippy::exhaustive_structs` issues triggered by the `Archive` derive macro using module-level `#![allow]` combined with detailed reasoning.
  - Reorganized internal test structure and module item ordering to achieve zero-warning status under strict project restriction lints.
- **Improved Isolation**: Refined table definition management to ensure clean separation even when sharing a single database instance across different cache types.
- Achieved full quality gate compliance (fmt, clippy, tests, pre-commit).
- **Adversarial Review Fixes**:
    - Made `RedbCache::new` async and moved database creation to a blocking task to satisfy project safety invariants.
    - Eliminated memory leaks by replacing `Box::leak` with `Arc<str>` for dynamic table name management.
    - Enhanced observability by including `table_name` and `key` attributes in tracing spans.
    - Refactored `rkyv` trait bounds into consolidated blocks for better maintainability and cleaner method signatures.

### File List
- `crates/adapters/src/spi/cache/redb.rs` - Implementation file.
- `crates/adapters/src/spi/cache/mod.rs` - Module declaration.
