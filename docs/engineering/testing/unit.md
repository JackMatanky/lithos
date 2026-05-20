---
title: "Unit Testing"
status: "active"
owner: "engineering"
last_updated: "2026-05-20"
scope: "Unit test boundaries and patterns"
---

# Unit Testing

## Scope

- Test pure logic and local invariants.
- Avoid external I/O by default; allow it only when I/O behavior is the unit under test, and keep it deterministic and local.
- Keep tests in the same file/module as implementation.
- Use unit tests to protect behavior contracts and edge cases, not implementation noise.

## Placement

- Place unit tests in `#[cfg(test)] mod tests` within `lithos-core/src/**/*.rs`.
- Unit tests may access private functions/items through module scope.

## Execution

- Use `mise run test:unit` for unit suites and `mise run test` for full verification.
- Run targeted modules while iterating (for example `cargo test -p lithos-core schema::property`).
- Prefer `nextest` output for diagnosis; names should be readable without opening source.

## Suite planning (before writing tests)

- Identify the unit(s) of work in the file.
- Enumerate happy path, boundary conditions, and failure paths.
- Define invariants that must always hold.
- Choose test structure:
  - Simple file: flat tests may be enough.
  - Multi-unit file: prefer submodules by unit of work.
- Decide whether `rstest` or `proptest` adds value for repeated/edge input spaces.

## Naming and structure

- Use descriptive snake_case names that encode behavior and condition.
- Prefer either style used in the codebase today:
  - `returns_error_when_invalid_input`
  - `should_return_error_when_invalid_input`
- Avoid generic names like `test_foo`, `test_1`, or `it_works`.
- Use focused submodules for larger units (for example `mod constructor`, `mod validation`, `mod conversions`, `mod proptests`).

Structure A/B naming rules and module selection are authoritative in [Unit Test Naming](./unit-naming.md).

## Patterns

- `#[cfg(test)] mod tests` with focused submodules for complex units.
- Local fixtures/helpers for readability and isolation.
- Explicit assertions and variant checks (for example with `matches!`).
- Use `rstest` named cases for table-driven tests with clear case names.
- Use `proptest` for invariants and edge-case exploration.

## Recommended suite shape

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;
        // setup helpers only
    }

    mod constructor {
        use super::*;

        #[test]
        fn returns_error_when_input_is_invalid() {}
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_value_when_rule_is_violated() {}
    }

    mod proptests {
        use super::*;
        // property-based suites only
    }
}
```

## Assertion guidance

- Include context in assertion messages.
- Prefer:

```rust
assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
```

- For enum errors, prefer `matches!`-based checks.
- For richer comparisons, prefer `assert_eq!` (or `pretty_assertions::assert_eq!` where already used).
- Prefer returning `Result<(), E>` from tests only when `?` materially improves readability.
- For panic contracts, use `#[should_panic(expected = "...")]` only when panic is intentional API behavior.

## Arrange/Act/Assert discipline

- Arrange: `unwrap`/`expect` is acceptable for test setup.
- Act: capture `Result` values; do not immediately unwrap.
- Assert: verify outcomes explicitly; avoid hidden pass/fail helpers.

## Fixture guidance

- Keep fixtures local to the module under test.
- Prefer small helper functions over complex builders unless complexity is unavoidable.
- Avoid sharing mutable fixture state across tests.
- Keep helper names concrete (`valid_schema`, `temp_vault`) rather than generic (`setup`).

## Determinism and speed

- Keep unit tests fast: most tests should complete in under 10ms, and module suites should usually complete in under 1s unless justified.
- Avoid time and randomness unless seeded/controlled.
- Use per-test isolated fixtures instead of shared mutable global state.
- Keep assertions local and specific so failures point to one behavior.
- Seed randomness and control clock/time inputs when behavior depends on time.

## Coverage intent for unit suites

- Verify domain invariants and validation failures first.
- Cover error variants and boundary values, not only happy paths.
- Add regression tests for every bugfix that changes behavior in this file.
- Use integration/e2e tests for cross-boundary workflows; do not overload unit tests with system behavior.

## Definition of done for a unit test suite

- Test names follow [Unit Test Naming](./unit-naming.md).
- Happy path and key failures are covered.
- Assertions are explicit and diagnostic.
- Tests pass via `mise run test:unit`.
- No unnecessary flakiness sources (time, random, shared mutable state).

## Anti-patterns

- Hidden assertions in helpers.
- Shared mutable state across tests.
- Non-deterministic random/time behavior.
- Unwrap/expect in Act or Assert phases.
- Test names that bundle multiple behaviors with `and`.
- Assertion-only smoke tests that do not verify domain behavior.
