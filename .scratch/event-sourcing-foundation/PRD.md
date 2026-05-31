# PRD: Event Sourcing Foundation

**Status**: draft
**Created**: 2026-05-31
**Updated**: 2026-05-31
**Triage**: ready-for-agent

## Problem Statement

Lithos needs a shared event-log foundation that all bounded contexts can rely on for restartability and auditability. Today, event behavior is at risk of drifting because each context can otherwise choose different append, ordering, and compaction semantics.

The missing piece is not domain event modeling. The missing piece is a strict infrastructure contract for how events are identified, written, loaded, and compacted safely.

## Solution

Define a contract-first event-sourcing foundation that provides reusable infrastructure primitives and behavioral guarantees:

- canonical event identity (`EventId`)
- typed event table wrappers (`EventTable<V>`)
- repository-level append/load/compact contracts (`EventStore`)
- transactional allocation+append atomicity
- deterministic replay ordering
- compaction safety rules

Contexts own their event payload types, event lifecycles, and orchestration semantics. This PRD only standardizes event-log mechanics.

## Scope

This PRD defines shared infrastructure contracts only:

1. `EventId` newtype semantics
2. `EventTable<V>` typed table-definition wrapper semantics
3. `EventStore` contract semantics for append/load/compact
4. Required transactional and ordering invariants
5. Serialization and compatibility expectations with existing DB codec conventions
6. Compaction safety and failure behavior contracts

## Non-Goals

- Defining context-specific event enums (Discovery/Schema/Note/Template/Config)
- Defining projector state models or rehydration logic
- Defining pipeline orchestration phase ordering
- Defining cross-context completion tracking or coordination policies
- Defining a centralized global sequence table inside `db/`

## Boundary Rule

This foundation is reusable event-log infrastructure only. It defines *how* events are stored and recovered, not *what* events mean.

- `db/` exposes primitives and contracts.
- Context repositories outside `db/` own concrete table definitions and sequence-state storage.

## User Stories

1. As a persistence maintainer, I want a canonical `EventId` type so all event logs use one identity model.
2. As a repository maintainer, I want a typed `EventTable<V>` wrapper so event tables follow existing table abstractions.
3. As a reliability-focused engineer, I want ID allocation and event append to commit atomically so crashes cannot create ghost IDs.
4. As a context maintainer, I want a shared `EventStore` contract so each context implements event logging consistently.
5. As a platform engineer, I want per-context sequence isolation so contexts evolve without sequence collisions.
6. As a debugging engineer, I want deterministic per-context replay order so incident analysis is reproducible.
7. As a maintainer, I want compaction rules that preserve replay correctness so logs can be bounded safely.
8. As a test author, I want explicit crash and failure semantics so transaction guarantees can be validated.
9. As an architecture reviewer, I want domain event modeling explicitly excluded so this PRD remains infrastructural.

## Core Contracts

### 1) Event Identity Contract (`EventId`)

- `EventId` is the canonical key type for event logs.
- IDs are monotonically increasing within a context.
- Uniqueness is required within a context log.
- Cross-context global ordering is not required.
- `EventId` must implement both `redb::Key` and `redb::Value` so it can be used as a primary key in table definitions without adapter types.
- `EventId` construction follows "parse, don't validate": callers do not create unchecked IDs, and invariants are guaranteed by constructors/parsers that return domain-safe values.
- Invalid external representations must fail at parse time (fallible construction), not be tolerated and checked later.

#### EventId Construction Rules

- Construction APIs must make invalid states unrepresentable to downstream code.
- Prefer fallible constructors/parsers at boundaries (`TryFrom`, parser functions) and infallible constructors only where invariants are already proven.
- Sequence allocation paths should return `EventId` directly from trusted transactional state, while untrusted inputs must go through parse paths.
- Conversion/parse errors must be explicit typed errors, not silent coercions.

### 2) Typed Table Contract (`EventTable<V>`)

