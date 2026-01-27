# Epic 9: Storage Layer & Persistence **[MVP CORE]**

## Overview

System has zero-copy persistent storage with ACID transactions using Redb + rkyv that supports high-performance queries and maintains data consistency.

**FRs covered:** Architecture requirements (Redb + rkyv storage per ADR 0002), NFR2 (2s indexing), NFR9 (500MB memory)

## Implementation Notes

- **Storage Technology**: Redb + rkyv per ADR 0002 (no SQLite - architectural decision already made)
- **Zero-Copy Deserialization**: rkyv enables direct memory mapping without deserialization overhead
- **ACID Transactions**: Redb provides MVCC concurrency and transaction isolation
- **CQRS Pattern**: Storage ports split into CacheWriterPort (commands) and CacheReaderPort (queries) for clarity
- **Unit of Work Pattern**: TransactionContext manages atomic multi-table operations with rollback
- **Clean Slate Protocol**: Storage corruption triggers full rebuild from vault markdown files
- **Integration Points**:
  - Epic 5: Reuses RedbCache foundation (same Redb database, different tables)
  - Epic 8: Listens to NoteIndexed events for storage updates
  - Epic 10: Receives indexed note data to persist
  - Epic 11: Provides query read path via CacheReaderPort
- **Storage Schema Design**: Optimized for Epic 11 query patterns (by path, by schema, by fileClass)
- **Performance Targets**:
  - NFR2: Full vault indexing (1000 notes) completes in <2 seconds
  - NFR9: Storage operations stay within 500MB memory budget
  - Write operations: <10ms per note
  - Read operations: <1ms cache hit, <10ms cache miss
- **Location**: `crates/adapters/src/spi/storage/` contains ports, redb implementation, mocks
- **Storage Tables**:
  - `notes` table: PathBuf → Arc<Note> (main entity storage)
  - `schema_index` table: SchemaName → Vec<PathBuf> (fileClass queries)
  - `alias_index` table: Alias → PathBuf (wiki-link resolution)
  - `metadata_index` table: (FieldName, FieldValue) → Vec<PathBuf> (metadata queries)
- **Error Handling**: StorageError enum with variants for Corruption, TransactionFailed, SchemaConflict, OutOfMemory
- **Backup Strategy**: Periodic snapshots to `.lithos/backups/` with configurable retention
- **Migration Strategy**: Schema versioning with forward/backward compatibility checks
- **May Create**: ADR for storage schema patterns if design decisions need documentation

## Story 9.1: Create Storage Domain Interface and Ports

As a developer implementing data persistence,
I want clean domain interfaces for storage operations,
So that data can be stored and retrieved through well-defined contracts following hexagonal architecture.

**Acceptance Criteria:**

**Given** I need storage contracts following CQRS pattern
**When** I create storage domain ports in `crates/domain/src/ports/storage.rs`
**Then** `CacheWriterPort` trait defines write operations (insert, update, delete, batch_write)
**And** `CacheReaderPort` trait defines read operations (get, list, query, exists)
**And** both traits are async-compatible for integration with Epic 8 event system

**Given** ports must support multiple entity types
**When** I design trait signatures
**Then** ports are generic over entity types `T: Serialize + for<'a> Deserialize<'a> + Send + Sync`
**And** path-based keys use `PathBuf` for type safety

**Given** Epic 11 requires query capabilities
**When** I define CacheReaderPort
**Then** it includes methods for:
- `get_by_path(path: &Path) -> Result<Option<Arc<T>>>`
- `list_by_schema(schema: &str) -> Result<Vec<Arc<T>>>`
- `query_metadata(field: &str, value: &str) -> Result<Vec<Arc<T>>>`
- `resolve_alias(alias: &str) -> Result<Option<PathBuf>>`

**Given** Epic 10 indexing requires batch operations
**When** I define CacheWriterPort
**Then** it includes methods for:
- `insert(key: PathBuf, value: Arc<T>) -> Result<()>`
- `batch_insert(entries: Vec<(PathBuf, Arc<T>)>) -> Result<()>`
- `delete(key: &Path) -> Result<()>`
- `clear_schema(schema: &str) -> Result<()>`

