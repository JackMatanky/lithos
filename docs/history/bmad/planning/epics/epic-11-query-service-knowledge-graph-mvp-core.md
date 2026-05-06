# Epic 11: Query Service & Knowledge Graph **[MVP CORE]**

## Overview

Users can perform fast lookups by filename, path, or schema keys, resolve wiki-links and aliases, and query metadata from other notes for template use.

**FRs covered:** FR21 (schema-based queries), FR22 (wiki-link resolution), FR23 (metadata access), NFR1 (<500ms query performance)

## Implementation Notes

- **Query Architecture**: CQRS read side (Epic 10 is write side for indexing)
- **QueryPort**: Domain interface for query operations (trait in `crates/domain/src/ports/query.rs`)
- **Query Service**: Application orchestrator in `crates/app/src/services/query_service.rs`
- **Integration Points**:
  - Epic 9: Uses CacheReaderPort for storage access (zero-copy rkyv reads)
  - Epic 8: Subscribes to NoteIndexed/NoteDeleted events for cache invalidation
  - Epic 10: Reads indexed data persisted by indexing service
  - Epic 12: Provides query results for template variable resolution
- **Query Types Supported**:
  - **Path Lookup**: Direct note retrieval by file path (O(log n) via Redb index)
  - **Schema Filtering**: Find all notes with specific fileClass (FR21)
  - **Metadata Queries**: Filter notes by frontmatter fields (FR23)
  - **Alias Resolution**: Resolve [[wiki-link]] to actual file path (FR22)
  - **Composition**: Combine multiple query conditions with AND/OR logic
- **Performance Targets** (NFR1):
  - Path lookups: <1ms (cache hit), <10ms (cache miss)
  - Schema queries: <50ms typical, <500ms worst-case
  - Metadata queries: <100ms typical, <500ms worst-case
  - Alias resolution: <5ms (hash map lookup)
- **Caching Strategy**:
  - LRU cache for frequently accessed query results
  - TTL-based expiration (default 60 seconds, configurable via Epic 6)
  - Event-driven invalidation on NoteIndexed/NoteDeleted events
  - Cache hit rate target: >90% for repeated queries
- **Result Formatting**: Query results formatted as lists, tables, or structured data for CLI/template use
- **Query Mocks**: Test doubles in `crates/domain/src/ports/query/mocks.rs` for isolated testing
- **Observability**: Query performance metrics tracked (latency, cache hit rate, error rate) with `#[tracing::instrument]` per architecture.md FR40
- **Tracing Levels**: `debug` for query operations/cache hits/misses, `warn` for slow queries (>500ms), `error` for query failures
- **Location**: `crates/app/src/services/query_service.rs`, `crates/domain/src/ports/query.rs`
- **Error Handling**: QueryError enum with variants for NotFound, InvalidFilter, StorageError, TimeoutExceeded

## Story 11.1: Create Query Domain Interface and Port

As a developer implementing query operations,
I want clean domain interfaces for query access,
So that queries follow hexagonal architecture principles.

**Acceptance Criteria:**

**Given** I need query operation contracts following CQRS read-side pattern
**When** I create QueryPort trait in `crates/domain/src/ports/query.rs`
**Then** it defines async methods for all query types:
- `async fn get_by_path(&self, path: &Path) -> Result<Option<Arc<Note>>, QueryError>` (FR21)
- `async fn list_by_schema(&self, schema: &str) -> Result<Vec<Arc<Note>>, QueryError>` (FR21)
- `async fn query_metadata(&self, field: &str, value: &str) -> Result<Vec<Arc<Note>>, QueryError>` (FR23)
- `async fn resolve_alias(&self, alias: &str) -> Result<Option<PathBuf>, QueryError>` (FR22)
- `async fn compose_query(&self, conditions: QueryConditions) -> Result<Vec<Arc<Note>>, QueryError>`

**Given** QueryPort must integrate with Epic 12 templates
**When** I design return types
**Then** query results use `Arc<Note>` for zero-copy sharing across template execution
**And** `Arc` prevents cloning large note entities during template rendering
**And** Note entities include all frontmatter fields for metadata access (FR23)

**Given** queries must support complex filtering
**When** I define QueryConditions domain type
**Then** it represents composable query logic:
- `And(Vec<QueryCondition>)` - all conditions must match
- `Or(Vec<QueryCondition>)` - any condition matches
- `FieldEquals(field, value)` - exact metadata match
- `SchemaMatches(schema)` - fileClass filtering
- `PathMatches(glob)` - path pattern matching

