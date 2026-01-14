# Lithos Test Suite

This directory contains the automated test suite for Lithos, organized by the Hexagonal Architecture testing pyramid.

## Testing Architecture

We follow a tiered testing strategy to ensure reliability, performance, and correctness across all layers of the system.

### 1. Unit Tests (Domain & Logic)
- **Location**: Inline in source files (`#[cfg(test)]`) or in `crates/domain`.
- **Focus**: Pure business logic, value object validation, and state transitions.
- **Tools**: `cargo test`, `proptest` (property-based testing).
- **Rules**: Zero external dependencies, zero I/O, extremely fast.

### 2. Integration Tests (Application & Adapters)
- **Location**: `tests/suite/integration/` and crate-level `tests/` directories.
- **Focus**: Port/Adapter contracts, CQRS command/query flows, and component interactions.
- **Tools**: `nextest`, `mockall` (for SPI port mocking), `lithos-test-utils`.
- **Rules**: Use mocks for external systems (Storage, Network, FS) unless specifically testing an adapter implementation.

### 3. End-to-End (E2E) Tests (CLI & Workflows)
- **Location**: `tests/suite/e2e/`.
- **Focus**: Complete user journeys and CLI binary behavior.
- **Tools**: `assert_cmd`, `predicates`, `tempfile`.
- **Rules**: Black-box testing against the compiled binary. Every test runs in an isolated temporary vault.

### 4. Performance Benchmarks (NFRs)
- **Location**: `benches/`.
- **Focus**: Indexing speed, rendering latency, and memory usage.
- **Tools**: `criterion` (benchmarking), `dhat` (heap profiling).
- **Rules**: Mandatory for any changes affecting the storage or template engines.

## Quality Gates & "Definition of Done"

A test is considered "Production Ready" only if it meets these criteria:

1. **Deterministic**: Must have 0% flakiness. No `sleep()` calls; use proper synchronization.
2. **Isolated**: Must not depend on or affect other tests. E2E tests must use unique temporary directories.
3. **Explicit**: Assertions must be visible in the test body. Avoid "hidden" assertions in helpers.
4. **Fast**: Unit tests < 10ms, Integration < 100ms, E2E < 2s.
5. **Self-Cleaning**: Must clean up all temporary files or database entries upon completion.

## Running Tests

We recommend using `cargo-nextest` for faster, parallel test execution.

```bash
# Run all tests
mise run test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo nextest run --test suite

# Run benchmarks
mise run bench
```

## Best Practices

- **Use `data-testid`**: For any UI or CLI output markers that tests depend on.
- **Favor `proptest`**: For complex logic where edge cases are hard to find manually.
- **Mock at Boundaries**: Use `automock` on traits in `crates/domain/src/ports`.
- **Trace on Failure**: CI is configured to retain traces and logs only for failed tests.

## Knowledge Base

Refer to the Test Engineering Architect (TEA) fragments in `_bmad/bmm/testarch/knowledge/` for deep-dives into specific patterns:
- `fixture-architecture.md`: Composable setup patterns.
- `data-factories.md`: Efficient test data generation.
- `test-quality.md`: Detailed quality standards.
