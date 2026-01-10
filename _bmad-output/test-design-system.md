# System-Level Test Design

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

| ASR ID | Requirement | Category | Probability | Impact | Score | Mitigation Strategy |
| ------ | ----------- | -------- | ----------- | ------ | ----- | ------------------- |
| ASR-01 | Template execution < 500ms | PERF | 2 | 3 | 6 | Criterion benchmarks for MiniJinja rendering and rkyv lookups. |
| ASR-02 | Vault indexing < 2s (1000+ files) | PERF | 3 | 3 | 9 | Mandatory 10k-note vault benchmarks; parallel indexing tests. |
| ASR-03 | Memory usage < 500MB | PERF | 2 | 2 | 4 | Memory profiling in CI; bounded MPSC channels for indexing. |
| ASR-04 | Vault consistency/link resolution | DATA | 2 | 3 | 6 | Property-based testing (proptest) for graph consistency. |
| ASR-05 | Configuration encryption | SEC | 1 | 3 | 3 | Specialized security tests for SPI crypto adapters. |

## Test Levels Strategy

- **Unit: 70%**
  - Focus: Pure business logic in `crates/domain`, template parsing, schema validation rules, and CQRS command/query logic.
  - Rationale: High cyclomatic complexity in schema inheritance and template composition requires granular, fast feedback.

- **Integration: 20%**
  - Focus: `Redb` persistence, `pulldown-cmark` extraction accuracy, and event-bus delivery reliability across planes.
  - Rationale: Validates the hexagonal boundary contracts and asynchronous coordination between the Indexer Actor and Query Service.

- **E2E: 10%**
  - Focus: CLI command structure, interactive prompts, and full user journeys (e.g., `lithos new` to note creation).
  - Rationale: Ensures the "parsimonious setup" and guided UX meet success metrics without over-testing implementation details.

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

## Test Environment Requirements

- **Local:** `mise` managed toolchain (Rust 1.92+, pre-commit hooks).
- **CI:** GitHub Actions with multi-OS support (macOS/Linux) and artifact preservation for benchmark results.
- **Data:** Sharded sample vaults (docs/refs/obsidian/) for scaling tests.

## Testability Concerns (if any)

- **Concern:** `rkyv` zero-copy buffers require careful lifetime management in the adapter layer. If leaked into the domain, it may complicate unit testing.
- **Mitigation:** Ensure `rkyv` types are mapped to ergonomic domain entities in `adapters/spi/storage` before passing to the `app` layer.

## Recommendations for Sprint 0

1. Initialize `crates/domain` with pure tests and NO external dependencies.
2. Setup `mise` tasks for `test`, `coverage`, and `bench` immediately.
3. Implement `MockStoragePort` and `MockVaultPort` early to unblock `app` layer development.
4. Establish the `tests/arch` suite to prevent `app` -> `adapters` dependency leakage.
