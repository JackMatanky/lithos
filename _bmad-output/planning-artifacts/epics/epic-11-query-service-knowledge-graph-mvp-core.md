# Epic 11: Query Service & Knowledge Graph **[MVP CORE]**

Users can perform fast lookups by filename, path, or schema keys, resolve wiki-links and aliases, and query metadata from other notes for template use.
**FRs covered:** FR21, FR22, FR23
**Implementation Notes:**

- QueryPort and mocks created in this epic
- CQRS read side (Epic 10 is write side)
- Performance benchmarking stories for NFR1 validation (<500ms queries)
- Observability/metrics for query performance
- File class queries for schema-based filtering
- Integration with Epic 9 storage and Epic 8 events

## Story 11.1: Create Query Domain Interface and Port

As a developer implementing query operations,
I want clean domain interfaces for query access,
So that queries follow hexagonal architecture principles.

**Acceptance Criteria:**

**Given** I need query operation contracts
**When** I create QueryPort trait
**Then** it includes methods for lookups, filtering, and resolution

**Given** QueryPort is defined
**When** I implement mocks for testing
**Then** test doubles are available for isolated query testing

**Given** the domain interface exists
**When** I validate the design
**Then** it follows hexagonal principles with clear separation between domain and infrastructure

## Story 11.2: Integrate Query Service with Storage Layer

As a developer coordinating queries with persistence,
I want query service integrated with storage layer,
So that queries retrieve data from the persisted index efficiently.

**Acceptance Criteria:**

**Given** I need storage integration
**When** I connect with Epic 9 storage
**Then** queries use storage ports for data retrieval

**Given** storage integration works
**When** I handle large datasets
**Then** queries maintain performance through storage optimization

**Given** integration is complete
**When** I validate data consistency
**Then** queries return data matching the indexed state

## Story 11.3: Implement Basic Query Operations

As a user needing to find notes,
I want basic lookup operations by filename and path,
So that I can quickly locate specific notes in the vault.

**Acceptance Criteria:**

**Given** I need basic queries
**When** I implement lookup operations
**Then** queries by filename and path return correct results

**Given** basic queries work
**When** I test with indexed data
**Then** results are retrieved from Epic 10 indexed data

**Given** lookups are implemented
**When** I validate performance
**Then** basic queries complete within acceptable time limits

## Story 11.4: Implement Schema-Based Query Filtering

As a user organizing notes by schema,
I want to filter queries by schema keys and metadata,
So that I can find notes with specific properties or schemas (when schemas are used).

**Acceptance Criteria:**

**Given** schemas are available
**When** I implement schema-based filtering
**Then** queries can filter by schema-defined metadata fields

**Given** schema filtering works
**When** I test with different schemas
**Then** results are correctly filtered by schema properties

**Given** filtering is implemented
**When** I validate edge cases
**Then** queries handle missing metadata gracefully and work without schemas

**Given** users don't use schemas
**When** they run queries
**Then** filtering works through direct frontmatter field queries

## Story 11.5: Implement File Class Query Operations

As a user categorizing notes by type,
I want to query notes by fileClass for schema-based organization,
So that I can find all "contact" notes or "project" notes efficiently.

**Acceptance Criteria:**

**Given** I need fileClass queries
**When** I implement fileClass filtering
**Then** queries can find all notes with specific fileClass values

**Given** fileClass queries work
**When** I test with schema inheritance
**Then** queries respect schema hierarchies and inheritance

**Given** fileClass operations are implemented
**When** I validate performance
**Then** fileClass queries are optimized for large result sets

## Story 11.6: Add Wiki-Link and Alias Resolution

As a user working with interconnected notes,
I want wiki-links and aliases resolved to actual note paths,
So that links work correctly across the knowledge graph.

**Acceptance Criteria:**

**Given** I need link resolution
**When** I implement wiki-link resolution
**Then** [[link]] syntax resolves to actual file paths

**Given** alias resolution is implemented
**When** I handle alias lookups
**Then** alias references resolve to correct targets

**Given** link resolution works
**When** I validate completeness
**Then** all wiki-link and alias patterns are properly resolved

## Story 11.7: Implement Query Cache Invalidation via Events

As a developer maintaining query performance,
I want cache invalidation through event system,
So that query results stay current when index updates occur.

**Acceptance Criteria:**

**Given** I need cache invalidation
**When** I integrate with Epic 8 events
**Then** query caches invalidate when NoteIndexed events are received

**Given** event integration works
**When** I test cache consistency
**Then** queries return updated results after index changes

**Given** invalidation is implemented
**When** I monitor performance
**Then** cache hit rates remain high while data stays current

## Story 11.8: Add Query Performance Optimization and Caching

As a developer optimizing query speed,
I want performance optimization with intelligent caching,
So that queries complete in <500ms meeting NFR1 requirements.

**Acceptance Criteria:**

**Given** I need performance optimization
**When** I implement caching with LRU strategy
**Then** frequently accessed query results are cached

**Given** caching is implemented
**When** I set TTL policies
**Then** cache entries expire appropriately to stay current

**Given** optimization works
**When** I benchmark queries
**Then** average query time is <500ms meeting NFR1

**Given** performance is validated
**When** I monitor metrics
**Then** cache hit rates and query latencies are tracked

## Story 11.9: Implement Query Result Formatting

As a user consuming query results,
I want results formatted appropriately for different use cases,
So that query output can be used directly or displayed clearly.

**Acceptance Criteria:**

**Given** I need result formatting
**When** I implement formatters
**Then** results can be formatted as lists, tables, or structured data

**Given** formatting works
**When** I test different output needs
**Then** formats are appropriate for CLI display and programmatic use

**Given** formatting is implemented
**When** I validate completeness
**Then** all query types have appropriate default formatting

## Story 11.10: Implement Advanced Query Composition

As a power user needing complex searches,
I want to compose multiple query conditions,
So that I can perform sophisticated searches across multiple criteria.

**Acceptance Criteria:**

**Given** I need complex queries
**When** I implement query composition
**Then** multiple conditions can be combined with AND/OR logic

**Given** composition works
**When** I test nested conditions
**Then** complex queries execute correctly and efficiently

**Given** advanced queries are implemented
**When** I validate performance
**Then** complex queries still meet NFR1 timing requirements

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
