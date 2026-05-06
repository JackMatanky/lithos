---
title: "Safety and Determinism"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
scope: "Deterministic and safe test execution constraints"
---

# Safety and Determinism

## Determinism rules

- Use fixed seeds when randomness is involved.
- Avoid time-dependent flakiness; control time where needed.
- Ensure test outcomes are independent of execution order.

## Sync-first test posture

- Core domain logic remains sync-first by default.
- Introduce async test runtime only where behavior genuinely requires it.
- Keep blocking and long-running operations explicit in test design.

## Isolation rules

- Use `tempfile::TempDir` for filesystem isolation.
- Avoid global mutable state in tests.

## Concurrency safety

- Use explicit synchronization for concurrent behavior.
- Ensure async tests do not rely on arbitrary sleeps.

## Flake mitigation

- Use repeat-run (`mise run test:burn-in`) for suspected nondeterminism.
- Remove hidden shared state and order-coupling between tests.
