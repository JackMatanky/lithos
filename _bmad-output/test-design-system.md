---
title: "System-Level Test Design"
description: "High-level test strategy, architecture assessment, and design decisions for Lithos"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-02-08"
section: "Testing & Quality"
---

# System-Level Test Design

This document outlines the high-level test strategy and architectural decisions for the Lithos project. For detailed implementation and usage, see [Lithos Test Developer Guide](./test-developer-guide.md).

## Testability Assessment

- **Controllability: PASS**
  - Hexagonal architecture with strict trait-based ports (`VaultReaderPort`, `StoragePort`, etc.) ensures all I/O and external dependencies are easily mockable using `mockall` or manual test doubles.
  - Dependency Injection via constructor injection in the `lithos` crate allows for full control of the implementation stack during integration tests.
  - `uuid v7` provides deterministic identity generation if seeded, aiding in reproducible test cases.

- **Observability: PASS**
  - `miette` provides high-fidelity error reporting that can be validated in E2E tests.
  - `tracing` integration with structured spans allows for deep inspection of async execution paths.
  - `app/metrics` calculator provides built-in instrumentation for vault-wide state validation.

- **Reliability: PASS**
  - Workspace-based crate separation prevents architectural drift and ensures pure domain logic remains stateless and parallel-safe.
  - Unit of Work pattern in the storage layer enables atomic test setups and clean rollbacks.
  - `rkyv` zero-copy deserialization reduces the risk of memory-related crashes during large vault operations.

## Test Architecture

### Hexagonal Architecture & Testability

Lithos follows a **hexagonal (ports-and-adapters) architecture** that fundamentally shapes our testing strategy:

```
┌─────────────────────────────────────────────────────────────┐
│                      Test Layers                           │
├─────────────────────────────────────────────────────────────┤
│  E2E Tests                                                 │
│  └─> CLI Layer (lithos-cli)                               │
│      └─> Full stack: CLI → App → Domain → Adapters       │
├─────────────────────────────────────────────────────────────┤
│  Integration Tests                                         │
│  ├─> App Layer Tests (port contracts, orchestration)     │
│  └─> Adapter Tests (persistence, external APIs)          │
├─────────────────────────────────────────────────────────────┤
│  Unit Tests                                                │
│  └─> Domain Layer (pure business logic, zero I/O)        │
│      └─> Commands, Queries, Entities, Value Objects      │
└─────────────────────────────────────────────────────────────┘
```

#### Port-Based Testing Patterns

**1. Domain Unit Tests (No Ports)**

- Pure logic with no I/O dependencies
- Fast, deterministic, highly parallel
- Test business invariants, state transitions, validation rules

**2. Port Contract Tests**

- Verify that adapters correctly implement port traits
- Use mock ports for app layer testing
- Use real adapters for integration testing

**3. Adapter Integration Tests**

- Test real implementations (redb, filesystem)
- Use temporary resources (TempDir, in-memory DB)
- Verify side effects and persistence

### CQRS Testing Strategy

Lithos implements **CQRS (Command Query Responsibility Segregation)** with split storage ports:

| Aspect        | Command Testing                         | Query Testing                      |
| ------------- | --------------------------------------- | ---------------------------------- |
| **Focus**     | State mutations, side effects           | Data retrieval, projections        |
| **Port**      | `SchemaCommandPort`, `NoteCommandPort`  | `SchemaQueryPort`, `NoteQueryPort` |
| **Test Type** | Integration (with transaction rollback) | Unit/Integration (read-only)       |
| **Isolation** | Serial or transaction-scoped            | Fully parallel                     |
| **Fixtures**  | Setup → Execute → Assert → Rollback     | Pre-seeded data, read-only         |

**Testing CQRS Commands:**

```rust
// Test the command handler, not the storage directly
#[test]
fn create_schema_command_persists_valid_schema() {
    let mut storage = InMemoryCommandPort::new();
    let cmd = CreateSchemaCommand::new("Task", vec![...]);

    let result = cmd.execute(&mut storage);

    assert!(result.is_ok());
    assert!(storage.contains_schema("Task"));
}
```

**Testing CQRS Queries:**

