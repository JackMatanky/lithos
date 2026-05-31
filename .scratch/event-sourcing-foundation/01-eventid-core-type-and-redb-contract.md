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

## Approved clarifications (2026-05-31)

- Backing representation is `u64`.
- Monotonicity semantics are strict (`next > previous`) within a context, with gaps allowed.
- `EventId` is internal infrastructure; parsing should prioritize explicit boundary conversions and avoid broad string parsing APIs unless required by an existing boundary.
- Preferred location is `db/events.rs` for this foundation slice (infrastructure-local), not a new top-level `src/events/` module.

## EventId parse and construction policy

- Trusted internal allocation paths create `EventId` from validated numeric sequence state.
- Untrusted/external representations are accepted only through fallible conversion paths.
- Required conversion paths for this slice:
  - `TryFrom<u64>` (range/invariant-preserving constructor)
  - `TryFrom<&[u8]>` for redb/byte boundary compatibility
  - Optional `TryFrom<[u8; 8]>` if used by implementation ergonomics
- Do not add permissive string parsing by default (`FromStr`) unless required by a concrete boundary in this slice.

## Typed error contract (best-practice baseline)

Introduce a dedicated typed error (`EventIdError`) using `thiserror` with variants that describe boundary failures without backend leakage:

- `EmptyBytes` - byte input was empty.
- `InvalidLength { expected: usize, got: usize }` - byte input length mismatch.
- `ZeroNotAllowed` - reserved/invalid zero value when invariant requires positive IDs.
- `OutOfRange` - parsed numeric value violates representable domain constraints.
- `NonCanonicalEncoding` - bytes are structurally decodable but violate canonical encoding rule (if canonical rule is enforced).

Notes:

- Keep variants stable and semantic; avoid stringly error branching.
- Preserve parse-dont-validate: conversion returns `Result<EventId, EventIdError>` and never silently coerces.

## TDD implementation plan (approved)

Follow RED -> GREEN vertical slices and unit-test conventions in `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`.

1. `validation` tests (first): reject invalid byte forms and zero/out-of-range values with typed errors.
2. `constructor` tests: accept valid trusted numeric values and preserve identity.
3. `ordering` tests: verify deterministic strict ordering semantics for `EventId` comparisons and redb key compare behavior.
4. `serialization` tests: verify redb `Value` round-trip and stable deterministic encoding.
5. `integrity` tests: verify trait shape contracts needed by typed table wrappers (`redb::Key`, `redb::Value`, fixed width behavior as designed).

Naming/style:

- Prefer Structure A modules (`validation`, `constructor`, `ordering`, `serialization`, `integrity`).
- Use verb-first test names (`returns_*`, `rejects_*`, `preserves_*`).

## Scope guardrails for this issue

- In scope: `EventId` type, parse/construction invariants, direct redb key/value contract, ordering and round-trip tests.
- Out of scope: `EventTable<V>`, `EventStore`, transactional append behavior, compaction semantics (covered by later slices).
