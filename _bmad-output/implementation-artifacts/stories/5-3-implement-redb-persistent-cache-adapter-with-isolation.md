# Story 5.3: Implement Redb Persistent Cache Adapter with Table Isolation

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a DevOps engineer requiring persistence,
I want a robust `Redb` adapter implementing the `Cache` trait with rkyv serialization and table isolation,
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
**And** the entire struct is rkyv-serialized for zero-copy deserialization per ADR 0002

**Given** the trait must be implemented
**When** I implement `Cache<K, V>` for `RedbCache<K, V>`
**Then** `get()` deserializes the `CachedEntry<V>` and returns `Some(entry.value)` on hit
**And** `put()` wraps the value in `CachedEntry` with current timestamp and empty metadata, then serializes
**And** `delete()` removes the entry and returns true if it existed
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
**When** I run `mise run test:unit:adapters redb_cache`
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
**When** I run `mise run test:unit:adapters --doc`
**Then** all doc tests demonstrate table isolation and metadata usage
**And** examples demonstrate proper database path handling

## TDD Tasks / Subtasks

### Phase 1: Test Infrastructure and Scaffolding
- [ ] Task 1: Initialize implementation file and verify module linkage
  - [ ] Subtask 1.1: Create empty file at `crates/adapters/src/spi/cache/redb.rs`
  - [ ] Subtask 1.2: Add `pub(crate) mod redb;` to `crates/adapters/src/spi/cache/mod.rs`
  - [ ] Subtask 1.3: Write a unit test in `redb.rs` under `#[cfg(test)]` that fails to import `RedbCache`
  - [ ] Subtask 1.4: Write failing test that fails to find `CachedEntry` type
  - [ ] Subtask 1.5: Run `mise run test:unit:adapters redb` and verify failures (RED)
  - [ ] Subtask 1.6: Run `mise run lint` and ensure environment is clean
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 2: Schema & Serialization (Test-Driven)
- [ ] Task 2: Implement CachedEntry and rkyv integration
  - [ ] Subtask 2.1: Write failing test for `CachedEntry<V>` struct requiring `Archive`, `Serialize`, `Deserialize`
  - [ ] Subtask 2.2: Implement `CachedEntry` with `value`, `timestamp`, `metadata` fields
  - [ ] Subtask 2.3: Write failing test requiring `rkyv` round-trip for `CachedEntry`
  - [ ] Subtask 2.4: Apply `rkyv` macros and verify serialization
  - [ ] Subtask 2.5: Write failing test for `SerializationError` mapping
  - [ ] Subtask 2.6: Implement error mapping for failed `rkyv` operations
  - [ ] Subtask 2.7: Run `mise run test:unit:adapters redb_serialization` and verify pass (GREEN)
  - [ ] Subtask 2.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 3: Database & Table Management (Test-Driven)
- [ ] Task 3: Implement Redb initialization and table isolation
  - [ ] Subtask 3.1: Write failing test for `RedbCache::new(db_path, table_name)`
  - [ ] Subtask 3.2: Implement `RedbCache` struct wrapping `Arc<redb::Database>`
  - [ ] Subtask 3.3: Write failing test verifying lazy table creation
  - [ ] Subtask 3.4: Implement lazy table opening within operations
  - [ ] Subtask 3.5: Write failing test for table isolation: write to "table1", ensure not in "table2"
  - [ ] Subtask 3.6: Verify multiple instances share the same `redb::Database` but different tables
  - [ ] Subtask 3.7: Write failing test for `IoError` mapping during DB open
  - [ ] Subtask 3.8: Implement I/O error mapping
  - [ ] Subtask 3.9: Run `mise run test:unit:adapters redb_init` and verify pass (GREEN)
  - [ ] Subtask 3.10: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 4: Cache Trait Implementation (Test-Driven)
- [ ] Task 4: Implement core Cache operations with persistence
  - [ ] Subtask 4.1: Write failing test for `put` then `get` across instance drops
  - [ ] Subtask 4.2: Implement `put` using Redb write transaction
  - [ ] Subtask 4.3: Implement `get` using Redb read transaction and `rkyv` zero-copy
  - [ ] Subtask 4.4: Write failing test for `delete` returning existence status
  - [ ] Subtask 4.5: Implement `delete` operation
  - [ ] Subtask 4.6: Write failing test for `invalidate` functionality
  - [ ] Subtask 4.7: Implement `invalidate`
  - [ ] Subtask 4.8: Run `mise run test:unit:adapters redb_trait` and verify pass (GREEN)
  - [ ] Subtask 4.9: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 5: Metadata & Extended Operations (Test-Driven)