- `EventTable<V>` wraps a typed table definition keyed by `EventId`.
- Wrapper behavior mirrors existing DB table wrappers and remains storage-primitive focused.
- `EventTable<V>` must not embed domain semantics.

### 3) Sequence Allocation Contract

- Sequence state is repository-owned (context layer), not globally owned by `db/`.
- Foundation requirement: allocation must occur in the same write transaction as append.
- Storage layout for sequence state is an implementation choice of each repository, as long as contract invariants hold.

### 4) EventStore Behavior Contract

Shared trait behavior (exact signatures may adapt to existing repository patterns):

- `append_event(...) -> EventId`
  - Allocates next ID and appends event in one transaction.
  - Returns committed `EventId`.
- `load_all_events(...) -> Vec<Event>`
  - Returns events in deterministic ascending `EventId` order.
- `compact_events(...)`
  - Removes only events that are safe to remove under compaction rules.

## Transaction and Failure Model

Required behavior:

- **Crash before commit**: neither sequence advancement nor event append is visible.
- **Crash after commit**: both sequence advancement and appended event are visible.
- **Concurrent appends**: write serialization yields a deterministic committed order.
- **Recovery**: `load_all_events` must return all committed events in `EventId` order.

Explicitly rejected:

- Non-transactional counters for allocation (e.g., process-local atomics without transactional durability)
- Designs that can advance sequence state without persisting the corresponding event

## Replay Contract

- Replay consumers rely on strict ascending `EventId` order within a context.
- Replay determinism is guaranteed only within a context event log.
- Cross-context replay coordination is out of scope for this foundation.

## Serialization Contract

- Event payload serialization must align with existing DB codec conventions.
- Serialization format and validation strategy must preserve deterministic round-trip behavior for committed events.
- Foundation does not mandate domain payload schemas; it mandates consistent storage/codec behavior.
- `EventId` key/value encoding for redb must be stable and deterministic, with round-trip coverage in tests.

## Compaction Contract

Compaction is permitted only when removal cannot break required replay semantics for the owning context.

- Safe: events for work units that are terminal and no longer required for replay or recovery.
- Unsafe: compaction of events still required to reconstruct in-progress or unresolved state.
- Timing policy (when compaction runs) is owned by orchestrators/contexts, not by this foundation.

## Implementation Decisions

- Add `EventId` as canonical event-log key type.
- Add `EventTable<V>` as typed wrapper for event table definitions.
- Define shared `EventStore` behavior contract for append/load/compact.
- Require transactional allocate+append atomicity.
- Require deterministic ascending replay order per context.
- Keep sequence-state storage repository-owned, outside `db/` global ownership.
- Keep all domain event types and orchestration details out of scope.

## Testing Decisions

Good tests validate observable behavior and invariants, not internals.

Required coverage areas:

1. `EventId` ordering and comparison behavior
2. `EventId` parse/constructor behavior (invalid input rejected, invariants guaranteed through construction)
3. `EventId` `redb::Key`/`redb::Value` round-trip behavior
4. `EventTable<V>` integration with typed table definitions
5. Atomic append semantics (allocation+append commit together)
6. Crash model expectations (before commit vs after commit)
7. Per-context sequence isolation
8. Deterministic `load_all_events` ordering
9. Compaction safety (safe deletions succeed, unsafe policy violations are prevented by caller policy/tests)

## Acceptance Criteria

This PRD is satisfied when:

1. Shared infrastructure primitives are implemented without domain event leakage.
2. Repository implementations can prove transactional allocation+append atomicity.
3. Replay order is deterministic and validated in tests.
4. Compaction behavior is bounded by explicit safety rules.
5. No centralized domain-coupled sequence ownership is introduced in `db/`.

## Out of Scope

- Discovery-specific event enums and projector models
- Schema/Note/Template/Config event modeling
- Cross-context orchestration and completion tracking
- Parallel execution strategy decisions

## Further Notes

- This PRD should land before broad context-level event-sourcing adoption.
- Follow-on PRDs can define domain event models per context while reusing this foundation unchanged.