**Given** storage ports are defined
**When** I implement mock test doubles in `crates/domain/src/ports/storage/mocks.rs`
**Then** `MockCacheWriter` and `MockCacheReader` are available for isolated testing
**And** mocks track method calls for verification in tests

**Given** the domain interfaces exist
**When** I validate the design
**Then** they follow hexagonal principles with clear separation between domain and infrastructure
**And** no infrastructure concerns (Redb, rkyv) leak into domain layer

**Given** Epic 5 established CacheQuery/CacheCommand split
**When** I review port design
**Then** storage ports follow same CQRS pattern for consistency
**And** CacheWriterPort handles commands, CacheReaderPort handles queries

## Story 9.2: Implement Redb + rkyv Storage Foundation

As a developer needing high-performance persistence,
I want Redb + rkyv implementation with memory bounds,
So that data is stored efficiently with zero-copy deserialization and controlled memory usage.

**Acceptance Criteria:**

**Given** Epic 5 provides RedbCache infrastructure
**When** I implement storage foundation in `crates/adapters/src/spi/storage/redb_storage.rs`
**Then** it reuses same Redb database instance from Epic 5 with separate table namespaces
**And** `notes` table uses `redb::TableDefinition<&str, &[u8]>` for path → serialized Note mapping

**Given** I need ACID transaction support per ADR 0002
**When** I implement storage operations
**Then** all writes occur within Redb transactions (`WriteTransaction`)
**And** MVCC concurrency allows multiple readers with single writer
**And** transaction isolation ensures consistent reads during concurrent operations

**Given** rkyv serialization is implemented for zero-copy access
**When** I serialize Note entities
**Then** `#[derive(Archive, Serialize, Deserialize)]` is added to Note domain model
**And** rkyv produces `AlignedVec<u8>` for optimal memory alignment
**And** deserialized data is accessed via `ArchivedNote` without heap allocation

**Given** zero-copy deserialization is critical for performance
**When** I implement read operations
**Then** `rkyv::check_archived_root::<Note>(bytes)` validates integrity before access
**And** `ArchivedNote` reference is wrapped in `Arc<Note>` for shared ownership
**And** no intermediate heap allocations occur during read path

**Given** storage operations run under load
**When** I monitor memory usage with 1000+ notes
**Then** total memory consumption stays within NFR9 bounds (500MB limit)
**And** memory profiling identifies no leaks or unbounded growth
**And** Arc reference counting prevents duplicate entity allocations

**Given** Epic 8 event bus requires async storage
**When** I implement storage adapters
**Then** all storage operations are async-compatible using `tokio::task::spawn_blocking`
**And** Redb blocking operations are offloaded to dedicated thread pool
**And** async/await integration doesn't block event bus reactor threads

**Given** NFR2 requires fast vault indexing
**When** I benchmark batch write operations
**Then** inserting 1000 notes completes in <2 seconds total
**And** individual write operations complete in <10ms average
**And** batch operations use single transaction for atomic commits

**Given** storage corruption must be detectable
**When** I implement data integrity checks
**Then** rkyv checksum validation catches bitrot and corruption
**And** Redb transaction log detects incomplete writes
**And** StorageError::Corruption is raised with clear diagnostic messages

## Story 9.3: Add Unit of Work Pattern for Transactions

As a developer ensuring data consistency,
I want Unit of Work pattern for atomic operations,
So that multiple storage operations are committed together or rolled back as a unit.

**Acceptance Criteria:**

**Given** I need transactional consistency for multi-table writes
**When** I implement Unit of Work pattern in `crates/adapters/src/spi/storage/transaction.rs`
**Then** `TransactionContext` struct wraps Redb `WriteTransaction` with domain-level operations
**And** transaction provides atomic writes across notes, schema_index, alias_index, metadata_index tables

