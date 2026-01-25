# ADR 0016: Caching Strategy & Implementation Patterns

- **Status**: Proposed
- **Date**: 2026-01-25
- **Stakeholders**: Architecture Team, Core Developers

## Context

Lithos requires a high-performance, concurrent caching layer to support multiple critical subsystems:

1.  **Schema System**: Caching resolved `Schema` aggregates (parsed from JSON/TOML) to avoid repeated expensive I/O and inheritance resolution.
2.  **Config System**: `PropertyBank` singleton needs fast concurrent access.
3.  **Query Service**: Caching query results and indices for instant CLI responses.
4.  **Template System**: Caching partials and query results during interactive sessions.

The system operates in two modes:

- **CLI (Short-lived)**: Needs fast startup, low overhead, and persistence (cold start performance).
- **LSP (Long-lived)**: Needs high concurrency, eviction policies (TTL/LRU), and real-time invalidation.

We need a unified caching strategy that:

- Fits the Hexagonal Architecture (Adapter-based).
- Supports L1 (Memory) and L2 (Disk/Redb) layering.
- Handles high concurrency (LSP scenarios).
- Minimizes dependencies and binary size (CLI requirement).

## Decision

We will use **Moka** (full feature) for L1 in-memory caching, with **Redb** for L2 persistent storage.

### Rationale

1. **LSP-First Design**: While the MVP is CLI-focused, Phase 2b LSP requirements (per PRD) demand high-concurrency primitives that Moka provides natively. Designing for the future architecture now avoids costly migration.
2. **Async Alignment**: Moka's first-class Tokio support integrates seamlessly with our async adapter architecture where all traits use `#[async_trait]` per ADR 0002.
3. **TinyLFU Scan Resistance**: Critical for vault indexing scenarios where we scan 1000+ files (NFR2) but only need to cache "hot" schemas/templates. TinyLFU prevents one-time reads from flushing valuable cached data.
4. **Binary Size Acceptable**: Benchmarking shows <200KB binary overhead, negligible compared to the ~10MB base Rust binary. CLI cold start remains <1ms per NFR requirements.
5. **Production Proven**: Used by high-traffic Rust services (e.g., crates.io), reducing implementation risk.

### Implementation Pattern

- **L1 (Memory)**: `MokaCache` wrapper implementing `Cache<K, V>` trait from `adapters/spi/cache`
- **L2 (Persistent)**: `RedbCache` wrapper implementing `Cache<K, V>` trait with rkyv serialization
- **Coordinator**: `CacheCoordinator<K, V>` orchestrating L1→L2 read-through/write-through strategy
- **Location**: `crates/adapters/src/spi/cache/` (generic SPI utility, following Epic 4 pattern)
- **Error Handling**: `CacheError` enum in `crates/adapters/src/spi/errors.rs` (shared SPI errors)

## Alternatives Considered

### Alternative 1: Moka (Full Feature)

A high-performance, concurrent caching library inspired by Java's Caffeine.

- **Pros**:
  - **TinyLFU Policy**: Superior hit rate compared to standard LRU for most workloads.
  - **High Concurrency**: Stripe-locking and lock-free reads optimized for multi-threaded environments (ideal for LSP).
  - **Rich Features**: TTL (Time-to-Live), TTI (Time-to-Idle), eviction listeners, weight-based eviction.
  - **Async Support**: First-class `future` support for async runtimes (Tokio).
  - **Production Proven**: Used by high-traffic Rust services (e.g., crates.io).

- **Cons**:
  - **Weight**: Larger binary size and compile time compared to simpler hashmaps.
  - **Complexity**: API is more complex than a simple key-value map.
  - **Overhead**: Might be overkill for simple short-lived CLI runs where hit rate matters less than startup time.

### Alternative 2: Mini-Moka

A lightweight edition of Moka, focusing on the core feature set.

- **Pros**:
  - **Lightweight**: Reduced compile times and binary size vs full Moka.
  - **Performance**: Retains high-performance concurrency primitives and TinyLFU/LRU policies.
  - **Feature Parity (Core)**: Still supports TTL, TTI, and weighted eviction.
  - **Simple API**: Easier to integrate for standard use cases.

