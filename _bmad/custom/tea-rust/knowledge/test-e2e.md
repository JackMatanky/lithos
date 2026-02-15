# TEA Knowledge: Rust E2E Testing

## CONTEXT

- **Applies to**: End-to-end CLI testing in `lithos-cli/`
- **Purpose**: Testing complete user journeys and CLI behavior
- **Tools**: `assert_cmd`, `predicates`, `tempfile`
- **Scope**: Full stack from CLI → App → Domain → Adapters

## DECISION TREE: E2E vs Integration vs Unit

```
Does the test verify...
├── Complete user journey (CLI command to result)?
│   └── YES → E2E test in lithos-cli/
│
├── CLI argument parsing and validation?
│   └── YES → E2E test
│
├── Interactive prompts or TUI behavior?
│   └── YES → E2E test
│
├── Exit codes and output formatting?
│   └── YES → E2E test
│
├── Internal business logic?
│   └── YES → Unit test
│
└── Port/adapter implementations?
    └── YES → Integration test
```

## VALIDATION CHECKLIST

### Test Structure

- [ ] Test is in `lithos-cli/tests/` or `lithos-cli/src/main.rs` doc tests
- [ ] Uses `assert_cmd::Command` to invoke the binary
- [ ] Tests complete user flows (not implementation details)
- [ ] Focus on happy paths and critical error paths only

### CLI Testing

- [ ] Tests both success and failure exit codes
- [ ] Verifies stdout/stderr output content
- [ ] Tests with various argument combinations
- [ ] Uses `tempfile::TempDir` for filesystem isolation

### Isolation

- [ ] Each test has its own temp directory
- [ ] No dependence on existing files or environment
- [ ] Clean state before each test (fresh temp dir)

### Performance

- [ ] E2E tests complete in < 2s median
- [ ] Minimal CLI invocations (batch operations where possible)
- [ ] Focus on critical user journeys only (10% of test suite)

## ANTI-PATTERNS (FLAG THESE)

### Structure Issues

- ❌ **E2E test in `lithos-core/`** → Move to `lithos-cli/`
- ❌ **Testing implementation details** → Test behavior, not internals
- ❌ **Testing every edge case via CLI** → Use unit tests for edge cases
- ❌ **Complex setup in E2E tests** → Simplify or use integration tests

### CLI Issues

- ❌ **Not checking exit codes** → Always assert on success/failure
- ❌ **Brittle output assertions** → Use predicates, not exact string matching
- ❌ **Testing help text** → Only if it's part of UX requirements
- ❌ **Slow tests (> 2s)** → Optimize or move to integration layer

### Isolation Issues

- ❌ **Tests sharing temp directories** → Fresh `TempDir` per test
- ❌ **Tests depending on existing vault** → Create test vault in temp dir
- ❌ **Environment-dependent tests** → Mock environment variables

## CORRECT EXAMPLES

### Basic CLI Test

```rust
// lithos-cli/tests/cli_test.rs
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn cli_shows_help() {
    let mut cmd = Command::cargo_bin("lithos").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("USAGE"));
}

#[test]
fn new_command_creates_note() {
    // Arrange
    let temp = TempDir::new().unwrap();
    let vault_path = temp.path().join("vault");
    fs::create_dir(&vault_path).unwrap();

    // Act
    let mut cmd = Command::cargo_bin("lithos").unwrap();
    cmd.current_dir(&vault_path)
        .arg("new")
        .arg("hello.md")
        .arg("--title")
        .arg("Hello World");

    // Assert
    cmd.assert().success();
    assert!(vault_path.join("hello.md").exists());
}
```

### Testing Error Conditions

```rust
#[test]
fn new_command_fails_when_vault_not_initialized() {
    let temp = TempDir::new().unwrap();
    let not_a_vault = temp.path().join("not_a_vault");
    fs::create_dir(&not_a_vault).unwrap();

    let mut cmd = Command::cargo_bin("lithos").unwrap();
    cmd.current_dir(&not_a_vault)
        .arg("new")
        .arg("test.md");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a vault"));
}
```

### Complex User Journey

```rust
#[test]
fn full_workflow_create_index_and_search() {
    let temp = TempDir::new().unwrap();
    let vault = temp.path().join("vault");

    // Initialize vault
    let mut cmd = Command::cargo_bin("lithos").unwrap();
    cmd.current_dir(&vault.parent().unwrap())
        .arg("init")
        .arg(&vault);
    cmd.assert().success();

    // Create note
    let mut cmd = Command::cargo_bin("lithos").unwrap();
    cmd.current_dir(&vault)
        .arg("new")
        .arg("project/ideas.md");
    cmd.assert().success();

    // Index vault
    let mut cmd = Command::cargo_bin("lithos").unwrap();
    cmd.current_dir(&vault).arg("index");
    cmd.assert().success();

    // Search
    let mut cmd = Command::cargo_bin("lithos").unwrap();
    cmd.current_dir(&vault)
        .arg("search")
        .arg("ideas");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ideas.md"));
}
```

### Testing Output Formats

```rust
#[test]
fn list_command_outputs_json_when_requested() {
    let temp = setup_test_vault();

    let mut cmd = Command::cargo_bin("lithos").unwrap();
    cmd.current_dir(&temp)
        .arg("list")
        .arg("--format")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("}"));
}
```

## E2E TEST SCOPE

### DO Test (Critical User Journeys)

- Vault initialization (`lithos init`)
- Note creation (`lithos new`)
- Vault indexing (`lithos index`)
- Basic search (`lithos search`)
- CLI help and version
- Configuration handling

### DON'T Test (Use Unit/Integration Instead)

- Edge case validation (unit test)
- Schema inheritance logic (unit test)
- Port implementation details (integration test)
- Internal error handling (unit test)
- Parser/tokenizer behavior (unit test)

## QUICK REFERENCE

| Task            | Pattern                                    |
| --------------- | ------------------------------------------ |
| Run E2E tests   | `mise run test:e2e`                        |
| Invoke CLI      | `Command::cargo_bin("lithos")`             |
| Assert success  | `.assert().success()`                      |
| Assert failure  | `.assert().failure()`                      |
| Check output    | `.stdout(predicate::str::contains("..."))` |
| Temp directory  | `TempDir::new().unwrap()`                  |
| Set working dir | `.current_dir(&path)`                      |

## RELATED MODULES

- See `test-unit.md` for unit testing
- See `test-integration.md` for integration testing
- See `assertions.md` for assertion patterns