- [ ] Task 5: Implement metadata tracking and retrieval
  - [ ] Subtask 5.1: Write failing test for `put_with_metadata`
  - [ ] Subtask 5.2: Implement `put_with_metadata` storing custom HashMap
  - [ ] Subtask 5.3: Write failing test for `get_with_metadata`
  - [ ] Subtask 5.4: Implement `get_with_metadata` returning both value and metadata
  - [ ] Subtask 5.5: Write failing test verifying timestamp updates on every `put`
  - [ ] Subtask 5.6: Ensure current Unix timestamp is stored in `CachedEntry`
  - [ ] Subtask 5.7: Run `mise run test:unit:adapters redb_metadata` and verify pass (GREEN)
  - [ ] Subtask 5.8: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 6: Observability & Tracing (Test-Driven)
- [ ] Task 6: Implement tracing spans for Redb transactions
  - [ ] Subtask 6.1: Write failing test expecting `"redb_transaction"` span for operations
  - [ ] Subtask 6.2: Add `#[tracing::instrument]` to all methods with required attributes
  - [ ] Subtask 6.3: Write failing test expecting `cache_layer = "disk"` events
  - [ ] Subtask 6.4: Add events to `get`, `put`, and `delete`
  - [ ] Subtask 6.5: Run `mise run test:unit:adapters redb_tracing` and verify pass (GREEN)
  - [ ] Subtask 6.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 7: Documentation & Doc Testing (Test-Driven)
- [ ] Task 7: Implement module documentation and executable examples
  - [ ] Subtask 7.1: Write failing doc test for `RedbCache` with table isolation
  - [ ] Subtask 7.2: Add working doc test to module-level documentation
  - [ ] Subtask 7.3: Write failing doc test for metadata operations
  - [ ] Subtask 7.4: Add metadata example to `RedbCache` docs
  - [ ] Subtask 7.5: Run `mise run test:unit:adapters --doc` and verify all pass (GREEN)
  - [ ] Subtask 7.6: Run `mise run lint` and fix all warnings/errors
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions

### Phase 8: Final Quality Gate
- [ ] Task 8: Comprehensive project verification
  - [ ] Subtask 8.1: Run `mise run test:coverage` and verify `RedbCache` logic is fully exercised
  - [ ] Subtask 8.2: Run `mise run fmt` and verify formatting compliance
  - [ ] Subtask 8.3: Run `mise run lint` one final time
  - [ ] Subtask 8.4: Run `mise run verify` to ensure all Lithos quality gates are satisfied
  - [ ] Subtask 8.5: Run `pre-commit run --all-files` and verify all hooks pass (NEVER use `--no-verify`)
  - [ ] Subtask 8.6: Stage and commit all files created, deleted, or modified during the story implementation with a fully descriptive conventional commit style message (NEVER use `--no-verify`)

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
- **Alignment**: Consistent with ADR 0002 (Redb + rkyv foundation).
- **Conflicts**: None detected. Complements Story 5.2 (Moka).

### TDD Methodology
- **RED-GREEN-REFACTOR**: Strict adherence.
- **Co-located Tests**: Unit tests live in `redb.rs` under `#[cfg(test)]`.
- **Mise Orchestration**: Use `mise run test:unit:adapters` for verification.

### References
- [Source: project-context.md#Hexagonal-Boundary-Enforcement]
- [Source: project-context.md#Async-Resource-Safety]
- [Source: ADR 0002: Storage - Redb + rkyv]
- [Source: ADR 0016: Caching Strategy]
- [Source: Story 5.1: Define Cache Trait and Error Hierarchy]

## Dev Agent Record

### Agent Model Used
Claude-3.5-Sonnet (2024-10-22)

### Debug Log References
None - Story created through systematic analysis of artifacts and project context.

### Completion Notes List
- Applied TDD-optimized methodology with 51+ atomic subtasks.
- Preserved original Epic ACs.
- Integrated mandatory linting workflows and mise orchestration.
- Ensured co-located tests per Rust project standards.
- Provided detailed Redb table isolation logic.

### File List
- `crates/adapters/src/spi/cache/redb.rs` - Implementation file.
- `crates/adapters/src/spi/cache/mod.rs` - Module declaration.
    - **NOTE**: Review test-developer-guide.md Section 8 for comprehensive guidance on linting and code quality
    - **RULE**: Fix clippy issues properly rather than suppressing with `#[expect(...)]` attributes
    - **WORKFLOW**: `mise run lint` → Read diagnostic → Apply suggestions → Refactor for complexity → Verify with `mise run verify`
    - **ALLOWED USES**: `#[expect(...)]` only for intentional violations necessary for tests; `#[allow(...)]` primarily for generated code like `automock`
    - **COMMON FIXES**: Extract helper functions, use builder patterns, remove unnecessary collect(), avoid shadowing, document errors, use proper assertions