**Given** queries must handle errors gracefully
**When** I define QueryError enum
**Then** it includes variants:
- `NotFound { path }` - queried note doesn't exist
- `InvalidFilter { reason }` - malformed query condition
- `StorageError { cause }` - underlying storage failure
- `TimeoutExceeded { elapsed }` - query exceeded NFR1 limit
- `CacheInvalidationFailed { details }` - event integration failure

**Given** QueryPort is defined
**When** I implement mocks in `crates/domain/src/ports/query/mocks.rs`
**Then** `MockQueryPort` provides in-memory query simulation with call tracking
**And** mocks record query invocations: `query_calls: Vec<(QueryType, Params)>`
**And** mocks support error injection for resilience testing

**Given** the domain interface exists
**When** I validate the design
**Then** it follows hexagonal principles with no infrastructure leakage
**And** QueryPort has no dependencies on Redb, rkyv, or other adapter concerns
**And** application layer (Epic 12) depends only on QueryPort trait, not concrete implementations

## Story 11.2: Integrate Query Service with Storage Layer

As a developer coordinating queries with persistence,
I want query service integrated with storage layer,
So that queries retrieve data from the persisted index efficiently.

**Acceptance Criteria:**

**Given** Epic 9 provides CacheReaderPort for storage access
**When** I implement QueryService in `crates/app/src/services/query_service.rs`
**Then** it depends on `Arc<dyn CacheReaderPort<Note>>` injected via constructor
**And** QueryService implements QueryPort trait using storage backend
**And** all query operations delegate to storage layer via CacheReaderPort

**Given** Epic 9 storage uses rkyv zero-copy deserialization
**When** I implement query operations
**Then** `get_by_path()` directly returns `Arc<Note>` from storage without copying
**And** `list_by_schema()` wraps storage results in `Arc` for zero-copy sharing
**And** storage layer provides `Arc<T>` references, not owned values

**Given** Epic 9 storage provides multiple access patterns
**When** I map QueryPort methods to storage operations
**Then** `get_by_path()` → `CacheReaderPort::get_by_path()` (notes table lookup)
**And** `list_by_schema()` → `CacheReaderPort::list_by_schema()` (schema_index table scan)
**And** `query_metadata()` → `CacheReaderPort::query_metadata()` (metadata_index table query)
**And** `resolve_alias()` → `CacheReaderPort::resolve_alias()` (alias_index table lookup)

**Given** storage integration works
**When** I handle large result sets (1000+ notes matching schema)
**Then** queries use iterator-based access to avoid loading all results into memory
**And** pagination support limits result sets to configurable size (default 100, max 1000)
**And** memory usage stays within NFR9 bounds (500MB) even for large queries

**Given** storage performance varies by query type
**When** I implement performance optimization
**Then** path lookups leverage Redb B-tree index for O(log n) access
**And** schema queries leverage schema_index for O(1) lookup + O(k) iteration
**And** metadata queries leverage metadata_index composite keys for efficient filtering
**And** alias resolution uses hash map for O(1) average-case lookup

**Given** integration is complete
**When** I validate data consistency
**Then** queries return data matching the indexed state from Epic 10
**And** query results reflect most recent NoteIndexed events
**And** event-driven cache invalidation (Story 11.7) ensures consistency

**Given** storage operations may fail
**When** I handle storage errors
**Then** `CacheReaderPort` errors are wrapped in `QueryError::StorageError`
**And** transient failures (timeout, lock contention) trigger automatic retry (3 attempts)
**And** permanent failures (corruption) propagate to caller with diagnostic context

## Story 11.3: Implement Basic Query Operations

As a user needing to find notes,
I want basic lookup operations by filename and path,
So that I can quickly locate specific notes in the vault.

**Acceptance Criteria:**

**Given** FR21 requires note lookup by path
**When** I implement `get_by_path()` in QueryService
**Then** it queries Epic 9 notes table using PathBuf key
**And** returns `Some(Arc<Note>)` if note exists, `None` if not found
**And** path normalization ensures case-sensitive vault paths match correctly

**Given** users may query by partial paths
**When** I implement path matching
**Then** query supports glob patterns: `notes/**/*.md`, `projects/*/README.md`
**And** glob matching uses `globset` crate for efficient pattern evaluation
**And** glob queries scan notes table with path prefix optimization

**Given** basic queries work
**When** I test with indexed data from Epic 10
**Then** queries return Note entities with complete frontmatter fields
**And** Note entities include: path, content, frontmatter, schema, created_at, modified_at
**And** results reflect most recent indexing from Epic 10 indexing service

