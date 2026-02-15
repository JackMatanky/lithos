# TEA Knowledge: CI/CD Integration & Authorized Commands

## CONTEXT

- **Applies to**: Automation pipelines and gate enforcement
- **Entry Points**: All testing tasks MUST be orchestrated via `mise`.

## AUTHORIZED ENTRY POINTS (MISE)

| Command                     | Action                                 | Alias |
| :-------------------------- | :------------------------------------- | :---- |
| `mise run test`             | Run all tests (unit, integration, e2e) | `t`   |
| `mise run test:unit`        | Run all unit tests (nextest)           | `tu`  |
| `mise run test:integration` | Run all integration tests              | `ti`  |
| `mise run test:e2e`         | Run end-to-end tests                   | `te`  |
| `mise run test:coverage`    | Generate coverage report (tarpaulin)   | `tc`  |
| `mise run test:bench`       | Run performance benchmarks             | -     |
| `mise run test:watch`       | Watch mode (automatic re-run)          | `tw`  |
| `mise run test:burn-in`     | Stress test (detect flakes)            | `tb`  |
| `mise run test:changed`     | Test only affected crates              | `tc`  |
| `mise run verify`           | Full quality gate orchestration        | `v`   |

## CI QUALITY GATES

### 1. Test Execution

- **Unit Tests**: Required (nextest)
- **Integration Tests**: Required (nextest)
- **Benchmarks**: Required (criterion)

### 2. Quality Checks

- **Code Coverage**: >= 85% required
- **Linting**: No clippy warnings allowed (-D warnings)
- **Formatting**: Must follow rustfmt standards
- **Performance**: No regressions > 5%

### 3. Artifact Generation

- Test reports (JUnit XML)
- Coverage reports (HTML/LCOV)
- Benchmark result data

## AUTOMATED TESTING PIPELINE (TEMPLATE)

```bash
#!/bin/bash
# Comprehensive testing pipeline template

set -euo pipefail

echo "🧪 Starting comprehensive test suite"
echo "===================================="

# 1. Fast unit tests
echo "Running unit tests..."
cargo test --lib

# 2. Integration tests
echo "Running integration tests..."
cargo test --test '*'

# 3. Coverage analysis
echo "Generating coverage reports..."
cargo tarpaulin --out html

# 4. Benchmarks
echo "Running benchmarks..."
cargo bench

# 5. Quality checks
echo "Running quality checks..."
cargo clippy -- -D warnings
cargo fmt -- --check

echo "✅ All tests passed successfully!"
```

## RELATED MODULES

- See `quality-gates.md` for specific thresholds
- See `tools-nextest.md` for runner configuration
- See `linting.md` for clippy suppression policies