**Given** Epic 10 indexing requires atomic multi-entity updates
**When** I use TransactionContext
**Then** it supports chaining operations: `tx.insert_note().update_schema_index().update_alias_index().commit()`
**And** `commit()` method flushes all pending writes atomically
**And** `rollback()` method discards all pending writes on error

**Given** Unit of Work must prevent partial writes
**When** batch operations execute
**Then** all writes succeed together or none persist (all-or-nothing semantics)
**And** transaction isolation level is READ_COMMITTED per Redb defaults
**And** concurrent readers see consistent snapshot during write transaction

**Given** CQRS pattern separates reads from writes
**When** I handle concurrent operations
**Then** write transactions acquire exclusive write lock
**And** read operations use MVCC snapshots without blocking writers
**And** no deadlocks occur between CacheWriterPort and CacheReaderPort operations

**Given** transactions are used
**When** errors occur mid-transaction (e.g., out of disk space, validation failure)
**Then** automatic rollback on `Drop` preserves data consistency
**And** StorageError propagates with context about failed operation
**And** database remains in valid state after error recovery

**Given** Epic 8 event bus triggers storage updates
**When** I integrate transactions with async context
**Then** `TransactionContext` is `Send + Sync` for cross-thread usage
**And** transaction lifetime is scoped to single async task
**And** long-running transactions don't block event processing

**Given** transactions must have bounded duration
**When** I implement timeout protection
**Then** transactions automatically rollback after 30-second timeout
**And** timeout errors are logged with transaction details for debugging
**And** timeout threshold is configurable via Epic 6 configuration system

## Story 9.4: Implement Storage Schema Design with Query Requirements

As a developer optimizing data access,
I want storage schema designed for query performance,
So that Epic 11 queries can be executed efficiently against the storage layout.

**Acceptance Criteria:**

**Given** Epic 11 requires multiple query access patterns
**When** I design storage schema in `crates/adapters/src/spi/storage/schema.rs`
**Then** four tables support different query patterns:
- **notes table**: `PathBuf (String) → Arc<Note> (rkyv bytes)` - primary entity storage
- **schema_index table**: `SchemaName (String) → Vec<PathBuf> (rkyv bytes)` - fileClass queries (FR21)
- **alias_index table**: `Alias (String) → PathBuf (String)` - wiki-link resolution (FR22)
- **metadata_index table**: `(FieldName, FieldValue) (Composite) → Vec<PathBuf> (rkyv bytes)` - metadata filtering (FR23)

**Given** Epic 11 Story 11.3 requires path-based lookups
**When** I optimize notes table
**Then** PathBuf keys use lexicographic ordering for range queries
**And** path lookups are O(log n) via Redb B-tree index
**And** notes table supports efficient iteration for full vault scans

**Given** Epic 11 Story 11.5 requires fileClass queries
**When** I implement schema_index table
**Then** each SchemaName key maps to all PathBuf entities with that fileClass
**And** schema filtering is O(1) lookup + O(k) iteration where k = result count
**And** index is updated atomically when notes change fileClass property

**Given** Epic 11 Story 11.6 requires alias resolution
**When** I implement alias_index table
**Then** alias lookups resolve in O(log n) time via Redb index
**And** alias conflicts (duplicate aliases) are detected during indexing
**And** alias_index updates when note frontmatter changes

**Given** Epic 11 Story 11.4 requires metadata filtering
**When** I implement metadata_index table
**Then** composite keys `(field_name, field_value)` enable efficient metadata queries
**And** queries like "all notes with tag=rust" are O(log n) lookup
**And** index supports multiple field queries via iterator composition

**Given** storage schema must handle schema-less notes
**When** I design index behavior
**Then** notes without fileClass property are omitted from schema_index
**And** notes without aliases are omitted from alias_index
**And** empty metadata fields don't create index entries

**Given** storage writes must maintain index consistency
**When** I implement write operations
**Then** notes table insert triggers automatic index updates in same transaction
**And** TransactionContext ensures indexes never become stale or inconsistent
**And** index update failures rollback entire transaction

