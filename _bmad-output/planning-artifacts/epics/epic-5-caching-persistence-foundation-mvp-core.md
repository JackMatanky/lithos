# Epic 5: Caching & Persistence Foundation [MVP CORE]

## Overview

Establish the unified multi-layer caching architecture required by the `lithos` service. This epic implements the SPI (Service Provider Interface) traits, the L1 (Memory)/L2 (Disk) coordinator, and the concrete implementations for Moka and Redb. This foundation is a prerequisite for Rate Limiting (Epic 6) and Session Management (Epic 8).

## Implementation Notes

- **Architecture**: Hexagonal (Ports & Adapters).
- **Location**: `crates/adapters/src/spi/cache/` and `crates/adapters/src/spi/errors.rs`.
- **Libraries**: `moka` (L1), `redb` (L2), `rkyv` (Serialization), `async-trait`, `mockall`, `tracing`, `thiserror`.
- **Pattern**: Read-through/Write-through caching strategy.

## Story 5.1: Define Adapter Traits

As a Developer,
I want strictly typed, async traits for caching defined in the adapter layer,
So that I can swap implementations and automatically mock them for testing without changing consumers.

**Acceptance Criteria:**

**Given** the `crates/adapters/src/spi` directory
**When** I define the `CacheError` enum in `errors.rs` deriving `thiserror::Error`
**Then** it includes variants for `IoError`, `SerializationError`, and `BackendError`
**And** the error type implements `Send + Sync` to support async contexts

**Given** the `CacheError` definition
**When** I define `trait Cache<K, V>` with methods `get(k)`, `put(k, v)`, and `delete(k)` in `cache/mod.rs`
**Then** the trait compiles with `#[async_trait]`
**And** it returns `Result<Option<V>, CacheError>`
**And** it accepts generic constraints allowing `Send + Sync` types

**Given** the `Cache` trait definition
**When** I annotate the trait with `#[mockall::automock]`
**Then** a `MockCache` struct is automatically generated for testing
**And** I can set expectations on cache method calls in unit tests (no manual mocks allowed)

## Story 5.2: Implement Moka L1 Adapter

As a System Architect,
I want an in-memory `Moka` adapter implementing the `Cache` trait,
So that frequently accessed data is served with sub-millisecond latency and operations are fully observable.

**Acceptance Criteria:**

**Given** the `moka` crate dependency
**When** I implement the `MokaCache` struct
**Then** it satisfies the `Cache<K, V>` trait bounds
**And** all public methods are decorated with `#[tracing::instrument(skip(self), level = "debug")]`

**Given** a configured `MokaCache`
**When** I insert or retrieve a value
**Then** the value is returned successfully
**And** a `tracing::event!` is emitted with relevant attributes (key, hit/miss status)
**And** it respects the configured TTL (Time To Live)

## Story 5.3: Implement Redb L2 Adapter

As a DevOps Engineer,
I want a robust `Redb` adapter implementing the `Cache` trait using `rkyv` serialization,
So that data persists across application restarts with high performance and complete observability.

**Acceptance Criteria:**

**Given** the `redb` and `rkyv` dependencies
**When** I implement the `RedbCache` struct
**Then** it allows persisting generic types that implement `rkyv::Archive`
**And** database transactions are wrapped in `tracing` spans

**Given** a `RedbCache` instance backed by a file
**When** I put a value, restart the instance, and get the value
**Then** the value is successfully deserialized and returned
**And** serialization/deserialization errors are logged via `tracing::error!` and mapped to `CacheError`

## Story 5.4: Implement Cache Coordinator

As a System Architect,
I want a `CacheCoordinator` struct that orchestrates L1 and L2 access,
So that hits are served fast from memory, misses fall back to disk, and consistency is guaranteed.

**Acceptance Criteria:**

**Given** a `CacheCoordinator` wrapping L1 (Empty) and L2 (Has Data)
**When** I request a key
**Then** the value is retrieved from L2
**And** the value is backfilled into L1 for future access (Read-through)
**And** a `tracing` event at `Level::INFO` records the "L1 Miss / L2 Hit"

**Given** a `CacheCoordinator` wrapping L1 (Empty) and L2 (Empty)
**When** an L2 miss occurs
**Then** log an `info` event "L2 Miss"

**Given** a `CacheCoordinator`
**When** I request a key present in L1
**Then** the value is retrieved from L1
**And** a `tracing` event at `Level::DEBUG` records the "L1 Hit"

**Given** a `CacheCoordinator`
**When** I write a key
**Then** it is written to L2 first (Persistence)
**And** it is written to L1 immediately after (Write-through)
**And** log a `debug` event "Cache Write" with the key included
**And** "Consistency Coordination" ensures both layers are in sync
**And** any failure in L2 write prevents the L1 write to maintain consistency

## Story 5.5: Performance Benchmarking

As a Performance Engineer,
I want to benchmark the caching layer using `criterion`,
So that I can verify the throughput and memory overhead meets requirements.

**Acceptance Criteria:**

**Given** a standard `criterion` benchmark suite
**When** I run benchmarks for `MokaCache`, `RedbCache`, and `CacheCoordinator`
**Then** I get statistical reports on operations per second
**And** I can verify memory usage is within expected bounds

## Story 5.6: SPI Documentation

As a Developer,
I want clear documentation for the Cache SPI,
So that other developers know how to implement new adapters or use the coordinator.

**Acceptance Criteria:**

**Given** the completed implementation
**When** I write `crates/adapters/src/spi/cache/README.md`
**Then** it explains the `Cache` trait contract
**And** it provides examples of configuring `MokaCache` and `RedbCache`
**And** it documents the `rkyv` requirements for cached values
