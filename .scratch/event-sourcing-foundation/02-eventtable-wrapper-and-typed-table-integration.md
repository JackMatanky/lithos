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

## Approved clarifications (2026-06-01)

- This issue only introduces the wrapper and tests. No consumer migrations.
- Integration proof is satisfied purely within `db/table.rs` tests.
- Visibility is `pub(crate)` until the first concrete consumer lands.

## Blocked by

- `.scratch/event-sourcing-foundation/01-eventid-core-type-and-redb-contract.md`

## Agent Brief

**Category:** enhancement
**Summary:** Add `EventTable<V>` as an infrastructure-only typed DB wrapper keyed by `EventId`.

**Current behavior:**
DB wrappers currently cover UUID-keyed, path-keyed, UUID<->path, and generic table patterns. There is no dedicated event-id keyed wrapper in the typed wrapper catalog.

**Desired behavior:**
A new `EventTable<V>` wrapper is available for redb table definitions keyed by `EventId`, with the same wrapper ergonomics used by existing DB wrappers (`const fn new`, `definition()` accessor, no domain semantics).

**Key interfaces:**
- `EventTable<V>` wraps `TableDefinition<'static, EventId, V>` as a storage primitive.
- The DB wrapper catalog exports `EventTable` as `pub(crate)` for internal rollout.
- Wrapper tests in `db/table.rs` verify const construction and typed compatibility.

**Acceptance criteria:**
- [ ] `EventTable<V>` exists as an event-id keyed typed table wrapper.
- [ ] API ergonomics match established wrapper conventions (`new`, `definition`, const construction).
- [ ] `EventTable<V>` remains infrastructure-only and embeds no event stream/domain semantics.
- [ ] Test coverage is implemented in `db/table.rs` only and verifies representative table-definition usage.
- [ ] No storage migrations or downstream consumer rewiring are performed in this slice.

**Out of scope:**
- Event-store append/read/query semantics.
- Transaction policy changes.
- Existing table constant migration to `EventTable<V>`.
- Any domain repository behavior changes.

## Approved TDD plan

Follow RED -> GREEN vertical slices using `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`.

1. Add failing tests in `db/table.rs` under a dedicated `event_table` module.
2. Cover wrapper construction/ergonomics first (`new`, `definition`, const construction).
3. Cover type integration behavior (`EventId` key and generic value support).
4. Implement minimal production code to pass each test incrementally.
5. Keep tests behavior-focused and aligned to naming convention (verb-first, Structure A).
