# Epic 8: Event Bus & Orchestration Infrastructure **[PHASE 1.5]**

System has a robust event-driven architecture enabling loose coupling between services and supporting concurrent operations without god-objects.

**FRs covered:** Architecture requirements (event-driven, CQRS foundation)

**Implementation Notes:**

- **Hybrid Event Bus**: MPSC/Broadcast/Watch per ADR 0007 (event-orchestration.md)
- **Architecture**: Hexagonal (Ports & Adapters) - EventBusPort in domain, implementations in adapters
- **Location**: `crates/domain/src/ports/event_bus.rs` for port, `crates/adapters/src/spi/event/` for implementations
- **Libraries**: `tokio` (mpsc/broadcast/watch), `async-trait`, `mockall`, `tracing`, `thiserror`, `serde`, `rkyv`, `arc-swap`
- **Event Types**: Consolidated from Epics 3-7 domain events (Config, Schema, Template, Note)
- **Three Planes**:
  - **Data Plane (MPSC)**: Reliable indexing pipeline (10,000+ events/sec, bounded channels)
  - **Control Plane (Broadcast)**: Global signals (shutdown, error alerts, system-wide notifications)
  - **State Plane (Watch)**: LSP state sync (sub-50ms IDE responsiveness, latest-only semantics)
- **Caching Strategy**: Epic 5 `CacheCommand` trait for event persistence (last 1000 events OR 24 hours)
- **Adapter Structure**: `crates/adapters/src/spi/event/` contains bus.rs, mpsc_plane.rs, broadcast_plane.rs, watch_plane.rs, persistence.rs, errors.rs
- **Zero-Copy Payloads**: `Arc<T>` wrapping for broadcast/watch to prevent memory duplication
- **Prevents god-objects**: No central orchestrator (Go lesson learned from ADR 0007)
- **Testing**: `MockEventBus` with expectation API, concurrent subscription tests, backpressure scenarios
- **Performance Targets**: <1ms MPSC latency, <50ms LSP updates, <100μs persistence overhead

---

## Story 8.1: Create Event Bus Domain Interface and Port

As a developer implementing event-driven architecture,
I want a clean domain interface for event operations,
So that events can be published and subscribed to through well-defined contracts.

**Acceptance Criteria:**

### **EventBusPort Trait Definition:**

**Given** I need event bus contracts in the domain
**When** I create `EventBusPort` trait in `crates/domain/src/ports/event_bus.rs`
**Then** it defines async methods for event operations:
```rust
#[async_trait]
pub trait EventBusPort: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<(), EventError>;
    async fn subscribe(&self, event_type: EventType) -> Result<Receiver, EventError>;
    async fn shutdown(&self) -> Result<(), EventError>;
}
```
**And** the trait is annotated with `#[async_trait]` for async support
**And** trait is object-safe for dynamic dispatch if needed

**Given** event bus needs event type enumeration
**When** I define `DomainEvent` enum in `crates/domain/src/events/mod.rs`
**Then** it consolidates all domain events:
```rust
#[derive(Debug, Clone)]
pub enum DomainEvent {
    Config(ConfigEvents),
    Schema(SchemaEvents),
    Template(TemplateEvents),
    Note(NoteEvents),
}
```
**And** it derives `Clone` for Arc-wrapping
**And** it includes `Send + Sync` for cross-thread event passing

**Given** event type filtering is needed
**When** I define `EventType` enum
**Then** it provides subscription filters:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    ConfigUpdated,
    SchemaCreated,
    PropertyBankUpdated,
    TemplateCreated,
    NoteCreated,
    FrontmatterValidated,
    All,  // wildcard subscription
}
```
**And** it derives `Hash` for subscription map keys
**And** it includes `All` variant for debugging/logging subscribers

**Given** async receivers are needed
**When** I define `Receiver` type
**Then** it wraps tokio receivers:
```rust
pub enum Receiver {
    Mpsc(tokio::sync::mpsc::UnboundedReceiver<Arc<DomainEvent>>),
    Broadcast(tokio::sync::broadcast::Receiver<Arc<DomainEvent>>),
    Watch(tokio::sync::watch::Receiver<Arc<DomainEvent>>),
}
```
**And** all variants use `Arc<DomainEvent>` for zero-copy sharing
**And** receiver provides `async fn recv() -> Option<Arc<DomainEvent>>`

**Given** error handling is required
**When** I define `EventError` enum in `crates/adapters/src/spi/event/errors.rs`
**Then** it includes error variants:
```rust
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event channel closed for {event_type:?}")]
    ChannelClosed { event_type: EventType },

    #[error("Publish timeout after {timeout_ms}ms for {event_type:?}")]
    PublishTimeout { event_type: EventType, timeout_ms: u64 },

    #[error("Subscription failed: {message}")]
    SubscriptionFailed { message: String },

    #[error("Event bus already shutdown")]
    AlreadyShutdown,

    #[error("Invalid event payload: {message}")]
    InvalidPayload { message: String },
}
```
**And** all variants are `Send + Sync` for async contexts
**And** error messages follow ADR 0006 (actionable diagnostics)

### **Port Documentation:**

**Given** the EventBusPort must be documented
**When** I write module-level docs
**Then** it explains:
- **Purpose**: Async event bus for decoupled bounded context communication
- **Implementations**: HybridEventBus (MPSC+Broadcast+Watch per ADR 0007)
- **Consumers**: All application services (Config, Schema, Template, Note, Indexer, LSP)
- **Pattern**: Publish events after aggregate state changes, subscribe for cross-context reactions
- **Planes**: Data (MPSC), Control (Broadcast), State (Watch) - automatic routing by event type

**Given** port usage patterns must be clear
**When** I document publish pattern
**Then** example shows:
```rust
// After Config aggregate updates:
let event = DomainEvent::Config(ConfigEvents::ConfigUpdated(updated));
event_bus.publish(event).await?;
```

**Given** subscription pattern must be demonstrated
**When** I document subscribe pattern
**Then** example shows:
```rust
let mut receiver = event_bus.subscribe(EventType::SchemaCreated).await?;
while let Some(event) = receiver.recv().await {
    // React to schema creation (e.g., reload property bank)
}
```

### **Testing Support:**

**Given** testing requires mocks
**When** I annotate `EventBusPort` with `#[mockall::automock]`
**Then** `MockEventBusPort` is auto-generated
**And** mocks support expectation setting:
```rust
let mut mock = MockEventBusPort::new();
mock.expect_publish()
    .with(predicate::eq(EventType::ConfigUpdated))
    .times(1)
    .returning(|_| Ok(()));
```

**Given** port contract tests are needed
**When** I write unit tests in `crates/domain/src/ports/event_bus.rs`
**Then** test `DomainEvent` is `Send + Sync`
**And** test `EventType` derives `Hash + Eq`
**And** test `EventError` includes context fields

---

## Story 8.2: Define Complete Domain Event Types

As a developer coordinating events across the system,
I want complete domain event definitions,
So that all events from Epics 3-7 are properly defined and coordinated.

**Acceptance Criteria:**

### **Event Consolidation:**

**Given** events are defined in multiple domain modules
**When** I consolidate in `crates/domain/src/events/mod.rs`
**Then** re-export all event types:
```rust
pub use crate::config::events::{ConfigEvents, ConfigUpdated};
pub use crate::schema::events::{SchemaEvents, SchemaCreated, PropertyBankUpdated};
pub use crate::template::events::{TemplateEvents, TemplateCreated};
pub use crate::note::events::{NoteEvents, NoteCreated, FrontmatterValidated};
```
**And** define `DomainEvent` enum consolidating all:
```rust
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DomainEvent {
    Config(ConfigEvents),
    Schema(SchemaEvents),
    Template(TemplateEvents),
    Note(NoteEvents),
}
```

**Given** event payloads need serialization
**When** I verify existing event structs
**Then** all events derive `Serialize + Deserialize` (already implemented)
**And** all events derive `Clone` for Arc-wrapping
**And** all events include `timestamp: i64` (Unix seconds) for ordering

### **Event Inventory by Epic:**

**Given** I need complete event listing
**When** I document events by source epic
**Then** inventory shows:

