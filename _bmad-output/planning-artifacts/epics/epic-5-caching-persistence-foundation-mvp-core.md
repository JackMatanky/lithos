# Epic 5: Caching & Persistence Foundation [MVP CORE]

## Overview

Establish a unified multi-layer caching architecture as a generic SPI utility for the lithos service. This epic implements the Cache trait interface, concrete implementations for Moka (memory cache) and Redb (disk cache), and a coordinator for read-through/write-through strategies. This foundation enables high-performance caching for schema resolution, configuration management, query results, and template execution.

**FRs covered:** Architecture requirements (caching infrastructure per ADR 0016)

## Implementation Notes

- **Architecture**: Hexagonal (Ports & Adapters) - Generic SPI utility similar to Epic 4's PathValidator/FormatDispatcher
- **Location**: `crates/adapters/src/spi/cache/` for cache implementations, `crates/adapters/src/spi/errors.rs` for shared errors
- **Libraries**: `moka` (memory), `redb` (disk), `rkyv` (Serialization), `async-trait`, `mockall`, `tracing`, `thiserror`
- **Pattern**: Read-through/Write-through caching with memory layer (fast access) + disk layer (persistent storage) coordination
- **ADR References**: ADR 0016 (Caching Strategy - Moka + Redb), ADR 0002 (Redb + rkyv storage)

## Story 5.1: Define Cache Trait and Error Hierarchy

As a developer building adapter-layer caching,
I want strictly typed, async traits for cache operations with comprehensive error handling,
So that multiple cache backends can be swapped and automatically mocked for testing without changing consumers.

**Acceptance Criteria:**

**Given** the adapter layer needs shared error types
**When** I define the `CacheError` enum in `spi/errors.rs` deriving `thiserror::Error`
**Then** it includes variants for common failure modes:

- `IoError(#[from] std::io::Error)` for file system failures
- `SerializationError(String)` for rkyv serialization/deserialization failures
- `BackendError(String)` for cache-specific errors (Moka eviction, Redb transaction failures)

**And** all variants implement `Send + Sync` to support async contexts
**And** error messages follow ADR 0006 (actionable diagnostics with context)

**Given** cache consumers need standardized operations
**When** I define `trait Cache<K, V>` in `spi/cache/mod.rs`
**Then** it includes these async methods:

- `async fn get(&self, key: &K) -> Result<Option<V>, CacheError>` - retrieve value by key
- `async fn put(&self, key: K, value: V) -> Result<(), CacheError>` - store key-value pair
- `async fn delete(&self, key: &K) -> Result<bool, CacheError>` - remove entry (returns true if existed)
- `async fn invalidate(&self, key: &K) -> Result<bool, CacheError>` - alias for delete (cache-specific terminology)

**And** the trait is annotated with `#[async_trait]` for async support

**Given** type safety is critical
**When** I define trait bounds
**Then** the trait requires:

- `K: Clone + Eq + Hash + Send + Sync + 'static` for hashable, thread-safe keys
- `V: Clone + Send + Sync + 'static` for thread-safe values

**And** documentation explains when `V: rkyv::Archive + rkyv::Serialize + rkyv::Deserialize` is needed (for RedbCache)

**Given** testing requires mock implementations
**When** I annotate the trait with `#[mockall::automock]`
**Then** a `MockCache<K, V>` struct is automatically generated at compile time
**And** the mock allows setting expectations on method calls (no manual mocks required)
**And** documentation includes example test using `MockCache` with expectations

**Given** the trait contract must be clear
**When** I write module-level documentation
**Then** it explains:

- Purpose: Generic caching SPI for adapter-layer use
- Consumers: Schema adapters, Config adapters, Query adapters
- Implementations: MokaCache (memory layer), RedbCache (disk layer), Coordinator (memory+disk)
- Error semantics: When each `CacheError` variant is returned

## Story 5.2: Implement Moka In-Memory Cache Adapter

As a system architect optimizing for high concurrency,
I want an in-memory `Moka` adapter implementing the `Cache` trait with observability,
So that frequently accessed data is served with sub-millisecond latency and all operations are traced.

