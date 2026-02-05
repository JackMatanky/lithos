---
name: minimal-event-foundation-with-deferred-orchestration
status: accepted
stakeholders: [Jack (Developer), Architects]
date_proposed: 2026-01-08
date_decided: 2026-02-01
date_implemented: pending
date_updated: 2026-02-01
---

# ADR 0004: Minimal Event Foundation with Deferred Orchestration

## Context

Lithos requires event-driven architecture to prevent god-object orchestration patterns (validated by past Go implementation experience). However, implementing a full 3-tier event bus (MPSC/Broadcast/Watch) from day one introduces unnecessary complexity before core domain logic is established.

The system must balance:
- **Avoiding god-objects**: Direct orchestration leads to brittle coupling between aggregates
- **Domain purity**: Domain entities should not depend on infrastructure (event bus)
- **Evolutionary design**: Start minimal, defer complexity until needed

## Decision

We will implement a **Minimal Event Foundation** in Phase 1, deferring complex orchestration:

### Phase 1: Minimal Event Foundation (MVP)

**Domain Layer** (Pure):
- Aggregates return `(Entity, Vec<Event>)` - pure functions, no side effects
- Events are simple Rust enums: `enum DomainEvent { NoteCreated { id: Uuid }, NoteUpdated { id: Uuid, version: u32 } }`
- No event bus dependency in domain - keeps domain pure

**Application Layer** (Orchestration):
- Use cases collect events from aggregates: `let (note, events) = Note::create(...);`
- Single MPSC channel (`tokio::sync::mpsc`) for event dispatch
- Application layer dispatches events to handlers synchronously (for now)
- Event handlers are simple functions: `fn handle_note_created(db: &Database, event: &NoteCreated) -> Result<()>`

**Benefits**:
- Prevents god-object orchestration (aggregates emit events, don't directly call other services)
- Maintains domain purity (no infrastructure dependencies)
- Simple to implement and test
- Easy to extend to complex orchestration later

### Phase 2: Complex Orchestration (Deferred)

When performance profiling or feature requirements demand it, implement:

1. **The Data Plane: Indexer Actor (MPSC)**: High-throughput ordered processing (10,000+ events/sec)
2. **The Control Plane: System Broadcast**: Fire-and-forget notifications (shutdown, cache invalidation)
3. **The State Plane: Watch Channels**: Last-value-wins state synchronization (UI updates, LSP status)

**Triggers for Phase 2**:
- LSP implementation requires async state updates
- Vault indexing shows >100ms lag in event processing
- Multiple subscribers need different event delivery guarantees

## Alternatives Considered

### Alternative 1: Full 3-Tier Event Bus from Day One

- **Pros**: Production-ready architecture, handles all future use cases
- **Cons**: Over-engineering for MVP, significant complexity before domain is stable, harder to change once established
- **Verdict**: Defer to Phase 2 when requirements are validated

### Alternative 2: No Events (Direct Orchestration)

- **Pros**: Simplest possible implementation, no channels or async complexity
- **Cons**: Creates god-object orchestrators (validated anti-pattern from Go implementation), tight coupling between aggregates, hard to test and evolve
- **Verdict**: Rejected - past experience shows this leads to unmaintainable code

### Alternative 3: Full Event Sourcing Store

- **Pros**: Perfect history for auditing and recovery, replay capability
- **Cons**: Overkill for local CLI tool, significant storage and complexity overhead, not needed for current requirements
- **Verdict**: Not appropriate for Lithos use case

## Technical Validation

### Pattern Validation

**Event Emission Pattern**:
```rust
// Domain (pure function)
impl Note {
    pub fn create(id: Uuid, title: String) -> (Self, Vec<DomainEvent>) {
        let note = Note { id, title, version: 1 };
        let events = vec![DomainEvent::NoteCreated { id }];
        (note, events)
    }
}

// Application layer (orchestration)
pub fn create_note_use_case(db: &Database, title: String) -> Result<Uuid> {
    let id = Uuid::now_v7();
    let (note, events) = Note::create(id, title);

    db.save_note(&note)?;

    for event in events {
        event_dispatcher.dispatch(event)?;
    }

    Ok(id)
}
```

### Compatibility & Performance

- **Hexagonal Alignment**: Domain remains pure, events are POJOs (Plain Old Rust Objects)
- **Testability**: Easy to test domain logic (pure functions), easy to mock event dispatcher
- **Evolution Path**: Simple MPSC channel can be replaced with complex orchestration without changing domain

### Migration Path to Phase 2

When Phase 2 is triggered:
1. Replace simple event dispatcher with hybrid bus (MPSC/Broadcast/Watch)
2. Domain code remains unchanged (still returns `Vec<Event>`)
3. Application layer updated to route events to appropriate channels
4. Zero-copy payloads (`Arc<[u8]>` with rkyv) added only where profiling shows benefit

## Consequences

- **Positive**:
  - Prevents god-object anti-pattern while keeping implementation simple
  - Domain purity maintained (no infrastructure dependencies)
  - Easy to test (pure functions + mockable dispatcher)
  - Clear evolution path to complex orchestration when needed
  - Validated pattern from real-world Go experience
- **Negative**:
  - Simple MPSC may not scale to 10,000+ events/sec (acceptable for MVP)
  - Need to migrate to Phase 2 when LSP or high-throughput indexing is implemented
  - Developers must understand the phased approach (documented in this ADR)
