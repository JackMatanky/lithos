---
title: 03-eventstore-contract-and-transactional-append-semantics
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-31T00:00:00+00:00
---

# EventStore contract + transactional append semantics

## What to build

Define and apply a shared `EventStore` infrastructure contract for append/load/compact behavior, with explicit guarantees for transactional sequence allocation and deterministic event loading.

This slice must enforce:

- Append allocates sequence state and persists the event in the same write transaction.
- Load returns deterministic ascending `EventId` order.
- Sequence state remains repository-owned by each context (no centralized global ownership in `db/`).
- Per-context sequence isolation is preserved.

## Acceptance criteria

- [ ] Shared append/load/compact contract is defined and usable by context repositories.
- [ ] Append semantics guarantee atomic allocate+append behavior.
- [ ] `load_all_events` ordering is deterministic and verified.
- [ ] Repository-owned sequence-state storage is preserved; no global sequence table ownership is introduced in `db/`.
- [ ] Tests verify per-context sequence isolation.

## Blocked by

- `.scratch/event-sourcing-foundation/01-eventid-core-type-and-redb-contract.md`
- `.scratch/event-sourcing-foundation/02-eventtable-wrapper-and-typed-table-integration.md`

## Approved clarifications

- Contract shape is infrastructure-only (`db` context) and generic over a typed
  event payload requiring `ArchivedEntity`; no domain event enums inside the
  shared contract.
- Contract location is explicitly `lithos-core/src/db/events.rs`.
- This slice defines trait/API + contract tests only; no concrete redb
  EventStore adapter implementation in this issue.
- Append returns the allocated `EventId` from the same write transaction that
  persists the event record.
- Allocation source is per-context sequence state (for example, one sequence key
  per repository stream/table namespace), not a global cross-context allocator
  table.
- Deterministic loading requires explicit ascending `EventId` iteration and
  tests that fail if insertion order differs from load order.
- **Compaction semantic for this slice:** prefix compaction by cutoff
  (`compact_through` / `compact_before`) that removes old events up to a bound,
  never rewrites surviving `EventId` values, never reuses historical IDs, and
  never moves sequence state backward.
- Slice scope is strictly contract + infrastructure + tests only (no consumer
  migrations in this issue).
- Outbox/event-bus dispatch and cross-context fan-out are out of scope.

## Agent Brief

**Category:** enhancement
**Summary:** Define a shared EventStore append/load/compact contract with
transactional append semantics and per-context sequence isolation.

**Current behavior:**
`EventId` and `EventTable<V>` primitives exist, but there is no shared
EventStore contract for repository adapters. Existing repositories implement
context-specific write/read behavior with atomic transactions, but event-log
append/load semantics are not standardized across contexts.

**Desired behavior:**
Repository adapters can implement one shared infrastructure contract that
guarantees append allocates the next `EventId` and persists the event in the
same transaction. Event loads are deterministic (ascending `EventId`), and
sequence state remains owned by each context repository with no global
cross-context sequence ownership.

**Key interfaces:**
- Shared EventStore trait(s) in infrastructure (`db/events.rs`) with
  append/load/compact operations and typed error surfaces.
- Append operation contract that returns allocated `EventId` and enforces
  atomic allocate+persist behavior.
- Per-context sequence-state representation and repository-owned persistence
  for cursor/max-event tracking.
- Deterministic load operation contract (ascending `EventId`) with explicit
  ordering guarantees.

**Acceptance criteria:**
- [ ] Shared append/load/compact contract is defined and consumable by context
      repositories without leaking domain semantics into `db`.
- [ ] Append allocate+persist behavior is atomic inside a single write
      transaction and returns the committed `EventId`.
- [ ] `load_all_events` (or equivalent load API) returns deterministic ascending
      `EventId` order and is covered by tests.
- [ ] Sequence state ownership remains per-context; no global sequence table is
      introduced under `db/` for cross-context allocation.
- [ ] Tests verify per-context sequence isolation (context A allocation does not
      advance context B).
- [ ] Failure-path tests prove no partial append state is visible when append
      fails after allocation attempt.

**Out of scope:**
- Cross-context/global event bus semantics.
- Event replay projections and subscriber dispatch.
- Migrating existing context repositories to event-sourced storage in this
  slice.
- Introducing concrete EventStore adapter implementations (tracked in follow-up
  issue).

## Approved TDD plan

Follow RED -> GREEN vertical slices and unit-test standards in
`docs/engineering/testing/unit.md` and
`docs/engineering/testing/unit-naming.md`.

1. Add contract/API tests for append/load/compact trait shape and payload
   generic constraints in `db/events.rs`.
2. Verify contract signatures encode deterministic load ordering and compaction
   cutoff semantics at the API boundary.
3. Keep this slice implementation-free beyond test fixtures needed to validate
   trait contracts.