**Given** lookups must be performant per NFR1
**When** I validate performance with benchmarks
**Then** exact path lookups complete in <1ms (cache hit) or <10ms (cache miss)
**And** glob pattern queries complete in <100ms for typical vaults (1000 notes)
**And** worst-case glob queries complete in <500ms meeting NFR1 requirement

**Given** path lookups may fail
**When** I handle edge cases
**Then** non-existent paths return `None` without error
**And** invalid paths (path traversal attempts) return `QueryError::InvalidFilter`
**And** storage errors propagate as `QueryError::StorageError`

## Story 11.4: Implement Schema-Based Query Filtering

As a user organizing notes by schema,
I want to filter queries by schema keys and metadata,
So that I can find notes with specific properties or schemas (when schemas are used).

**Acceptance Criteria:**

**Given** FR21 requires schema-based note filtering
**When** I implement `list_by_schema(schema: &str)` in QueryService
**Then** it queries Epic 9 schema_index table using schema name as key
**And** returns `Vec<Arc<Note>>` for all notes with matching fileClass
**And** empty result set returned if no notes match schema

**Given** Epic 9 schema_index optimizes fileClass queries
**When** I leverage storage optimization
**Then** schema queries are O(1) lookup + O(k) iteration where k = result count
**And** schema_index is maintained by Epic 10 indexing when note fileClass changes
**And** query performance is <50ms typical, <500ms worst-case per NFR1

**Given** schema filtering works
**When** I test with different schemas (e.g., "contact", "project", "daily-note")
**Then** results include only notes with exact fileClass match
**And** schema names are case-sensitive per domain rules
**And** schema inheritance (if implemented in future) is respected in query results

**Given** users may query metadata fields beyond fileClass
**When** I implement metadata filtering
**Then** `query_metadata(field, value)` queries Epic 9 metadata_index table
**And** metadata queries support frontmatter fields: tags, author, status, etc.
**And** metadata queries work independently of schema system

**Given** filtering is implemented
**When** I validate edge cases
**Then** queries handle notes without fileClass gracefully (empty result for schema queries)
**And** queries handle notes without schemas by using direct frontmatter field queries
**And** missing metadata fields return empty result set, not errors

**Given** users don't use schemas
**When** they run metadata queries
**Then** `query_metadata()` works through direct frontmatter field matching
**And** fileClass is optional: metadata queries work without schema system
**And** query performance is identical whether schemas are used or not

## Story 11.5: Implement File Class Query Operations

As a user categorizing notes by type,
I want to query notes by fileClass for schema-based organization,
So that I can find all "contact" notes or "project" notes efficiently.

**Acceptance Criteria:**

**Given** FR21 requires fileClass as primary categorization mechanism
**When** I implement fileClass queries
**Then** `list_by_schema(schema)` is the primary interface for fileClass filtering
**And** fileClass query implementation is identical to schema queries (Story 11.4)
**And** fileClass is stored in note frontmatter as `fileClass: "contact"` field

**Given** Epic 9 schema_index optimizes fileClass lookups
**When** I leverage storage design
**Then** fileClass queries use schema_index table for O(1) + O(k) performance
**And** schema_index is updated atomically when note fileClass changes
**And** index consistency is guaranteed via Epic 9 Unit of Work pattern

**Given** fileClass queries return large result sets
**When** I validate performance with 1000+ notes
**Then** queries returning 100 notes complete in <50ms
**And** queries returning 500 notes complete in <200ms
**And** worst-case queries (1000+ results) complete in <500ms meeting NFR1

**Given** Epic 7 defines schema validation rules
**When** I implement fileClass queries
**Then** queries respect validated fileClass values from schema definitions
**And** invalid fileClass values (not in schema) are not indexed by Epic 10
**And** query results include only schema-compliant notes

**Given** schema inheritance may be added in future (Phase 1.5+)
**When** I design for extensibility
**Then** fileClass query interface supports future schema hierarchy traversal
**And** current implementation treats fileClass as flat categorization
**And** inheritance logic (if added) would be handled in Epic 7 schema system, not query layer

**Given** fileClass operations are implemented
**When** I test with realistic vault data
**Then** common queries ("contact", "project", "daily-note") complete in <20ms typical
**And** rare queries (uncommon fileClass) complete in <50ms
**And** no query type exceeds 500ms threshold per NFR1

## Story 11.6: Add Wiki-Link and Alias Resolution

