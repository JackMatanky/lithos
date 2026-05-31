# PRD: Event Sourcing Foundation

**Status**: draft
**Created**: 2026-05-31
**Triage**: ready-for-agent

## Problem Statement

Lithos needs a shared, transaction-safe event foundation that context processors can rely on for restartability and auditability. Today, event behavior is fragmented and context-specific event modeling risks diverging without a common base contract.

## Solution

Introduce a generic event-sourcing foundation that standardizes event identity, event table typing, append/load/compact behavior, and transactional allocation guarantees. Contexts will build their own event models on top of this foundation later.

## User Stories

1. As a persistence maintainer, I want a canonical `EventId` type, so that event logs are uniformly keyed.
2. As a repository maintainer, I want a typed event table wrapper, so that event storage is consistent with existing table abstractions.
3. As a reliability-focused engineer, I want event append and ID allocation in the same transaction, so that crashes cannot produce ghost IDs.
4. As a context maintainer, I want a shared `EventStore` contract, so that each context can implement event sourcing consistently.
5. As a platform engineer, I want per-context event sequences, so that contexts can evolve independently.
6. As a debugging engineer, I want deterministic event ordering, so that replay and incident analysis are reproducible.
7. As a maintainer, I want compaction primitives, so that event logs remain bounded after completion.
8. As a test author, I want explicit crash-safety expectations, so that transaction semantics are verifiable.
9. As an architecture reviewer, I want context event modeling out of this PRD, so that foundation scope stays narrow.
10. As a future refactor owner, I want this foundation reusable across discovery, config, schema, note, and template contexts, so that event behavior remains coherent across the codebase.

## Implementation Decisions

- Add an `EventId` newtype as the canonical event key.
- Add an `EventTable<V>` wrapper for typed event-table definitions.
- Add an `EVENT_SEQUENCES` table for per-context transactional sequence allocation.
- Define a shared `EventStore` trait with append, load-all, and compact operations.
- Require append semantics to allocate the next event ID and persist the event in one write transaction.
- Require monotonic event ordering per context.
- Keep context event enums out of scope; this PRD only delivers shared infrastructure.
- Require serialization consistency with existing database codec conventions.

## Testing Decisions

- Good tests validate external behavior (ordering, allocation, compaction, transaction atomicity), not private implementation details.
- Test modules: EventId behavior, EventTable integration, transactional append semantics, sequence initialization, per-context isolation, compaction correctness.
- Prior art: repository seam tests and typestate/restartability-oriented tests already present in core modules.

## Out of Scope

- Context-specific event enum definitions.
- Cross-context orchestration policies.
- Processor-specific replay/projector logic.
- Parallel execution policy decisions.

## Further Notes

- This PRD is intentionally foundational and should ship before context-level event sourcing adoption.
