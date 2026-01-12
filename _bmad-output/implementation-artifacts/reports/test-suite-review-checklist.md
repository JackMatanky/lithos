# Test Suite Review Checklist

This checklist provides criteria for auditing the test suite to ensure efficiency, clarity, and signal quality, aligned with Rust testing best practices.

## 1. Naming Rules
Tests must have descriptive, behavior-first names that read like sentences.

- **DO**: `maintains_event_bus_api_contract_across_boundaries`
- **DO**: `fails_to_load_vault_with_invalid_permissions`
- **DON'T**: `test_event_bus`
- **DON'T**: `fix_issue_123`
- **DON'T**: `check_everything`

**Rule**: Use snake_case, start with a verb, and describe the expected outcome and/or the condition.

## 2. Test Description Template
For non-obvious tests or modules, include a brief doc comment or intent comment.

```rust
/// [Intent]: Describes what this test proves.
/// [Context]: Any necessary setup or state information.
/// [Rationale]: (Mandatory for ignore/only/needs flags) Why is this test gated?
```

## 3. Minimal Test Content
Only include the code strictly necessary to prove the behavior.

- Avoid unrelated errors or complex setup that doesn't contribute to the assertion.
- Use helpers or fixtures to hide boilerplate.
- Ensure the failure message is clear and diagnostic.

## 4. Scope and Placement
- **Unit Tests**: Colocated in module with `#[cfg(test)]`. Focused on implementation details and private logic.
- **Integration Tests**: Located in `tests/` directory. Validate public API behavior and cross-module interactions.
- **Doc Tests**: Used for public API examples. Keep them minimal and focused on documentation accuracy.
- **Benchmarks**: Located in `benches/`. Isolated from functional tests to prevent interference.

## 5. Assertion and Signal Quality
- **One Behavior Per Test**: A test should prove exactly one thing.
- **Single Primary Assertion**: Prefer one primary `assert!` or `assert_eq!` per test. Multiple assertions are allowed if they are part of the same logical check.
- **No Vanity Coverage**: Coverage must be tied to defect prevention or risk.
- **Descriptive Table Labels**: If using table-driven tests, each case must have a label describing the scenario.

## 6. Determinism and Flakiness
- **Fixed Data**: Use fixed UUIDs, timestamps, or seeds.
- **Redactions**: Redact unstable fields (e.g., current time, random strings) in snapshot tests.
- **Async Stability**: Use `tokio::time::pause` for time-based tests. Use timeouts to prevent infinite hangs.
- **Mocking**: Mock external I/O and services for stability.

## 7. Snapshot Testing (Insta)
- **Small Payloads**: Avoid large blobs; focus on the data under test.
- **Named Snapshots**: Use descriptive names for snapshot files.
- **No Primitives**: Use `assert_eq!` for simple values instead of snapshots.
- **Redaction**: Mandatory for UUIDs and timestamps.
