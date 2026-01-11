# Story 2.3: establish-cqrs-testing-patterns

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Executive Summary

**Objective:** Establish comprehensive CQRS testing framework with command/query isolation, event sourcing validation, eventual consistency testing, security integration, and CI/CD automation.

**Key Deliverables:**
- 8 testing patterns covering unit, integration, security, and performance testing
- cqrs-es TestFramework integration with Given-When-Then aggregate testing
- Mock repositories (commands) and stubbed data stores (queries) with Arc<dyn Trait> patterns
- Eventual consistency validation with precise timing control
- Security, observability, and CI/CD testing integration

**Business Value:** Ensures reliable CQRS implementation with 90%+ test coverage, preventing production bugs in complex event-driven workflows.

## Story

As a developer testing CQRS command and query separation,
I want patterns for testing write operations and read models separately,
So that command side and query side code are tested in isolation with proper verification.

## Acceptance Criteria

**Command Handler Testing:**
- **Given** CQRS testing patterns are established
- **When** testing a command handler
- **Then** verifies command validation logic, state changes are applied correctly, domain events are published, and error cases are handled appropriately

**Query Handler Testing:**
- **Given** CQRS testing patterns are established
- **When** testing a query handler
- **Then** verifies query execution returns correct data, query performance meets requirements, query isolation from write operations, and caching behavior if applicable

**CQRS Separation Verification:**
- **Given** I have researched CQRS testing patterns
- **When** reviewing CQRS testing infrastructure
- **Then** patterns are established for command/query separation verification and eventual consistency testing between write and read models

**DDD Integration:**
- **Given** I have researched CQRS testing in Rust ecosystems
- **When** checking the implementation
- **Then** addresses common CQRS testing challenges: testing eventual consistency, mocking read model updates, verifying command/query separation, and testing cross-aggregate consistency

## Tasks / Subtasks

- [ ] Research CQRS testing patterns and implement core framework **[Effort: 4-5 hours | Complexity: Medium]**
  - [ ] Implement cqrs-es TestFramework for Given-When-Then aggregate testing with event verification
  - [ ] Create mock repository implementations using Arc<dyn RepositoryPort> for command isolation
  - [ ] Develop stubbed query data stores with configurable test data for read model testing
  - [ ] Build event verification utilities for published domain events with serde payload comparison
- [ ] Establish command handler testing patterns **[Effort: 3-4 hours | Complexity: High]**
  - [ ] Implement mock-based command testing with event capture and payload verification
  - [ ] Create validation testing for command inputs, business rules, and error scenarios
  - [ ] Develop async command testing with tokio integration and timeout handling
  - [ ] Build error scenario testing for repository failures and constraint violations
- [ ] Establish query handler testing patterns **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Implement stub-based query testing with predictable data isolation
  - [ ] Create result transformation testing for sorting, filtering, and aggregation
  - [ ] Develop performance validation for query execution time and resource usage
  - [ ] Build caching behavior testing with mock cache layers and invalidation logic
- [ ] Implement event sourcing aggregate testing **[Effort: 4-5 hours | Complexity: High]**
  - [ ] Establish Given-When-Then testing with initial event history loading
  - [ ] Create command execution testing with parameter validation and error handling
  - [ ] Develop event sequence verification with exact payload matching and ordering
  - [ ] Build versioning support for event schema evolution and migration testing
- [ ] Create eventual consistency testing patterns **[Effort: 3-4 hours | Complexity: High]**
  - [ ] Implement timing control for write/read model synchronization testing
  - [ ] Develop cross-aggregate event flow testing with multiple read model updates
  - [ ] Create failure recovery testing for read model rebuild scenarios
  - [ ] Build race condition prevention testing for concurrent operations
- [ ] Establish integration and validation testing **[Effort: 3-4 hours | Complexity: Medium]**
  - [ ] Implement command-to-query workflow integration testing
  - [ ] Create multi-aggregate saga testing for complex business transactions
  - [ ] Develop security testing integration for authorization and input sanitization
  - [ ] Build observability testing for logging, metrics, and tracing validation