As a user working with interconnected notes,
I want wiki-links and aliases resolved to actual note paths,
So that links work correctly across the knowledge graph.

**Acceptance Criteria:**

**Given** FR22 requires wiki-link resolution for interconnected notes
**When** I implement `resolve_alias(alias: &str)` in QueryService
**Then** it queries Epic 9 alias_index table using alias string as key
**And** returns `Some(PathBuf)` if alias exists, `None` if not found
**And** alias resolution completes in <5ms per NFR1 (hash map O(1) lookup)

**Given** wiki-links use [[link]] syntax
**When** I implement wiki-link parsing
**Then** link text is extracted: `[[Contact Note]]` → `"Contact Note"` alias
**And** link resolution uses alias_index for path lookup
**And** resolved paths are absolute vault-relative: `contacts/contact-note.md`

**Given** notes may have multiple aliases
**When** I handle alias indexing (Epic 10 responsibility)
**Then** frontmatter aliases field: `aliases: ["Contact Note", "CN"]` creates multiple index entries
**And** each alias maps to same target PathBuf in alias_index
**And** alias conflicts (duplicate aliases pointing to different notes) are detected during indexing

**Given** alias resolution is implemented
**When** I handle alias conflicts
**Then** `resolve_alias()` returns first indexed note if conflict exists
**And** conflict warnings are logged by Epic 10 indexing service
**And** users are notified of duplicate aliases via Epic 14 CLI warnings

**Given** wiki-links may include section headers
**When** I implement link parsing
**Then** `[[Note#Section]]` syntax is parsed as alias + fragment
**And** alias portion `"Note"` is resolved to PathBuf
**And** fragment portion `"Section"` is preserved for caller (Epic 12) to handle
**And** query layer only resolves alias, not section navigation

**Given** links may use display text
**When** I handle link syntax variations
**Then** `[[Alias|Display Text]]` is parsed to extract `"Alias"` for resolution
**And** display text is ignored by query layer (relevant for rendering, not resolution)
**And** all standard Obsidian wiki-link formats are supported

**Given** link resolution works
**When** I validate completeness with test cases
**Then** all wiki-link patterns resolve correctly:
- `[[Simple Link]]` → alias lookup
- `[[Alias|Display]]` → alias lookup (display ignored)
- `[[Note#Section]]` → alias lookup + fragment return
- `[[path/to/note]]` → path-based lookup fallback if no alias
**And** unresolved links return `None` without error

**Given** alias_index may become stale
**When** I integrate with Epic 8 events
**Then** NoteIndexed events trigger alias_index updates
**And** NoteDeleted events remove aliases from index
**And** alias resolution always reflects most recent indexed state

## Story 11.7: Implement Query Cache Invalidation via Events

As a developer maintaining query performance,
I want cache invalidation through event system,
So that query results stay current when index updates occur.

**Acceptance Criteria:**

**Given** Epic 8 provides event bus for system coordination
**When** I integrate QueryService with event system
**Then** QueryService subscribes to `NoteIndexed` and `NoteDeleted` events on Broadcast channel
**And** event subscription happens during QueryService initialization
**And** event handlers run on dedicated async task to avoid blocking queries

**Given** query results are cached for performance (Story 11.8)
**When** I implement cache invalidation
**Then** `NoteIndexed` event invalidates cached queries affecting that note:
- Path-based cache entries for affected PathBuf
- Schema-based cache entries if note fileClass changed
- Metadata cache entries if note frontmatter changed
- Alias cache entries if note aliases changed
**And** invalidation is selective: only affected cache entries are cleared

**Given** NoteDeleted events indicate removed notes
**When** I handle deletion events
**Then** all cache entries referencing deleted PathBuf are invalidated
**And** alias_index entries for deleted note are cleared
**And** schema_index entries are updated to remove deleted note

**Given** cache invalidation must be reliable
**When** I implement event handling
**Then** event processing uses `at-least-once` delivery semantics
**And** duplicate events are idempotent (safe to process multiple times)
**And** event processing failures are logged with retry logic (3 attempts)

**Given** event integration works
**When** I test cache consistency with integration tests
**Then** test scenario: index note → query → update note → re-query → results reflect update
**And** cache invalidation completes in <10ms per event
**And** queries after invalidation fetch fresh data from storage

**Given** invalidation is implemented
**When** I monitor cache performance metrics
**Then** cache hit rates remain >90% even with frequent updates
**And** selective invalidation minimizes cache churn
**And** cache effectiveness is tracked: hits, misses, invalidations, evictions

