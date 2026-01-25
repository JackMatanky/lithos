# Epic 7: Event Bus & Orchestration Infrastructure **[PHASE 1.5]**

System has a robust event-driven architecture enabling loose coupling between services and supporting concurrent operations without god-objects.
**FRs covered:** Architecture requirements (event-driven, CQRS foundation)
**Implementation Notes:**

- Hybrid Event Bus (MPSC/Broadcast/Watch per ADR 0007)
- Event payload schema design and validation
- Event persistence for debugging and recovery
- EventBusPort mocks for testing
- Integration contracts for other epics
- Prevents god-object orchestrators (Go lesson learned)
- May create ADR for event patterns if architectural decisions made

## Story 7.1: Create Event Bus Domain Interface and Port

As a developer implementing event-driven architecture,
I want a clean domain interface for event operations,
So that events can be published and subscribed to through well-defined contracts.

**Acceptance Criteria:**

**Given** I need event bus contracts
**When** I create the EventBusPort trait
**Then** it includes publish and subscribe methods for domain events

**Given** EventBusPort is defined
**When** I implement mocks for testing
**Then** test doubles are available for isolated event testing

**Given** the domain interface exists
**When** I validate the design
**Then** it follows hexagonal principles with async event handling

## Story 7.2: Define Complete Domain Event Types

As a developer coordinating events across the system,
I want complete domain event definitions,
So that all events from Epics 3-6 are properly defined and coordinated.

**Acceptance Criteria:**

**Given** events are defined in Epics 3, 4, 5, 6
**When** I consolidate all domain events
**Then** complete event type definitions exist with consistent naming and payloads

**Given** event types are defined
**When** I validate consistency
**Then** all events follow EventBusPort contracts and have proper serialization

**Given** events are consolidated
**When** I check for completeness
**Then** all system events are defined (ConfigurationLoaded, SchemaLoaded, NoteIndexed, etc.)

## Story 7.3: Implement MPSC Data Plane

As a developer needing reliable event delivery,
I want MPSC data plane for indexing operations,
So that events are delivered reliably without loss in the indexing pipeline.

**Acceptance Criteria:**

**Given** I need reliable event delivery
**When** I implement MPSC data plane per ADR 0007
**Then** actor-based mailbox pattern handles indexing events

**Given** MPSC data plane is implemented
**When** I test event delivery
**Then** events are processed in order without loss

**Given** high event volume occurs
**When** I monitor performance
**Then** bounded channels prevent memory issues during indexing

## Story 7.4: Implement Broadcast Control Plane

As a developer needing global signaling,
I want broadcast control plane for system signals,
So that shutdown and global notifications work across all components.

**Acceptance Criteria:**

**Given** I need global signaling
**When** I implement broadcast control plane per ADR 0007
**Then** shutdown and system-wide notifications are supported

**Given** broadcast control plane is implemented
**When** I send global signals
**Then** all subscribers receive notifications reliably

**Given** system shutdown occurs
**When** I broadcast shutdown signal
**Then** graceful shutdown happens across all components

## Story 7.5: Implement Watch State Plane

As a developer needing state synchronization,
I want watch state plane for LSP integration,
So that real-time state changes are communicated to IDE integrations.

**Acceptance Criteria:**

**Given** I need state synchronization
**When** I implement watch state plane per ADR 0007
**Then** LSP clients receive real-time vault state updates

**Given** watch state plane is implemented
**When** I monitor state changes
**Then** subscribers get immediate notifications of state updates

**Given** LSP integration is active
**When** I change vault state
**Then** watch notifications enable sub-50ms IDE responsiveness

## Story 7.6: Implement Event Publishing and Subscription

As a developer using the event system,
I want complete publish/subscribe functionality,
So that components can publish events and subscribe to relevant notifications.

**Acceptance Criteria:**

**Given** event planes are implemented
**When** I add publishing functionality
**Then** components can publish events to appropriate planes

**Given** publishing is implemented
**When** I add subscription functionality
**Then** components can subscribe to event types they need

**Given** publish/subscribe is complete
**When** I test end-to-end
**Then** events flow from publishers to subscribers correctly

## Story 7.7: Add Event Payload Validation and Error Handling

As a developer ensuring event integrity,
I want event payload validation and error handling,
So that malformed events are caught and handled gracefully.

**Acceptance Criteria:**

**Given** events are published
**When** I validate payloads
**Then** event structure and required fields are checked

**Given** validation fails
**When** I handle errors
**Then** clear error messages are logged without crashing the system

**Given** malformed events occur
**When** I process them
**Then** system continues operating with degraded functionality for bad events

## Story 7.8: Implement Event Persistence for Debugging

As a developer debugging event flows,
I want event persistence capabilities,
So that event history can be inspected for troubleshooting and system analysis.

**Acceptance Criteria:**

**Given** I need event debugging
**When** I implement event persistence
**Then** recent events are stored for inspection

**Given** event persistence is implemented
**When** I debug issues
**Then** event sequences can be replayed and analyzed

**Given** persistence is active
**When** I check performance impact
**Then** persistence adds minimal overhead to normal operations

## Story 7.9: Define Event Bus Integration Contracts

As a developer integrating with the event system,
I want clear integration contracts,
So that other epics know how to publish and subscribe to events.

**Acceptance Criteria:**

**Given** event system is implemented
**When** I define integration contracts
**Then** clear patterns exist for event publishing in each epic

**Given** integration contracts exist
**When** other epics integrate
**Then** consistent subscription patterns are followed

**Given** contracts are defined
**When** I validate system integration
**Then** all epics properly integrate with the event bus

## Story 7.10: Create Event Bus Mocks for Testing

As a developer testing event-driven code,
I want comprehensive event bus mocks,
So that event interactions can be tested in isolation.

**Acceptance Criteria:**

**Given** I need to test event interactions
**When** I create event bus mocks
**Then** mock implementations allow testing different event scenarios

**Given** mocks are available
**When** I write event-driven tests
**Then** tests can verify event publishing and subscription logic

**Given** integration tests are needed
**When** I use mocks
**Then** they simulate real event bus behavior for comprehensive testing

## Story 7.11: Review Epic 7 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 7 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 7 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests

**Given** all Epic 7 public APIs are documented
**When** I verify doc test coverage
**Then** all public components have runnable doc tests demonstrating usage

**Given** all Epic 7 components are implemented with tests
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
**Then** test execution completes in <30 seconds for the full Epic 7 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation

## Story 7.12: Document Event Bus Integration for Developers

As a developer integrating with the event system,
I want comprehensive developer documentation for event bus usage,
So that other epics can properly publish and subscribe to events.

**Acceptance Criteria:**

**Given** event system is implemented
**When** I create developer documentation
**Then** it includes event publishing/subscription patterns and integration contracts

**Given** documentation exists
**When** developers read it
**Then** they understand how to integrate with the event bus in their epics

**Given** integration docs are complete
**When** other epics implement event integration
**Then** they follow consistent patterns without architectural review
