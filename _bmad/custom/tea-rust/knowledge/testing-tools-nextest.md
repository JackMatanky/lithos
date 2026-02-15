# TEA Knowledge: Nextest Configuration

## CONTEXT

- **Tool**: `cargo-nextest` - Primary test runner for Lithos
- **Purpose**: Fast, parallel test execution with superior isolation
- **Configuration**: `.config/nextest.toml`

## DECISION TREE: When to Use Profiles

```
What environment are you running in?
├── Local development (default)?
│   └── → Use default profile (fast feedback, fail-fast)
│
├── CI/CD pipeline?
│   └── → Use ci profile (all tests, JUnit output, retries)
│
├── Investigating flaky tests?
│   └── → Use stress profile (run repeatedly, single-threaded)
│
├── Quick feedback needed?
│   └── → Use fast profile (skip slow tests)
│
└── Running specific test types?
    └── → Use filters or test groups
```

## VALIDATION CHECKLIST

### Configuration File

- [ ] File exists at `.config/nextest.toml`
- [ ] Has `profile.default` section
- [ ] Has `profile.ci` section for CI
- [ ] Test groups defined for serial tests
- [ ] Timeout overrides for slow tests

### Profile Settings

- [ ] `test-threads` set appropriately (num-cpus for default)
- [ ] `fail-fast` true for local, false for CI
- [ ] `retries` configured for flaky test detection
- [ ] `slow-timeout` set per environment

### Test Groups

- [ ] Serial tests identified and marked
- [ ] Database tests limited (`max-threads`)
- [ ] Network tests have longer timeouts
- [ ] Platform-specific overrides (macOS, Windows)

### CI Integration

- [ ] JUnit XML output configured
- [ ] Test results archived as artifacts
- [ ] Flaky test detection enabled
- [ ] Success output stored for analysis

## ANTI-PATTERNS (FLAG THESE)

### Configuration Issues

- ❌ **No nextest.toml** → Must have configuration
- ❌ **Default profile only** → Need CI profile
- ❌ **No test groups** → Tests that need serial execution may fail
- ❌ **No timeout overrides** → Slow tests hang CI

### CI Issues

- ❌ **No JUnit output** → Can't integrate with CI dashboards
- ❌ **No retries** → Flaky tests fail builds
- ❌ **fail-fast in CI** → Miss multiple failures

### Test Organization

- ❌ **Tests without serial annotation** → Should use test groups
- ❌ **Database tests running parallel unbounded** → Limit connections
- ❌ **No platform-specific overrides** → macOS tests slower

## CORRECT EXAMPLES

### Basic Configuration

```toml
# .config/nextest.toml

[profile.default]
test-threads = "num-cpus"
fail-fast = true
slow-timeout = "60s"
retries = 2
failure-output = "immediate"

[profile.ci]
fail-fast = false
retries = { backoff = "exponential", count = 3, delay = "1s" }
slow-timeout = "120s"
success-output = "final"
junit = { path = "junit.xml" }
```

### Test Groups

```toml
[test-groups]
serial = { max-threads = 1 }
db = { max-threads = 4 }
network = { max-threads = 8 }

[[profile.default.overrides]]
filter = 'test(/\btest_db_/)'
test-group = "db"

[[profile.default.overrides]]
filter = 'test(/\btest_serial_/)'
test-group = "serial"

[[profile.default.overrides]]
filter = 'test(/\btest_network_/)'
test-group = "network"
slow-timeout = "300s"
retries = 5
```

### Platform-Specific Overrides

```toml
[[profile.ci.overrides]]
platform = 'cfg(target_os = "macos")'
filter = 'test(/\btest_fs_/)'
slow-timeout = "180s"

[[profile.ci.overrides]]
platform = 'cfg(target_os = "windows")'
filter = 'test(/\btest_path_/)'
slow-timeout = "120s"
```

### Timeout with Termination

```toml
[[profile.default.overrides]]
filter = 'test(/\btest_hanging_/)'
slow-timeout = { period = "30s", terminate-after = 2 }
# Tests taking >30s are marked slow
# Tests taking >60s are terminated
```

## USAGE COMMANDS

### Running Tests

```bash
# Default profile (local development)
mise run test:unit
# or
cargo nextest run

# CI profile
 cargo nextest run --profile ci

# Stress testing (find flaky tests)
cargo nextest run --profile stress --test-threads 1

# Fast profile (skip slow tests)
cargo nextest run --profile fast

# Specific test group
cargo nextest run --test-group db
```

### Filtering Tests

```bash
# Run tests matching pattern
cargo nextest run -- test_foo

# Run tests NOT matching pattern
cargo nextest run -- --exclude test_slow

# Complex filter expression
cargo nextest run -E 'test(/\bparse/) & !test(/\bslow/)'

# Run tests in specific crate
cargo nextest run -p lithos-core
```

## COMPARISON: nextest vs cargo test

| Feature            | cargo test    | nextest           |
| ------------------ | ------------- | ----------------- |
| Parallelism        | Thread-based  | Process-based     |
| Isolation          | Shared memory | Process isolation |
| Flaky detection    | No            | Yes               |
| Timeout per test   | No            | Yes               |
| Retry failed       | No            | Yes               |
| JUnit XML          | No            | Yes               |
| Stress testing     | No            | Yes               |
| Filter expressions | Simple        | Rich              |
| Doctests           | Yes           | No\*              |

\*Doctests must be run with `cargo test --doc`

## MISE INTEGRATION

### Available Commands

```bash
mise run test:unit          # Run all unit tests
mise run test:unit:core     # Core crate only
mise run test:unit:cli      # CLI crate only
mise run test:integration   # Integration tests
mise run test:e2e          # E2E tests
mise run test:burn-in      # Stress testing
mise run test:changed      # Tests for changed crates
mise run test:watch        # Watch mode
```

### Aliases

```bash
mise run tu    # test:unit
mise run tucore # test:unit:core
mise run ti    # test:integration
mise run te    # test:e2e
mise run tb    # test:burn-in
mise run tw    # test:watch
```

## QUICK REFERENCE

| Task                | Command                                  |
| ------------------- | ---------------------------------------- |
| Run all tests       | `cargo nextest run`                      |
| Run with CI profile | `cargo nextest run --profile ci`         |
| Run specific test   | `cargo nextest run -- test_name`         |
| Filter tests        | `cargo nextest run -E 'test(/pattern/)'` |
| Stress test         | `cargo nextest run --profile stress`     |
| List tests          | `cargo nextest list`                     |

## RELATED MODULES

- See `testing-unit.md` for unit testing
- See `testing-integration.md` for integration testing
- See `testing-e2e.md` for E2E testing
- See `testing-anti-patterns.md` for comprehensive anti-patterns