**Given** Epic 8 event bus may experience lag under load
**When** I handle event processing delays
**Then** query results may be slightly stale (eventual consistency model)
**And** staleness is bounded by event processing latency (<50ms per Epic 8)
**And** critical queries can bypass cache for strong consistency if needed

## Story 11.8: Add Query Performance Optimization and Caching

As a developer optimizing query speed,
I want performance optimization with intelligent caching,
So that queries complete in <500ms meeting NFR1 requirements.

**Acceptance Criteria:**

**Given** NFR1 requires query performance <500ms
**When** I implement query caching in `crates/app/src/services/query_cache.rs`
**Then** LRU cache stores frequently accessed query results with configurable capacity
**And** cache key is hash of query parameters: `(query_type, params)` → `Vec<Arc<Note>>`
**And** cache uses `moka` crate for concurrent LRU implementation with async support

**Given** caching is implemented
**When** I configure cache policies via Epic 6
**Then** `query.cache.enabled = true` (default)
**And** `query.cache.max_entries = 1000` (default, configurable)
**And** `query.cache.ttl_seconds = 60` (default, configurable)
**And** cache respects configuration at runtime without restart

**Given** query cache must balance freshness and performance
**When** I set TTL policies
**Then** cache entries expire after TTL seconds (default 60s)
**And** expired entries are lazily evicted on next access
**And** TTL prevents stale data even without event-driven invalidation (Story 11.7)

**Given** cache must stay within memory bounds
**When** I implement memory management
**Then** LRU eviction removes least-recently-used entries when capacity reached
**And** cache memory usage is bounded: ~1KB per entry × 1000 entries = 1MB typical
**And** cache stays within NFR9 memory budget (500MB total, cache is small fraction)

**Given** optimization works
**When** I benchmark queries with criterion
**Then** cache hit queries complete in <1ms (no storage access)
**And** cache miss queries complete in <10ms (storage access + cache population)
**And** average query time across cache hits/misses is <50ms typical, <500ms worst-case
**And** cache hit rate exceeds 90% for typical workloads

**Given** performance is validated
**When** I implement observability metrics per architecture.md FR40
**Then** query latency is tracked: p50, p95, p99 percentiles
**And** cache metrics are tracked: hit_rate, miss_rate, evictions, size
**And** metrics are exposed via Epic 8 event bus for monitoring
**And** slow queries (>500ms) are logged via `tracing::warn!(?query, duration_ms, "Slow query detected")`

**Given** query operations require instrumentation
**When** I add tracing to QueryService methods
**Then** all query methods use `#[tracing::instrument(skip(self, params), fields(operation, query_type), level = "debug")]`
**And** log cache hits: `tracing::debug!(query_type, cache_key, "Query cache hit")`
**And** log cache misses: `tracing::debug!(query_type, cache_key, "Query cache miss - executing query")`
**And** log query completion: `tracing::debug!(query_type, result_count, duration_ms, cache_hit, "Query completed")`
**And** span attributes include: query_type, result_count, duration_ms, cache_hit (bool), storage_accessed (bool)

**Given** Epic 9 storage provides fast access
**When** I avoid redundant caching
**Then** simple path lookups may bypass cache (storage is fast enough <10ms)
**And** expensive queries (metadata filtering, glob patterns) are cached aggressively
**And** cache effectiveness is measured per query type

**Given** cache invalidation (Story 11.7) maintains consistency
**When** I integrate caching with events
**Then** cached results are invalidated on NoteIndexed/NoteDeleted events
**And** cache + invalidation provides low-latency with eventual consistency
**And** cache staleness is bounded by event processing latency (<50ms)

## Story 11.9: Implement Query Result Formatting

As a user consuming query results,
I want results formatted appropriately for different use cases,
So that query output can be used directly or displayed clearly.

**Acceptance Criteria:**

**Given** Epic 12 templates consume query results
**When** I implement result formatters in `crates/app/src/services/query_formatter.rs`
**Then** formatters transform `Vec<Arc<Note>>` into template-friendly representations
**And** `ListFormatter` produces simple list: `["note1.md", "note2.md"]`
**And** `TableFormatter` produces structured data: `[{path, title, fileClass}, ...]`
**And** `JsonFormatter` produces JSON output for programmatic use

**Given** Epic 14 CLI displays query results
**When** I implement CLI formatters
**Then** `CliTableFormatter` produces human-readable table with columns: Path, Title, Schema, Modified
**And** table formatting uses `comfy-table` crate for aligned columns
**And** table output respects terminal width and truncates long values