```rust
// Queries are read-only and highly parallelizable
#[test]
fn find_schema_by_id_returns_correct_schema() {
    let storage = InMemoryQueryPort::with_seed(vec![task_schema()]);
    let query = FindSchemaById::new(task_schema_id());

    let result = query.execute(&storage);

    assert_eq!(result.unwrap().name, "Task");
}
```

### Context Isolation Testing

Per architectural constraints, business contexts (note, schema, template) **must not import each other**. Testing enforces this:

- **Compilation Boundaries**: Each context has independent tests
- **No Cross-Context Fixtures**: Test data remains context-local
- **Integration Tests Verify Boundaries**: Cross-context scenarios tested at app layer only

## Architecturally Significant Requirements (ASRs)

### Performance ASRs

| ASR ID  | Requirement                     | Target         | Category | Probability | Impact | Score | Mitigation Strategy                                           |
| ------- | ------------------------------- | -------------- | -------- | ----------- | ------ | ----- | ------------------------------------------------------------- |
| ASR-P01 | Unit test execution time        | < 10ms median  | PERF     | 2           | 3      | 6     | Fast feedback loops; parallel execution via nextest           |
| ASR-P02 | Integration test execution time | < 100ms median | PERF     | 2           | 3      | 6     | In-memory adapters; TempDir for FS isolation                  |
| ASR-P03 | E2E test execution time         | < 2s median    | PERF     | 2           | 2      | 4     | Minimal CLI invocation; focused user journeys                 |
| ASR-P04 | Full test suite execution       | < 60s          | PERF     | 3           | 3      | 9     | nextest parallelization; workspace-level caching              |
| ASR-P05 | Template execution              | < 500ms        | PERF     | 2           | 3      | 6     | Criterion benchmarks for MiniJinja rendering and rkyv lookups |
| ASR-P06 | Vault indexing (1000+ files)    | < 2s           | PERF     | 3           | 3      | 9     | Mandatory 10k-note vault benchmarks; parallel indexing tests  |
| ASR-P07 | Memory usage during indexing    | < 500MB        | PERF     | 2           | 2      | 4     | Memory profiling in CI; bounded MPSC channels for indexing    |

### Coverage ASRs

| ASR ID  | Requirement               | Target          | Category | Probability | Impact | Score | Mitigation Strategy                            |
| ------- | ------------------------- | --------------- | -------- | ----------- | ------ | ----- | ---------------------------------------------- |
| ASR-C01 | Unit test coverage        | ≥ 70%           | QUALITY  | 2           | 3      | 6     | tarpaulin CI integration; coverage gating      |
| ASR-C02 | Integration test coverage | ≥ 20%           | QUALITY  | 2           | 3      | 6     | Public API testing; port contract verification |
| ASR-C03 | Critical path coverage    | 100%            | QUALITY  | 1           | 3      | 3     | Manual review; mutation testing for hot paths  |
| ASR-C04 | Doc test coverage         | All public APIs | QUALITY  | 2           | 2      | 4     | Living documentation; example-driven design    |

### Tool-Specific ASRs

| ASR ID  | Requirement                      | Tool           | Category        | Probability | Impact | Score | Mitigation Strategy                                       |
| ------- | -------------------------------- | -------------- | --------------- | ----------- | ------ | ----- | --------------------------------------------------------- |
| ASR-T01 | Flaky test detection             | nextest        | RELIABILITY     | 2           | 3      | 6     | Automatic retry configuration; stress testing             |
| ASR-T02 | Performance regression detection | criterion      | PERF            | 2           | 3      | 6     | Baseline tracking; CI comparison                          |
| ASR-T03 | Edge case discovery              | proptest       | QUALITY         | 2           | 3      | 6     | Shrinking for minimal failing cases; deterministic seeds  |
| ASR-T04 | Test parallelization             | nextest        | PERF            | 2           | 2      | 4     | Process-per-test isolation; resource contention detection |
| ASR-T05 | Coverage accuracy                | cargo-llvm-cov | QUALITY         | 2           | 2      | 4     | Line + branch coverage; PR coverage reports               |
| ASR-T06 | Parameterized test execution     | rstest         | MAINTAINABILITY | 2           | 2      | 4     | Named test cases; fixture injection                       |

### Data Integrity ASRs

