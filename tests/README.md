# Lithos Physical Test Suite

This directory contains the physical infrastructure, utilities, and high-level test suites for the Lithos project.

## Directory Structure

*   **`utils/`**: The `lithos-test-utils` crate. This is the "Core OS" for testing, providing `TestVault`, `IsolatedTestContext`, and CQRS mocking infrastructure.
*   **`macros/`**: The `lithos-test-macros` crate, providing procedural macros like `#[derive(TestFactory)]`.
*   **`suite/`**: High-level test suites that span multiple crates:
    *   **`suite/integration/`**: Cross-module orchestration and port contract validation.
    *   **`suite/e2e/`**: Black-box CLI binary testing using `assert_cmd`.
    *   **`suite/arch/`**: Architecture sentinels (e.g., `purity.rs`) to prevent hexagonal drift.

## Key Infrastructure

### `lithos-test-utils`
Centralized utilities for ensuring test isolation and determinism.
- `fs/`: Vault and temp directory management.
- `cqrs/`: Given-When-Then frameworks and repository mocks.
- `obs/`: Tracing and metrics capture.

### `lithos-test-macros`
Generators for boilerplate reduction.
- `TestFactory`: Generates type-safe builders for domain entities.

## Guides & Strategy

- **Quick Start**: See [docs/test_guide.md](../docs/test_guide.md) for mise commands and naming conventions.
- **Architectural Strategy**: See [_bmad-output/test-design-system.md](../_bmad-output/test-design-system.md) for the 70/20/10 pyramid and ASRs.
- **Tactical Patterns**: See `docs/testing/` for deep-dives into Async, CQRS, and Event-Driven testing.
