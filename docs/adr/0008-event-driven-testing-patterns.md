# ADR 0008: Event-Driven Testing Patterns for Domain Events and Event Bus

*   **Status**: Accepted
*   **Date**: 2026-01-11
*   **Stakeholders**: Jack (Developer)

## Context

Lithos implements event-driven architecture using domain events and a hybrid event bus (ADR 0007) for decoupling components in the hexagonal architecture. The testing strategy requires comprehensive patterns for testing event publishing, subscription, payload verification, and async event flows to ensure reliability and prevent regressions.

Research of event-driven testing patterns revealed several established approaches:

**CQRS Event Sourcing Testing:**
- Given-When-Then framework for aggregate testing
- Event payload verification using serialization
- Command execution and event emission validation

**Event Bus Testing:**
- Mock implementations for isolation
- Async channel testing with tokio
- Subscriber verification patterns

**Domain Event Testing:**
- Payload integrity validation
- Event ordering and timing verification
- Race condition prevention in concurrent scenarios

The project uses CQRS patterns where commands publish events that update read models asynchronously. Testing must cover:
- Event publishing verification
- Subscriber event handling
- Event payload integrity
- Async timing and ordering
- Mock isolation for unit tests

## Decision

We will establish comprehensive event-driven testing patterns based on research of CQRS and event sourcing testing practices in Rust ecosystems:

### 1. CQRS Event Sourcing Testing Framework
Implement Given-When-Then testing framework for event sourcing aggregates, following rust-cqrs and eventastic patterns:
- `given(vec![previous_events])` - Setup aggregate state with event history
- `when(command)` - Execute command and capture resulting events
- `then_expect_events(vec![expected_events])` - Verify exact event sequence and payloads

### 2. Mock Event Bus Implementation
Create trait-based mock event bus using `Arc<dyn EventBusPort>` for unit test isolation:
- Event capture storage with `Vec<EventRecord>` for verification
- Async subscription verification using `tokio::sync::mpsc::Receiver`
- Payload inspection for domain event contracts using serde comparison
- Support for both broadcast and MPSC channel patterns from ADR 0007

### 3. Event Payload Verification
Use serde serialization for comprehensive event payload verification:
- `assert_eq!` on JSON-serialized event data for human-readable diffs
- Domain event contract testing with required field validation
- Immutable event structure validation preventing accidental mutations

### 4. Async Event Testing Patterns
Implement tokio-based async event testing with proper concurrency handling:
- `tokio::sync::mpsc` bounded channels for reliable event flow testing
- `tokio::time::timeout` integration to prevent hanging tests
- Race condition prevention using `tokio::sync::Mutex` for shared state
- Concurrent event publishing simulation with `tokio::spawn`

### 5. Event Ordering and Timing Verification
Establish patterns for event sequence and timing validation:
- Generation counters in event records for deterministic ordering
- Timestamp-based verification using `chrono::DateTime`
- Event stream ordering tests for CQRS read model consistency
- Concurrent event interleaving simulation and verification

### 6. Integration Testing Patterns
Complement unit tests with integration patterns for event flows:
- Real event bus testing with isolated tokio runtimes
- Cross-aggregate event flow validation
- Performance testing for event throughput under load
- Failure scenario testing with event bus disruptions

## Alternatives Considered

### Alternative 1: Manual Event Testing
- **Pros**: Simple implementation, no additional frameworks, direct control over test logic
- **Cons**: Error-prone due to manual event verification, inconsistent testing approaches across the codebase, missing edge cases like race conditions, poor maintainability with code duplication, no standardized patterns for async event handling

### Alternative 2: Full Event Sourcing Framework (e.g., eventastic)
- **Pros**: Comprehensive tooling with built-in aggregate testing, standardized CQRS patterns, event store testing utilities, reduced boilerplate for complex event scenarios
- **Cons**: Additional dependencies increasing binary size, complexity for simple event testing needs, potential over-engineering for basic domain events, framework lock-in and learning curve

### Alternative 3: Integration-Only Testing
- **Pros**: Tests real event flows end-to-end, catches integration issues and timing problems, validates complete event-driven workflows
- **Cons**: Slow execution due to real async operations, brittle tests affected by external factors, hard to isolate failures to specific components, difficult to test error scenarios

### Alternative 4: Property-Based Testing Only
- **Pros**: Comprehensive coverage of edge cases, automatic test case generation, mathematical verification of event properties
- **Cons**: Complex setup for domain-specific constraints, hard to debug failing cases, doesn't test specific business scenarios, requires deep understanding of property testing

## Consequences

*   **Positive**:
  - Comprehensive event testing ensures reliability of CQRS patterns and prevents regressions in critical event-driven features
  - Enables confident refactoring of event handling code with solid test coverage
  - Standardizes testing approaches across the hexagonal architecture layers
  - Supports the hybrid event bus design (ADR 0007) with appropriate testing for both MPSC and broadcast patterns
  - Improves developer productivity by providing reusable testing utilities and patterns

*   **Negative**:
  - Increased test setup complexity compared to simple unit tests
  - Additional development time for creating mock implementations and testing utilities
  - Potential for over-testing if not balanced with integration tests
  - Learning curve for developers new to event-driven testing patterns

*   **Risks**:
  - Mock implementations may drift from real event bus behavior if not maintained
  - Async testing complexity could introduce flaky tests if timeouts aren't handled properly
  - Performance impact of comprehensive event testing in CI/CD pipelines

*   **Mitigation**:
  - Regular review of mock implementations against real interfaces
  - Standardized timeout values and retry logic for async tests
  - Selective application of comprehensive testing to critical event paths

## Technical Validation

### Research Findings Validation
The selected patterns are validated against established Rust ecosystem practices:

**CQRS Testing**: The Given-When-Then pattern from rust-cqrs provides proven aggregate testing with clear separation of setup, execution, and verification phases.

**Mock Implementation**: Trait-based mocking using `Arc<dyn Trait>` is the Rust idiomatic approach for dependency injection testing, ensuring compile-time safety and runtime flexibility.

**Async Patterns**: Tokio's channel-based testing aligns with the async runtime choice (ADR 0002) and provides reliable concurrent testing without race conditions.

**Event Ordering**: Generation counters and timestamps follow event sourcing best practices for maintaining event stream integrity.

### Compatibility with Architecture
The patterns fully support Lithos' architectural decisions:
- Hexagonal architecture (domain ports enable mocking)
- CQRS separation (command testing vs. query testing)
- Hybrid event bus (ADR 0007) requires both MPSC and broadcast testing patterns
- Async-first design requires tokio-native testing approaches

### Performance Considerations
The testing patterns are designed for performance:
- Mock implementations avoid real I/O overhead
- Async testing leverages tokio's efficient runtime
- Selective comprehensive testing focuses on critical paths
- CI/CD optimization through parallel test execution

## Status Tracking

*   **Proposed**: 2026-01-11
*   **Accepted**: 2026-01-11
*   **Implemented**: 2026-01-11
