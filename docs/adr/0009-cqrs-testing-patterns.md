---
name: cqrs-testing-patterns-and-best-practices
status: accepted
stakeholders: [Jack (Developer)]
date_proposed: 2026-01-11
date_decided: 2026-01-11
date_implemented: 2026-01-11
---

# ADR 0009: CQRS Testing Patterns and Best Practices

## Context

Lithos implements CQRS (Command Query Responsibility Segregation) with event sourcing, separating write operations (commands) from read operations (queries) for optimal performance and scalability. The testing strategy requires comprehensive patterns to ensure command handlers, query handlers, aggregates, and eventual consistency work correctly in isolation and integration.

Research revealed several established CQRS testing approaches:

**Command Testing Patterns:**

- Mock repositories for command isolation
- Event publishing verification
- Validation logic testing
- Error handling for invalid commands

**Query Testing Patterns:**

- Stubbed data stores for predictable results
- Performance testing for read operations
- Caching behavior verification
- Data transformation testing

**Event Sourcing Testing:**

- Given-When-Then framework for aggregates
- Event sequence verification
- State reconstruction testing
- Event versioning validation

**Integration Testing:**

- Command-to-query eventual consistency
- Cross-aggregate interactions
- Eventual consistency timing verification

The project needs testing patterns that support hexagonal architecture, async operations, and the hybrid event bus from ADR 0007.

## Decision

We will establish comprehensive CQRS testing patterns based on industry best practices and Rust ecosystem capabilities:

### 1. Command Handler Testing Framework

Implement mock-based command testing with event verification using cqrs-es TestFramework:

- **Mock Repositories**: Use `Arc<dyn RepositoryPort>` mocks that record interactions and return controlled data
- **Event Verification**: Capture published events with exact payload matching using serde comparison
- **Validation Testing**: Test command validation logic with invalid inputs, boundary conditions, and business rule violations
- **Error Scenarios**: Verify proper error handling for repository failures, constraint violations, and concurrent modification conflicts
- **Async Testing**: Ensure tokio-compatible mocking for async command execution and event publishing

### 2. Query Handler Testing Framework

Implement stub-based query testing with predictable data isolation:

- **Stubbed Data Stores**: Use in-memory stubs that return predefined datasets without external dependencies
- **Result Transformation Testing**: Verify query logic transforms data correctly (sorting, filtering, aggregation)
- **Performance Validation**: Test query execution time bounds and resource usage patterns
- **Caching Verification**: Mock cache layers to test hit/miss scenarios and cache invalidation logic
- **Pagination Testing**: Validate pagination logic with various page sizes and edge cases

### 3. Event Sourcing Aggregate Testing

Establish Given-When-Then testing for aggregates with comprehensive state verification:

- **Given Phase**: Load initial event history with proper ordering and versioning
- **When Phase**: Execute commands with parameter validation and error handling
- **Then Phase**: Verify exact event sequence, payload correctness, and aggregate state reconstruction
- **Versioning Support**: Test event schema evolution and migration scenarios
- **Concurrent Command Testing**: Simulate concurrent command execution and conflict resolution

### 4. Eventual Consistency Testing

Create sophisticated patterns for write/read model synchronization testing:

- **Timing Control**: Use tokio::time for precise control over event propagation delays
- **Cross-Aggregate Verification**: Test event flows that update multiple read models
- **Consistency Windows**: Define acceptable consistency lag times and test boundary conditions
- **Failure Recovery**: Test read model rebuild scenarios and eventual consistency after failures
- **Race Condition Prevention**: Implement deterministic testing for concurrent write/read operations

### 5. Integration Testing Strategy

Complement unit tests with comprehensive integration approaches:

- **Command-to-Query Workflows**: Execute commands then verify read model updates through queries
- **Event Bus Integration**: Test complete event flows using real event bus implementations
- **Multi-Aggregate Sagas**: Test complex business transactions spanning multiple aggregates
- **Performance Benchmarking**: Measure CQRS operation throughput and latency under load
- **Resource Usage Testing**: Validate memory usage, connection pooling, and database load patterns