- [ ] Create testing documentation and CI/CD integration **[Effort: 2-3 hours | Complexity: Low]**
  - [ ] Write comprehensive CQRS testing guidelines with examples and anti-patterns
  - [ ] Implement CI/CD integration with automated test execution and quality gates
  - [ ] Create performance benchmarking for CQRS operations under load
  - [ ] Establish test data management with factories and fixtures for maintainability

## Dev Notes

- **ADR 0008 Analysis Integration**: Follow validated CQRS testing patterns for optimal command/query separation testing and eventual consistency verification.

- **Architecture Compliance**: Builds on CQRS architecture (ADR 0002, ADR 0007) with comprehensive testing for write/read model separation.

- **Implementation Priority**: Start with TestFramework infrastructure (Priority 1), then mock implementations (Priority 2), finally advanced patterns (Priority 3-4).

- **Source Tree Components**: CQRS test utilities in crates/test-utils/src/cqrs.rs, mock repositories in tests/, guidelines in docs/testing/cqrs-testing.md.

- **Quality Assurance**: CQRS tests ensure proper command/query separation, event publishing verification, and eventual consistency across aggregates.

### Project Structure Notes

- **Alignment with unified project structure**: CQRS testing integrates with event testing from Story 2.2 and hexagonal architecture domain ports.

- **Detected conflicts or variances**: None - extends existing testing infrastructure with CQRS-specific patterns.

### References

