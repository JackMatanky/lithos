---
title: 01-eventid-core-type-and-redb-contract
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-31T00:00:00+00:00
---

# EventId core type + redb contract

## What to build

Implement the foundation `EventId` type as the canonical event-log identifier with deterministic ordering semantics, redb compatibility, and parse-dont-validate construction guarantees.

This slice must establish:

- `EventId` supports monotonic ordering within a context.
- `EventId` implements `redb::Key` and `redb::Value` directly (no adapter indirection).
- Construction APIs enforce invariants through parsing/construction rather than post-hoc validation.
- Invalid external representations fail at parse time with explicit typed errors.

## Acceptance criteria

- [ ] `EventId` is introduced as the canonical event identifier primitive.
- [ ] `EventId` implements both `redb::Key` and `redb::Value`.
- [ ] Construction follows parse-dont-validate; untrusted inputs use fallible parse paths.
- [ ] Invalid inputs are rejected via typed errors (no silent coercion).
- [ ] Ordering semantics are deterministic and verified in tests.
- [ ] redb key/value round-trip encoding is verified in tests.

## Blocked by

None - can start immediately.