### 6. Validation and Error Testing

Ensure comprehensive error scenario coverage with systematic failure testing:

- **Command Validation**: Test all validation rules with invalid inputs, missing fields, and constraint violations
- **Query Error Handling**: Verify graceful degradation for data access failures and partial result scenarios
- **Event Processing**: Test malformed events, duplicate processing, and error recovery mechanisms
- **Aggregate Invariants**: Validate business rule enforcement and state consistency across operations
- **Infrastructure Failures**: Test behavior during repository outages, network failures, and timeout scenarios

### 7. Security Testing Integration

Incorporate security validation into CQRS testing patterns:

- **Authorization Testing**: Verify command/query access controls and permission enforcement
- **Input Sanitization**: Test for injection vulnerabilities and malicious payload handling
- **Audit Trail Verification**: Ensure security events are properly recorded and retrievable
- **Data Leakage Prevention**: Validate that queries don't expose unauthorized information

### 8. Observability and Monitoring Testing

Test logging, metrics, and tracing in CQRS operations:

- **Event Tracing**: Verify correlation IDs propagate through command-event-query flows
- **Performance Metrics**: Test metric collection for latency, throughput, and error rates
- **Health Checks**: Validate system health indicators during various CQRS operation states
- **Debug Logging**: Ensure appropriate log levels and structured logging for troubleshooting

## Alternatives Considered

### Alternative 1: End-to-End Only Testing

- **Pros**: Tests complete workflows, catches integration issues
- **Cons**: Slow execution, difficult to isolate failures, brittle tests, hard to test error scenarios

### Alternative 2: Manual Testing Frameworks

- **Pros**: Flexible, no dependencies
- **Cons**: Inconsistent approaches, error-prone, poor maintainability, duplicate code across tests

### Alternative 3: Generic Testing Libraries Only

- **Pros**: Standardized tooling, community support
- **Cons**: May not fit CQRS-specific needs, potential over-abstraction, learning curve

### Alternative 4: Integration Testing Only

- **Pros**: Tests realistic scenarios, verifies system behavior, catches environment-specific issues
- **Cons**: Slow feedback, hard to debug failures, external dependencies required, difficult to test edge cases

### Alternative 5: Record-Replay Testing

- **Pros**: Tests with real production data, validates against actual usage patterns
- **Cons**: Complex setup, data privacy concerns, brittle tests that break with schema changes

### Alternative 6: Contract Testing

- **Pros**: Validates API contracts between commands/queries and external systems
- **Cons**: Additional tooling overhead, requires consumer-driven contract definition

## Technical Validation

### Research Alignment

The selected patterns align with CQRS testing best practices:

**Command Testing**: Mock repositories follow "Mocks for Commands" principle from testing literature
**Query Testing**: Stubbed data stores implement "Stubs for Queries" approach
**Event Sourcing**: Given-When-Then framework matches aggregate testing standards
**Integration**: Eventual consistency patterns address CQRS-specific challenges

### Lithos Architecture Fit

The patterns perfectly support Lithos' design:

- Hexagonal architecture enables clean mocking of ports
- Async-first design requires tokio-compatible testing
- Hybrid event bus (ADR 0007) supports event flow testing
- CQRS separation demands isolated command/query testing

### Performance Considerations

Testing patterns designed for efficiency:

- Mock implementations avoid I/O overhead
- Isolated unit tests provide fast feedback
- Selective integration testing for critical paths
- Async testing leverages tokio for performance

## Consequences

- **Positive**:
  - Ensures CQRS command/query separation works correctly in isolation and integration
  - Provides confidence in event sourcing aggregate behavior and state reconstruction
  - Validates eventual consistency between write and read models
  - Enables fast feedback during development with isolated unit tests
  - Supports hexagonal architecture with mockable ports and adapters
- **Negative**:
  - Additional complexity in setting up mock repositories and stubbed data stores
  - Potential for mock implementations to drift from real interfaces
  - Learning curve for Given-When-Then testing patterns
  - Maintenance overhead for keeping test data in sync with domain changes
