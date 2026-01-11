# ADR 0007: Hybrid Event Orchestration Strategy

*   **Status**: Accepted
*   **Date**: 2026-01-11
*   **Stakeholders**: Jack (Developer), Architects

## Context

Lithos Rust must coordinate a write-heavy Redb indexer, a read-heavy LSP, and a latency-sensitive UI. During a vault scan, the system produces 10,000+ events in seconds. We must ensure the indexer isn't blocked, the LSP misses no state changes, and data is passed with zero-copy efficiency.

## Decision

We will implement a **Hybrid Orchestration** model:

1. **The Data Plane: Indexer Actor (MPSC):** Uses `tokio::sync::mpsc`. Ensures every "File Change" is processed exactly once in order. Backpressure naturally slows the producer to match disk I/O speeds.
2. **The Control Plane: System Broadcast (Broadcast):** General events (e.g., `Shutdown`, `IndexingComplete`) via `tokio::sync::broadcast`. Payloads wrapped in `Arc<T>` to avoid memory duplication.
3. **The State Plane: Diagnostics Path (Watch):** Uses `tokio::sync::watch` to hold the latest "Knowledge Graph Version." LSP pulls data only when idle, preventing "event storms" during rapid typing.

## Alternatives Considered

### Single Broadcast Bus
- **Pros**: Simpler mental model.
- **Cons**: Forces a choice between reliability (blocking) and performance (dropping events). Not suitable for high-frequency (10,000+ per sec) indexing events.

### Full Event Sourcing Store
- **Pros**: Perfect history for auditing and recovery.
- **Cons**: Overkill for a local CLI tool; significant storage and complexity overhead.

## Technical Validation

### Research Findings
- **Mechanical Sympathy**: Using `Arc<[u8]>` with `rkyv` allows passing large metadata payloads zero-copy.
- **Indexer Health**: The MPSC mailbox prevents the Indexer from being blocked by slow UI/LSP subscribers, as they pull from the State Plane independently.

### Compatibility & Performance
- **Hexagonal Alignment**: `EventBus` trait in `domain`, implementation in `adapters`.
- **Latency**: Sub-10ms LSP state synchronization achieved via the `watch` channel "Atomic Notification" pattern.

## Consequences

*   **Positive**: Decoupled components, reliable indexing, responsive UI/LSP, zero-copy data flow.
*   **Negative**: Increased complexity (three channel types); requires clear guidelines for developers on which "wire" to use for each event.

## Status Tracking

*   **Proposed**: 2026-01-08
*   **Accepted**: 2026-01-11
*   **Implemented**: 2026-01-11
