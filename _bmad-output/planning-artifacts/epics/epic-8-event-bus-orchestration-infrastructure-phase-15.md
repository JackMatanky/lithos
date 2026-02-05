# Epic 8: Event Bus & Orchestration Infrastructure **[PHASE 1.5]**

System has a robust event-driven architecture enabling loose coupling between services and supporting concurrent operations without god-objects.

**FRs covered:** Architecture requirements (event-driven, CQRS foundation)

## Implementation Notes

- **Hybrid Event Bus**: Three planes per ADR 004 (event-orchestration.md)
  - **MPSC Data Plane**: Reliable indexing pipeline (bounded channels, FIFO ordering)
  - **Broadcast Control Plane**: Global signals (shutdown, system-wide notifications)
  - **Watch State Plane**: LSP state sync (latest-only, sub-50ms responsiveness)
- **Architecture**: Hexagonal (EventBusPort in domain, HybridEventBus in adapters)
- **Location**: `crates/domain/src/ports/event_bus.rs` for port, `crates/adapters/src/spi/event/` for implementations
- **Libraries**: tokio (mpsc/broadcast/watch), async-trait, mockall, tracing, thiserror, serde, rkyv
- **Event Types**: Consolidated from domain events (ConfigEvents, SchemaEvents, TemplateEvents, NoteEvents)
- **Persistence**: Epic 5 CacheCommand integration for event history (debugging/replay)
- **Zero-Copy**: Arc-wrapped payloads for broadcast/watch (prevent memory duplication)
- **Testing**: MockEventBusPort with expectation API, concurrent subscription tests
- **Prevents God Objects**: No central orchestrator (lesson from Go experience per ADR 004)

---

## Story 8.1: Create Event Bus Domain Interface and Port

As a developer implementing event-driven architecture,
I want a clean domain interface for event operations,
So that events can be published and subscribed to through well-defined contracts.

**Acceptance Criteria:**

**Given** I need event bus contracts in the domain
**When** I create EventBusPort trait in `crates/domain/src/ports/event_bus.rs`
**Then** it defines async methods: publish, subscribe, shutdown
**And** it uses async-trait for async support
**And** it is object-safe for dynamic dispatch

**Given** domain events need consolidation
**When** I define DomainEvent enum in `crates/domain/src/events/mod.rs`
**Then** it consolidates all domain events: ConfigEvents, SchemaEvents, TemplateEvents, NoteEvents
**And** it derives Clone for Arc-wrapping
**And** it includes Send + Sync for cross-thread event passing

**Given** event type filtering is needed
**When** I define EventType enum
**Then** it provides subscription filters: ConfigUpdated, SchemaCreated, PropertyBankUpdated, TemplateCreated, NoteCreated, FrontmatterValidated, All (wildcard)
**And** it derives Hash + Eq for subscription map keys

**Given** async receivers are needed
**When** I define Receiver type
**Then** it wraps tokio receivers: Mpsc, Broadcast, Watch variants
**And** all variants use Arc<DomainEvent> for zero-copy sharing
**And** receiver provides unified async recv() interface

**Given** error handling is required
**When** I define EventError enum in `crates/adapters/src/spi/event/errors.rs`
**Then** it includes variants: ChannelClosed, PublishTimeout, SubscriptionFailed, AlreadyShutdown, InvalidPayload
**And** all variants are Send + Sync for async contexts
**And** error messages follow ADR 005 (actionable diagnostics)

**Given** testing requires mocks
**When** I annotate EventBusPort with mockall::automock
**Then** MockEventBusPort is auto-generated
**And** mocks support expectation setting for verification

---

## Story 8.2: Define Complete Domain Event Types

As a developer coordinating events across the system,
I want complete domain event definitions,
So that all events from Epics 3-7 are properly defined and coordinated.

**Acceptance Criteria:**

**Given** events are defined in multiple domain modules
**When** I consolidate in `crates/domain/src/events/mod.rs`
**Then** re-export all event types from Epic 6 (ConfigEvents), Epic 7 (SchemaEvents), Epic 12 (TemplateEvents), Epic 10 (NoteEvents)
**And** define DomainEvent enum wrapping all event types

**Given** event inventory is needed
**When** I document events by source epic
**Then** Epic 6: ConfigUpdated (source, timestamp)
**And** Epic 7: SchemaCreated (id, name, timestamp), PropertyBankUpdated (property_count, timestamp)
**And** Epic 12: TemplateCreated (id, name, timestamp)
**And** Epic 10: NoteCreated (id, path, timestamp), FrontmatterValidated (note_id, field_count, timestamp)

