# Epic 10: Vault File System Integration & Indexing Engine **[MVP CORE]**

Users can index large vaults (1000+ files) in <2 seconds with incremental updates, reliable crash-free operation, and persistent storage.
**FRs covered:** FR20, FR24, FR25
**Implementation Notes:**

- VaultReaderPort, VaultWriterPort, VaultScannerPort, MarkdownPort and mocks created
- pulldown-cmark for markdown parsing (adapter layer per ADR 0004)
- Sample vault notes from docs/refs/obsidian/ as test fixtures
- Performance benchmarking stories for NFR2 validation (<2s for 1000+ files)
- Observability/metrics for indexing performance
- Integration with Epic 8 (event publishing) and Epic 9 (storage persistence)

## Story 10.1: Implement Note CQRS Ports from Epic 3

As a developer completing the note bounded context,
I want to implement the NoteCommand and NoteQuery ports defined in Epic 3,
So that note operations follow CQRS separation with proper command/query handling.

**Acceptance Criteria:**

**Given** Epic 3 defined NoteCommand and NoteQuery trait interfaces
**When** I implement the concrete ports
**Then** NoteCommand handles note creation, updates, and indexing

**Given** NoteCommand is implemented
**When** I validate command operations
**Then** it supports parsing notes from filesystem, validating against schemas, and storing indexed notes

**Given** NoteQuery is implemented
**When** I validate query operations
**Then** it supports retrieving notes by various criteria (path, frontmatter, links, etc.)

**Given** both ports are implemented
**When** I test integration
**Then** commands and queries work together for complete note management and querying

## Story 10.2: Create Vault Domain Interfaces and Ports

As a developer implementing vault operations,
I want clean domain interfaces for vault access,
So that vault operations follow hexagonal architecture principles.

**Acceptance Criteria:**

**Given** I need vault operation contracts
**When** I create vault domain ports
**Then** VaultReaderPort, VaultWriterPort, VaultScannerPort, MarkdownPort are defined

**Given** vault ports are defined
**When** I implement mocks for testing
**Then** test doubles are available for isolated vault testing

**Given** the domain interfaces exist
**When** I validate the design
**Then** they follow hexagonal principles with clear separation between domain and infrastructure

## Story 10.3: Implement Vault File System Scanner

As a developer scanning vault directories,
I want efficient filesystem scanning with concurrent access handling,
So that vault files can be discovered and processed safely.

**Acceptance Criteria:**

**Given** I need to scan vault directories
**When** I implement filesystem scanner
**Then** recursive directory traversal discovers all markdown files

**Given** file scanning is implemented
**When** I handle concurrent access
**Then** proper file locking or change detection prevents conflicts

**Given** vault scanning runs
**When** I monitor performance
**Then** large vaults (1000+ files) are scanned efficiently

## Story 10.4: Implement Markdown Parser for Frontmatter Extraction

As a developer parsing vault files,
I want reliable frontmatter extraction from markdown,
So that note metadata can be indexed and queried.

**Acceptance Criteria:**

**Given** I need to parse markdown files
**When** I implement frontmatter extraction using pulldown-cmark
**Then** YAML frontmatter is correctly parsed from markdown files

**Given** markdown parsing is implemented
**When** I handle malformed files
**Then** parsing errors are handled gracefully without crashing

**Given** frontmatter extraction works
**When** I validate completeness
**Then** all standard frontmatter fields are properly extracted

## Story 10.5: Implement Frontmatter Validation Service

As a user editing notes,
I want frontmatter validated against schemas with clear warnings,
So that I know if my metadata is incorrect without blocking note usage.

**Acceptance Criteria:**

### **Service Location (Application Layer):**

**Given** frontmatter validation is application-layer orchestration
**When** I create the service
**Then** implement in `crates/app/src/services/frontmatter_validator.rs`
**And** it is NOT an adapter (does not implement port trait)
**And** it orchestrates domain logic and adapters (SchemaQuery, Note validation)

**Given** service dependencies
**When** I design the service
**Then** it depends on `SchemaQuery` port (Epic 7) for schema lookup
**And** it uses `Note` and `Frontmatter` domain aggregates (Epic 3)
**And** it uses `Property` and `PropertySpec` for constraint validation (Epic 3)

### **Validation Logic:**