**Given** schema design is complete
**When** I benchmark query performance with 1000+ notes
**Then** path lookups complete in <1ms (cache hit) or <10ms (cache miss)
**And** fileClass queries complete in <50ms for typical result sets
**And** metadata queries complete in <500ms meeting NFR1 requirements
**And** full vault iteration completes in <2 seconds meeting NFR2 requirements

## Story 9.5: Add Storage Validation and Error Handling

As a developer ensuring storage reliability,
I want comprehensive validation and error recovery,
So that storage corruption is detected and recovered gracefully.

**Acceptance Criteria:**

**Given** rkyv provides checksum validation
**When** I implement read operations in `crates/adapters/src/spi/storage/validator.rs`
**Then** `rkyv::check_archived_root::<T>(bytes)` validates data integrity before access
**And** corrupted bytes trigger `StorageError::Corruption` with diagnostic context (table, key, checksum)
**And** validation failures prevent unsafe memory access

**Given** Redb provides transaction integrity
**When** storage operations occur
**Then** incomplete writes are detected via transaction log validation
**And** Redb automatically rolls back partially-written transactions on crash
**And** database file integrity is verified on startup

**Given** Epic 10 may write invalid data structures
**When** I implement domain validation before persistence
**Then** Note entities are validated against domain rules before serialization
**And** invalid entities trigger `StorageError::ValidationFailed` with field-level errors
**And** validation prevents persisting invalid state

**Given** corruption is detected during read operations
**When** I implement recovery in `crates/adapters/src/spi/storage/recovery.rs`
**Then** clean slate protocol triggers automatic rebuild from vault markdown files
**And** recovery process scans `.lithos/backups/` for recent snapshot before full rebuild
**And** recovery preserves user data (vault files) while recreating corrupted indexes

**Given** clean slate protocol requires full vault re-index
**When** corruption recovery executes
**Then** Epic 10 indexing service is invoked to rebuild all tables
**And** recovery completes in <5 minutes for typical vaults (1000 notes)
**And** recovery progress is logged with percentage completion

**Given** storage errors occur (disk full, permissions, corruption)
**When** I handle them with miette-based diagnostics
**Then** `StorageError` enum provides variants:
- `Corruption { table, key, details }` - data integrity failure
- `TransactionFailed { operation, cause }` - transaction commit failure
- `OutOfSpace { required, available }` - disk space exhaustion
- `PermissionDenied { path }` - file system access denied
- `SchemaConflict { expected, actual }` - schema version mismatch

**Given** storage errors are raised
**When** errors propagate to application layer
**Then** miette diagnostic messages suggest recovery actions:
- Corruption → "Run `lithos rebuild-index` to recover"
- OutOfSpace → "Free up X MB of disk space"
- PermissionDenied → "Check file permissions on .lithos/ directory"

**Given** storage errors must not lose user data
**When** errors occur during write operations
**Then** automatic rollback ensures vault markdown files remain authoritative
**And** no user data is lost even on storage corruption
**And** worst case: storage rebuild from markdown takes <5 minutes

## Story 9.6: Implement Storage Backup and Corruption Recovery

As a developer protecting against data loss,
I want backup and recovery mechanisms,
So that storage corruption can be recovered without losing vault data.

**Acceptance Criteria:**

**Given** storage database can become corrupted
**When** I implement backup strategy in `crates/adapters/src/spi/storage/backup.rs`
**Then** periodic snapshots are created in `.lithos/backups/storage-{timestamp}.redb`
**And** backups are triggered every 100 write transactions or daily (whichever comes first)
**And** backup creation uses Redb's atomic snapshot feature without blocking operations

**Given** backup snapshots accumulate over time
**When** I implement retention policy
**Then** last 7 daily backups are retained (configurable via Epic 6)
**And** backups older than retention period are automatically pruned
**And** retention policy is enforced on startup and after each new backup

