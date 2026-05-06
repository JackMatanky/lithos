---
title: "Integration Testing"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
scope: "Integration boundaries, public API, and adapter verification"
---

# Integration Testing

## Scope

- Test coordination between components and public contracts.
- Validate adapter behavior with realistic dependencies.
- Use temporary resources for filesystem/persistence tests.

## Placement

- Prefer integration test files in crate integration test locations.
- Test through public APIs and adapter contracts.

## Typical use cases

- Persistence adapter behavior (real temporary DB/files).
- Filesystem and ingestion workflows with temporary vault directories.
- Cross-component behavior where unit tests are insufficient.

## Patterns

- Place tests in integration test locations for cross-module behavior.
- Verify side effects explicitly.
- Keep tests independent and environment-agnostic.

## Mocking posture

- Use mocks for external services/boundaries when real dependency is unnecessary.
- Prefer real adapters for adapter contract tests.
- Avoid over-mocking core behavior that can be validated with in-memory implementations.

## Isolation rules

- Fresh temporary directories/datastores per test.
- No shared mutable fixtures between test cases.
