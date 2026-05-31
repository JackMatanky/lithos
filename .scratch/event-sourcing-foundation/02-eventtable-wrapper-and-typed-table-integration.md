---
title: 02-eventtable-wrapper-and-typed-table-integration
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-31T00:00:00+00:00
---

# EventTable wrapper + typed table integration

## What to build

Add an infrastructure-only `EventTable<V>` wrapper that provides typed event-table definitions keyed by `EventId` and aligns with existing DB table wrapper patterns.

This slice must keep strict boundaries:

- The wrapper is a storage primitive, not a domain abstraction.
- The wrapper does not encode context-specific event semantics.
- Integration demonstrates compatibility with existing table-definition usage patterns.

## Acceptance criteria

- [ ] `EventTable<V>` is implemented as a typed wrapper keyed by `EventId`.
- [ ] Behavior matches established DB wrapper ergonomics and conventions.
- [ ] No domain-specific behavior is embedded in `EventTable<V>`.
- [ ] Integration tests prove it works in representative table-definition flows.

## Blocked by

- `.scratch/event-sourcing-foundation/01-eventid-core-type-and-redb-contract.md`
