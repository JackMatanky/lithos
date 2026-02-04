# Lithos Testing Guides

Tactical testing patterns and best practices for Lithos development.

## Overview

These guides provide **tactical implementation patterns** for testing Lithos code. For **architectural decisions** about testing infrastructure, see the relevant ADRs in `docs/adr/`.

## Available Guides

### Core Testing Patterns

| Guide | Purpose | When to Use |
|-------|---------|-------------|
| [CQRS Testing](cqrs.md) | Testing Command/Query separation patterns | Testing domain aggregates, use cases, read models |
| [Event Testing](event.md) | Testing event-driven architecture | Testing event emission, handlers, async flows |
| [Async Testing](async.md) | Testing async Rust code with Tokio | Testing async functions, channels, timeouts |

### Quick Reference

**Testing domain logic?** → See [CQRS Testing](cqrs.md)
- Command handler testing (behavioral)
- Query handler testing (predictable)
- Aggregate state verification
- Validation and error testing

**Testing events?** → See [Event Testing](event.md)
- Event emission verification
- Subscriber testing
- Event payload contracts
- Ordering and timing

**Testing async code?** → See [Async Testing](async.md)
- Tokio runtime setup
- Channel testing
- Timeout handling
- Concurrency patterns

## Testing Stack

Lithos uses the following testing tools:

| Tool | Purpose | Documentation |
|------|---------|---------------|
| `nextest` | Fast test runner with parallel execution | [docs/refs/rust/quality-tooling.md](../refs/rust/quality-tooling.md) |
| `criterion` | Performance benchmarking | [docs/refs/rust/quality-tooling.md](../refs/rust/quality-tooling.md) |
| `tarpaulin` | Code coverage reporting | [docs/refs/rust/quality-tooling.md](../refs/rust/quality-tooling.md) |
| `insta` | Snapshot testing | [docs/refs/rust/quality-tooling.md](../refs/rust/quality-tooling.md) |
| `tokio::test` | Async test runtime | [async.md](async.md) |

## Common Commands

Run tests with `mise` task runner:

```bash
# Run all tests
mise run test

# Run unit tests only
mise run test:unit

# Run specific module tests
mise run test:unit:note
mise run test:unit:schema

# Run integration tests
mise run test:integration

# Run end-to-end tests
mise run test:e2e

# Generate coverage report
mise run test:coverage

# Watch mode (auto-run on changes)
mise run test:watch
```

See `mise.toml` for full list of test tasks.

## Test Organization

Lithos follows Rust standard test organization:

```
lithos-rust/
├── lithos-core/            # Core crate
│   ├── src/                # Unit tests (#[cfg(test)] modules)
│   ├── tests/              # Integration tests (when added)
│   └── benches/            # Performance benchmarks
└── lithos-cli/             # CLI crate (E2E binaries)
```

**Unit tests**: Co-located with code using `#[cfg(test)]` modules
**Integration tests**: In `lithos-core/tests/` for cross-module testing (when added)
**E2E tests**: In `lithos-cli/` for full CLI workflows

## Historical Context

**Note**: Prior to 2026-02-01, testing patterns were documented as ADRs (0008-0011). These have been **deleted and consolidated** into these tactical guides:

- **ADR 0008** (Event Testing) → [event.md](event.md)
- **ADR 0009** (CQRS Testing) → [cqrs.md](cqrs.md)
- **ADR 0010** (Test Utilities) → Retired; use inline fixtures and tempfile patterns
- **ADR 0011** (Integration Testing) → Cross-module patterns in guides

**Rationale**: Testing patterns are tactical implementation details, not architectural decisions. ADRs should document "what" decisions were made (e.g., "use CQRS"), while these guides document "how" to implement them (e.g., "how to test CQRS patterns").

For architectural testing decisions, see:
- **ADR 0008** (Benchmarking Infrastructure) - Performance testing strategy

## Related Documentation

- [Test Guide (High-Level)](../test_guide.md) - Strategic testing approach
- [Quality Tooling Reference](../refs/rust/quality-tooling.md) - Tool configuration and usage
- [Rust Idioms](../refs/rust/idioms.md) - Rust testing idioms and patterns
- [ADR 0008](../adr/0008-benchmarking-infrastructure.md) - Benchmarking strategy (architectural decision)

## Contributing

When adding new testing patterns:

1. **Is it tactical "how-to" guidance?** → Add to these guides
2. **Is it an architectural decision?** → Create an ADR in `docs/adr/`
3. **Is it tool configuration?** → Update `docs/refs/rust/quality-tooling.md`

Keep guides focused on **practical implementation patterns** with code examples and anti-patterns.