**Given** FrontmatterValidator.validate(note: &Note) method
**When** I implement validation
**Then** extract `fileClass` from note.frontmatter.fields
**And** if no `fileClass`, return Ok(vec![]) (no schema defined, no validation)
**And** call `schema_query.get(fileClass)` → Option<Schema>
**And** if schema not found, return `Ok(vec![ComplianceWarning::SchemaNotFound(fileClass)])`
**And** if schema found, validate all properties against schema

**Given** property validation
**When** I check each schema property
**Then** for required properties: verify field exists in frontmatter
**And** if missing, add `ComplianceWarning::MissingField { property, schema }`
**And** for present properties: validate type and constraints

**Given** type validation
**When** I check property type
**Then** for PropertySpec::String: verify frontmatter value is string
**And** for PropertySpec::Number: verify frontmatter value is number
**And** for PropertySpec::Bool: verify frontmatter value is bool
**And** for PropertySpec::Date: verify frontmatter value is valid date string
**And** for PropertySpec::File: verify frontmatter value is string (file reference)
**And** if type mismatch, add `ComplianceWarning::TypeError { property, expected, actual }`

**Given** constraint validation (string)
**When** PropertySpec::String has constraints
**Then** if `enum` is set: verify value is in allowed list
**And** if `pattern` is set: verify value matches regex
**And** if violation, add `ComplianceWarning::ConstraintViolation { property, constraint, value }`

**Given** constraint validation (number)
**When** PropertySpec::Number has constraints
**Then** if `min` is set: verify value >= min
**And** if `max` is set: verify value <= max
**And** if `step` is set: verify (value - min) % step == 0 (if min exists)
**And** if violation, add `ComplianceWarning::ValueOutOfRange { property, value, min, max }`

**Given** array property validation
**When** property has `array: true`
**Then** verify frontmatter value is YAML array (Vec<Value>)
**And** validate each array element against PropertySpec
**And** if any element violates constraints, add warning for that index

### **ComplianceWarning Types:**

**Given** validation returns warnings (not errors)
**When** I define warning types
**Then** create enum in `crates/app/src/services/compliance_warning.rs`:
```rust
pub enum ComplianceWarning {
    SchemaNotFound { file_class: String },
    MissingField { property: String, schema: String },
    TypeError { property: String, expected: String, actual: String },
    ConstraintViolation { property: String, constraint: String, value: String },
    ValueOutOfRange { property: String, value: String, min: Option<f64>, max: Option<f64> },
}
```

**Given** warnings are non-blocking
**When** validation returns warnings
**Then** note usage continues (not rejected)
**And** warnings can be surfaced to user (CLI, LSP future)
**And** warnings stored in index metadata (for later query)

### **Integration with Indexing:**

**Given** NoteIndexer (Story 10.5) processes notes
**When** I integrate frontmatter validation
**Then** after parsing frontmatter (Story 10.4), call `FrontmatterValidator::validate(note)`
**And** store warnings in indexed note metadata
**And** log warnings at debug level: "Note {path} has {count} frontmatter warnings"
**And** indexing continues regardless of warnings (non-blocking)

### **Error Handling:**

**Given** validation operations
**When** I handle errors
**Then** SchemaQuery errors are converted to warnings (e.g., SchemaNotFound)
**And** regex compilation errors in pattern constraints are logged + skipped (config error)
**And** unexpected errors (e.g., schema cache failure) are logged but don't block validation

### **Testing:**

**Given** FrontmatterValidator is implemented
**When** I write unit tests
**Then** test schema not found (fileClass undefined) → no warnings
**And** test schema not found (fileClass="unknown") → SchemaNotFound warning
**And** test required field missing → MissingField warning
**And** test type mismatch (string expected, number provided) → TypeError warning
**And** test enum violation (value not in list) → ConstraintViolation warning
**And** test pattern violation (regex mismatch) → ConstraintViolation warning
**And** test number out of range (value > max) → ValueOutOfRange warning
**And** test array property validation (each element checked)
**And** test valid note (all constraints satisfied) → no warnings
**And** use `example_vault` schemas + notes as test fixtures

**Given** integration with indexing
**When** I test end-to-end
**Then** index note with invalid frontmatter → warnings stored in index
**And** query note → warnings accessible via index metadata
**And** note still usable despite warnings

## Story 10.6: Create Vault Indexing Engine with Incremental Updates

As a developer building the indexing system,
I want an indexing engine that supports incremental updates and rename detection,
So that only changed files are reprocessed and note identity is preserved across renames.

**Acceptance Criteria:**

**Given** I need vault indexing
**When** I create the indexing engine
**Then** it processes vault files and builds searchable index

