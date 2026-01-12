# Story 2.2: create-event-driven-testing-patterns
Status: done


<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer testing event-driven systems,
I want patterns for testing domain events and event bus interactions,
So that event-driven code is thoroughly tested with proper isolation and verification.

## Acceptance Criteria

**Infrastructure Setup:**
- **Given** researched event-driven testing patterns
- **When** reviewing event testing infrastructure
- **Then** established patterns for publishing/subscription testing, payload verification, ordering/timing verification, and mock event bus implementations

**Publisher Testing:**
- **Given** established event testing patterns
- **When** testing event publisher
- **Then** verifies correct events published, payloads contain expected data, events published at correct time, and error handling for failed publishing

**Subscriber Testing:**
- **Given** established event testing patterns
- **When** testing event subscriber
- **Then** verifies subscriber receives expected events, handling logic executes correctly, graceful malformed event handling, and subscription lifecycle management

**DDD Compliance:**
- **Given** researched event testing in domain-driven design
- **When** checking patterns
- **Then** follows DDD best practices: event sourcing verification, event storming validation, domain event contract testing, and integration testing for event flows

## Tasks / Subtasks

- [x] Research event-driven testing patterns for domain events and event bus **[Effort: 2-3 hours | Complexity: Medium]**
  - [x] Analyze CQRS event sourcing testing with Given-When-Then patterns (e.g., AccountTestFramework::given().when().then_expect_events())
  - [x] Study mock event bus implementations using Arc<dyn EventBusPort> for trait-based testing isolation
  - [x] Review event payload verification using serde serialization and domain event contract testing
  - [x] Examine event ordering and timing verification with tokio::time for async event sequencing
- [x] Create event testing infrastructure and utilities **[Effort: 4-6 hours | Complexity: High]**
  - [x] Implement mock event bus with captured events storage and subscription verification
  - [x] Develop event payload verification helpers for domain event structs with assert_eq! on serialized data
  - [x] Create event ordering utilities using Vec<EventRecord> with timestamp and sequence validation
  - [x] Build async event subscription lifecycle management with tokio::sync::mpsc for reliable testing
- [x] Establish event-driven testing patterns and guidelines **[Effort: 3-4 hours | Complexity: Medium]**
  - [x] Define publisher testing patterns: setup mock bus, execute command, verify published events with payloads
  - [x] Create subscriber testing patterns: mock event publishing, verify handler execution and side effects
  - [x] Implement error handling patterns for malformed events and failed subscriptions
  - [x] Develop integration testing patterns for event flows using real event bus with isolated channels
- [x] Test event testing infrastructure **[Effort: 2-3 hours | Complexity: Medium]**
  - [x] Validate mock event bus captures events correctly with proper async handling
  - [x] Test event payload verification handles serde serialization/deserialization
  - [x] Verify event ordering patterns work with concurrent event publishing
  - [x] Ensure async event patterns integrate with tokio runtime without race conditions

### Quality Assurance and Commit (MANDATORY FINAL TASK)
- [x] Run `mise run fmt` to format all code according to project standards
- [x] Run `mise run lint` to check for all code quality issues and anti-patterns
- [x] Run `mise run verify` for comprehensive verification (fmt + lint + tests)
- [x] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [x] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING
- [x] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [x] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [x] **MANDATORY:** Verify 80%+ test coverage is maintained (80.71%)
- [x] **MANDATORY:** Confirm all code passes clippy cognitive complexity limits (<25)
- [x] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [x] Stage all files created or modified during story development
- [x] Commit with conventional commit message: `feat: establish event-driven testing patterns with mock bus and async verification`

## Dev Notes

- Relevant architecture patterns and constraints: Event-driven architecture with hybrid event bus (ADR 0007), domain events for CQRS separation
- Source tree components to touch: Event testing utilities in test-utils crate, domain event definitions, event bus ports
- Testing standards summary: Event-driven tests ensure proper event flow, payload verification, and async handling with mock isolation

### Project Structure Notes

- Alignment with unified project structure: Event testing integrates with async testing patterns from Story 2.1
- Detected conflicts or variances: None - extends async testing infrastructure

### References

