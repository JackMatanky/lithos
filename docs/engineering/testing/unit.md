---
title: "Unit Testing"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
scope: "Unit test boundaries and patterns"
---

# Unit Testing

## Scope

- Test pure logic and local invariants.
- Prefer no external I/O.
- Keep tests in the same file/module as implementation.

## Placement

- Place unit tests in `#[cfg(test)] mod tests` within `lithos-core/src/**/*.rs`.
- Unit tests may access private functions/items through module scope.

## Naming and structure

- Use verb-first names such as `returns_error_when_invalid_input`.
- Avoid generic names like `test_foo`.
- Use submodules for complex units (for example `mod validation`, `mod conversions`).

## Patterns

- `#[cfg(test)] mod tests` with focused submodules for complex units.
- Local fixtures/helpers for readability and isolation.
- Explicit assertions and variant checks (for example with `matches!`).

## Assertion guidance

- Include context in assertion messages.
- Prefer:

```rust
assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
```

- For enum errors, prefer `matches!`-based checks.

## Arrange/Act/Assert discipline

- Arrange: `unwrap`/`expect` is acceptable for test setup.
- Act: capture `Result` values; do not immediately unwrap.
- Assert: verify outcomes explicitly; avoid hidden pass/fail helpers.

## Anti-patterns

- Hidden assertions in helpers.
- Shared mutable state across tests.
- Non-deterministic random/time behavior.
