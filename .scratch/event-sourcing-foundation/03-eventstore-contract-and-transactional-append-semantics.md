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
