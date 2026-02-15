# TEA Knowledge: Testing Behavioral Rules & Policies

## CONTEXT

- **Applies to**: Test authoring and code review standards
- **Purpose**: Ensure consistency, maintainability, and clarity across the test suite

## BEHAVIORAL RULES

1.  **One Behavior per Test**: Describe exactly one thing the unit does. If you find yourself using `and` in a test name, split it.
2.  **Single Assertion Preference**: Use one logical assertion per test to make failures easy to diagnose.
3.  **No `test_` Prefix**: The attribute `#[test]` is sufficient. Prefixes are redundant.
4.  **Verb-First Naming**: Names should describe actions (e.g., `returns_ok_when_valid`).

## SNAPSHOT TESTING POLICY

**Lithos currently does NOT use snapshot testing.** We prefer explicit assertions for all test verification.

### Why no snapshots?

1.  **Explicit Assertions**: `assert_eq!` and `matches!` are clearer and more maintainable.
2.  **Debugging**: Explicit assertions show exactly what failed and why.
3.  **Review Friction**: Snapshot diffs in PRs require careful review to catch regressions.
4.  **Determinism**: Snapshot tests can hide timing issues and non-deterministic output.

### Future Exceptions

If snapshot testing is added in the future, it must be restricted to structural correctness (CLI output, complex JSON) and must always redact unstable fields (UUIDs, timestamps).

## ADVANCED VERIFICATION

- **Observability**: Use `tracing-test` to verify emitted spans and events.
- **Domain Purity**: Programmatic enforcement ensures `lithos-core` domain contexts remain free of I/O dependencies.

## VALIDATION CHECKLIST

- [ ] Tests describe exactly one behavior
- [ ] No `test_` prefix in names
- [ ] No snapshot testing crates used
- [ ] Explicit assertions for all outcomes

## RELATED MODULES

- See `naming.md` for naming formulas
- See `assertions.md` for assertion patterns
- See `test-unit.md` for unit testing rules