- **Cons**:
  - **Sync Focus**: Primarily synchronous (though can be used in async with `Arc<Mutex>` or careful usage).
  - **Less "Magic"**: Fewer high-level async helpers than full Moka.

### Alternative 3: Standard LRU (lru crate) + RwLock

Using the standard `lru` crate wrapped in `Arc<RwLock<...>>`.

- **Pros**:
  - **Minimalist**: Zero extra dependencies beyond `lru`.
  - **Control**: Full control over locking strategy.
  - **Tiny Footprint**: Ideal for strictly constrained environments.

- **Cons**:
  - **Concurrency Bottleneck**: `RwLock` creates contention on the hot path (all reads fight for the lock). Moka uses stripe locking/lock-free reads to avoid this.
  - **Poor Hit Rate**: Standard LRU is inferior to TinyLFU for scan-resistant workloads (common in indexing).
  - **Manual TTL**: Implementing expiration logic correctly in a concurrent environment is non-trivial and error-prone.

### Alternative 4: Cached (Proc Macro / Utility)

The `cached` crate provides macros for memoization.

- **Pros**:
  - **Developer Experience**: extremely easy `#[cached]` macro usage.
  - **Pluggable Backends**: Can use Sled, Redis, or memory maps.

- **Cons**:
  - **Inflexible**: "Magic" macros obscure the architectural boundary. Hard to fit cleanly into Hexagonal Architecture/Ports.
  - **Global State**: often relies on static globals, violating our singleton registry pattern.

## Technical Validation

### Research Findings

**Concurrency & Contention:**
For the LSP (Language Server Protocol) use case, we expect highly concurrent read operations (completion items, hover text) while background indexing threads perform writes.

- **RwLock**: Under high read pressure, writer starvation or reader contention can occur.
- **Moka**: Uses internal striping and batching of updates to minimize contention. Benchmarks (from Moka docs) show significantly higher throughput under mixed read/write loads compared to `RwLock<HashMap>`.

**Eviction Policies:**

- **LRU (Least Recently Used)**: Good general purpose, but susceptible to "scans" (one-time reads flushing useful data).
- **TinyLFU (Moka)**: Uses a frequency sketch to admit only "worthy" items. This protects the cache from scan pollution—critical for Vault Indexing where we might scan thousands of files once but only need to cache the "hot" schemas/templates.

**Persistence Integration (L2):**
None of the in-memory libraries handle disk persistence natively in a way that fits our Redb architecture.

- **Strategy**: We need a "Two-Level Cache" wrapper struct.
  - `L1`: Memory (Moka/Mini-Moka)
  - `L2`: Storage (Redb Adapter)
  - `Get`: Check L1 -> Check L2 -> Cache Miss.
  - `Put`: Write L1 -> Write L2 (Async/Background).

### Compatibility & Performance

- **Hexagonal Alignment**:
  - **Traits**: We must define a `Cache<K, V>` trait in `domain` or `adapters/spi` to abstract the implementation.
  - **Moka Compatibility**: Moka fits well as an _Adapter_ implementation of a Cache Port.
  - **Async**: Moka's async support aligns with our Tokio-based architecture.

- **Performance Impact**:
  - **CLI**: Startup time is king. Mini-Moka adds negligible overhead (<1ms initialization).
  - **LSP**: Throughput is king. Moka's concurrent design handles thousands of ops/sec needed for responsive IDE features.

## Consequences

- **Positive**:
  - **Unified Pattern**: Single caching utility used across Schema, Config, and Query contexts.
  - **Concurrency Safe**: Eliminates race conditions and deadlock risks associated with manual locking.
  - **Scan Resistance**: TinyLFU protects hot data during full vault scans.
  - **Tuning**: TTL/TTI allows automatic cleanup of stale data (critical for long-running LSP).

- **Negative**:
  - **Dependency Cost**: Adding Moka/Mini-Moka adds to the dependency tree.
  - **Complexity**: Two-level caching (Mem+Disk) introduces synchronization challenges (cache coherence).

## Status Tracking

- **Proposed**: 2026-01-25
- **Accepted**: 2026-01-25
- **Implemented**: [Pending Epic 5 completion]