| ASR ID  | Requirement                       | Category | Probability | Impact | Score | Mitigation Strategy                                         |
| ------- | --------------------------------- | -------- | ----------- | ------ | ----- | ----------------------------------------------------------- |
| ASR-D01 | Vault consistency/link resolution | DATA     | 2           | 3      | 6     | Property-based testing (proptest) for graph consistency     |
| ASR-D02 | Configuration encryption          | SEC      | 1           | 3      | 3     | Specialized security tests for SPI crypto adapters          |
| ASR-D03 | Zero-copy safety                  | DATA     | 2           | 2      | 4     | rkyv validation at trust boundaries; access_unchecked audit |

## Testing Tools Stack

### Core Test Runner: nextest vs cargo test

**Why nextest:**

- **Up to 3x faster**: Process-per-test parallelism vs thread-per-test
- **Superior isolation**: Tests run in separate processes, preventing state leakage
- **Rich filtering**: Expression-based test selection (`test(a) & !test(b)`)
- **Flaky detection**: Automatic detection and retry of flaky tests
- **JUnit output**: Native CI integration with machine-readable results
- **Stress testing**: Built-in loop mode for detecting intermittent failures

**When to use cargo test:**

- Doc tests (nextest doesn't support doctests yet)
- Single test debugging (simpler output)

| Feature            | cargo test    | nextest              |
| ------------------ | ------------- | -------------------- |
| Parallelism        | Thread-based  | Process-based        |
| Isolation          | Shared memory | Process isolation    |
| Flaky detection    | No            | Yes                  |
| Timeout per test   | No            | Yes                  |
| Retry failed       | No            | Yes                  |
| JUnit XML          | No            | Yes                  |
| Stress testing     | No            | Yes                  |
| Filter expressions | Simple        | Rich (boolean ops)   |
| Doctests           | Yes           | No (Rust limitation) |

### Coverage Tools: cargo-llvm-cov vs tarpaulin

**Current Stack: tarpaulin**

- Proven integration with Rust test ecosystem
- HTML and LCOV report generation
- CI-friendly exit codes

**Future Consideration: cargo-llvm-cov**

- More accurate coverage (uses LLVM instrumentation)
- Better support for async code
- Faster execution for large codebases
- Consider migration if coverage accuracy issues arise

### Property-Based Testing: proptest

**Use Cases:**

- Mathematical invariants (serialization round-trips)
- Graph consistency (link resolution)
- Input validation edge cases
- State machine transitions

**Best Practices:**

- Use deterministic seeds for reproducibility
- Leverage shrinking for minimal failing cases
- Combine with example-based tests for clarity

### Parameterized Testing: rstest

**Patterns:**

- **Fixtures**: Shared test setup via function injection
- **Cases**: Table-driven tests with named parameters
- **Values**: Cross-product testing for combinations

```rust
#[rstest]
#[case::empty_string("", 0)]
#[case::single_char("a", 1)]
#[case::multiple_words("hello world", 11)]
fn string_length_matches_expected(#[case] input: &str, #[case] expected: usize) {
    assert_eq!(input.len(), expected);
}
```

### Benchmarking: criterion

**Requirements:**

- Statistical confidence (outlier detection)
- Baseline comparison for regression detection
- HTML reports for trend analysis
- Async support for I/O benchmarks

**Critical Paths Requiring Benchmarks:**

- Vault indexing (ASR-P06)
- Template rendering (ASR-P05)
- rkyv serialization/deserialization
- Markdown parsing

## Test Levels Strategy

- **Unit: 70%**
  - Focus: Pure business logic in `lithos-core/src/`, template parsing, schema validation rules, and CQRS command/query logic.
  - Rationale: High cyclomatic complexity in schema inheritance and template composition requires granular, fast feedback.
  - Tools: `mise run test:unit`, `proptest`, `rstest`.
  - Coverage Target: ≥ 70% line coverage.

- **Integration: 20%**
  - Focus: `Redb` persistence, `pulldown-cmark` extraction accuracy, and event-bus delivery reliability across planes.
  - Rationale: Validates the hexagonal boundary contracts and asynchronous coordination between the Indexer Actor and Query Service.
  - Tools: `cargo nextest` (orchestrated via `mise`), `mockall`, `tempfile`.
  - Coverage Target: ≥ 20% line coverage (public API focus).

- **E2E: 10%**
  - Focus: CLI command structure, interactive prompts, and full user journeys (e.g., `lithos new` to note creation).
  - Rationale: Ensures the "parsimonious setup" and guided UX meet success metrics without over-testing implementation details.
  - Tools: `assert_cmd`, `predicates`, `tempfile`.
  - Coverage Target: Critical user journeys only.

## Test Data Strategy

### Tiered Data Approach

Lithos uses a tiered approach to test data to ensure reproducibility and scale:

1. **Inline Fixtures**: For unit tests, data is defined directly in the test body or a local `setup` function.
2. **Deterministic Randomness**: Using `proptest` with fixed seeds for complex edge-case discovery.
3. **Reference Vaults**: Located in `docs/refs/obsidian/`, these provide a standard "Golden Set" of markdown files for integration and E2E testing.
4. **Isolated Contexts**: Every test that touches the filesystem MUST use `tempfile::TempDir` to prevent cross-test interference.

### rstest Fixtures Pattern

**Basic Fixture:**

```rust
#[fixture]
fn test_schema() -> Schema {
    Schema::builder()
        .name("Task")
        .field(Field::string("title"))
        .build()
        .expect("Valid test schema")
}

#[rstest]
fn schema_validates_correctly(test_schema: Schema) {
    assert!(test_schema.validate().is_ok());
}
```

**Async Fixture Support:**

```rust
#[fixture]
async fn temp_vault() -> TempDir {
    TempDir::new().expect("Create temp directory")
}

#[rstest]
#[tokio::test]
async fn vault_indexes_correctly(#[future] temp_vault: TempDir) {
    let vault = temp_vault.await;
    // Test implementation
}
```

**Fixture Composition:**

```rust
#[fixture]
fn schema_with_notes(test_schema: Schema) -> (Schema, Vec<Note>) {
    let notes = vec![
        Note::new(&test_schema, "Task 1"),
        Note::new(&test_schema, "Task 2"),
    ];
    (test_schema, notes)
}
```

### Property-Based Testing Data Generation

**Custom Strategies:**

```rust
use proptest::prelude::*;

// Generate valid schema names
fn schema_name_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,63}"
}

// Generate valid note paths
fn note_path_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-/]+\\.md"
}

// Composite strategy for complete notes
prop_compose! {
    fn note_strategy()
        (path in note_path_strategy(),
         title in "[^\n]{1,100}") -> Note {
        Note::builder()
            .path(path)
            .title(title)
            .build()
            .unwrap()
    }
}
```

**State Machine Testing:**

```rust
// Test vault operations as state transitions
proptest! {
    #[test]
    fn vault_operations_maintain_consistency(
        ops in prop::collection::vec(vault_operation_strategy(), 1..100)
    ) {
        let mut vault = Vault::empty();
        for op in ops {
            vault.apply(op).unwrap();
            assert!(vault.check_invariants().is_ok());
        }
    }
}
```

### Reference Data Management

**Golden Files:**

- Location: `docs/refs/obsidian/`
- Purpose: Standard test vault for integration/E2E tests
- Version Control: Tracked in git for reproducibility
- Update Process: Manual review required for changes

**Test Data Categories:**
| Category | Location | Use Case |
|----------|----------|----------|
| Minimal valid | Inline fixtures | Unit tests, edge cases |
| Complex valid | `tests/fixtures/` | Integration tests |
| Invalid/malformed | `tests/fixtures/invalid/` | Error handling tests |
| Performance scale | Generated | Benchmarks, load tests |
| Golden reference | `docs/refs/obsidian/` | E2E validation |

## Quality Gates & "Definition of Done"

A test is considered "Production Ready" only if it meets these five criteria:

1. **Deterministic**: 0% flakiness. No `sleep()` calls; use proper synchronization or paused clocks in async tests.
2. **Isolated**: Must not depend on or affect other tests. Use `tempfile::TempDir` for unique environments.
3. **Explicit**: Assertions must be visible in the test body. Avoid hidden "pass-through" assertions in helpers.
4. **Fast**: Unit tests < 10ms, Integration < 100ms, E2E < 2s.
5. **Self-Cleaning**: Must clean up all temporary files or database entries upon completion (ensured by `RAII` patterns).

### Coverage Gates

| Gate                   | Threshold | Enforcement                        |
| ---------------------- | --------- | ---------------------------------- |
| Overall coverage       | ≥ 80%     | CI failure on drop below threshold |
| New code coverage      | ≥ 90%     | PR gate via diff coverage          |
| Critical path coverage | 100%      | Manual review + mutation testing   |
| Unit test coverage     | ≥ 70%     | Workspace-level tracking           |
| Integration coverage   | ≥ 20%     | Module-level tracking              |

**Coverage Reporting:**

- Tool: `tarpaulin` (with `cargo-llvm-cov` under evaluation)
- Format: HTML (local), LCOV (CI integration)
- PR Integration: Coverage diff comments

### Performance Regression Gates

| Metric                  | Baseline           | Threshold | Action      |
| ----------------------- | ------------------ | --------- | ----------- |
| Template render         | Criterion baseline | +10%      | Warning     |
| Vault index (1k notes)  | Criterion baseline | +10%      | Warning     |
| Vault index (10k notes) | Criterion baseline | +5%       | Block merge |
| Test suite duration     | 60s                | +20%      | Investigate |

**Benchmark CI Integration:**

- Criterion baseline stored per-branch
- Performance alerts on regression
- Trend visualization in CI artifacts

### Mutation Testing Considerations

**Purpose:** Verify test suite effectiveness by introducing artificial bugs.

**Current Status:** Under evaluation (not yet integrated)

**Future Integration:**

- Tool: `cargo-mutants` or `necessist`
- Target: Critical business logic modules
- Threshold: 80% mutation score
- Frequency: Weekly CI run (not per-PR due to cost)

## NFR Testing Approach

- **Security:**
  - Automated validation of config encryption/decryption at the SPI layer.
  - Audit log verification via the `AuditSubscriber` to ensure FR40 compliance.
- **Performance:**
  - `criterion` benchmarks integrated into `mise run bench`.
  - Regression testing in CI for indexing speed and query latency.
- **Reliability:**
  - Fault injection in the MPSC data plane to test indexing recovery.
  - "Clean slate protocol" tests to verify recovery from Redb corruption.
- **Maintainability:**
  - 80%+ coverage target enforced by `tarpaulin`.
  - Architecture tests ensuring context isolation boundaries.

## Risk Assessment

### Technical Debt Risks

| Risk ID  | Description                                   | Impact | Likelihood | Mitigation                                                |
| -------- | --------------------------------------------- | ------ | ---------- | --------------------------------------------------------- |
| RISK-T01 | Insufficient test coverage in critical paths  | High   | Medium     | 100% critical path coverage requirement; mutation testing |
| RISK-T02 | Complex test fixtures becoming unmaintainable | Medium | Medium     | Inline fixture policy; rstest for composition             |
| RISK-T03 | Slow test suite discouraging frequent runs    | High   | Low        | nextest parallelization; performance budgets              |
| RISK-T04 | Brittle tests breaking on refactoring         | Medium | Medium     | Test behavior not implementation; hexagonal boundaries    |
| RISK-T05 | Test data drift from production patterns      | Medium | Low        | Reference vault maintenance; proptest for diversity       |

### Flaky Test Mitigation

**Detection:**

- nextest automatic flaky detection
- `mise run test:burn-in` for stress testing
- CI tracking of test failure patterns

**Prevention:**

- No `sleep()` calls; use synchronization primitives
- Deterministic fixtures with fixed seeds
- Process isolation via nextest
- No shared mutable state

**Remediation Process:**

1. Quarantine flaky test (mark with `#[ignore]`)
2. Investigate root cause (timing, shared state, randomness)
3. Fix and verify with stress testing
4. Re-enable test

### Test Maintenance Burden

**Indicators:**

- Test code > 50% of production code
- Tests require frequent updates on refactors
- High cognitive complexity in test modules

**Mitigation Strategies:**

- Behavior-focused testing (not implementation)
- Stable port contracts
- Automated fixture generation
- Regular test suite refactoring

## Test Environment Requirements

- **Local:** `mise` managed toolchain (Rust 1.92+, pre-commit hooks).
- **CI:** GitHub Actions with multi-OS support (macOS/Linux) and artifact preservation for benchmark results.
- **Data:** Sharded sample vaults (docs/refs/obsidian/) for scaling tests.

### CI Pipeline Integration

```yaml
# Simplified CI flow
1. Checkout & mise setup
2. Quality gates (fmt, lint, adr:validate)
3. Unit tests (nextest, parallel)
4. Integration tests (nextest, parallel)
5. Coverage report (tarpaulin)
6. Benchmarks (criterion, compare to baseline)
7. E2E tests (serial, isolated)
```

## Testability Concerns (if any)

- **Concern:** `rkyv` zero-copy buffers require careful lifetime management in the adapter layer. If leaked into the domain, it may complicate unit testing.
- **Mitigation:** Ensure `rkyv` types are mapped to ergonomic domain entities in `adapters/spi/storage` before passing to the `app` layer.

- **Concern:** CQRS split ports may lead to inconsistent test doubles between command and query tests.
- **Mitigation:** Provide unified in-memory storage implementations for testing that implement both port types.

## Current Implementation Status

### Fully Implemented ✅

| Component                      | Status | Tools                   |
| ------------------------------ | ------ | ----------------------- |
| Hexagonal testing architecture | ✅     | nextest, mockall        |
| Unit/Integration/E2E split     | ✅     | Standard Rust + nextest |
| CI/CD pipeline                 | ✅     | GitHub Actions          |
| Coverage reporting             | ✅     | tarpaulin               |
| Performance benchmarking       | ✅     | criterion               |
| Property-based testing         | ✅     | proptest                |
| Parameterized testing          | ✅     | rstest                  |
| Domain purity enforcement      | ✅     | Architecture tests      |
| Temporary file isolation       | ✅     | tempfile                |

### Partially Implemented ⚠️

| Component                        | Status | Gap                              |
| -------------------------------- | ------ | -------------------------------- |
| Coverage gating                  | ⚠️     | No PR-level diff coverage yet    |
| Performance regression detection | ⚠️     | No automated baseline comparison |
| Mutation testing                 | ⚠️     | Under evaluation                 |
| Flaky test tracking              | ⚠️     | Manual process only              |

### Planned for Future 📋

| Component                  | Priority | Timeline |
| -------------------------- | -------- | -------- |
| cargo-llvm-cov evaluation  | Medium   | Q1 2026  |
| Automated mutation testing | Low      | Q2 2026  |
| Load testing framework     | Low      | Q2 2026  |
| Fuzz testing integration   | Low      | Q3 2026  |

## Implementation Details

For detailed implementation guides, patterns, and examples, see [Lithos Test Developer Guide](./test-developer-guide.md). Key implementation achievements include:

### Test Infrastructure

- **Standard Rust testing**: All tests use `#[test]` and `#[cfg(test)]` with no external test frameworks
- **Mise orchestration**: All testing workflows managed through `mise run` commands with proper environment setup
- **Quality gates**: Automated linting, formatting, and coverage checks in CI/CD
- **nextest**: Fast, parallel test runner for improved CI/CD performance
- **criterion**: Performance regression detection with statistical analysis

### Testing Patterns

- **Co-located tests**: Unit tests live in `#[cfg(test)] mod tests` within the same file as implementation
- **Inline fixtures**: Test data and helpers defined locally within test modules
- **Property-based testing**: `proptest` with inline strategies for edge case discovery
- **Clear assertions**: `assert!`, `assert_eq!`, `assert_ne!`, and `matches!` for explicit verification
- **Custom error messages**: All assertions include context for debugging failures

### Testing Best Practices

- **One behavior per test**: Each test verifies a single expected outcome
- **Descriptive names**: Test names follow `action_expected_condition` pattern
- **Explicit failure context**: All assertions include formatted error messages
- **Result-based tests**: Use `Result<(), Error>` return types for complex test logic
- **Module organization**: Group related tests into sub-modules for clarity

### Coverage Goals

- **Unit tests**: 70% coverage focusing on business logic and edge cases
- **Integration tests**: 20% coverage for component interaction and contracts
- **E2E tests**: 10% coverage for user journey validation
- **Overall target**: 80%+ code coverage with performance benchmarking

---

_Last Updated: 2026-02-08_
_For questions or updates, refer to the [Test Developer Guide](./test-developer-guide.md)_