| Epic    | Event Type              | Payload Fields                          | Purpose                          |
|---------|-------------------------|-----------------------------------------|----------------------------------|
| Epic 6  | ConfigUpdated           | source: String, timestamp: i64          | Config file reloaded             |
| Epic 7  | SchemaCreated           | id: Uuid, name: String, timestamp: i64  | New schema defined               |
| Epic 7  | PropertyBankUpdated     | property_count: usize, timestamp: i64   | PropertyBank reloaded            |
| Epic 12 | TemplateCreated         | id: Uuid, name: String, timestamp: i64  | Template registered              |
| Epic 10 | NoteCreated             | id: Uuid, path: String, timestamp: i64  | Note parsed and indexed          |
| Epic 10 | FrontmatterValidated    | note_id: Uuid, field_count: usize, ts   | Frontmatter schema validation OK |

**Given** event naming conventions
**When** I validate event names
**Then** all events use past tense (Created, Updated, Validated)
**And** all events are suffixed with context (ConfigUpdated, not just Updated)

### **Event Routing Specification:**

**Given** events route to different planes
**When** I define routing logic (implementation detail for Story 8.6)
**Then** routing rules are:
- **MPSC Data Plane**: `NoteCreated`, `FrontmatterValidated` (indexing pipeline, high volume)
- **Broadcast Control Plane**: `ConfigUpdated`, `PropertyBankUpdated` (system-wide signals, all services react)
- **Watch State Plane**: `SchemaCreated`, `TemplateCreated` (LSP state sync, latest-only)

**Given** routing must be deterministic
**When** I document routing rationale
**Then** explanation includes:
- **NoteCreated** → MPSC: 10,000+ events during vault scan, must not block
- **ConfigUpdated** → Broadcast: All services must reload config atomically
- **SchemaCreated** → Watch: LSP only needs latest schema version, not history

### **Event Payload Validation:**

**Given** event payloads must be valid
**When** I define validation rules
**Then** all events require:
- `timestamp` must be positive i64 (Unix seconds since epoch)
- UUIDs must be v7 (time-ordered for indexing)
- String fields must not be empty (enforced by domain aggregates)

**Given** validation errors occur
**When** I handle invalid payloads
**Then** return `EventError::InvalidPayload { message }` with specific failure reason

### **Serialization Strategy:**

**Given** events persist for debugging (Story 8.8)
**When** I choose serialization format
**Then** use `rkyv` for zero-copy deserialization (per ADR 0002)
**And** add `rkyv` derive to `DomainEvent`:
```rust
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum DomainEvent { ... }
```

**Given** event size matters
**When** I measure event payloads
**Then** typical sizes:
- `ConfigUpdated`: ~64 bytes
- `SchemaCreated`: ~128 bytes (includes UUID + name)
- `NoteCreated`: ~256 bytes (includes path string)

### **Testing:**

**Given** event types are implemented
**When** I write unit tests in `crates/domain/src/events/mod.rs`
**Then** test `DomainEvent` is `Send + Sync + Clone`
**And** test serialization round-trip for all event variants
**And** test timestamp ordering (v7 UUIDs correlate with timestamps)
**And** test payload size measurements (ensure <512 bytes per event)

---

## Story 8.3: Implement MPSC Data Plane

As a developer needing reliable event delivery,
I want MPSC data plane for indexing operations,
So that events are delivered reliably without loss in the indexing pipeline.

**Acceptance Criteria:**

### **MPSC Channel Configuration:**

**Given** I need reliable indexing event delivery
**When** I implement MPSC plane in `crates/adapters/src/spi/event/mpsc_plane.rs`
**Then** create bounded channel with capacity:
```rust
const MPSC_CHANNEL_SIZE: usize = 1000;
let (tx, rx) = tokio::sync::mpsc::channel::<Arc<DomainEvent>>(MPSC_CHANNEL_SIZE);
```
**And** channel size 1000 handles burst indexing (10,000 events in 10 seconds = 1000/sec avg)
**And** bounded channel provides backpressure (slows publisher when full)

**Given** channel must be persistent
**When** I store channel endpoints
**Then** structure is:
```rust
pub struct MpscPlane {
    sender: tokio::sync::mpsc::Sender<Arc<DomainEvent>>,
    // Note: receivers created per subscriber via subscribe()
}
```
**And** sender is `Clone` for multiple publishers
**And** receivers created on-demand per subscription

### **Publish Implementation:**

**Given** I need to publish events
**When** I implement `publish()` method
**Then** logic is:
```rust
pub async fn publish(&self, event: Arc<DomainEvent>) -> Result<(), EventError> {
    self.sender.send(event).await
        .map_err(|_| EventError::ChannelClosed {
            event_type: EventType::from(&event)
        })
}
```
**And** `send().await` applies backpressure (blocks if channel full)
**And** closed channel returns `ChannelClosed` error

**Given** backpressure must be handled
**When** publisher sends faster than indexer consumes
**Then** `send().await` blocks until capacity available
**And** blocking prevents unbounded memory growth
**And** tracing event emits `backpressure_applied` with wait duration

### **Subscribe Implementation:**

**Given** I need to subscribe to MPSC events
**When** I implement `subscribe()` method
**Then** create new receiver:
```rust
pub fn subscribe(&self) -> tokio::sync::mpsc::Receiver<Arc<DomainEvent>> {
    // MPSC supports single consumer - clone sender, create new receiver
    let (tx, rx) = tokio::sync::mpsc::channel(MPSC_CHANNEL_SIZE);
    // Store tx in subscriber registry for forwarding
    rx
}
```
**And** MPSC pattern: single producer → multiple consumers via fan-out
**And** each subscriber gets dedicated receiver (no shared state)

### **Event Ordering Guarantee:**

**Given** MPSC provides ordering guarantees
**When** events are published
**Then** receiver sees events in exact publish order (FIFO)
**And** ordering is critical for indexing (e.g., NoteCreated before FrontmatterValidated)
**And** ordering test: publish 1000 events → verify recv() preserves order

### **Performance Targets:**

**Given** indexing requires low latency
**When** I measure MPSC performance
**Then** publish latency <1ms (99th percentile)
**And** throughput >10,000 events/sec on single core
**And** memory overhead <1MB for 1000-event buffer

**Given** performance must be validated
**When** I write benchmarks in `crates/adapters/benches/event_mpsc.rs`
**Then** benchmark:
- Publish 10,000 events → measure total time
- Single publisher, single consumer → verify <1s total
- Concurrent publishers (4 threads) → verify no contention

### **Error Handling:**

**Given** MPSC operations can fail
**When** I handle errors
**Then** scenarios:
- **Sender dropped**: `send()` returns `Err` → `EventError::ChannelClosed`
- **Receiver dropped**: sender continues (buffered until capacity)
- **Channel full**: `send().await` blocks (backpressure, not error)

**Given** error recovery is needed
**When** channel closes
**Then** log error with `tracing::error!` + event_type
**And** do NOT retry (closed channel is unrecoverable)
**And** emit metric `event_bus.channel_closed{plane="mpsc"}`

### **Tracing & Observability:**

**Given** MPSC operations need observability
**When** I instrument methods
**Then** all methods use `#[tracing::instrument(skip(self, event), level = "debug")]`
**And** publish emits event with attributes:
```rust
tracing::event!(
    Level::DEBUG,
    plane = "mpsc",
    operation = "publish",
    event_type = ?event_type,
    buffer_size = self.sender.capacity(),
);
```

**Given** backpressure is critical to observe
**When** backpressure occurs
**Then** emit warning-level event:
```rust
tracing::warn!(
    plane = "mpsc",
    event_type = ?event_type,
    wait_duration_ms = wait_ms,
    "Backpressure applied: channel full",
);
```

### **Testing:**

**Given** MPSC plane is implemented
**When** I write unit tests
**Then** test successful publish + receive (happy path)
**And** test channel closed error (drop sender → publish fails)
**And** test backpressure (fill channel → publish blocks)
**And** test event ordering (1000 events → verify FIFO)
**And** test concurrent publishers (4 threads → no lost events)
**And** test single consumer semantics (multiple receivers each get full stream)

---

## Story 8.4: Implement Broadcast Control Plane

As a developer needing global signaling,
I want broadcast control plane for system signals,
So that shutdown and global notifications work across all components.

**Acceptance Criteria:**

### **Broadcast Channel Configuration:**

**Given** I need global signal broadcasting
**When** I implement Broadcast plane in `crates/adapters/src/spi/event/broadcast_plane.rs`
**Then** create broadcast channel:
```rust
const BROADCAST_CAPACITY: usize = 16;
let (tx, _rx) = tokio::sync::broadcast::channel::<Arc<DomainEvent>>(BROADCAST_CAPACITY);
```
**And** capacity 16 is sufficient (control events are low-frequency: <10/sec)
**And** broadcast delivers to ALL active subscribers simultaneously