**Given** Epic 6 provides configuration system
**When** I integrate backup configuration
**Then** `global.toml` includes settings:
- `storage.backup.enabled = true` (default)
- `storage.backup.retention_days = 7` (default)
- `storage.backup.interval_transactions = 100` (default)
**And** backup behavior respects configuration at runtime

**Given** corruption is detected (via Story 9.5)
**When** I trigger recovery in `recovery.rs`
**Then** recovery attempts restoration from most recent backup snapshot
**And** backup restoration completes in <10 seconds for typical databases
**And** restoration validates backup integrity before replacing corrupted database

**Given** backup restoration fails (backup also corrupted)
**When** I implement fallback recovery
**Then** clean slate protocol triggers full vault re-index from markdown files
**And** Epic 10 indexing service rebuilds all storage tables from scratch
**And** clean slate recovery completes in <5 minutes for 1000-note vaults

**Given** clean slate protocol is the ultimate fallback
**When** I implement rebuild process
**Then** corrupted database is moved to `.lithos/corrupted/{timestamp}/` for forensics
**And** new empty database is created at `.lithos/storage.redb`
**And** Epic 10 indexing scans entire vault and rebuilds all indexes
**And** rebuild progress is logged with file count and percentage completion

**Given** backup/recovery is implemented
**When** I test disaster scenarios with integration tests
**Then** test cases validate:
- Corrupt database → backup restoration succeeds
- Corrupt backup → clean slate rebuild succeeds
- Missing database → clean slate rebuild succeeds
- Mid-transaction crash → Redb recovery succeeds
**And** all disaster scenarios recover without user data loss

## Story 9.7: Implement Storage Schema Migration and Evolution

As a developer updating storage requirements,
I want schema evolution capabilities,
So that storage format can change safely across versions without data loss.

**Acceptance Criteria:**

**Given** storage schema will evolve across lithos versions
**When** I implement migration framework in `crates/adapters/src/spi/storage/migrations.rs`
**Then** `SchemaVersion` metadata is stored in dedicated `_metadata` table
**And** current schema version is `const STORAGE_SCHEMA_V1: u32 = 1` for MVP
**And** version check occurs on database open before any operations

**Given** future versions may add new tables or fields
**When** I design migration infrastructure
**Then** `Migration` trait defines `fn up()` and `fn down()` methods for forward/backward migration
**And** migration registry maintains ordered list of migrations (V1→V2, V2→V3, etc.)
**And** migrations are executed sequentially in transaction for atomicity

**Given** rkyv serialization is not schema-flexible
**When** I plan for schema evolution
**Then** breaking changes require full data migration (deserialize old, serialize new)
**And** non-breaking changes (adding optional fields) use rkyv versioning features
**And** migration strategy is documented in `docs/storage-migrations.md` for future reference

**Given** migration may fail mid-process
**When** I implement migration safety
**Then** entire migration executes in single Redb transaction
**And** migration failure triggers automatic rollback to previous version
**And** failed migrations log diagnostic errors with rollback instructions

**Given** schema version mismatch is detected
**When** I implement version validation
**Then** newer schema version (future lithos) → error: "Database created by newer version, upgrade lithos"
**And** older schema version (past lithos) → automatic migration attempt
**And** schema version matches current → no migration needed

**Given** migrations must preserve user data integrity
**When** I implement migration validation
**Then** pre-migration backup is created automatically in `.lithos/backups/pre-migration-v{N}.redb`
**And** post-migration validation confirms data integrity (row counts, checksums)
**And** validation failure triggers automatic rollback and error report

**Given** schema evolution is complete for MVP
**When** I validate V1 schema stability
**Then** V1 schema supports all Epic 11 query requirements
**And** V1 schema has no planned breaking changes for MVP scope
**And** future schema changes (Phase 1.5+) will use migration framework

**Given** backward compatibility may be needed
**When** I implement rollback support
**Then** `down()` migrations allow schema downgrade for version rollback
**And** downgrade preserves data where possible, warns on data loss
**And** downgrade validation ensures older lithos versions can read downgraded schema