**Given** indexing engine is implemented
**When** I handle incremental updates
**Then** only modified files are re-indexed based on change detection

**Given** incremental indexing works
**When** I validate efficiency
**Then** large vaults show significant performance improvement over full rebuilds

**Given** a note file is renamed between indexing runs (Lithos not running)
**When** I perform incremental indexing
**Then** the engine detects potential renames using a three-tier strategy: filesystem metadata → frontmatter matching → content hash fallback
**And** UUID v7 identity is preserved for correctly detected renames

**Given** filesystem metadata suggests a rename (unchanged mtime but changed ctime)
**When** I validate with frontmatter matching
**Then** I parse the file's Frontmatter using domain's Frontmatter struct
**And** I compare against cached Frontmatter using PartialEq
**And** exact frontmatter matches result in high-confidence rename (0.90)
**And** partial matches (title + created date) result in medium-confidence (0.70)

**Given** frontmatter matching is inconclusive
**When** content hashing is enabled in configuration
**Then** I compute SHA256 hash of file content
**And** I compare against cached content hashes for missing files
**And** exact hash matches result in very-high-confidence rename (0.95)

**Given** rename detection produces confidence scores
**When** I apply rename decisions
**Then** I auto-accept renames above user-configured threshold (high/medium/never)
**And** I prompt for confirmation if interactive mode is enabled
**And** I log all rename decisions with confidence scores and detection signals

**Given** rename detection completes
**When** I update storage persistence
**Then** the original UUID is preserved for detected renames
**And** the notes table is updated to reflect the new path
**And** NoteRenamed event is published with old_path, new_path, and uuid

**Given** rename detection performance is measured
**When** I benchmark with 1000-file vault containing 10 renames
**Then** metadata-first approach avoids content hashing in 95%+ of cases
**And** total indexing time remains within NFR2 (<2s for 1000+ files)

**Given** rename detection is configurable
**When** users set preferences in lithos.toml
**Then** they can configure: rename_detection strategy, auto_accept_threshold, interactive_prompts, allow_content_hashing, frontmatter_signature_fields
**And** they can disable rename detection entirely for performance-critical scenarios

## Story 10.7: Add Indexing Performance Optimization and Monitoring

As a developer optimizing indexing performance,
I want performance monitoring and optimization for NFR2 compliance,
So that vault indexing completes in <2 seconds for 1000+ files.

**Acceptance Criteria:**

**Given** I need performance optimization
**When** I implement monitoring
**Then** indexing operations are timed and metrics collected

**Given** performance monitoring is active
**When** I optimize bottlenecks
**Then** concurrent processing and memory management improve performance

**Given** optimizations are implemented
**When** I benchmark with 1000+ files
**Then** indexing completes in <2 seconds meeting NFR2

**Given** memory usage is monitored
**When** I validate bounds
**Then** indexing stays within NFR9 500MB memory limit

## Story 10.8: Implement Indexing Error Recovery and Crash Prevention

As a developer ensuring indexing reliability,
I want error recovery and crash prevention mechanisms,
So that indexing failures don't corrupt the system or lose data.

**Acceptance Criteria:**

**Given** I need error recovery
**When** I implement failure handling
**Then** individual file parsing errors don't stop the entire indexing process

**Given** crash prevention is implemented
**When** I handle system interruptions
**Then** indexing can resume from interruption point

**Given** error recovery works
**When** I validate robustness
**Then** indexing achieves zero crashes during normal vault operations (NFR25)

## Story 10.9: Integrate Indexing with Storage Persistence

As a developer coordinating indexing with storage,
I want indexing results persisted to storage,
So that indexed data is available for queries and survives restarts.

**Acceptance Criteria:**

**Given** indexing produces results
**When** I integrate with Epic 9 storage
**Then** indexed data is persisted using storage ports

**Given** storage integration works
**When** I handle large indexes
**Then** storage operations maintain performance within bounds

**Given** persistence is implemented
**When** I restart the system
**Then** indexed data is available without re-indexing

## Story 10.10: Implement Indexing Event Publishing

As a developer coordinating indexing with the event system,
I want indexing to publish events for system coordination,
So that other components are notified of indexing progress and completion.

**Acceptance Criteria:**

**Given** indexing operations occur
**When** I integrate with Epic 8 event bus
**Then** indexing publishes NoteIndexed, VaultIndexingStarted, VaultIndexingCompleted events

**Given** event publishing works
**When** I monitor indexing progress
**Then** subscribers receive real-time updates on indexing status

