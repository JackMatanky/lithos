---
title: 04-compaction-safety-and-crash-model-verification
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-31T00:00:00+00:00
---

# Compaction safety + crash model verification

## What to build

Implement and verify event-foundation safety behavior for compaction and failure scenarios, ensuring replay correctness and crash semantics are preserved.

This slice must cover:

- Crash-before-commit semantics (no visible sequence/event effects).
- Crash-after-commit semantics (sequence/event effects both visible).
- Compaction safety boundaries (safe vs unsafe removal relative to replay needs).
- Preservation of deterministic replay for retained events.

## Acceptance criteria

- [ ] Tests verify crash-before-commit behavior (no partial visibility).
- [ ] Tests verify crash-after-commit behavior (fully committed visibility).
- [ ] Compaction tests verify safe-removal behavior and guardrails against replay-breaking assumptions.
- [ ] Replay determinism remains intact after allowed compaction operations.

## Blocked by

- `.scratch/event-sourcing-foundation/03-eventstore-contract-and-transactional-append-semantics.md`

## Approved clarifications

- This slice verifies behavior at the EventStore contract boundary and adapter
  failure surfaces; it does not introduce new storage primitives.
- Crash semantics are modeled using deterministic fault injection in tests
  (for example, failpoints around allocation/persist boundaries), not by
  process-kill integration harnesses in this issue.
- "Crash-before-commit" means the write transaction does not commit and no
  allocated sequence/event record becomes observable in subsequent loads.
- "Crash-after-commit" means allocation and event persistence are both visible
  together after recovery; no state where one is visible without the other.
- Compaction is prefix-only by cutoff (`compact_through` / `compact_before`),
  never rewrites surviving `EventId` values, never reuses IDs, and never moves
  sequence state backward.
- Replay determinism for retained events is defined as stable ascending
  `EventId` order and stable payload reconstruction for the same retained set.
- This slice focuses on correctness and safety properties; performance tuning,
  retention-policy UX, and projection rebuild tooling are out of scope.

## Agent Brief

**Category:** enhancement
**Summary:** Verify compaction and crash safety semantics so event replay
remains deterministic and append visibility is all-or-nothing.

**Current behavior:**
EventStore contract work defines append/load/compact semantics, but explicit
verification of crash boundaries and compaction safety is not yet codified as a
dedicated test slice. Without these checks, regressions could allow partial
visibility or replay-breaking assumptions.

**Desired behavior:**
Event append and sequence visibility obey strict transactional crash semantics:
no visibility before commit, full visibility after commit. Compaction only
removes events within allowed prefix bounds and preserves deterministic replay
for retained events.

**Key interfaces:**
- `EventStore` append/load/compact behavior contract and error/failure paths.
- Transaction boundary semantics around sequence allocation + event persistence.
- Compaction cutoff semantics (`compact_through` / `compact_before`) relative
  to replay requirements.
- Replay/load behavior for retained events (deterministic ascending `EventId`
  ordering and stable decoded payload sequence).

**Acceptance criteria:**
- [ ] Tests prove crash-before-commit yields zero observable effects (no event,
      no sequence advancement visibility).
- [ ] Tests prove crash-after-commit yields fully observable effects (event and
      sequence state both visible together).
- [ ] Tests enforce compaction safety boundaries: allowed prefix removal passes;
      replay-breaking assumptions are rejected or fail contract checks.
- [ ] Tests prove deterministic replay for retained events after allowed
      compaction.
- [ ] Failure-path tests avoid `unwrap()`/`expect()` in production paths and use
      explicit error assertions.

**Out of scope:**
- OS-level crash harnesses or kill -9 durability experiments.
- New compaction algorithms beyond existing prefix-cutoff semantics.
- Projection/event-bus subscriber behavior outside EventStore contract scope.
- Performance benchmarking or retention policy product decisions.

## Approved TDD plan

Follow RED -> GREEN slices in `docs/engineering/testing/unit.md`:

1. Add failing tests for crash-before-commit and crash-after-commit
   observability boundaries.
2. Add failing tests for safe compaction cutoff behavior and replay guardrails.
3. Make minimal implementation updates required by tests, then refactor while
   preserving deterministic replay assertions.
