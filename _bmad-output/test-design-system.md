---
title: "System-Level Test Design"
description: "High-level test strategy, architecture assessment, and design decisions for Lithos"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-01-23"
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

## Architecturally Significant Requirements (ASRs)

| ASR ID | Requirement                       | Category | Probability | Impact | Score | Mitigation Strategy                                            |
| ------ | --------------------------------- | -------- | ----------- | ------ | ----- | -------------------------------------------------------------- |
| ASR-01 | Template execution < 500ms        | PERF     | 2           | 3      | 6     | Criterion benchmarks for MiniJinja rendering and rkyv lookups. |
| ASR-02 | Vault indexing < 2s (1000+ files) | PERF     | 3           | 3      | 9     | Mandatory 10k-note vault benchmarks; parallel indexing tests.  |
| ASR-03 | Memory usage < 500MB              | PERF     | 2           | 2      | 4     | Memory profiling in CI; bounded MPSC channels for indexing.    |
| ASR-04 | Vault consistency/link resolution | DATA     | 2           | 3      | 6     | Property-based testing (proptest) for graph consistency.       |
| ASR-05 | Configuration encryption          | SEC      | 1           | 3      | 3     | Specialized security tests for SPI crypto adapters.            |

## Test Levels Strategy

- **Unit: 70%**
  - Focus: Pure business logic in `crates/domain`, template parsing, schema validation rules, and CQRS command/query logic.
  - Rationale: High cyclomatic complexity in schema inheritance and template composition requires granular, fast feedback.
  - Tools: `mise run test:unit`, `proptest`.

- **Integration: 20%**
  - Focus: `Redb` persistence, `pulldown-cmark` extraction accuracy, and event-bus delivery reliability across planes.
  - Rationale: Validates the hexagonal boundary contracts and asynchronous coordination between the Indexer Actor and Query Service.
  - Tools: `cargo nextest` (orchestrated via `mise`), `mockall`.

- **E2E: 10%**
  - Focus: CLI command structure, interactive prompts, and full user journeys (e.g., `lithos new` to note creation).
  - Rationale: Ensures the "parsimonious setup" and guided UX meet success metrics without over-testing implementation details.
  - Tools: `assert_cmd`, `predicates`, `TestVault`.

## Quality Gates & "Definition of Done"

A test is considered "Production Ready" only if it meets these five criteria:

1. **Deterministic**: 0% flakiness. No `sleep()` calls; use proper synchronization or virtual clock (`time_test!`).
2. **Isolated**: Must not depend on or affect other tests. Use `TestVault` or `IsolatedTestContext` for unique environments.
3. **Explicit**: Assertions must be visible in the test body. Avoid hidden "pass-through" assertions in helpers.
4. **Fast**: Unit tests < 10ms, Integration < 100ms, E2E < 2s.
5. **Self-Cleaning**: Must clean up all temporary files or database entries upon completion (ensured by `RAII` patterns).

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
  - Architecture tests (`tests/arch/`) to enforce hexagonal boundaries.

## Test Data Strategy

Lithos uses a tiered approach to test data to ensure reproducibility and scale:

1. **Inline Fixtures**: For unit tests, data is defined directly in the test body or a local `setup` function.
2. **Deterministic Randomness**: Using `proptest` with fixed seeds for complex edge-case discovery.
3. **Reference Vaults**: Located in `docs/refs/obsidian/`, these provide a standard "Golden Set" of markdown files for integration and E2E testing.
4. **Isolated Contexts**: Every test that touches the filesystem MUST use `IsolatedTestContext` to prevent cross-test interference.

## Test Environment Requirements

- **Local:** `mise` managed toolchain (Rust 1.92+, pre-commit hooks).
- **CI:** GitHub Actions with multi-OS support (macOS/Linux) and artifact preservation for benchmark results.
- **Data:** Sharded sample vaults (docs/refs/obsidian/) for scaling tests.

## Testability Concerns (if any)

- **Concern:** `rkyv` zero-copy buffers require careful lifetime management in the adapter layer. If leaked into the domain, it may complicate unit testing.
- **Mitigation:** Ensure `rkyv` types are mapped to ergonomic domain entities in `adapters/spi/storage` before passing to the `app` layer.

## Current Implementation Status

- ✅ Hexagonal testing architecture (Unit/Integration/E2E split)
- ✅ Test utilities crate with async helpers and fixtures
- ✅ CI/CD pipeline with coverage reporting and quality gates
- ✅ Domain purity testing and architectural boundary enforcement
- 🔄 Performance benchmarking framework (in progress)
- 🔄 Property-based testing expansion (in progress)

## Implementation Details

For detailed implementation guides, patterns, and examples, see [Lithos Test Guide](../docs/test_guide.md). Key implementation achievements include:

### Test Infrastructure
- **lithos-test-utils crate**: Centralized testing utilities with async helpers, time control, and fixture management
- **Mise orchestration**: All testing workflows managed through `mise run` commands with proper environment setup
- **Quality gates**: Automated linting, formatting, and coverage checks in CI/CD

### Testing Patterns
- **Hexagonal testing**: Separate test strategies for domain, application, infrastructure, and E2E layers
- **Async testing**: Multi-threaded runtime with deterministic time control and proper error handling
- **Fixture management**: Isolated test contexts and vault simulation for reliable testing

### Coverage Goals
- **Unit tests**: 70% coverage focusing on business logic and edge cases
- **Integration tests**: 20% coverage for component interaction and contracts
- **E2E tests**: 10% coverage for user journey validation
- **Overall target**: 80%+ code coverage with performance benchmarking