**Given** events are published
**When** I validate integration
**Then** other epics can subscribe to indexing events without tight coupling

## Story 10.11: Implement Indexing State Persistence

As a developer enabling resumable indexing,
I want indexing state persisted for interruption recovery,
So that long-running indexing operations can resume after interruptions.

**Acceptance Criteria:**

**Given** I need resumable indexing
**When** I implement state persistence
**Then** current indexing progress is saved periodically

**Given** state persistence works
**When** I interrupt indexing
**Then** indexing can resume from saved state without restarting

**Given** resumption works
**When** I validate reliability
**Then** large vault indexing survives system interruptions gracefully

## Story 10.12: Create Sample Vault Test Data

As a developer testing indexing functionality,
I want representative sample vault data,
So that indexing can be tested with realistic data volumes and patterns.

**Acceptance Criteria:**

**Given** I need test data
**When** I create sample vaults from docs/refs/obsidian/
**Then** samples include various file types, frontmatter patterns, and link structures

**Given** sample data exists
**When** I test indexing
**Then** samples validate all indexing scenarios and edge cases

**Given** samples are comprehensive
**When** I benchmark performance
**Then** test results are representative of real vault indexing performance

## Story 10.13: Create Vault Operation Mocks for Testing

As a developer testing vault-dependent code,
I want comprehensive mocks for vault operations,
So that vault interactions can be tested in isolation without filesystem access.

**Acceptance Criteria:**

**Given** I need to test vault operations
**When** I create mocks for vault ports
**Then** mock implementations simulate all vault port behaviors

**Given** mocks are available
**When** I write vault-dependent tests
**Then** tests verify correct vault operations without real filesystem

**Given** integration tests are needed
**When** I use mocks
**Then** they simulate realistic vault behavior for comprehensive testing

## Story 10.14: Performance Benchmarking for Vault Indexing (NFR2 Validation)

As a performance engineer, I want comprehensive benchmarks for vault indexing operations, so that NFR2 (<2s for 1000+ files) is validated and monitored.
**Acceptance Criteria:**
**Given** vault indexing system is implemented
**When** I run performance benchmarks
**Then** indexing 1000+ files completes in <2 seconds
**And** incremental updates are measured and validated
**And** memory usage stays within NFR9 bounds (<500MB)

**Given** performance benchmarks are established
**When** I monitor indexing performance
**Then** metrics are collected for optimization
**And** performance regressions are detected
**And** scaling characteristics are documented

## Story 10.15: Vault Operation Monitoring and Health Checks

As a system administrator, I want continuous monitoring of vault operations, so that performance issues and failures are detected before they impact users.
**Acceptance Criteria:**
**Given** vault indexing is running
**When** I monitor system health
**Then** indexing performance metrics are collected continuously
**And** memory usage is tracked against NFR9 limits
**And** alerts trigger when performance degrades beyond thresholds

**Given** vault operations encounter issues
**When** health checks run
**Then** they detect corrupted indexes or inconsistent state
**And** they trigger automatic recovery procedures
**And** they log detailed diagnostic information

## Story 10.16: Redb Storage Performance Regression Testing

As a performance engineer, I want automated regression tests for Redb storage operations, so that the architectural choice of Redb + rkyv remains optimal and performance degradation is caught immediately.
**Acceptance Criteria:**
**Given** Redb storage implementation
**When** performance regression tests run
**Then** read/write benchmarks are compared against established baselines
**And** memory usage is validated against winning olympics metrics
**And** query performance regressions trigger alerts and investigation
**And** storage benchmarks run in CI/CD pipeline for every change

## Story 10.17: Review Epic 10 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 10 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 10 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests

**Given** all Epic 10 public APIs are documented
**When** I verify doc test coverage
**Then** all public components have runnable doc tests demonstrating usage

**Given** all Epic 10 components are implemented with tests
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
**Then** test execution completes in <30 seconds for the full Epic 10 suite

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

## Story 10.18: Document Vault Indexing System for Developers

As a developer working with vault operations,
I want comprehensive developer documentation for indexing,
So that vault indexing can be properly understood and maintained.

**Acceptance Criteria:**

**Given** indexing system is implemented
**When** I create developer documentation
**Then** it includes indexing algorithms, performance characteristics, and maintenance procedures

**Given** documentation exists
**When** developers read it
**Then** they understand indexing operations and troubleshooting procedures

**Given** indexing docs are complete
**When** other components integrate
**Then** they can work with indexed data effectively
