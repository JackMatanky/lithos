# ADR 0007: Hybrid Event Orchestration Strategy

## Status
Accepted

## Context
Lithos Rust must coordinate a write-heavy Redb indexer, a read-heavy LSP, and a latency-sensitive UI. During a vault scan, the system produces 10,000+ events in seconds. We must ensure:
1. The Indexer is never blocked by a slow UI.
2. The LSP never misses a state-change event.
3. Data is passed with zero-copy efficiency using rkyv and Arc.

## Decision
We will implement a **Hybrid Orchestration** model:

### 1. The Data Plane: Indexer Actor (MPSC)
The `Indexer` will be implemented as an Actor with a dedicated `tokio::sync::mpsc` channel.
- **Reason:** Ensuring every "File Change" is processed exactly once in order.
- **Backpressure:** If the mailbox fills, the filesystem watcher will block, naturally slowing down the producer to match disk I/O speeds.

### 2. The Control Plane: System Broadcast (Broadcast)
General system events (e.g., `IndexingComplete`, `VaultReloadRequested`, `Shutdown`) will use `tokio::sync::broadcast`.
- **Payloads:** All payloads must be wrapped in `Arc<T>` to satisfy the `Clone` requirement without duplicating memory.

### 3. The Diagnostics Path: State Watch (Watch)
To satisfy the sub-10ms LSP requirement, we will use a specialized "Atomic Notification" pattern for diagnostics:
- A `tokio::sync::watch` channel will hold the latest "Knowledge Graph Version."
- Subscribers (LSP) check the version and pull data only when idle, preventing "event storms" during rapid typing.

## Mechanical Sympathy
- **rkyv + Arc:** Instead of passing structs, we will pass `Arc<[u8]>` containing rkyv-serialized metadata where appropriate, allowing subscribers to map the data zero-copy.
- **Trait-Based Bus:** To support Hexagonal Architecture, the `EventBus` trait will live in the `domain` crate.

## Rationale
Using a single pattern (like just Broadcast) would force us to choose between reliability (blocking) and performance (dropping events). This hybrid approach uses the right tool for each flow: MPSC for reliable writes, Broadcast for status, and Watch for state synchronization.

## Consequences
- **Complexity:** The system uses three types of channels (`mpsc`, `broadcast`, `watch`), which requires clear documentation of which "wire" to use for each event type.
- **Safety:** We must be careful to avoid circular dependencies between actors.
- **Observability:** Centralizing these channels in a single `EventBus` adapter makes it easier to trace the application's "nervous system."