**Given** broadcast channel semantics
**When** I understand broadcast behavior
**Then** late subscribers receive events published AFTER subscription (no history)
**And** slow subscribers cause sender lag (if buffer fills, oldest message dropped)
**And** dropped messages return `Err(RecvError::Lagged(n))` to receiver

### **Publish Implementation:**

**Given** I need to broadcast control events
**When** I implement `publish()` method
**Then** logic is:
```rust
pub fn publish(&self, event: Arc<DomainEvent>) -> Result<usize, EventError> {
    let subscriber_count = self.sender.send(event)
        .map_err(|_| EventError::ChannelClosed {
            event_type: EventType::from(&event)
        })?;
    Ok(subscriber_count)  // returns # of active subscribers
}
```
**And** `send()` is non-blocking (returns immediately)
**And** return value indicates how many subscribers received event

**Given** no subscribers exist
**When** I publish to empty broadcast
**Then** `send()` succeeds but returns `0` (no subscribers)
**And** event is lost (broadcast requires active listeners)
**And** log warning: `"Broadcast event published with 0 subscribers"`

### **Subscribe Implementation:**

**Given** I need to subscribe to control events
**When** I implement `subscribe()` method
**Then** clone receiver from sender:
```rust
pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Arc<DomainEvent>> {
    self.sender.subscribe()
}
```
**And** `subscribe()` creates new receiver sharing same channel
**And** all receivers get copies of every event (fan-out pattern)

### **Subscriber Lifecycle:**

**Given** subscribers can join/leave dynamically
**When** I manage subscriber lifecycle
**Then** subscription count tracked via `self.sender.receiver_count()`
**And** dropping receiver automatically unsubscribes
**And** no explicit unsubscribe needed (RAII cleanup)

**Given** subscriber count affects system behavior
**When** I emit tracing events
**Then** log subscriber changes:
```rust
tracing::info!(
    plane = "broadcast",
    subscriber_count = count,
    "New broadcast subscriber",
);
```

### **Graceful Shutdown Pattern:**

**Given** system shutdown requires coordination
**When** I implement shutdown signal
**Then** pattern is:
```rust
// Publisher side (shutdown coordinator):
event_bus.publish(DomainEvent::SystemShutdown).await?;

// Subscriber side (all services):
while let Ok(event) = receiver.recv().await {
    match event.as_ref() {
        DomainEvent::SystemShutdown => {
            // Cleanup and exit
            break;
        }
        _ => { /* process event */ }
    }
}
```
**And** all services listen for `SystemShutdown` event
**And** shutdown completes when all subscribers acknowledge

**Given** graceful shutdown must complete
**When** I add shutdown timeout
**Then** coordinator waits max 5 seconds for all subscribers
**And** after timeout, force shutdown remaining services

### **Lag Handling (Slow Subscribers):**

**Given** slow subscribers can lag
**When** receiver buffer fills
**Then** oldest messages dropped (lag occurs)
**And** receiver gets `Err(RecvError::Lagged(n))` indicating `n` messages lost
**And** subscriber logs warning:
```rust
tracing::warn!(
    plane = "broadcast",
    lagged_count = n,
    "Subscriber lagged: {n} messages dropped",
);
```

**Given** critical events must not be lost
**When** I detect lag
**Then** subscriber can choose to:
- **Ignore lag**: continue with next message (acceptable for non-critical events)
- **Panic on lag**: unacceptable for critical control events (fail fast)
- **Re-sync state**: query current state from source (e.g., reload config)

### **Performance Targets:**

**Given** broadcast is for control events
**When** I measure performance
**Then** publish latency <10μs (non-blocking)
**And** delivery to 16 subscribers <100μs (all receive simultaneously)
**And** memory overhead <64KB (16 * 4KB per subscriber buffer)

### **Error Handling:**

**Given** broadcast operations can fail
**When** I handle errors
**Then** scenarios:
- **All receivers dropped**: `send()` returns `Err` → `EventError::ChannelClosed`
- **Receiver lags**: `recv()` returns `Err(Lagged(n))` → log warning, continue
- **Sender dropped**: all receivers return `None` → shutdown signal

### **Testing:**

**Given** Broadcast plane is implemented
**When** I write unit tests
**Then** test successful broadcast to multiple subscribers (3 subscribers receive same event)
**And** test late subscriber (subscribe after publish → no historical events)
**And** test no subscribers (publish returns 0, event lost)
**And** test lag scenario (slow subscriber → verify `RecvError::Lagged`)
**And** test graceful shutdown (publish shutdown → all subscribers receive)
**And** test concurrent subscribers (spawn 16 subscribers → all receive all events)

---

## Story 8.5: Implement Watch State Plane

As a developer needing state synchronization,
I want watch state plane for LSP integration,
So that real-time state changes are communicated to IDE integrations.

**Acceptance Criteria:**

### **Watch Channel Configuration:**

**Given** I need latest-state synchronization
**When** I implement Watch plane in `crates/adapters/src/spi/event/watch_plane.rs`
**Then** create watch channel:
```rust
let initial_state = Arc::new(DomainEvent::SystemStarted);
let (tx, rx) = tokio::sync::watch::channel::<Arc<DomainEvent>>(initial_state);
```
**And** watch holds ONLY latest value (no history)
**And** subscribers always see most recent state

**Given** watch semantics
**When** I understand watch behavior
**Then** late subscribers immediately receive current state (not `None`)
**And** state updates replace previous value (no queue)
**And** subscribers notified only when state changes (deduplication)

### **Publish Implementation:**

**Given** I need to update state
**When** I implement `publish()` method
**Then** logic is:
```rust
pub fn publish(&self, event: Arc<DomainEvent>) -> Result<(), EventError> {
    self.sender.send(event)
        .map_err(|_| EventError::ChannelClosed {
            event_type: EventType::from(&event)
        })
}
```
**And** `send()` is non-blocking (returns immediately)
**And** `send()` replaces previous state (not appending)

