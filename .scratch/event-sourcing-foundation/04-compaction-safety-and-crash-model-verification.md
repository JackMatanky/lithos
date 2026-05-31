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