**Given** event naming conventions
**When** I validate event names
**Then** all events use past tense (Created, Updated, Validated)
**And** all events are suffixed with context (ConfigUpdated, not just Updated)

**Given** event routing specification
**When** I define routing rules for implementation
**Then** MPSC Data Plane routes: NoteCreated, FrontmatterValidated (high volume indexing)
**And** Broadcast Control Plane routes: ConfigUpdated, PropertyBankUpdated (system-wide signals)
**And** Watch State Plane routes: SchemaCreated, TemplateCreated (LSP state sync, latest-only)

**Given** event payloads must be valid
**When** I define validation rules
**Then** timestamp must be positive i64 (Unix seconds)
**And** UUIDs must be v7 (time-ordered)
**And** String fields must not be empty (enforced by domain aggregates)

**Given** event serialization is needed for persistence
**When** I choose serialization format
**Then** use rkyv for zero-copy deserialization (per ADR 006)
**And** add rkyv derives to DomainEvent enum

**Given** testing is needed
**When** I write unit tests
**Then** test DomainEvent is Send + Sync + Clone
**And** test serialization round-trip for all event variants

---

## Story 8.3: Implement MPSC Data Plane

As a developer needing reliable event delivery,
I want MPSC data plane for indexing operations,
So that events are delivered reliably without loss in the indexing pipeline.

**Acceptance Criteria:**

**Given** I need reliable indexing event delivery
**When** I implement MPSC plane in `crates/adapters/src/spi/event/mpsc_plane.rs`
**Then** it uses tokio::sync::mpsc with bounded channel
**And** channel capacity handles burst indexing (size configurable, default: 1000)
**And** bounded channel provides backpressure (slows publisher when full)

**Given** publish operations must be reliable
**When** I implement publish()
**Then** it uses async send().await for backpressure support
**And** closed channel returns EventError::ChannelClosed
**And** backpressure blocking is traced with wait duration

**Given** subscription supports multiple consumers
**When** I implement subscribe()
**Then** each subscriber gets dedicated receiver (fan-out pattern)
**And** MPSC pattern: multiple receivers, each sees all events

**Given** event ordering is critical
**When** events are published
**Then** receiver sees events in exact publish order (FIFO guarantee)
**And** ordering is essential for indexing (e.g., NoteCreated before FrontmatterValidated)

**Given** performance targets exist
**When** I measure MPSC performance
**Then** publish latency <1ms (99th percentile)
**And** throughput >10,000 events/sec (vault scan use case)

**Given** error scenarios must be handled
**When** channel operations fail
**Then** sender dropped → ChannelClosed error
**And** channel full → backpressure blocks (not error)
**And** errors include event_type context for debugging

**Given** observability is required
**When** I instrument operations
**Then** use tracing::instrument with plane="mpsc", operation, event_type attributes
**And** emit backpressure warnings when channel fills

**Given** testing is needed
**When** I write unit tests
**Then** test successful publish + receive (happy path)
**And** test channel closed error
**And** test backpressure (fill channel, verify blocking)
**And** test event ordering (publish 1000 events, verify FIFO)
**And** test concurrent publishers (no lost events)

---

## Story 8.4: Implement Broadcast Control Plane

As a developer needing global signaling,
I want broadcast control plane for system signals,
So that shutdown and global notifications work across all components.

**Acceptance Criteria:**

**Given** I need global signal broadcasting
**When** I implement Broadcast plane in `crates/adapters/src/spi/event/broadcast_plane.rs`
**Then** it uses tokio::sync::broadcast channel
**And** channel capacity supports expected subscriber count (default: 16)
**And** broadcast delivers to ALL active subscribers simultaneously

**Given** broadcast semantics must be understood
**When** I document broadcast behavior
**Then** late subscribers receive events published AFTER subscription (no history)
**And** slow subscribers cause lag (oldest message dropped if buffer fills)
**And** dropped messages return RecvError::Lagged(n) to receiver

**Given** publish operations are non-blocking
**When** I implement publish()
**Then** it returns immediately (non-blocking)
**And** returns subscriber count (number who received event)
**And** zero subscribers is valid (event lost, logged as warning)