**Acceptance Criteria:**

**Given** the `moka` crate dependency is added to `adapters/Cargo.toml`
**When** I implement the `MokaCache<K, V>` struct in `spi/cache/moka.rs`
**Then** it wraps `moka::future::Cache<K, V>` with configuration options:

- `max_capacity: usize` - maximum number of entries
- `time_to_live: Option<Duration>` - TTL for automatic expiration
- `time_to_idle: Option<Duration>` - TTI for idle eviction

**And** the struct provides a builder pattern for configuration

**Given** the adapter must implement the trait
**When** I implement `Cache<K, V>` for `MokaCache<K, V>`
**Then** all trait methods satisfy the async trait bounds
**And** `get()` returns `None` for cache misses, `Some(V)` for hits
**And** `put()` stores values respecting TTL/TTI policies
**And** `delete()` removes entries and returns true if key existed
**And** `invalidate()` delegates to `delete()` for semantic clarity

**Given** observability is required per project standards
**When** I instrument all public methods
**Then** each method is decorated with `#[tracing::instrument(skip(self, value), level = "debug")]`
**And** `get()` emits a `tracing::event!` with attributes:

- `cache_layer = "memory"`
- `operation = "get"`
- `hit = true/false`
  **And** `put()` emits events with `cache_layer = "memory"`, `operation = "put"`
  **And** `delete()` emits events with `cache_layer = "memory"`, `operation = "delete"`, `existed = true/false`

