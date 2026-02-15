# TEA Knowledge: CI/CD Integration

## CONTEXT

- **Applies to**: Automation pipelines and gate enforcement
- **Purpose**: Ensure 100% green state for every commit
- **Tools**: `cargo nextest`, `cargo tarpaulin`, `cargo clippy`, `cargo fmt`

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