**Given** subscription is dynamic
**When** I implement subscribe()
**Then** it creates new receiver sharing same channel
**And** all receivers get copies of every event (fan-out)
**And** dropping receiver automatically unsubscribes (RAII)

**Given** graceful shutdown pattern is critical
**When** I implement shutdown signal support
**Then** shutdown coordinator publishes shutdown event to broadcast plane
**And** all services subscribe to shutdown events
**And** services cleanup and exit when shutdown received
**And** coordinator waits for acknowledgment with timeout

**Given** lag handling is needed
**When** subscriber buffer fills
**Then** receiver gets RecvError::Lagged(n) indicating dropped messages
**And** subscriber logs warning with lag count
**And** subscriber chooses: ignore lag, panic, or re-sync state

**Given** performance targets exist
**When** I measure performance
**Then** publish latency <10μs (non-blocking)
**And** delivery to all subscribers <100μs

**Given** testing is needed
**When** I write unit tests
**Then** test broadcast to multiple subscribers (all receive same event)
**And** test late subscriber (no historical events)
**And** test no subscribers (returns 0, event lost)
**And** test lag scenario (slow subscriber, verify RecvError::Lagged)
**And** test graceful shutdown (all subscribers receive shutdown signal)

---

## Story 8.5: Implement Watch State Plane

As a developer needing state synchronization,
I want watch state plane for LSP integration,
So that real-time state changes are communicated to IDE integrations.

**Acceptance Criteria:**

**Given** I need latest-state synchronization
**When** I implement Watch plane in `crates/adapters/src/spi/event/watch_plane.rs`
**Then** it uses tokio::sync::watch channel
**And** watch holds ONLY latest value (no history, no queue)
**And** subscribers always see most recent state

**Given** watch semantics must be understood
**When** I document watch behavior
**Then** late subscribers immediately receive current state (not None)
**And** state updates replace previous value (no append)
**And** subscribers notified only when state changes (deduplication)

**Given** publish operations replace state
**When** I implement publish()
**Then** it is non-blocking (returns immediately)
**And** it replaces previous state (not appending to queue)
**And** rapid updates skip intermediate states (subscriber sees latest only)

**Given** subscription provides current state
**When** I implement subscribe()
**Then** it returns receiver with current state immediately available
**And** receiver can read current state without waiting (borrow())
**And** receiver can wait for state changes (changed().await)

**Given** LSP state synchronization is the use case
**When** I use watch for LSP integration
**Then** SchemaCreated updates trigger LSP autocomplete reload
**And** TemplateCreated updates trigger LSP template suggestions
**And** LSP achieves sub-50ms responsiveness (from state change to IDE notification)

**Given** state deduplication saves resources
**When** I prevent unnecessary notifications
**Then** watch channel deduplicates: if new_state == current_state, no notification
**And** changed().await only returns when state actually changes
**And** deduplication relies on PartialEq implementation for DomainEvent

**Given** late subscriber pattern is important
**When** subscriber joins late (e.g., LSP connects after system startup)
**Then** borrow() immediately returns current state
**And** subscriber syncs to latest without waiting

**Given** performance targets exist
**When** I measure performance
**Then** publish latency <1μs (atomic pointer swap)
**And** notification delivery <10μs (wakes all subscribers)
**And** end-to-end LSP latency <50ms (schema change → autocomplete update)

**Given** testing is needed
**When** I write unit tests
**Then** test initial state (new subscriber sees current state immediately)
**And** test state update (publish → changed() returns)
**And** test deduplication (publish same event twice → changed() returns once)
**And** test late subscriber (subscribe after updates → sees latest state)
**And** test performance (10,000 updates complete quickly)

---

## Story 8.6: Implement Event Publishing and Subscription

As a developer using the event system,
I want complete publish/subscribe functionality,
So that components can publish events and subscribe to relevant notifications.

**Acceptance Criteria:**

**Given** I need unified event bus
**When** I implement HybridEventBus in `crates/adapters/src/spi/event/bus.rs`
**Then** it combines all three planes: mpsc_plane, broadcast_plane, watch_plane
**And** it maintains routing table: EventType → PlaneType mapping
**And** it provides shutdown coordination via tokio::sync::Notify

**Given** event routing must be deterministic
**When** I define routing table
**Then** NoteCreated, FrontmatterValidated → MPSC (high-volume indexing)
**And** ConfigUpdated, PropertyBankUpdated → Broadcast (system-wide signals)
**And** SchemaCreated, TemplateCreated → Watch (LSP state sync, latest-only)