## Story 9.8: Implement Storage Performance Benchmarking

As a developer validating performance requirements,
I want comprehensive storage benchmarking,
So that NFR2 (2s vault indexing) and NFR9 (500MB memory) are validated at the storage layer.

**Acceptance Criteria:**

**Given** NFR2 requires 1000-note indexing in <2 seconds
**When** I implement write performance benchmarks in `crates/adapters/benches/storage_write.rs`
**Then** benchmark `batch_insert_1000_notes` measures total time for 1000 sequential inserts
**And** benchmark `batch_insert_parallel` measures time for batched transaction writes
**And** benchmark validates <2 second total time requirement

**Given** write performance must be optimized
**When** I analyze benchmark results
**Then** individual note writes complete in <10ms average
**And** batched writes (100 notes/transaction) complete in <500ms per batch
**And** parallel batch writes across multiple transactions complete in <2 seconds total

**Given** NFR1 requires query operations <500ms
**When** I implement read performance benchmarks in `crates/adapters/benches/storage_read.rs`
**Then** benchmark `lookup_by_path` measures single note retrieval time
**And** benchmark `query_by_schema` measures fileClass filtering performance
**And** benchmark `query_metadata` measures metadata filtering performance
**And** all query benchmarks validate <500ms requirement

**Given** read performance must be optimized
**When** I analyze query benchmarks
**Then** cache hit reads complete in <1ms (rkyv zero-copy access)
**And** cache miss reads complete in <10ms (Redb B-tree lookup)
**And** complex queries (metadata filtering) complete in <100ms typical, <500ms worst-case

**Given** NFR9 requires memory usage <500MB
**When** I implement memory benchmarks in `crates/adapters/benches/storage_memory.rs`
**Then** benchmark `memory_usage_1000_notes` measures peak memory during full vault load
**And** benchmark uses criterion with memory profiling enabled
**And** benchmark validates total memory (Redb cache + Arc<Note> references) stays <500MB

**Given** memory usage must be bounded
**When** I analyze memory benchmarks
**Then** rkyv zero-copy reduces heap allocations (no deserialization overhead)
**And** Arc<Note> sharing prevents duplicate entity allocations
**And** Redb cache size is configurable with 256MB default (Epic 6 config)
**And** total memory footprint: ~50KB per note × 1000 = 50MB + 256MB cache = 306MB < 500MB

**Given** benchmarks use criterion framework
**When** I implement benchmarks
**Then** criterion provides statistical analysis (mean, stddev, regression detection)
**And** benchmarks run with multiple iterations for reliable results
**And** benchmark results are tracked in git for regression detection

**Given** performance benchmarks run in CI
**When** I integrate with mise tasks
**Then** `mise run test:bench` executes all storage benchmarks
**And** CI fails if benchmarks regress beyond threshold (>10% slowdown)
**And** benchmark reports are generated in `target/criterion/` for analysis

**Given** performance benchmarks validate requirements
**When** I analyze results before Epic 11 integration
**Then** NFR2 (2s indexing) is validated with 1000-note dataset
**And** NFR1 (<500ms queries) is validated across all query types
**And** NFR9 (500MB memory) is validated under peak load
**And** performance regressions are caught before integration

## Story 9.9: Create Storage Mocks for Testing

As a developer testing storage-dependent code,
I want comprehensive storage mocks,
So that storage interactions can be tested in isolation without database setup.

**Acceptance Criteria:**

**Given** Epic 11 query service needs testable storage
**When** I create storage mocks in `crates/domain/src/ports/storage/mocks.rs`
**Then** `MockCacheWriter` implements `CacheWriterPort` with in-memory HashMap backend
**And** `MockCacheReader` implements `CacheReaderPort` with in-memory HashMap backend
**And** mocks are thread-safe using `Arc<RwLock<HashMap<PathBuf, Arc<T>>>>`