- [ADR 0008: Event-Driven Testing Patterns](docs/adr/0008-event-driven-testing-patterns.md) - CQRS testing patterns and validation
- [Source: epics/epic-2-test-architecture-patterns-utilities-mvp-core.md#Story 2.3]
- [ADR 0002: Storage - Redb + rkyv](docs/adr/0002-storage-redb-rkyv.md) - CQRS foundation
- [ADR 0009: CQRS Testing Patterns](docs/adr/0009-cqrs-testing-patterns.md) - Comprehensive CQRS testing framework and detailed implementation patterns
- [ADR 0008: Event-Driven Testing Patterns](docs/adr/0008-event-driven-testing-patterns.md) - Event testing foundation for CQRS validation
- [ADR 0007: Hybrid Event Orchestration](docs/adr/0007-event-orchestration.md) - Event bus architecture for CQRS operations
- [Research: CQRS Testing Best Practices - https://reintech.io/blog/testing-strategies-cqrs-applications]
- [Research: Mocks for Commands, Stubs for Queries - https://blog.ploeh.dk/2013/10/23/mocks-for-commands-stubs-for-queries/]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Dev Notes

- **ADR 0009 Analysis Integration**: Follow the comprehensive CQRS testing framework from ADR 0009 for optimal command/query separation testing, including security, observability, and CI/CD integration.

- **Architecture Compliance**: Implements all ADR 0009 testing patterns supporting hexagonal architecture, async operations, and hybrid event bus (ADR 0007) with full CQRS separation validation.

- **Implementation Priority**: Start with core TestFramework infrastructure (Priority 1), then command/query patterns (Priority 2-3), followed by advanced features (Priority 4-6).

- **Source Tree Components**: CQRS test utilities in crates/test-utils/src/cqrs.rs, mock implementations across crates/test-utils/src/mocks/, security helpers, performance utilities, and comprehensive testing guidelines.

- **Quality Assurance**: CQRS tests ensure complete validation of command/query separation, event publishing, eventual consistency, cross-aggregate workflows, security, and observability with production-ready patterns.

### Project Structure Notes

- **Alignment with unified project structure**: CQRS testing builds on Story 2.2 event patterns and integrates with existing test infrastructure.

- **Detected conflicts or variances**: None - follows established hexagonal architecture and async testing conventions.

### Technical Requirements

**Command Handler Testing (ADR 0009 Decision 1):**
- Mock repositories using Arc<dyn RepositoryPort> that record interactions and return controlled data
- Event verification with exact payload matching using serde comparison
- Command validation testing with invalid inputs, boundary conditions, and business rule violations
- Async testing with tokio integration for command execution and event publishing

**Query Handler Testing (ADR 0009 Decision 2):**
- Stubbed data stores returning predefined datasets without external dependencies
- Result transformation validation for sorting, filtering, aggregation, and pagination
- Performance testing with execution time bounds and resource usage patterns
- Caching verification with mock cache layers for hit/miss scenarios

**Event Sourcing Testing (ADR 0009 Decision 3):**
- Given-When-Then framework with initial event history loading and proper ordering
- Command execution with parameter validation and error handling
- Event sequence verification with exact payload correctness and aggregate state reconstruction
- Versioning support for event schema evolution and concurrent command testing

**Eventual Consistency Testing (ADR 0009 Decision 4):**
- Controlled timing simulation using tokio::time for write/read model synchronization
- Cross-aggregate verification for event flows updating multiple read models
- Failure recovery testing for read model rebuild scenarios and consistency after failures
- Race condition prevention with deterministic testing for concurrent operations

**Integration Testing (ADR 0009 Decision 5):**
- Command execution followed by query verification through real event bus implementations
- Multi-aggregate saga testing for complex business transactions spanning aggregates
- Performance benchmarking for CQRS operation throughput and latency under load
- Security testing integration for authorization, input sanitization, and audit trails

**Validation & Error Testing (ADR 0009 Decision 6):**
- Comprehensive error scenario coverage for invalid commands, query failures, and event processing
- Security testing for authorization controls and malicious payload handling
- Observability testing for event tracing, performance metrics, and health indicators

### File Structure Requirements

- CQRS test utilities in crates/test-utils/src/cqrs.rs with cqrs-es TestFramework integration and Given-When-Then helpers
- Mock repositories in crates/test-utils/src/mocks/repository.rs using Arc<dyn RepositoryPort> traits
- Mock query stores in crates/test-utils/src/mocks/query_store.rs with configurable test data and pagination support
- Event verification utilities in crates/test-utils/src/events/verification.rs with serde payload comparison
- CQRS testing examples in crates/*/tests/cqrs_*.rs demonstrating all ADR 0009 patterns (see examples below)
- Security testing helpers in crates/test-utils/src/security/ with authorization and input sanitization mocks
- Performance testing utilities in crates/test-utils/src/benchmarking/ for CQRS operation load testing
- CQRS testing guidelines in docs/testing/cqrs-testing.md with comprehensive examples, anti-patterns, and CI/CD integration

### Code Examples for Complex Patterns

**Eventual Consistency Testing:**
```rust
#[tokio::test]
async fn command_eventually_updates_read_model() {
    // Setup
    let mock_repo = Arc::new(MockUserRepository::new());
    let event_bus = Arc::new(TestEventBus::new());
    let read_model = Arc::new(InMemoryUserReadModel::new(event_bus.clone()));

    // Execute command
    let cmd = CreateUserCommand { /* ... */ };
    let handler = CreateUserHandler::new(mock_repo, event_bus);
    handler.handle(cmd).await.unwrap();

    // Wait for eventual consistency with timeout
    tokio::time::timeout(
        Duration::from_millis(100),
        read_model.wait_for_user(user_id)
    ).await.expect("Read model not updated within timeout");

    // Verify read model state
    assert_eq!(read_model.get_user(user_id).unwrap().email, expected_email);
}
```

**Cross-Aggregate Saga Testing:**
```rust
#[tokio::test]
async fn order_saga_updates_inventory_and_payment() {
    let inventory_repo = Arc::new(MockInventoryRepository::new());
    let payment_repo = Arc::new(MockPaymentRepository::new());
    let event_bus = Arc::new(TestEventBus::new());

    // Setup saga participants
    let inventory_handler = InventoryHandler::new(inventory_repo, event_bus.clone());
    let payment_handler = PaymentHandler::new(payment_repo, event_bus.clone());

    // Execute order command
    let order_cmd = PlaceOrderCommand { /* ... */ };
    let order_handler = OrderHandler::new(event_bus);
    order_handler.handle(order_cmd).await.unwrap();

    // Verify saga completion - both aggregates updated
    assert!(inventory_repo.item_reserved(item_id).await);
    assert!(payment_repo.payment_processed(payment_id).await);
}
```

### Testing Requirements

- Unit tests for all CQRS testing utilities, mock implementations, and framework components
- Integration tests for complete command-to-query workflows with real event bus validation
- Async testing with tokio runtime verification for all concurrent CQRS operations
- Eventual consistency tests with precise timing control and race condition prevention
- Cross-aggregate testing for multi-aggregate sagas and complex business transaction workflows
- Security tests for authorization, input sanitization, and audit trail verification
- Performance tests with benchmarking for CQRS operation throughput, latency, and resource usage
- Observability tests for logging, metrics collection, tracing correlation, and health indicators
- CI/CD validation with automated quality gates for test coverage (>90%) and performance regression detection
- Performance tests ensuring CQRS patterns don't introduce overhead

### Previous Story Intelligence

- Story 2.2 established event testing patterns (ADR 0008) - integrate with CQRS event verification, publishing validation, and hybrid event bus testing
- Story 2.1 established async testing infrastructure - leverage tokio runtime for all CQRS async operations, timing control, and concurrent testing
- ADR 0008 provides event testing foundation - combine with ADR 0009 for complete CQRS and event testing coverage across all patterns

### Git Intelligence Summary

- Recent commits show event bus patterns - CQRS testing builds on event publishing foundations
- CQRS patterns align with established domain event and aggregate conventions

### Latest Tech Information

- CQRS Testing: cqrs-es TestFramework enables Given-When-Then aggregate testing with comprehensive event verification and versioning support
- Command Patterns: Mock repositories with Arc<dyn Trait> provide complete isolation while enabling event capture and validation testing
- Query Patterns: Stubbed data stores with configurable fixtures support performance, caching, and transformation testing
- Eventual Consistency: tokio::time enables precise timing control for write/read synchronization and race condition testing
- Security Integration: Authorization testing and input sanitization validation integrated into CQRS workflows
- Observability: Event tracing, metrics collection, and health checks validated through CQRS operation testing
- CI/CD: Automated test execution with quality gates, performance regression detection, and environment-specific testing

### Project Context Reference

- Lithos implements CQRS with event sourcing for optimal read/write separation and scalability
- Commands publish domain events that asynchronously update read models through the hybrid event bus (ADR 0007)
- Testing requires complete isolation of command side (mocks) from query side (stubs) operations
- Domain aggregates must be tested with event sourcing patterns for state reconstruction and business rule validation
- Eventual consistency between write and read models needs precise timing control and race condition prevention
- Security, observability, and performance requirements demand comprehensive testing coverage
- CI/CD integration requires automated testing with quality gates and performance regression detection

### Migration Path for Existing CQRS Tests

**Phase 1: Foundation (Week 1)**
- Install cqrs-es dependency and set up basic TestFramework
- Create initial mock repositories and query stubs
- Migrate 2-3 simple command handler tests to use new patterns

**Phase 2: Core Patterns (Weeks 2-3)**
- Implement Given-When-Then for existing aggregate tests
- Replace manual mocks with Arc<dyn Trait> implementations
- Add event verification to command handler tests

**Phase 3: Advanced Features (Weeks 4-5)**
- Introduce eventual consistency testing for complex workflows
- Add security and observability test layers
- Implement performance benchmarking

**Phase 4: Optimization (Week 6)**
- Refactor remaining manual tests to use framework
- Implement CI/CD integration and quality gates
- Add comprehensive documentation and team training

**Migration Benefits:**
- 60-70% reduction in test boilerplate code
- Improved test reliability and maintainability
- Enhanced coverage of edge cases and error scenarios
- Automated performance regression detection

### Story Completion Status

- Status: ready-for-dev
- All acceptance criteria defined with comprehensive CQRS testing requirements including security and observability
- Technical requirements complete with all ADR 0009 patterns: command/query testing, event sourcing, eventual consistency, integration, security, and CI/CD
- Integration points identified with event testing (ADR 0008), async infrastructure (Story 2.1), and hybrid event bus (ADR 0007)
- Migration path provided for teams with existing CQRS tests to transition smoothly
- Risk assessment: Low risk, follows validated ADR 0009 patterns with extensive research and real-world case studies
- Execution Optimization: Follow ADR 0009 comprehensive framework for maximum efficiency, security, and production readiness