**Given** rapid state updates occur
**When** publisher sends faster than subscriber consumes
**Then** intermediate states are skipped (subscriber only sees latest)
**And** skipping is intentional (LSP doesn't need every intermediate schema version)

### **Subscribe Implementation:**

**Given** I need to watch state changes
**When** I implement `subscribe()` method
**Then** clone receiver from sender:
```rust
pub fn subscribe(&self) -> tokio::sync::watch::Receiver<Arc<DomainEvent>> {
    self.sender.subscribe()
}
```
**And** `subscribe()` returns receiver with current state immediately available

**Given** subscriber reads state
**When** I implement receiver usage
**Then** pattern is:
```rust
let mut receiver = watch_plane.subscribe();
loop {
    receiver.changed().await?;  // waits for state change
    let state = receiver.borrow().clone();  // reads latest state
    // React to state update
}
```
**And** `changed().await` blocks until state changes
**And** `borrow()` reads current state without waiting

### **LSP State Synchronization:**

**Given** LSP needs vault state updates
**When** I use watch for LSP integration
**Then** state updates include:
- **SchemaCreated**: LSP reloads schema definitions → updates autocomplete
- **TemplateCreated**: LSP indexes available templates → suggests in commands
- **Current vault version**: Increments on every change → LSP detects staleness

**Given** LSP responsiveness is critical
**When** I measure state sync latency
**Then** schema update → LSP notification <50ms (99th percentile)
**And** LSP client polls `borrow()` during idle (no blocking during typing)

### **State Deduplication:**

**Given** duplicate state updates waste resources
**When** I prevent unnecessary notifications
**Then** watch channel deduplicates: if `new_state == current_state`, no notification sent
**And** `changed().await` only returns when state actually changes
**And** deduplication relies on `PartialEq` implementation for `DomainEvent`

**Given** deduplication must work correctly
**When** I test state changes
**Then** publish same event twice → `changed()` returns once
**And** publish different events → `changed()` returns each time

### **Late Subscriber Behavior:**

**Given** LSP connects after system startup
**When** subscriber joins late
**Then** `borrow()` immediately returns current state (not `None`)
**And** subscriber syncs to latest state without waiting
**And** example: LSP starts 10 seconds after system → immediately gets current schema version

**Given** late subscriber pattern
**When** I implement LSP connection
**Then** pattern is:
```rust
let receiver = event_bus.subscribe(EventType::SchemaCreated).await?;
let current_schema = receiver.borrow().clone();  // immediate state
// Now wait for future updates
while receiver.changed().await.is_ok() {
    let updated_schema = receiver.borrow().clone();
    // React to change
}
```

### **Performance Targets:**

**Given** watch is for LSP state sync
**When** I measure performance
**Then** publish latency <1μs (atomic pointer swap via `ArcSwap`)
**And** notification delivery <10μs (wakes all subscribers)
**And** memory overhead: single `Arc<DomainEvent>` (shared across all subscribers)

**Given** LSP responsiveness requirement
**When** I validate end-to-end latency
**Then** schema file change → LSP autocomplete update <50ms
**And** breakdown: file watch (10ms) + event publish (1μs) + LSP processing (40ms)

### **Error Handling:**

**Given** watch operations can fail
**When** I handle errors
**Then** scenarios:
- **Sender dropped**: `changed()` returns `Err` → all subscribers exit
- **No state updates**: `changed()` blocks indefinitely (timeout on consumer side)
- **Initial state missing**: not possible (watch requires initial value at creation)

### **Testing:**

**Given** Watch plane is implemented
**When** I write unit tests
**Then** test initial state (new subscriber sees current state immediately)
**And** test state update (publish → `changed()` returns)
**And** test deduplication (publish same event twice → changed() returns once)
**And** test late subscriber (subscribe after 3 updates → sees latest state)
**And** test multiple subscribers (all see same state changes)
**And** test performance (10,000 updates <10ms total)

---

## Story 8.6: Implement Event Publishing and Subscription

As a developer using the event system,
I want complete publish/subscribe functionality,
So that components can publish events and subscribe to relevant notifications.

**Acceptance Criteria:**

### **HybridEventBus Implementation:**

**Given** I need unified event bus
**When** I implement `HybridEventBus` in `crates/adapters/src/spi/event/bus.rs`
**Then** structure combines all three planes:
```rust
pub struct HybridEventBus {
    mpsc_plane: Arc<MpscPlane>,
    broadcast_plane: Arc<BroadcastPlane>,
    watch_plane: Arc<WatchPlane>,
    routing_table: HashMap<EventType, PlaneType>,
    shutdown_signal: Arc<tokio::sync::Notify>,
}
```
**And** routing table determines which plane handles each event type
**And** shutdown signal coordinates graceful termination

**Given** event routing is deterministic
**When** I define routing rules
**Then** routing table maps:
```rust
let mut routing = HashMap::new();
// Data Plane (MPSC): High-volume indexing events
routing.insert(EventType::NoteCreated, PlaneType::Mpsc);
routing.insert(EventType::FrontmatterValidated, PlaneType::Mpsc);

// Control Plane (Broadcast): System-wide signals
routing.insert(EventType::ConfigUpdated, PlaneType::Broadcast);
routing.insert(EventType::PropertyBankUpdated, PlaneType::Broadcast);

// State Plane (Watch): LSP state sync (latest-only)
routing.insert(EventType::SchemaCreated, PlaneType::Watch);
routing.insert(EventType::TemplateCreated, PlaneType::Watch);
```

### **Publish Method Implementation:**

**Given** I need to publish events
**When** I implement `EventBusPort::publish()`
**Then** logic is:
```rust
async fn publish(&self, event: DomainEvent) -> Result<(), EventError> {
    let event_type = EventType::from(&event);
    let plane = self.routing_table.get(&event_type)
        .ok_or_else(|| EventError::InvalidPayload {
            message: format!("No route for {:?}", event_type)
        })?;

    let arc_event = Arc::new(event);
    match plane {
        PlaneType::Mpsc => self.mpsc_plane.publish(arc_event).await,
        PlaneType::Broadcast => self.broadcast_plane.publish(arc_event),
        PlaneType::Watch => self.watch_plane.publish(arc_event),
    }
}
```
**And** routing is transparent to publisher (automatic plane selection)
**And** Arc-wrapping happens once (zero-copy across all subscribers)

### **Subscribe Method Implementation:**

**Given** I need to subscribe to events
**When** I implement `EventBusPort::subscribe()`
**Then** logic is:
```rust
async fn subscribe(&self, event_type: EventType) -> Result<Receiver, EventError> {
    let plane = self.routing_table.get(&event_type)
        .ok_or_else(|| EventError::SubscriptionFailed {
            message: format!("No route for {:?}", event_type)
        })?;

    let receiver = match plane {
        PlaneType::Mpsc => Receiver::Mpsc(self.mpsc_plane.subscribe()),
        PlaneType::Broadcast => Receiver::Broadcast(self.broadcast_plane.subscribe()),
        PlaneType::Watch => Receiver::Watch(self.watch_plane.subscribe()),
    };
    Ok(receiver)
}
```
**And** subscriber receives typed receiver matching plane semantics
**And** subscriber doesn't need to know which plane is used

### **Integration Examples by Epic:**

**Given** I need concrete integration patterns
**When** I document integration for each epic
**Then** examples show:

#### **Epic 6 → Epic 7 Integration** (Config reload triggers schema reload):
```rust
// Epic 6: Config adapter publishes after reload
let event = DomainEvent::Config(ConfigEvents::ConfigUpdated(updated));
event_bus.publish(event).await?;

// Epic 7: Schema adapter subscribes and reacts
let mut receiver = event_bus.subscribe(EventType::ConfigUpdated).await?;
while let Some(event) = receiver.recv().await {
    match event.as_ref() {
        DomainEvent::Config(ConfigEvents::ConfigUpdated(_)) => {
            // Reload property bank from updated config path
            property_bank.reload()?;
        }
        _ => {}
    }
}
```

#### **Epic 7 → Epic 10 Integration** (Schema loaded triggers note validation):
```rust
// Epic 7: Schema adapter publishes after loading
let event = DomainEvent::Schema(SchemaEvents::SchemaCreated(created));
event_bus.publish(event).await?;

// Epic 10: Indexer subscribes and validates notes
let mut receiver = event_bus.subscribe(EventType::SchemaCreated).await?;
while let Some(event) = receiver.recv().await {
    match event.as_ref() {
        DomainEvent::Schema(SchemaEvents::SchemaCreated(created)) => {
            // Trigger re-validation of all notes against new schema
            indexer.revalidate_notes(created.name).await?;
        }
        _ => {}
    }
}
```

#### **Epic 10 → LSP Integration** (Note indexed triggers IDE update):
```rust
// Epic 10: Indexer publishes after indexing
let event = DomainEvent::Note(NoteEvents::NoteCreated(created));
event_bus.publish(event).await?;

// LSP: Watches for note changes (Watch plane, latest-only)
let mut receiver = event_bus.subscribe(EventType::NoteCreated).await?;
while receiver.changed().await.is_ok() {
    let event = receiver.borrow().clone();
    // Update IDE autocomplete, diagnostics
    lsp_client.notify_workspace_change(event).await?;
}
```

### **Wildcard Subscription (Debugging):**

**Given** debugging requires seeing all events
**When** I implement wildcard subscription
**Then** `EventType::All` subscribes to all planes:
```rust
if event_type == EventType::All {
    // Fork all three planes into unified receiver
    let receivers = vec![
        self.mpsc_plane.subscribe(),
        self.broadcast_plane.subscribe(),
        self.watch_plane.subscribe(),
    ];
    return Ok(Receiver::Multi(receivers));
}
```
**And** `Receiver::Multi` merges streams using `tokio::select!`

### **Publisher-Subscriber Patterns:**

**Given** different patterns exist
**When** I document usage patterns
**Then** examples include:

- **Publish-Subscribe (1:N)**: Config update → all services reload
- **Point-to-Point (1:1)**: Note created → single indexer processes
- **Fan-Out (1:N, selective)**: Schema created → LSP + Indexer both react differently
- **State Sync (latest-only)**: Watch plane for LSP (skips intermediate updates)

### **Error Propagation:**

**Given** publish/subscribe can fail
**When** I handle errors
**Then** propagate errors from underlying planes:
- MPSC closed → `EventError::ChannelClosed`
- Broadcast lag → logged warning, subscription continues
- Watch sender dropped → subscription returns `None`

### **Testing:**

**Given** event bus is implemented
**When** I write integration tests
**Then** test publish → subscribe roundtrip for each event type
**And** test routing (ConfigUpdated → Broadcast, NoteCreated → MPSC)
**And** test Epic 6 → Epic 7 integration (config reload triggers schema reload)
**And** test Epic 7 → Epic 10 integration (schema change triggers note validation)
**And** test wildcard subscription (receives all events)
**And** test concurrent publishers (4 threads publishing simultaneously)
**And** test subscriber resilience (one subscriber crashes, others unaffected)

---

## Story 8.7: Add Event Payload Validation and Error Handling

As a developer ensuring event integrity,
I want event payload validation and error handling,
So that malformed events are caught and handled gracefully.

**Acceptance Criteria:**

### **Event Validation Rules:**

**Given** events must be valid before publishing
**When** I implement validation in `publish()` method
**Then** validate required fields:
```rust
fn validate_event(event: &DomainEvent) -> Result<(), EventError> {
    match event {
        DomainEvent::Config(ConfigEvents::ConfigUpdated(e)) => {
            if e.source.is_empty() {
                return Err(EventError::InvalidPayload {
                    message: "ConfigUpdated.source cannot be empty".into()
                });
            }
            if e.timestamp <= 0 {
                return Err(EventError::InvalidPayload {
                    message: "ConfigUpdated.timestamp must be positive".into()
                });
            }
        }
        DomainEvent::Schema(SchemaEvents::SchemaCreated(e)) => {
            if e.name.is_empty() {
                return Err(EventError::InvalidPayload {
                    message: "SchemaCreated.name cannot be empty".into()
                });
            }
            // UUID v7 validation: check timestamp bits are reasonable
            validate_uuid_v7(e.id)?;
        }
        // ... validate all event types
    }
    Ok(())
}
```
**And** validation runs before routing to planes
**And** invalid events return error immediately (not published)

**Given** UUID v7 must be time-ordered
**When** I validate UUIDs
**Then** extract timestamp from UUID v7 and verify:
- Timestamp is within reasonable range (not year 1970 or 3000)
- Timestamp <= current time (no future UUIDs)
- Timestamp is monotonically increasing per producer (optional)

### **Error Recovery Strategies:**

**Given** publish can fail
**When** I handle publish errors
**Then** strategies by error type:

| Error Type          | Recovery Strategy                              | Retry? |
|---------------------|------------------------------------------------|--------|
| `ChannelClosed`     | Log error, stop publishing, return error       | No     |
| `PublishTimeout`    | Log warning, retry with exponential backoff    | Yes (3 attempts) |
| `InvalidPayload`    | Log error, emit `EventValidationFailed` event  | No     |
| `AlreadyShutdown`   | Silently discard event (graceful shutdown)     | No     |

**Given** retry logic is needed for timeouts
**When** I implement exponential backoff
**Then** retry delays: 10ms, 100ms, 1000ms
**And** after 3 failures, return `PublishTimeout` error
**And** log each retry attempt with attempt number

### **Graceful Degradation:**

**Given** event bus failures should not crash system
**When** publish fails
**Then** fallback behavior:
```rust
match event_bus.publish(event).await {
    Ok(_) => { /* success */ }
    Err(EventError::ChannelClosed { event_type }) => {
        tracing::error!(
            ?event_type,
            "Event bus closed, entering degraded mode"
        );
        // Continue operation without events (e.g., skip cache invalidation)
    }
    Err(EventError::InvalidPayload { message }) => {
        tracing::error!(
            ?message,
            "Invalid event payload, discarding"
        );
        // Emit fallback event for monitoring
        metrics::counter!("event_bus.invalid_payload").increment(1);
    }
    Err(e) => {
        tracing::warn!(?e, "Event publish failed, retrying");
        // Retry logic or queue for later
    }
}
```

**Given** degraded mode must be detectable
**When** event bus is unavailable
**Then** set global flag `EVENT_BUS_DEGRADED` (atomic bool)
**And** subscribers check flag before waiting for events
**And** health check endpoint reports degraded status

### **EventValidationFailed Event:**

**Given** validation failures need observability
**When** I detect invalid payload
**Then** emit new event type:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventValidationFailed {
    pub original_event_type: String,
    pub validation_error: String,
    pub timestamp: i64,
}
```
**And** publish to Broadcast plane (system-wide notification)
**And** monitoring systems subscribe to detect payload issues

### **Payload Size Limits:**

**Given** large payloads impact performance
**When** I validate payload size
**Then** enforce limits:
- Maximum event size: 64 KB (prevents memory exhaustion)
- String fields: max 1024 characters (prevent unbounded growth)
- Array fields: max 100 elements (e.g., excludes list in SchemaCreated)

**Given** size limit exceeded
**When** I handle oversized events
**Then** return `EventError::InvalidPayload { message: "Event exceeds 64KB limit" }`
**And** log oversized event type + actual size for investigation

### **Serialization Error Handling:**

**Given** rkyv serialization can fail
**When** persisting events (Story 8.8)
**Then** catch serialization errors:
```rust
let archived = rkyv::to_bytes::<_, 256>(&event)
    .map_err(|e| EventError::InvalidPayload {
        message: format!("Serialization failed: {}", e)
    })?;
```
**And** serialization errors indicate programming bugs (should never happen in production)
**And** emit critical alert if serialization fails

### **Testing:**

**Given** validation is implemented
**When** I write unit tests
**Then** test validation rejects empty strings (`ConfigUpdated.source = ""`)
**And** test validation rejects negative timestamps
**And** test validation rejects oversized payloads (65KB event)
**And** test retry logic for `PublishTimeout` (3 attempts with backoff)
**And** test graceful degradation (ChannelClosed → system continues)
**And** test `EventValidationFailed` emission (invalid event → new event published)
**And** test UUID v7 validation (future timestamp rejected)

---

## Story 8.8: Implement Event Persistence for Debugging

As a developer debugging event flows,
I want event persistence capabilities,
So that event history can be inspected for troubleshooting and system analysis.

**Acceptance Criteria:**

### **Epic 5 Cache Integration:**

**Given** I need persistent event storage
**When** I integrate Epic 5 caching
**Then** use `CacheCommand` trait for event writes:
```rust
use crate::spi::cache::CacheCommand;

pub struct EventPersistence {
    cache: Arc<dyn CacheCommand<String, ArchivedEvent>>,
}
```
**And** `ArchivedEvent` is rkyv-serialized `DomainEvent`
**And** cache backend is `RedbCache` (persistent disk storage per ADR 0002)

**Given** cache table isolation is needed
**When** I create event cache
**Then** table name is `"event_history"` (separate from schema/config caches)
**And** Redb table structure:
```rust
TableDefinition<&str, &[u8]>  // key: timestamp+UUID, value: rkyv bytes
```

### **Event Storage Strategy:**

**Given** I need to persist events
**When** I implement storage logic
**Then** persist after successful publish:
```rust
async fn publish(&self, event: DomainEvent) -> Result<(), EventError> {
    // ... publish to appropriate plane ...

    // Persist event for debugging
    if let Err(e) = self.persistence.store(&event).await {
        tracing::warn!(?e, "Failed to persist event, continuing");
        // Non-fatal: event was published, persistence is best-effort
    }

    Ok(())
}
```
**And** persistence failure does NOT fail publish (decoupled)
**And** persistence is async (non-blocking)

**Given** event key must be unique and time-ordered
**When** I generate cache keys
**Then** key format: `"{timestamp_ms}:{uuid}"` (e.g., `"1704067200000:01933e3a-1234-7000-8000-000000000000"`)
**And** millisecond timestamp enables sorting by time
**And** UUID suffix ensures uniqueness (multiple events per millisecond)

### **Retention Policy:**

**Given** event storage must be bounded
**When** I implement retention policy
**Then** keep last 1000 events OR 24 hours, whichever is larger
**And** cleanup runs every hour (background task)
**And** cleanup deletes events where `timestamp < now() - 24 hours` AND `count > 1000`

**Given** cleanup must not block event bus
**When** I implement cleanup task
**Then** spawn background task:
```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour
    loop {
        interval.tick().await;
        if let Err(e) = persistence.cleanup().await {
            tracing::warn!(?e, "Event cleanup failed");
        }
    }
});
```

### **Query API:**

**Given** I need to retrieve historical events
**When** I implement query methods
**Then** provide API:
```rust
impl EventPersistence {
    /// Get all events since timestamp (Unix milliseconds).
    pub async fn get_events_since(&self, timestamp_ms: i64)
        -> Result<Vec<DomainEvent>, CacheError>;