**Given** publish method orchestrates routing
**When** I implement EventBusPort::publish()
**Then** determine event type from DomainEvent
**And** lookup plane from routing table
**And** Arc-wrap event once (zero-copy across all subscribers)
**And** delegate to appropriate plane (MPSC/Broadcast/Watch)
**And** routing is transparent to publisher

**Given** subscribe method orchestrates routing
**When** I implement EventBusPort::subscribe()
**Then** lookup plane from routing table based on EventType
**And** create receiver from appropriate plane
**And** return typed receiver (Receiver enum wrapping plane-specific receiver)
**And** subscriber doesn't need to know which plane is used

**Given** integration examples are needed
**When** I document cross-epic integration patterns
**Then** Epic 6 → Epic 7: ConfigUpdated (Broadcast) triggers PropertyBank reload in Schema system
**And** Epic 7 → Epic 10: SchemaCreated (Watch) triggers note revalidation in Indexer
**And** Epic 10 → LSP: NoteCreated (Watch for state sync) updates IDE diagnostics

**Given** error handling propagates correctly
**When** publish/subscribe fails
**Then** propagate plane-specific errors: ChannelClosed, SubscriptionFailed
**And** no route error: EventError::InvalidPayload

**Given** testing is needed
**When** I write integration tests
**Then** test publish → subscribe roundtrip for each event type
**And** test routing (ConfigUpdated → Broadcast, NoteCreated → MPSC)
**And** test cross-epic integration patterns
**And** test concurrent publishers (multiple threads)
**And** test subscriber resilience (one crashes, others unaffected)

---

## Story 8.7: Add Event Payload Validation and Error Handling

As a developer ensuring event integrity,
I want event payload validation and error handling,
So that malformed events are caught and handled gracefully.

**Acceptance Criteria:**

**Given** events must be valid before publishing
**When** I implement validation in publish()
**Then** validate required fields: source (non-empty), timestamp (positive), UUIDs (v7 format)
**And** validation runs before routing to planes
**And** invalid events return EventError::InvalidPayload immediately (not published)

**Given** error recovery strategies must be defined
**When** publish fails
**Then** ChannelClosed: log error, stop publishing, return error (unrecoverable)
**And** PublishTimeout: retry with exponential backoff (transient failure)
**And** InvalidPayload: log error, emit EventValidationFailed event (programming bug)
**And** AlreadyShutdown: silently discard (graceful shutdown in progress)

**Given** graceful degradation is required
**When** event bus failures occur
**Then** system continues operating without events (degraded mode)
**And** set global flag EVENT_BUS_DEGRADED (atomic bool)
**And** log error with context (event type, failure reason)
**And** health check endpoint reports degraded status

**Given** validation failures need observability
**When** invalid payload detected
**Then** emit EventValidationFailed event (Broadcast plane for system-wide notification)
**And** event includes: original_event_type, validation_error, timestamp
**And** monitoring systems subscribe to detect payload issues

**Given** payload size limits prevent memory exhaustion
**When** I validate payload size
**Then** maximum event size: 64 KB
**And** return InvalidPayload error if exceeded
**And** log oversized event type + actual size

**Given** testing is needed
**When** I write unit tests
**Then** test validation rejects empty strings (source, name fields)
**And** test validation rejects negative timestamps
**And** test validation rejects oversized payloads (>64KB)
**And** test graceful degradation (ChannelClosed → system continues)
**And** test EventValidationFailed emission

---

## Story 8.8: Implement Event Persistence for Debugging

As a developer debugging event flows,
I want event persistence capabilities,
So that event history can be inspected for troubleshooting and system analysis.

**Acceptance Criteria:**

**Given** I need persistent event storage
**When** I integrate Epic 5 caching
**Then** use CacheCommand trait for event writes
**And** cache backend is RedbCache (persistent disk storage per ADR 006)
**And** table name is "event_history" (isolated from schema/config caches)

**Given** event storage must be non-blocking
**When** I implement storage logic
**Then** persist asynchronously after successful publish
**And** persistence failure does NOT fail publish (decoupled, best-effort)
**And** persistence failures logged as warnings

**Given** event keys must be unique and time-ordered
**When** I generate cache keys
**Then** key format: "{timestamp_ms}:{uuid}" (e.g., "1704067200000:01933e3a-...")
**And** millisecond timestamp enables time-range queries
**And** UUID suffix ensures uniqueness (multiple events per millisecond)