**Given** Moka's TinyLFU policy must be utilized
**When** I configure the cache
**Then** the default eviction policy is TinyLFU (Moka's default)
**And** documentation explains TinyLFU prevents scan pollution during vault indexing

**Given** the adapter must handle errors gracefully
**When** Moka operations fail (rare, but possible with listeners)
**Then** errors are mapped to `CacheError::BackendError` with descriptive context

## Story 5.3: Implement Redb Persistent Cache Adapter with Table Isolation

As a DevOps engineer requiring persistence,
I want a robust `Redb` adapter implementing the `Cache` trait with rkyv serialization and table isolation,
So that data persists across application restarts and multiple cache consumers can coexist without conflicts.

**Acceptance Criteria:**

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

## Story 5.4: Refactor Cache for Modularity and CQRS

As a system architect optimizing for modularity and scalability,
I want to refactor the cache implementations to use the Handle/Inner pattern and separate Reader/Writer traits,
So that the codebase is more maintainable, supports zero-copy operations more effectively, and adheres to CQRS principles.

**Acceptance Criteria:**

**Given** the need for better modularity
**When** I implement the `CacheCodec` trait in `deserializer.rs`
**Then** it provides a unified interface for serialization/deserialization
**And** `RkyvCodec` implements this trait for zero-copy Redb storage

**Given** the need for a common builder interface
**When** I create `builder.rs`
**Then** it defines a `CacheBuilder` trait with a `build()` method
**And** both Moka and Redb provide builders implementing this trait

**Given** the Handle/Inner pattern is required
**When** I refactor `MokaCache` and `RedbCache`
**Then** the implementation is split into `Inner` structs (state) and Handles (interface)
**And** Handles are cheaply cloneable `Arc` wrappers

**Given** CQRS principles
**When** I implement separate `Reader` and `Writer` handles
**Then** consumers can request read-only or write-only access to the cache
**And** the `CacheCoordinator` uses these separate handles for orchestration

**Given** the refactor must maintain quality
**When** I follow strict TDD
**Then** tests are written before implementation for each phase
**And** zero-copy verification ensures `rkyv` is used correctly in the new structure

## Story 5.5: Implement Cache Coordinator for Memory/Disk Read-Through and Write-Through

As a system architect ensuring consistency and extreme performance,
I want a `CacheCoordinator` split into Reader and Writer handles that orchestrates memory and disk cache access,
So that cache hits are served fast, consistency is guaranteed, and the system follows strict CQRS principles with decoupled background backfill.

**Acceptance Criteria:**

**Given** coordinated caching requires both layers
**When** I implement `CoordinatorInner<K, V>` in `spi/cache/coordinator.rs`
**Then** it leverages the modular `Reader` and `Writer` handles from Story 5.4
**And** it encapsulates:
- `memory_reader: Arc<dyn CacheReaderPort<K, V>>`
- `memory_writer: Arc<dyn CacheWriterPort<K, V>>`
- `disk_reader: Arc<dyn CacheReaderPort<K, V>>`
- `disk_writer: Arc<dyn CacheWriterPort<K, V>>`

**Given** the need for CQRS consistency
**When** I implement `CacheCoordinatorReader` and `CacheCoordinatorWriter`
**Then** they share the `CoordinatorInner` state via `Arc`
**And** the `Reader` handle ONLY implements the `CacheReaderPort` trait
**And** the `Writer` handle ONLY implements the `CacheWriterPort` trait

**Given** read-through caching must be high-performance
**When** I implement `get()` for the coordinator reader
**Then** the flow is:
1. Check memory cache via `memory_reader`.
2. **Memory Hit**: Return value immediately; emit `tracing::event!` at `Level::DEBUG` with "Memory Hit".
3. **Memory Miss**: Check disk cache via `disk_reader`.
4. **Disk Hit**:
    - Trigger an **Asynchronous Backfill** to memory via an internal `mpsc` channel.
    - Emit `tracing::event!` at `Level::INFO` with "Memory Miss / Disk Hit".
    - Return the disk value to the caller IMMEDIATELY without waiting for the memory write to complete.
5. **Disk Miss**: Emit `tracing::event!` at `Level::INFO` with "Disk Miss"; return `None`.

**Given** write-through caching must ensure consistency
**When** I implement `put()` for the coordinator writer
**Then** the flow is:
1. Attempt write to disk via `disk_writer` to ensure persistence first.
2. **Disk Success**: Proceed to write the value to the memory layer via `memory_writer`.
3. **Disk Failure**: Return the error immediately and PREVENT writing to memory (ensuring the cache does not contain data that failed to persist).
4. Emit `tracing::event!` at `Level::DEBUG` with "Cache Write" including key (if serializable).

**Given** invalidation must affect both layers
**When** I implement `delete()`, `invalidate()`, and `clear()`
**Then** the operations are coordinated across both memory and disk layers.
**And** for `delete` and `clear`, both layers are invalidated in parallel (best effort) using `tokio::join!` to minimize latency.
**And** `delete`/`invalidate` returns true if the key existed in either layer.

**Given** observability is critical for debugging
**When** I trace coordinator operations
**Then** spans nest correctly: `coordinator` → `memory operation` → `disk operation`
**And** backfill events are emitted with `operation = "backfill"` and `status = "triggered"`

## Story 5.6: Cache Performance & Zero-Copy Refactor

As a performance engineer optimizing the storage layer,
I want to implement a "Guard-Based Trait" design and leverage advanced crate features,
So that we achieve zero-copy reads/writes and significant performance improvements.

**Context:**
We have completed a deep analysis of `redb`, `moka`, and `rkyv` and identified significant performance gaps in the current implementation. We need a dedicated story to execute the "Guard-Based Trait" design (Level 2 architecture) and leverage advanced crate features.

**Acceptance Criteria:**

**Given** the need for zero-copy reads
**When** I implement the `CacheReader` trait
**Then** it returns `Result<Option<Self::Guard>>`
**And** `redb` returns `AccessGuard` wrappers (zero-copy aligned or single-copy unaligned)
**And** `moka` returns `Arc` wrappers without allocation

**Given** the need for zero-copy writes
**When** I update the `Codec`
**Then** it supports `serialize_into` and `serialized_size`
**And** `redb` writes use `insert_reserve` to write directly to the memory-mapped file

**Given** the need for efficient batch processing
**When** I implement operations on `redb`
**Then** it supports single-transaction `get_many` and `put_many`
**And** benchmarks show ~10x speedup for batch scans

**Given** alignment requirements in `redb`
**When** I access data
**Then** the implementation handles alignment correctly (fallback to copy for unaligned reads)

**Given** Moka optimization goals
**When** I update the `moka` implementation
**Then** it stores `Entry<V>` instead of `V` to enable zero-copy timestamp checks
**And** it exposes `metrics()` and explicit maintenance hooks
**And** benchmarks show ~47x speedup for timestamp checks

**Given** architectural requirements for the Guard pattern
**When** I refactor `coordinator.rs`
**Then** it uses monomorphic generics instead of trait objects

**Given** existing functionality
**When** I run the test suite
**Then** all existing tests pass

## Story 5.7: Implement Performance Benchmarking Suite

As a performance engineer validating cache performance,
I want comprehensive benchmarks using `criterion`,
So that I can verify throughput, latency, and memory usage meet requirements.

**Acceptance Criteria:**

**Given** benchmarking infrastructure exists per ADR 0012
**When** I create `benches/cache_benchmarks.rs` in the adapters crate
**Then** it includes benchmark suites for:

- `MokaCache` standalone operations
- `RedbCache` standalone operations
- `CacheCoordinator` full memory+disk flow

**Given** throughput is critical for LSP scenarios
**When** I benchmark `MokaCache` concurrent operations
**Then** the benchmark:

- Spawns 100 concurrent tasks performing mixed get/put operations
- Runs 1000 operations per second
- Measures p50, p95, p99 latency
- Reports ops/sec throughput

**And** p99 latency is <5ms for get() operations
**And** p99 latency is <10ms for put() operations

**Given** cold start performance matters for CLI
**When** I benchmark `RedbCache` initialization
**Then** the benchmark:

- Measures database open + table creation time
- Measures first read after cold start

**And** database open completes in <10ms

**Given** memory usage must stay within bounds
**When** I benchmark `CacheCoordinator` with large datasets
**Then** the benchmark:

- Caches 10,000 entries of typical size (e.g., 1KB each)
- Measures peak memory usage for memory + disk layers combined

**And** memory usage stays below 100MB (memory layer typically capped at 50MB, disk layer is file-backed)

**Given** scan resistance is a key Moka feature
**When** I benchmark scan scenarios
**Then** the benchmark:

- Performs 10,000 sequential reads (simulating vault scan)
- Followed by 1,000 random reads from a small "hot set"
- Measures cache hit rate for the hot set

**And** TinyLFU policy maintains >80% hit rate for hot data despite scan pollution

**Given** results must be tracked over time
**When** benchmarks are run
**Then** criterion generates statistical reports comparing against baseline
**And** reports are saved to `target/criterion/` for CI integration

## Story 5.8: Review Epic 5 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 5 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before adapter integration.

**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guides during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 5 public components are implemented (Cache trait, MokaCache, RedbCache, Coordinator)
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests
**And** each `CacheError` variant has a test case ensuring proper error propagation

**Given** all Epic 5 public APIs are documented
**When** I verify doc test coverage
**Then** all public components (traits, structs, enums, methods) have runnable doc tests in `# Examples` sections demonstrating usage
**And** doc tests cover both success cases and error handling
**And** doc tests compile and pass when run via `cargo test --doc`

**Given** all Epic 5 components are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate:

- False positives (tests that pass but don't validate behavior)
- Redundant tests (duplicate coverage)
- Inadequate edge case coverage (error paths, boundary conditions)

**Given** I take adversarial position against the test suite
**When** I critique test quality
**Then** I assess if tests validate actual cache behavior vs implementation details
**And** tests verify contract adherence (trait semantics) not internal state

**Given** the test suite is implemented
**When** I review for redundancy
**Then** I eliminate duplicate test cases and consolidate overlapping coverage

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 5 suite

**Given** concurrency is critical per ADR 0016
**When** I test MokaCache and Coordinator
**Then** tests include concurrent read/write scenarios with 100+ spawned tasks
**And** tests verify no data races or deadlocks under load using `tokio::test` with multi-threaded runtime

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify:

- Tests use proper fixtures (test data builders, sample types)
- Tests avoid flaky behavior (no timing dependencies, no hard-coded sleep)
- Test intent is clear (descriptive names, Given/When/Then structure in comments)

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code
**And** complex test scenarios have inline documentation explaining setup

**Given** mocking is critical for isolation
**When** I review mock usage
**Then** `MockCache` is used appropriately in coordinator tests to isolate memory/disk behavior
**And** no manual mocks exist (all use `#[mockall::automock]`)

**Given** documentation quality is critical
**When** I review all doc comments
**Then** every public component has:

- Precise `///` doc comments explaining purpose and behavior
- Well-written doc tests in `# Examples` sections
- Error cases documented with `# Errors` sections where applicable
- Panic conditions documented with `# Panics` sections where applicable

**And** doc tests demonstrate realistic usage patterns
**And** doc comments follow project standards from `project-context.md`

**Given** RedbCache persistence must be validated
**When** I test persistence behavior
**Then** integration tests verify:

- Value survives process restart (create cache, write, drop, recreate, read)
- rkyv serialization round-trips correctly for complex types
- Metadata is preserved across reads/writes

**Given** tests are written
**When** I review test documentation
**Then** all tests include BDD-style comments (GIVEN-WHEN-THEN)
**And** test names clearly describe behavior being tested
**And** any developer can understand test purpose without reading implementation
**And** BDD comments explain business context, not just technical steps

## Story 5.9: Document Cache System Foundation

As a developer integrating caching in adapter implementations,
I want clear documentation for the Cache SPI with concrete examples and comprehensive doc comments,
So that I understand how to use the generic primitives in domain-specific contexts.

**Acceptance Criteria:**

**Given** all Epic 5 code is implemented
**When** I review all doc comments
**Then** they are accurate, precise, and follow project standards from `project-context.md`
**And** every public component uses proper `///` documentation format

**Given** all Epic 5 public components are documented
**When** I verify doc comments
**Then** all public traits, structs, enums, functions, and methods have:

- Clear `///` doc comments explaining their purpose
- `# Examples` sections with runnable, well-written doc tests
- `# Errors` sections documenting error conditions where applicable
- `# Panics` sections documenting panic conditions where applicable

**And** doc tests demonstrate realistic usage patterns
**And** doc tests compile and pass via `cargo test --doc`

**Given** the Cache SPI is implemented
**When** I create `crates/adapters/src/spi/cache/README.md`
**Then** it includes:

- **Overview**: Purpose of the Cache SPI as generic infrastructure
- **Trait Contract**: Explanation of `Cache<K, V>` methods and semantics
- **Implementations**: MokaCache (memory), RedbCache (disk), Coordinator (memory+disk)
- **Example 1**: Using RedbCache with table isolation for configuration storage
- **Example 2**: Using Coordinator for schema caching with metadata tracking
- **rkyv Requirements**: Types cached in RedbCache must derive Archive + Serialize + Deserialize
- **Table Naming Conventions**: Suggested patterns (e.g., "schemas", "config", "query_results")

**Given** developers need architectural context
**When** I create `docs/spi/cache-foundation.md`
**Then** it explains:

- **Memory/Disk Architecture**: Why we use two-level caching (speed vs persistence)
- **When to Use What**:
  - MokaCache alone: Temporary session data, template execution caching
  - RedbCache alone: Persistent data without frequent access (cold storage)
  - Coordinator: High-performance persistent caching (schemas, config)
- **Metadata Storage**: How to use `CachedEntry` metadata for versioning, hash tracking, rollback
- **Integration Patterns**: How adapter implementations compose Cache primitives
- **Performance Characteristics**: Latency targets, memory bounds, concurrency behavior

**Given** examples must be runnable
**When** I include code examples in documentation
**Then** they compile and demonstrate:

- Creating a MokaCache with TTL configuration
- Creating a RedbCache with table isolation
- Composing a Coordinator with both layers
- Using metadata for hash-based invalidation
- Using metadata for versioned snapshots