    /// Get events by type (filter by event type).
    pub async fn get_events_by_type(&self, event_type: EventType)
        -> Result<Vec<DomainEvent>, CacheError>;

    /// Get last N events (for debugging).
    pub async fn get_last_events(&self, count: usize)
        -> Result<Vec<DomainEvent>, CacheError>;
}
```
**And** queries return deserialized `DomainEvent` structs (not raw bytes)
**And** queries use Redb range scans (efficient for time-ordered keys)

### **Performance Overhead:**

**Given** persistence must not slow publish
**When** I measure overhead
**Then** persistence adds <100μs to publish latency (99th percentile)
**And** rkyv serialization: <10μs
**And** Redb write: <90μs (disk I/O)
**And** total publish latency budget: 1ms (persistence is 10% of budget)

**Given** performance must be validated
**When** I write benchmarks in `crates/adapters/benches/event_persistence.rs`
**Then** benchmark:
- Publish 10,000 events with persistence enabled → measure total time
- Verify persistence overhead <1ms per event
- Query last 1000 events → measure latency (<10ms)

### **Debugging CLI Integration (Future Epic):**

**Given** developers need to inspect events
**When** I design CLI interface (not implemented in this epic)
**Then** plan commands:
```bash
lithos events list --since="10m ago"
lithos events filter --type=SchemaCreated
lithos events tail -f  # follow mode (real-time)
lithos events export --format=json > events.json
```
**And** CLI uses `get_events_since()` and `get_events_by_type()` methods

### **Event Replay (Future Feature):**

**Given** event sourcing may be needed later
**When** I design for future replay capability
**Then** ensure:
- Events are immutable (never modified after persistence)
- Events include full payload (not just metadata)
- Events are time-ordered (enables sequential replay)
- Schema versioning supports forward compatibility (new fields don't break old events)

### **Testing:**

**Given** persistence is implemented
**When** I write unit tests
**Then** test event storage (publish → query returns same event)
**And** test retention policy (insert 1100 events → cleanup removes oldest 100)
**And** test query by timestamp (`get_events_since()` returns correct range)
**And** test query by type (`get_events_by_type()` filters correctly)
**And** test persistence failure (Redb unavailable → publish still succeeds)
**And** test performance (persistence overhead <100μs)
**And** test deserialization (rkyv roundtrip preserves event data)

---

## Story 8.9: Define Event Bus Integration Contracts

As a developer integrating with the event system,
I want clear integration contracts,
So that other epics know how to publish and subscribe to events.

**Acceptance Criteria:**

### **Integration Contract Table:**

**Given** I need comprehensive integration documentation
**When** I define contracts for each epic
**Then** create table in `docs/architecture/event-bus-integration.md`:

| Epic    | Events Published                 | Events Subscribed                    | Purpose                                      |
|---------|----------------------------------|--------------------------------------|----------------------------------------------|
| Epic 6  | `ConfigUpdated`                  | -                                    | Notify system when config reloads            |
| Epic 7  | `SchemaCreated`, `PropertyBankUpdated` | `ConfigUpdated`                     | Reload property bank when config changes     |
| Epic 10 | `NoteCreated`, `FrontmatterValidated` | `SchemaCreated`, `PropertyBankUpdated` | Revalidate notes when schemas update         |
| Epic 12 | `TemplateCreated`                | `ConfigUpdated`, `SchemaCreated`     | Reload templates when config/schemas change  |
| LSP     | -                                | `SchemaCreated`, `TemplateCreated`, `NoteCreated` | Sync IDE with vault state changes           |

### **Publisher Contract Pattern:**

**Given** epics need to publish events
**When** I define publisher contract
**Then** pattern is:
```rust
// 1. After domain aggregate state change:
let config = Config::from_merged(...)?;
config.take_events();  // Get pending events from aggregate