**Given** retention policy bounds storage
**When** I implement retention
**Then** keep last 1000 events OR 24 hours (whichever is larger)
**And** cleanup runs hourly (background task)
**And** cleanup deletes events: timestamp < now() - 24h AND count > 1000

**Given** query API enables debugging
**When** I implement query methods
**Then** get_events_since(timestamp_ms) returns events after timestamp
**And** get_events_by_type(event_type) filters by event type
**And** get_last_events(count) returns most recent N events
**And** queries use Redb range scans (efficient for time-ordered keys)

**Given** performance overhead must be minimal
**When** I measure overhead
**Then** persistence adds <100μs to publish latency (99th percentile)
**And** rkyv serialization: <10μs
**And** Redb write: <90μs

**Given** testing is needed
**When** I write unit tests
**Then** test event storage (publish → query returns same event)
**And** test retention policy (insert 1100 events → cleanup removes oldest 100)
**And** test query by timestamp (get_events_since filters correctly)
**And** test persistence failure (Redb unavailable → publish still succeeds)
**And** test performance (overhead <100μs)

---

## Story 8.9: Define Event Bus Integration Contracts

As a developer integrating with the event system,
I want clear integration contracts,
So that other epics know how to publish and subscribe to events.

**Acceptance Criteria:**

**Given** I need comprehensive integration documentation
**When** I define contracts for each epic
**Then** create table documenting: Events Published, Events Subscribed, Purpose
**And** Epic 6: Publishes ConfigUpdated, Subscribes to none
**And** Epic 7: Publishes SchemaCreated/PropertyBankUpdated, Subscribes to ConfigUpdated
**And** Epic 10: Publishes NoteCreated/FrontmatterValidated, Subscribes to SchemaCreated/PropertyBankUpdated
**And** Epic 12: Publishes TemplateCreated, Subscribes to ConfigUpdated/SchemaCreated
**And** LSP: Publishes none, Subscribes to SchemaCreated/TemplateCreated/NoteCreated

**Given** publisher contract pattern is needed
**When** I define publishing contract
**Then** after domain aggregate state change, call take_events()
**And** publish each pending event via event_bus.publish()
**And** handle errors: ChannelClosed (log, continue), other errors (propagate)