**Given** mocks must track method calls for verification
**When** I implement call tracking
**Then** mocks record method invocations: `insert_calls: Vec<(PathBuf, Arc<T>)>`
**And** test code can assert: `assert_eq!(mock.insert_calls.len(), 3)`
**And** call tracking helps verify integration behavior in tests

**Given** mocks must simulate realistic behavior
**When** I implement mock operations
**Then** `MockCacheWriter::insert()` stores data in HashMap and records call
**And** `MockCacheReader::get_by_path()` retrieves data from HashMap
**And** mocks support query operations (list_by_schema, query_metadata) via filtering

**Given** mocks need error simulation for resilience testing
**When** I implement error injection
**Then** mocks support `set_error_mode(StorageError)` to trigger failures
**And** error mode simulates: `Corruption`, `TransactionFailed`, `OutOfSpace`
**And** error simulation validates error handling in Epic 10/11 integration

**Given** mocks are used in unit tests
**When** I write storage-dependent tests
**Then** tests verify correct storage operations without real Redb database
**And** tests run faster (<30 seconds for full suite) without disk I/O
**And** tests are deterministic (no timing-dependent behavior)

**Given** Epic 10 indexing uses storage mocks
**When** I test indexing logic
**Then** mocks verify: indexer calls `batch_insert()` with correct note entities
**And** mocks verify: indexer updates schema_index and alias_index correctly
**And** mock call tracking validates indexing workflow without database

**Given** Epic 11 queries use storage mocks
**When** I test query logic
**Then** mocks verify: query service calls correct CacheReaderPort methods
**And** mocks provide test data for query filtering tests
**And** mock responses validate query result transformation logic

**Given** integration tests need realistic storage
**When** I implement test fixtures
**Then** mocks provide `with_test_data()` constructor preloaded with sample notes
**And** fixtures include: notes with/without schemas, duplicate aliases, complex metadata
**And** fixtures enable comprehensive integration testing without Redb setup

## Story 9.10: Storage Error Recovery and Data Integrity

As a user experiencing storage issues, I want the system to handle corruption, crashes, and recovery gracefully, so that my vault data remains safe and recoverable.
**Acceptance Criteria:**
**Given** storage corruption is detected
**When** the system attempts to read corrupted data
**Then** it provides clear error messages and recovery suggestions
**And** it can restore from backup or recreate corrupted indexes
**And** data integrity checks prevent silent corruption

**Given** storage operations fail mid-transaction
**When** the system recovers
**Then** it maintains ACID properties and data consistency
**And** failed operations are properly rolled back
**And** system state remains valid after recovery

## Story 9.11: Review Epic 9 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 9 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 9 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests

**Given** all Epic 9 public APIs are documented
**When** I verify doc test coverage
**Then** all public components have runnable doc tests demonstrating usage

**Given** all Epic 9 components are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate false positives, redundant tests, and inadequate edge case coverage

**Given** I take adversarial position against the test suite
**When** I critique test quality
**Then** I assess if tests actually validate business requirements vs implementation details

**Given** the test suite is implemented
**When** I review for redundancy
**Then** I eliminate duplicate test cases and consolidate overlapping coverage

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 9 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation

**Given** tests are written
**When** I review test documentation
**Then** all tests include BDD-style comments (GIVEN-WHEN-THEN)
**And** test names clearly describe behavior being tested
**And** any developer can understand test purpose without reading implementation
**And** BDD comments explain business context, not just technical steps

## Story 9.12: Document Storage System for Developers

As a developer working with data persistence,
I want comprehensive developer documentation for storage operations,
So that storage can be properly used and maintained across the application.

**Acceptance Criteria:**

**Given** storage system is implemented
**When** I create developer documentation
**Then** it includes storage operations, migration procedures, and performance characteristics

**Given** documentation exists
**When** developers read it
**Then** they understand storage operations and maintenance procedures

**Given** storage docs are complete
**When** other epics need storage integration
**Then** they can implement proper storage usage without architectural review