// 2. Publish each event:
for event in pending_events {
    event_bus.publish(DomainEvent::Config(event)).await?;
}

// 3. Handle publish errors:
match result {
    Ok(_) => { /* continue */ }
    Err(EventError::ChannelClosed { .. }) => {
        // Event bus unavailable, log and continue (degraded mode)
        tracing::warn!("Event bus unavailable, continuing without events");
    }
    Err(e) => return Err(e.into()),  // Propagate other errors
}
```

**Given** error handling must be consistent
**When** publish fails
**Then** guidelines:
- `ChannelClosed`: Log error, continue operation (best-effort event delivery)
- `InvalidPayload`: Programming error, fix aggregate or validation logic
- `PublishTimeout`: Retry with backoff (transient failure)

### **Subscriber Contract Pattern:**

**Given** epics need to subscribe to events
**When** I define subscriber contract
**Then** pattern is:
```rust
// 1. Subscribe during service initialization:
let mut receiver = event_bus.subscribe(EventType::ConfigUpdated).await?;

// 2. Spawn background task to handle events:
tokio::spawn(async move {
    while let Some(event) = receiver.recv().await {
        match event.as_ref() {
            DomainEvent::Config(ConfigEvents::ConfigUpdated(updated)) => {
                // React to config change
                if let Err(e) = handle_config_update(updated).await {
                    tracing::error!(?e, "Failed to handle config update");
                    // Continue processing (don't crash on single event failure)
                }
            }
            _ => { /* ignore other events */ }
        }
    }
});

// 3. Graceful shutdown: drop receiver or check for shutdown signal
```

**Given** subscriber resilience is critical
**When** event handler fails
**Then** guidelines:
- Log error but continue processing next event (don't crash)
- Emit metric for failed event handling
- Consider retry for transient failures (e.g., database unavailable)

### **Integration Examples by Epic:**

#### **Epic 6 (Config) Publisher:**
```rust
// crates/adapters/src/spi/config/command.rs
impl ConfigCommand {
    async fn reload_config(&self) -> Result<Config, ConfigError> {
        let config = self.loader.load()?;

        // Publish event after successful reload
        let event = ConfigEvents::ConfigUpdated(ConfigUpdated::new(
            "vault".to_string(),
            now(),
        ));
        self.event_bus.publish(DomainEvent::Config(event)).await?;

        Ok(config)
    }
}
```

#### **Epic 7 (Schema) Subscriber:**
```rust
// crates/adapters/src/spi/schema/registry.rs
impl SchemaRegistry {
    pub async fn start_event_listener(&self) -> Result<(), SchemaError> {
        let mut receiver = self.event_bus.subscribe(EventType::ConfigUpdated).await?;
        let property_bank = self.property_bank.clone();

        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let DomainEvent::Config(ConfigEvents::ConfigUpdated(_)) = event.as_ref() {
                    tracing::info!("Config updated, reloading property bank");
                    if let Err(e) = property_bank.reload().await {
                        tracing::error!(?e, "Property bank reload failed");
                    }
                }
            }
        });

        Ok(())
    }
}
```

#### **Epic 10 (Indexer) Publisher + Subscriber:**
```rust
// crates/app/src/services/indexer.rs
impl Indexer {
    async fn index_note(&self, path: PathBuf) -> Result<(), IndexerError> {
        let note = Note::from_file(path)?;

        // Publish NoteCreated event
        let event = NoteEvents::NoteCreated(NoteCreated::new(
            note.id(),
            note.path().to_string(),
            now(),
        ));
        self.event_bus.publish(DomainEvent::Note(event)).await?;

        Ok(())
    }

    pub async fn start_schema_listener(&self) -> Result<(), IndexerError> {
        let mut receiver = self.event_bus.subscribe(EventType::SchemaCreated).await?;

        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let DomainEvent::Schema(SchemaEvents::SchemaCreated(created)) = event.as_ref() {
                    tracing::info!("Schema created, revalidating notes");
                    // Trigger note revalidation
                }
            }
        });

        Ok(())
    }
}
```

### **Event Bus Lifecycle:**

**Given** event bus must be initialized before use
**When** I define initialization contract
**Then** pattern is:
```rust
// main.rs or service initialization
let event_bus = Arc::new(HybridEventBus::new()?);

// Pass event_bus to all services needing it
let config_service = ConfigService::new(event_bus.clone());
let schema_service = SchemaService::new(event_bus.clone());
let indexer_service = IndexerService::new(event_bus.clone());

// Start all subscribers
config_service.start_listeners().await?;
schema_service.start_listeners().await?;
indexer_service.start_listeners().await?;

// Graceful shutdown
event_bus.shutdown().await?;
```

### **Contract Validation:**

**Given** integrations must be tested
**When** I validate contracts
**Then** integration tests verify:
- Epic 6 publishes `ConfigUpdated` → Epic 7 receives it
- Epic 7 publishes `SchemaCreated` → Epic 10 receives it
- Epic 10 publishes `NoteCreated` → LSP receives it
- All events use correct plane (MPSC/Broadcast/Watch)
- No events are lost during normal operation

### **Testing:**

**Given** contracts are defined
**When** I write integration tests in `crates/adapters/tests/event_integration.rs`
**Then** test Epic 6 → Epic 7 contract (ConfigUpdated triggers PropertyBank reload)
**And** test Epic 7 → Epic 10 contract (SchemaCreated triggers note validation)
**And** test Epic 10 → LSP contract (NoteCreated updates IDE state)
**And** test publisher error handling (ChannelClosed → graceful degradation)
**And** test subscriber error handling (handler failure → continue processing)

---

## Story 8.10: Create Event Bus Mocks for Testing

As a developer testing event-driven code,
I want comprehensive event bus mocks,
So that event interactions can be tested in isolation.

**Acceptance Criteria:**

### **MockEventBus Implementation:**

**Given** I need to mock event bus for testing
**When** I use `mockall` crate
**Then** `EventBusPort` trait auto-generates `MockEventBusPort`:
```rust
use mockall::predicate::*;
use mockall::mock;