**Given** formatting works
**When** I test different output needs
**Then** template use case: formatters provide structured data for iteration
**And** CLI use case: formatters provide human-readable display
**And** API use case (future): formatters provide machine-readable JSON

**Given** formatting is implemented
**When** I validate completeness
**Then** all query types have appropriate default formatting
**And** path queries → list format (simple paths)
**And** schema queries → table format (multiple fields)
**And** metadata queries → table format (filterable data)

## Story 11.10: Implement Advanced Query Composition

As a power user needing complex searches,
I want to compose multiple query conditions,
So that I can perform sophisticated searches across multiple criteria.

**Acceptance Criteria:**

**Given** users need complex multi-criteria searches
**When** I implement `compose_query(conditions: QueryConditions)` in QueryService
**Then** it supports composable query logic defined in Story 11.1:
- `And(vec![condition1, condition2])` - all conditions must match
- `Or(vec![condition1, condition2])` - any condition matches
- `FieldEquals(field, value)` - metadata field matching
- `SchemaMatches(schema)` - fileClass filtering
- `PathMatches(glob)` - path pattern matching

**Given** query composition must be efficient
**When** I implement execution strategy
**Then** `And` conditions use intersection: execute most selective condition first
**And** `Or` conditions use union: execute all conditions and merge results
**And** result merging uses `Arc<Note>` pointer equality to avoid duplicates

**Given** composition works
**When** I test nested conditions
**Then** complex query: `And([SchemaMatches("contact"), FieldEquals("status", "active")])` executes correctly
**And** nested query: `Or([And([...]), And([...])])` supports arbitrary nesting depth
**And** query optimizer reorders conditions for performance (most selective first)

**Given** composed queries access multiple indexes
**When** I implement multi-index queries
**Then** `SchemaMatches` uses schema_index table
**And** `FieldEquals` uses metadata_index table
**And** `PathMatches` scans notes table with prefix optimization
**And** index results are composed via set operations (intersection/union)

**Given** advanced queries are implemented
**When** I validate performance with benchmarks
**Then** simple composed queries (2-3 conditions) complete in <100ms
**And** complex composed queries (5+ conditions) complete in <500ms meeting NFR1
**And** query optimizer reduces redundant index scans

**Given** query composition may produce large result sets
**When** I implement result limiting
**Then** optional `limit` parameter caps result count (default no limit)
**And** pagination support allows: `compose_query(conditions, offset, limit)`
**And** result limiting prevents memory exhaustion on broad queries

## Story 11.11: Create Query Operation Mocks for Testing

As a developer testing query-dependent code,
I want comprehensive mocks for query operations,
So that query interactions can be tested in isolation.

**Acceptance Criteria:**

**Given** I need to test query interactions
**When** I create mocks for QueryPort
**Then** mock implementations simulate all query behaviors

**Given** mocks are available
**When** I write query-dependent tests
**Then** tests verify correct query usage without real data

**Given** integration tests are needed
**When** I use mocks
**Then** they simulate realistic query behavior for comprehensive testing

## Story 11.12: Performance Benchmarking for Query Operations

As a performance engineer, I want benchmarks for query operations to ensure fast lookups and efficient caching, so that query performance supports template execution requirements.
**Acceptance Criteria:**
**Given** query service is implemented
**When** I run query performance benchmarks
**Then** basic lookups (filename, path) complete in <100ms
**And** complex queries with metadata filtering complete in <500ms
**And** cache hit rates exceed 90% for repeated queries

**Given** performance benchmarks are established
**When** I monitor query performance
**Then** metrics are collected for optimization
**And** query performance regressions are detected
**And** memory usage for query caches stays within NFR9 bounds

## Story 11.13: Review Epic 11 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 11 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 11 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests

**Given** all Epic 11 public APIs are documented
**When** I verify doc test coverage
**Then** all public components have runnable doc tests demonstrating usage

**Given** all Epic 11 components are implemented with tests
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
**Then** test execution completes in <30 seconds for the full Epic 11 suite

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

## Story 11.14: Document Query Service for Developers

As a developer working with query operations,
I want comprehensive developer documentation for the query service,
So that query functionality can be properly understood and used.

**Acceptance Criteria:**

**Given** query service is implemented
**When** I create developer documentation
**Then** it includes query APIs, performance characteristics, and caching behavior

**Given** documentation exists
**When** developers read it
**Then** they understand query operations and integration patterns

**Given** query docs are complete
**When** other components integrate
**Then** they can use query service effectively and efficiently