**Given** subscriber contract pattern is needed
**When** I define subscription contract
**Then** subscribe during service initialization
**And** spawn background task to handle events
**And** log errors but continue processing (don't crash on single event failure)
**And** drop receiver on shutdown

**Given** integration examples demonstrate patterns
**When** I document cross-epic integration
**Then** Epic 6 publisher example: reload_config() publishes ConfigUpdated
**And** Epic 7 subscriber example: PropertyBank reloads on ConfigUpdated
**And** Epic 10 publisher + subscriber example: publishes NoteCreated, subscribes to SchemaCreated

**Given** event bus lifecycle must be managed
**When** I define initialization contract
**Then** initialize event_bus before services
**And** pass Arc<event_bus> to all services
**And** start all service listeners after initialization
**And** call event_bus.shutdown() for graceful termination

**Given** testing is needed
**When** I write integration tests
**Then** test Epic 6 → Epic 7 contract (ConfigUpdated triggers PropertyBank reload)
**And** test Epic 7 → Epic 10 contract (SchemaCreated triggers note validation)
**And** test Epic 10 → LSP contract (NoteCreated updates IDE)

---

## Story 8.10: Create Event Bus Mocks for Testing

As a developer testing event-driven code,
I want comprehensive event bus mocks,
So that event interactions can be tested in isolation.

**Acceptance Criteria:**

**Given** I need to mock event bus for testing
**When** I use mockall crate
**Then** EventBusPort trait auto-generates MockEventBusPort
**And** mock is available via use lithos_domain::MockEventBusPort

**Given** expectation API enables verification
**When** I set expectations
**Then** expect_publish() with predicate matching event type
**And** times(n) verifies exact publish count
**And** returning() configures return value or error

**Given** subscription mocking is needed
**When** I mock subscribe()
**Then** return controlled receiver (tokio::sync::mpsc channel)
**And** test code sends events via sender (tx.send())
**And** subscriber logic receives events from returned receiver

**Given** test fixtures simplify testing
**When** I create fixtures in `crates/domain/src/events/fixtures.rs`
**Then** provide factory functions: sample_config_updated(), sample_schema_created(), etc.
**And** fixtures return DomainEvent with realistic data

**Given** example test cases demonstrate usage
**When** I document mock usage
**Then** example: Config service publishes on reload (verify via expect_publish())
**And** example: Schema service subscribes to config events (send event, verify handler called)

**Given** mock limitations must be documented
**When** I explain mock behavior
**Then** mocks verify interactions, not actual event delivery
**And** mocks don't test concurrency (use real HybridEventBus for that)
**And** integration tests use real event bus for end-to-end validation

**Given** testing is needed
**When** I write tests demonstrating mock usage
**Then** test publish expectation (event published exactly once)
**And** test subscribe expectation (subscription created)
**And** test error simulation (ChannelClosed error returned)
**And** test fixture usage (sample events match expected structure)

---

## Story 8.11: Review Epic 8 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 8 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** tests are written
**When** I review test documentation
**Then** all tests include BDD-style comments (GIVEN-WHEN-THEN)
**And** test names clearly describe behavior being tested
**And** any developer can understand test purpose without reading implementation
**And** BDD comments explain business context, not just technical steps

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 8 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests
**And** all public APIs have runnable doc tests demonstrating usage

**Given** all Epic 8 components are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate false positives, redundant tests, and inadequate edge case coverage
**And** I assess if tests validate business requirements vs implementation details
**And** I eliminate duplicate test cases and consolidate overlapping coverage

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 8 suite

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior (no sleeps for sync), and maintain clear intent
**And** test code follows same quality standards as production code with proper documentation

**Given** adversarial scenarios must be tested
**When** I identify edge cases
**Then** test concurrent publishers (4+ threads, no lost events)
**And** test channel closed mid-publish (receiver dropped)
**And** test backpressure under load (MPSC channel fills completely)
**And** test broadcast lag (slow subscriber, verify RecvError::Lagged)
**And** test watch deduplication (publish same event twice)
**And** test event persistence failure (Redb unavailable, publish succeeds)

**Given** cross-epic integration must be validated
**When** I review integration tests
**Then** verify Epic 6 → Epic 7 integration (ConfigUpdated → PropertyBank reload)
**And** verify Epic 7 → Epic 10 integration (SchemaCreated → note validation)
**And** integration tests use real HybridEventBus (not mocks)

---

## Story 8.12: Document Event Bus Integration for Developers

As a developer integrating with the event system,
I want comprehensive developer documentation for event bus usage,
So that other epics can properly publish and subscribe to events.

**Acceptance Criteria:**

**Given** event system is implemented
**When** I create developer documentation
**Then** create `docs/architecture/event-bus.md` with architecture overview
**And** document three planes: MPSC (data), Broadcast (control), Watch (state)
**And** explain routing: which events go to which planes and why
**And** include performance characteristics table (latency, throughput per plane)

**Given** developer guide is needed
**When** I create `docs/guides/event-bus-integration.md`
**Then** provide step-by-step: publishing events (after aggregate state change)
**And** provide step-by-step: subscribing to events (during service init, spawn background task)
**And** explain error handling patterns (ChannelClosed, lag, degraded mode)

**Given** API reference must be complete
**When** I document EventBusPort trait
**Then** rustdoc includes architecture overview, plane descriptions, usage examples
**And** all public methods have doc tests demonstrating usage

**Given** integration patterns must be documented
**When** I document cross-epic patterns
**Then** include examples from Story 8.9 (Epic 6→7, 7→10, 10→LSP)
**And** show publisher pattern (take_events, publish, error handling)
**And** show subscriber pattern (subscribe, spawn task, match events, log errors)

**Given** troubleshooting guide helps developers
**When** I create troubleshooting section
**Then** document common problems: events not received (check routing), slow subscriber lag (increase buffer), event bus unavailable (check init order), memory leak (drop receivers)
**And** provide solutions for each problem

**Given** performance tuning guidance is needed
**When** I document tuning options
**Then** MPSC channel size (default 1000, increase for burst indexing)
**And** Broadcast capacity (default 16, rarely needs tuning)
**And** Event payload size (keep <1KB for optimal performance)

**Given** testing guidance helps developers
**When** I document testing patterns
**Then** use MockEventBusPort for unit tests (Story 8.10)
**And** use real HybridEventBus for integration tests
**And** use sample_*() fixtures for test events

---