- [ADR 0008: Event-Driven Testing Patterns](docs/adr/0008-event-driven-testing-patterns.md) - Comprehensive analysis and decisions for optimal testing patterns
- [Source: epics/epic-2-test-architecture-patterns-utilities-mvp-core.md#Story 2.2]
- [ADR 0007: Hybrid Event Orchestration](docs/adr/0007-event-orchestration.md) - Event bus architecture requiring specific testing patterns
- [Research: CQRS Event Sourcing Testing - https://doc.rust-cqrs.org/add_first_test.html]
- [Research: Domain Event Testing Patterns - https://verraes.net/2023/05/eventsourcing-testing-patterns/]
- [Research: Event Sourcing with Aggregates - https://medium.com/capital-one-tech/event-sourcing-with-aggregates-in-rust-4022af41cf67]

## Dev Agent Record

### Agent Model Used

opencode-codex

### Debug Log References

- Implemented event-driven testing utilities and mocks

### Implementation Plan

- Extend `lithos-test-utils` with event testing utilities and mock event bus
- Add integration tests demonstrating publisher/subscriber patterns
- Document event-driven testing guidelines aligned with ADR 0008

### Completion Notes List

- Added Given-When-Then event testing framework and payload/sequence assertions
- Implemented hybrid mock event bus with MPSC, broadcast, and watch planes
- Added integration tests covering publisher/subscriber and ordering verification
- Documented event-driven testing guidelines in project docs
- Expanded tests with descriptive integration coverage
- Ran `mise run fmt`, `mise run lint`, `mise run verify`, `mise run test:coverage`, and `pre-commit run --all-files`

### File List

- crates/test-utils/Cargo.toml
- crates/test-utils/src/async_helpers.rs
- crates/test-utils/src/lib.rs
- crates/test-utils/src/events.rs
- crates/test-utils/src/mocks/mod.rs
- crates/test-utils/src/mocks/event_bus.rs
- crates/app/Cargo.toml
- crates/app/tests/dummy_integration.rs
- crates/app/tests/event_testing_patterns.rs
- crates/cli/src/main.rs
- docs/testing/event-testing.md
- _bmad-output/implementation-artifacts/sprint-status.yaml
- _bmad-output/implementation-artifacts/stories/2-2-create-event-driven-testing-patterns.md

## Change Log

- 2026-01-12: Added event testing utilities, mock event bus, and docs
- 2026-01-12: Expanded tests with descriptions and 80%+ coverage

## Dev Notes

- **ADR 0008 Analysis Integration**: Follow validated testing patterns for optimal efficiency and complete coverage of Lithos' hybrid event bus architecture (ADR 0007).

- **Architecture Compliance**: Builds on CQRS patterns with comprehensive event verification across all three event bus planes.

- **Implementation Priority**: Start with mock infrastructure (Priority 1-2), then verification patterns (Priority 3-4), finally advanced features (Priority 5-6).

- **Source Tree Components**: Event test utilities in crates/test-utils/src/events.rs, mock implementations in tests/, guidelines in docs/testing/event-testing.md.

- **Quality Assurance**: Event tests ensure payload integrity, timing verification, and race-free async execution with proper isolation.

### Project Structure Notes

- **Alignment with unified project structure**: Event testing patterns integrate with async testing from Story 2.1 and hexagonal architecture event ports.

- **Detected conflicts or variances**: None - leverages existing async infrastructure for event-driven testing.

### Technical Requirements

**Priority 1 (Foundation):** Implement CQRS Event Sourcing Testing Framework with Given-When-Then patterns for aggregate testing (ADR 0008 Decision 1)

**Priority 2 (Infrastructure):** Create trait-based mock event bus using Arc<dyn EventBusPort> supporting all three ADR 0007 event bus planes:
- Data Plane (MPSC): Reliable indexer actor event testing
- Control Plane (Broadcast): System status event testing
- State Plane (Watch): LSP state synchronization testing

**Priority 3 (Verification):** Develop event payload verification using serde serialization with contract testing for domain events (ADR 0008 Decision 3)

**Priority 4 (Async Handling):** Establish tokio-based async event testing patterns with proper error scenario coverage:
- Channel overflow and backpressure testing
- Subscriber disconnect and cleanup verification
- Malformed payload error handling
- Race condition prevention in concurrent scenarios

**Priority 5 (Advanced):** Build event ordering and timing verification using generation counters and timestamps (ADR 0008 Decision 5)

**Priority 6 (Integration):** Implement performance testing patterns for event throughput (10,000+ events/sec target from ADR 0007) and cross-aggregate event flows

### File Structure Requirements

- Event test utilities in crates/test-utils/src/events.rs with Given-When-Then framework (ADR 0008 aligned)
- Mock event bus implementations in crates/test-utils/src/mocks/event_bus.rs supporting MPSC, Broadcast, and Watch channels
- Event testing examples in crates/*/tests/event_*.rs demonstrating all ADR 0008 testing patterns
- Event testing guidelines in docs/testing/event-testing.md with comprehensive CQRS and hybrid bus examples

### Testing Requirements

- Unit tests for event testing utilities and mock implementations covering all three event bus planes
- Integration tests for event publishing and subscription flows with cross-aggregate verification
- Async event testing with tokio runtime verification including error scenarios (channel overflow, disconnects)
- Event payload and timing validation tests with performance benchmarks for 10,000+ events/sec throughput
- Error scenario testing for malformed events, failed subscriptions, and race conditions

### Previous Story Intelligence

- Story 2.1 established async testing patterns - integrate event testing with tokio runtime
- Story 2.1 created async test utilities - extend for event-specific testing needs

### Git Intelligence Summary

- Recent commits show tokio integration - event testing builds on async patterns
- Event-driven patterns align with established domain event conventions

### Latest Tech Information

- CQRS: Given-When-Then framework (rust-cqrs); payload verification via serde
- Async: tokio::sync channels (MPSC/Broadcast/Watch) for concurrent testing
- Ordering: Generation counters + timestamps; trait-based mocks with Arc<dyn Trait>
- Performance: 10,000+ events/sec throughput testing required

### Project Context Reference

- Lithos uses hybrid event bus (ADR 0007) for domain events and CQRS
- Event-driven architecture requires comprehensive testing for reliability
- Domain layer events drive cross-aggregate consistency and UI updates

### Story Completion Status

- Status: ready-for-dev
- All acceptance criteria defined with testable event testing requirements validated by ADR 0008 analysis
- Technical requirements complete with mock event bus and verification patterns optimized per ADR 0008
- Integration points identified with async testing infrastructure and hybrid event bus architecture
- Risk assessment: Low risk, builds on established async and event patterns with comprehensive research validation
- Execution Optimization: Follow ADR 0008 decisions for maximum efficiency and coverage of Lithos' event-driven architecture