// Auto-generated by #[mockall::automock] on EventBusPort trait
let mut mock = MockEventBusPort::new();
```
**And** mock is available in test code via `use lithos_domain::MockEventBusPort;`

### **Expectation API:**

**Given** I need to verify event publishing
**When** I set expectations
**Then** API supports:
```rust
mock.expect_publish()
    .with(predicate::eq(DomainEvent::Config(...)))
    .times(1)
    .returning(|_| Ok(()));

mock.expect_publish()
    .with(predicate::function(|e: &DomainEvent| {
        matches!(e, DomainEvent::Schema(SchemaEvents::SchemaCreated(_)))
    }))
    .times(1)
    .returning(|_| Ok(()));
```
**And** expectations can match exact events or use predicates
**And** `times(n)` verifies event is published exactly n times

**Given** I need to simulate errors
**When** I configure error returns
**Then** API supports:
```rust
mock.expect_publish()
    .returning(|_| Err(EventError::ChannelClosed {
        event_type: EventType::ConfigUpdated
    }));
```

### **Subscription Mocking:**

**Given** I need to mock subscription
**When** I set subscribe expectations
**Then** pattern is:
```rust
use tokio::sync::mpsc;

let (tx, rx) = mpsc::unbounded_channel();
mock.expect_subscribe()
    .with(eq(EventType::SchemaCreated))
    .times(1)
    .return_once(|_| Ok(Receiver::Mpsc(rx)));

// Test code can send events via tx:
tx.send(Arc::new(DomainEvent::Schema(...))).unwrap();
```
**And** mock returns controlled receiver for testing subscriber logic

### **Assertion Helpers:**

**Given** I need to verify event contents
**When** I write test assertions
**Then** helper functions:
```rust
pub fn assert_event_published(
    mock: &MockEventBusPort,
    event_type: EventType,
) {
    // Verify event was published (uses mockall's verify_expectations)
    mock.checkpoint();
}

pub fn capture_published_events(
    mock: &mut MockEventBusPort,
) -> Vec<DomainEvent> {
    let mut captured = Vec::new();
    mock.expect_publish()
        .returning(move |e| {
            captured.push(e.clone());
            Ok(())
        });
    captured
}
```

### **Test Fixtures:**

**Given** I need sample events for testing
**When** I create test fixtures in `crates/domain/src/events/fixtures.rs`
**Then** provide factory functions:
```rust
pub fn sample_config_updated() -> DomainEvent {
    DomainEvent::Config(ConfigEvents::ConfigUpdated(
        ConfigUpdated::new("vault".to_string(), 1704067200),
    ))
}

pub fn sample_schema_created() -> DomainEvent {
    DomainEvent::Schema(SchemaEvents::SchemaCreated(
        SchemaCreated::new(
            Uuid::now_v7(),
            "task".to_string(),
            1704067200,
        )
    ))
}

// ... fixtures for all event types
```

### **Example Test Cases:**

**Given** I use mocks in tests
**When** I write integration tests
**Then** examples show:

#### **Test Config Service Publishes Event:**
```rust
#[tokio::test]
async fn config_service_publishes_on_reload() {
    // GIVEN a config service with mock event bus
    let mut mock_bus = MockEventBusPort::new();
    mock_bus.expect_publish()
        .with(predicate::function(|e: &DomainEvent| {
            matches!(e, DomainEvent::Config(ConfigEvents::ConfigUpdated(_)))
        }))
        .times(1)
        .returning(|_| Ok(()));

    let service = ConfigService::new(Arc::new(mock_bus));

    // WHEN config is reloaded
    service.reload_config().await.unwrap();

    // THEN ConfigUpdated event was published (verified by mockall)
}
```

#### **Test Schema Service Subscribes to Config Events:**
```rust
#[tokio::test]
async fn schema_service_reloads_on_config_update() {
    // GIVEN a schema service with mock event bus
    let mut mock_bus = MockEventBusPort::new();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    mock_bus.expect_subscribe()
        .with(eq(EventType::ConfigUpdated))
        .times(1)
        .return_once(|_| Ok(Receiver::Mpsc(rx)));

    let service = SchemaService::new(Arc::new(mock_bus));
    service.start_listeners().await.unwrap();

    // WHEN ConfigUpdated event is sent
    tx.send(Arc::new(sample_config_updated())).unwrap();

    // THEN schema service reloads property bank (verify via service state)
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(service.property_bank_reloaded());
}
```

### **Mock Limitations:**

**Given** mocks have limitations
**When** I document mock behavior
**Then** explain:
- Mocks don't test actual event delivery (use integration tests for end-to-end)
- Mocks don't test concurrency (use real `HybridEventBus` for concurrency tests)
- Mocks verify interactions, not event bus implementation

### **Testing:**

**Given** mocks are implemented
**When** I write tests demonstrating mock usage
**Then** test publish expectation (event published exactly once)
**And** test subscribe expectation (subscription created)
**And** test error simulation (ChannelClosed error returned)
**And** test event capture (collect all published events)
**And** test fixture usage (sample events match expected structure)

---

## Story 8.11: Review Epic 8 Test Suite

As a senior developer conducting adversarial code review,
I want to brutally critique and improve the Epic 8 test suite to its foundation,
So that tests are comprehensive, maintainable, and catch real-world issues before production deployment.

**Acceptance Criteria:**

**Given** `_bmad-output/test-design-system.md` and `_bmad-output/test-developer-guide.md` provide testing standards and tools
**When** I reference the guide during review
**Then** I validate compliance with Lithos testing hierarchy, async patterns, fixtures, and utilities

**Given** all Epic 8 public components are implemented
**When** I verify test coverage
**Then** all public functions, structs, and modules have corresponding unit tests
**And** coverage includes:
- `EventBusPort` trait methods (`publish`, `subscribe`, `shutdown`)
- `MpscPlane`, `BroadcastPlane`, `WatchPlane` implementations
- `HybridEventBus` routing logic
- Event validation functions
- Event persistence (`EventPersistence` storage/retrieval)
- All `DomainEvent` variants serialization/deserialization

**Given** all Epic 8 public APIs are documented
**When** I verify doc test coverage
**Then** all public components have runnable doc tests demonstrating usage
**And** doc tests include:
- `EventBusPort::publish()` example
- `EventBusPort::subscribe()` example
- `DomainEvent` creation for each variant
- `EventError` handling patterns
- `EventPersistence` query examples

**Given** all Epic 8 components are implemented with tests
**When** I conduct adversarial review
**Then** I identify and eliminate false positives, redundant tests, and inadequate edge case coverage
**And** adversarial scenarios include:
- Concurrent publishers (4+ threads) + concurrent subscribers
- Channel closed mid-publish (receiver dropped)
- Backpressure under extreme load (fill MPSC channel completely)
- Broadcast lag (slow subscriber, verify `RecvError::Lagged`)
- Watch deduplication (publish same event twice)
- Event persistence failure (Redb unavailable)
- Large payload validation (64KB limit enforcement)
- UUID v7 validation (future timestamp rejection)

**Given** I take adversarial position against the test suite
**When** I critique test quality
**Then** I assess if tests actually validate business requirements vs implementation details
**And** verify tests check:
- **Business requirement**: Events are delivered to all subscribers (not just "send() succeeds")
- **Business requirement**: Event ordering preserved in MPSC (not just "channel works")
- **Business requirement**: LSP sees latest state (not just "watch channel updates")
- **Business requirement**: Graceful degradation when event bus fails (not just "error returned")

**Given** the test suite is implemented
**When** I review for redundancy
**Then** I eliminate duplicate test cases and consolidate overlapping coverage
**And** remove tests that only verify Tokio channel behavior (trust the library)
**And** focus on Lithos-specific logic (routing, validation, persistence, integration)

**Given** tests are executed
**When** I measure performance
**Then** test execution completes in <30 seconds for the full Epic 8 suite
**And** breakdown:
- Unit tests: <10 seconds (all planes, validation, persistence)
- Integration tests: <15 seconds (epic-to-epic event flows)
- Benchmarks: <5 seconds (publish latency, throughput)

**Given** I conduct brutal foundation critique
**When** I assess test design
**Then** I verify tests use proper fixtures, avoid flaky behavior, and maintain clear intent
**And** check for:
- **BDD comments**: All tests include GIVEN-WHEN-THEN explaining business context (not just "test_publish_works")
- **Fixtures**: Use `sample_config_updated()` instead of inline event construction
- **Async patterns**: All async tests use `#[tokio::test]` (not `#[test]`)
- **No sleeps**: Avoid `tokio::time::sleep()` for synchronization (use channels/barriers)
- **Deterministic**: No race conditions (order-dependent assertions only when guaranteed)

**Given** test suite is reviewed
**When** I check maintainability
**Then** test code follows same quality standards as production code with proper documentation
**And** verify:
- Tests have module-level docs explaining what's being tested
- Helper functions have doc comments
- Test names describe scenario (not implementation): `publishes_event_to_all_broadcast_subscribers` not `test_broadcast_send`
- No magic numbers (use named constants: `const BACKPRESSURE_TIMEOUT: Duration = Duration::from_millis(100)`)

**Given** BDD comments are required
**When** I review all tests
**Then** every test includes GIVEN-WHEN-THEN comments:
```rust
#[tokio::test]
async fn publishes_event_to_mpsc_plane() {
    // GIVEN an MPSC plane with a subscriber
    let plane = MpscPlane::new();
    let mut receiver = plane.subscribe();

    // WHEN an event is published
    let event = Arc::new(sample_note_created());
    plane.publish(event.clone()).await.unwrap();

    // THEN the subscriber receives the event
    let received = receiver.recv().await.unwrap();
    assert_eq!(received, event);
}
```

**Given** integration tests must validate cross-epic contracts
**When** I review integration tests
**Then** verify tests for each contract in Story 8.9:
- Epic 6 publishes `ConfigUpdated` → Epic 7 property bank reloads
- Epic 7 publishes `SchemaCreated` → Epic 10 indexer revalidates notes
- Epic 10 publishes `NoteCreated` → LSP state updates
**And** integration tests use real `HybridEventBus` (not mocks)

---

## Story 8.12: Document Event Bus Integration for Developers

As a developer integrating with the event system,
I want comprehensive developer documentation for event bus usage,
So that other epics can properly publish and subscribe to events.

**Acceptance Criteria:**

### **Architecture Documentation:**

**Given** developers need architectural overview
**When** I create `docs/architecture/event-bus.md`
**Then** document includes:
- **Overview**: Hybrid event bus (MPSC/Broadcast/Watch) per ADR 0007
- **Three Planes**: Data (indexing), Control (signals), State (LSP)
- **Event Flow Diagram**: Publisher → Routing → Plane → Subscriber
- **Performance Characteristics**: Latency targets, throughput limits
- **Use Cases**: When to use each plane

**Given** plane selection is critical
**When** I explain plane characteristics
**Then** table shows:

| Plane     | Channel Type | Semantics        | Use Case                        | Latency   | Throughput   |
|-----------|--------------|------------------|---------------------------------|-----------|--------------|
| MPSC      | Data         | FIFO, reliable   | Indexing pipeline (high volume) | <1ms      | 10,000/sec   |
| Broadcast | Control      | Fan-out, lossy   | System signals (shutdown, config) | <10μs     | 100/sec      |
| Watch     | State        | Latest-only      | LSP state sync (latest version) | <1μs      | 1,000/sec    |

### **Developer Guide:**

**Given** developers need step-by-step integration guide
**When** I create `docs/guides/event-bus-integration.md`
**Then** sections include:

#### **1. Publishing Events:**
```rust
// Step 1: Get event bus instance (injected via constructor)
let event_bus: Arc<dyn EventBusPort> = ...;

// Step 2: Create domain event after aggregate state change
let event = DomainEvent::Config(ConfigEvents::ConfigUpdated(
    ConfigUpdated::new("vault".to_string(), now())
));

// Step 3: Publish event
event_bus.publish(event).await?;

// Step 4: Handle errors
match result {
    Ok(_) => { /* success */ }
    Err(EventError::ChannelClosed { .. }) => {
        // Degraded mode: log and continue
        tracing::warn!("Event bus unavailable");
    }
    Err(e) => return Err(e.into()),
}
```

#### **2. Subscribing to Events:**
```rust
// Step 1: Subscribe during service initialization
let mut receiver = event_bus.subscribe(EventType::ConfigUpdated).await?;

// Step 2: Spawn background task to handle events
tokio::spawn(async move {
    while let Some(event) = receiver.recv().await {
        match event.as_ref() {
            DomainEvent::Config(ConfigEvents::ConfigUpdated(updated)) => {
                // React to config change
                handle_config_update(updated).await?;
            }
            _ => { /* ignore */ }
        }
    }
});
```

#### **3. Choosing the Right Event Type:**
- **Use existing events** (ConfigUpdated, SchemaCreated, etc.) when possible
- **Add new events** only if cross-context communication is needed
- **Don't use events** for internal module communication (use method calls)

### **API Reference:**

**Given** developers need API documentation
**When** I document `EventBusPort` trait
**Then** rustdoc includes:
```rust
/// Event bus port for publishing and subscribing to domain events.
///
/// # Architecture
/// The event bus uses a hybrid approach with three planes:
/// - **MPSC (Data Plane)**: Reliable indexing pipeline
/// - **Broadcast (Control Plane)**: System-wide signals
/// - **Watch (State Plane)**: LSP state synchronization
///
/// # Examples
/// ```rust
/// use lithos_domain::{EventBusPort, DomainEvent, ConfigEvents, ConfigUpdated};
///
/// async fn publish_config_update(bus: Arc<dyn EventBusPort>) {
///     let event = DomainEvent::Config(ConfigEvents::ConfigUpdated(
///         ConfigUpdated::new("vault".to_string(), 1704067200)
///     ));
///     bus.publish(event).await.unwrap();
/// }
/// ```
```

### **Integration Patterns:**

**Given** common patterns need documentation
**When** I document integration patterns
**Then** include examples from Story 8.9:
- Epic 6 → Epic 7 (Config reload triggers schema reload)
- Epic 7 → Epic 10 (Schema change triggers note validation)
- Epic 10 → LSP (Note indexed triggers IDE update)

### **Troubleshooting Guide:**

**Given** developers encounter issues
**When** I create troubleshooting section
**Then** cover common problems:

| Problem                     | Symptom                          | Solution                                  |
|-----------------------------|----------------------------------|-------------------------------------------|
| Events not received         | Subscriber never gets events     | Check routing table, verify event type    |
| Slow subscriber lag         | `RecvError::Lagged` errors       | Increase buffer size or process faster    |
| Event bus unavailable       | `ChannelClosed` errors           | Check event bus initialization order      |
| Memory leak                 | Growing memory usage             | Ensure receivers are dropped when done    |
| Out-of-order events         | Events arrive in wrong sequence  | Use MPSC plane (not Broadcast/Watch)      |

### **Performance Tuning:**

**Given** performance is critical
**When** I document tuning options
**Then** include guidelines:
- **MPSC channel size**: Default 1000, increase for burst indexing (e.g., 5000)
- **Broadcast capacity**: Default 16, rarely needs tuning
- **Event payload size**: Keep <1KB for optimal performance (avoid large strings)
- **Persistence**: Disable if debugging not needed (saves 100μs per event)

### **Testing Guidance:**

**Given** developers need to test event-driven code
**When** I document testing patterns
**Then** include:
- Use `MockEventBusPort` for unit tests (Story 8.10)
- Use real `HybridEventBus` for integration tests
- Use `sample_*()` fixtures for test events
- Verify event publishing with mockall expectations
- Test subscriber error handling (event processing failures)

### **Migration Guide (Future):**

**Given** event bus may evolve
**When** I plan for future changes
**Then** document:
- Adding new event types (requires routing table update)
- Changing event payloads (use versioning, maintain backward compatibility)
- Replacing event bus implementation (trait abstraction allows swapping)

### **Testing:**

**Given** documentation is complete
**When** I validate documentation quality
**Then** verify:
- All code examples compile and run
- Examples demonstrate real use cases (not toy examples)
- Troubleshooting guide covers 80% of common issues (measured by support tickets)
- API docs have 100% coverage for public items
- Developer guide has step-by-step walkthroughs for all integration patterns
