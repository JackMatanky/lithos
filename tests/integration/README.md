# Integration Tests

## Purpose

Integration tests verify that multiple components work together correctly with real implementations (no mocks for domain services).

## Quick Start

```bash
# Run all integration tests
go test ./tests/integration -v

# Run specific test suite
go test ./tests/integration -run SchemaLookup -v

# Run with race detector
go test -race ./tests/integration

# Run in parallel
go test ./tests/integration -parallel 4
```

## Test Suites

### Schema-Driven Lookup Tests

**File:** `schema_lookup_test.go`

**Purpose:** Verify schema-driven template lookups work end-to-end

**What it tests:**
- Template helper functions (lookup, query, fileClass)
- QueryService integration with TemplateEngine
- FileSpec validation with real schemas
- Note creation workflow (NewNote command)
- Error handling and graceful degradation

**Run tests:**
```bash
# All schema lookup tests
go test ./tests/integration -run SchemaLookup -v

# Specific test
go test ./tests/integration -run SchemaLookup/LookupHelper -v
```

### Template Engine Tests

**File:** `template_engine_test.go`

**Purpose:** Verify template rendering with real services

**What it tests:**
- Template loading and parsing
- Function registration
- Context injection
- Error handling

### Event-Driven Tests

**File:** `event_driven_test.go`

**Purpose:** Verify event publishing and subscription

**What it tests:**
- Event bus integration
- Async event handling
- Event ordering guarantees

## Fixture Organization

Integration tests use fixtures from `testdata/`:

```
testdata/
├── schemas/          # Test schemas
│   ├── valid/       # Valid schema examples
│   └── properties/  # Property bank
├── notes/           # Test vault notes
│   ├── contacts/   # Contact notes
│   └── projects/   # Project notes
├── templates/       # Test templates
└── golden/          # Expected outputs
```

## Golden File Testing

### What are Golden Files?

Golden files store expected test outputs for regression testing. Tests compare actual output against golden files to detect unintended changes.

### Using Golden Files

**Normal testing:**
```bash
# Tests compare output to golden files
go test ./tests/integration -run SchemaLookup
```

**Regenerating golden files:**
```bash
# Update golden files when behavior changes intentionally
UPDATE_GOLDEN=1 go test ./tests/integration -run SchemaLookup -v
```

**Important:** Always review regenerated golden files before committing!

### When to Regenerate

Regenerate golden files when you intentionally change:
- Template rendering behavior
- Error message formatting
- Query result structure
- Validation output

### Review Process

1. Make your code changes
2. Run tests - they will fail with mismatches
3. Review the diff to understand changes
4. If changes are correct: `UPDATE_GOLDEN=1 go test ...`
5. Review regenerated files: `git diff testdata/golden/`
6. Commit golden files with clear explanation

## Test Isolation

### Workspace Isolation

Each test gets an isolated environment:

```go
func TestExample(t *testing.T) {
    ws := testutils.NewWorkspace(t)
    // ws.Root() = unique temp directory
    // Automatic cleanup via t.Cleanup()
}
```

**Benefits:**
- No shared state between tests
- Tests can run in parallel
- Automatic cleanup
- Fresh vault/cache/schemas for each test

### Parallel Execution

Tests are designed to run concurrently:

```bash
# Run 4 tests in parallel
go test ./tests/integration -parallel 4

# Verify thread safety
go test -race ./tests/integration
```

### Temp Directories

Integration tests use temp directories that are:
- Created via `testutils.NewWorkspace(t)`
- Unique per test
- Automatically cleaned up
- Isolated from other tests

## Writing Integration Tests

### Test Structure

```go
func TestMyIntegration(t *testing.T) {
    // Setup: Create isolated environment
    env := setupTestEnvironment(t)

    t.Run("specific scenario", func(t *testing.T) {
        // Arrange: Prepare test data
        // Act: Execute the workflow
        // Assert: Verify outcomes
    })
}
```

### Using Test Utilities

**Create workspace:**
```go
ws := testutils.NewWorkspace(t)
vaultPath := ws.Path("vault")
```

**Copy fixtures:**
```go
testutils.CopyFromTestdata(t, ws, "schemas", "schemas")
testutils.CopyFromTestdata(t, ws, "notes", "vault")
```

**Create config:**
```go
cfg := &domain.Config{
    VaultPath:  ws.Path("vault"),
    SchemasDir: ws.Path("schemas"),
    CacheDir:   ws.Path("cache"),
}
```

### Error Testing

Test both success and failure paths:

```go
// Test success case
result, err := service.Execute(ctx, validInput)
require.NoError(t, err)
assert.Equal(t, expected, result)

// Test error case
_, err = service.Execute(ctx, invalidInput)
require.Error(t, err)
var validationErr *errors.ValidationError
require.ErrorAs(t, err, &validationErr)
assert.NotEmpty(t, validationErr.Remediation())
```

## Running Tests

### Local Development

```bash
# Run all integration tests
go test ./tests/integration

# Run with verbose output
go test ./tests/integration -v

# Run specific test
go test ./tests/integration -run TestSchemaLookup_LookupHelper

# Run with coverage
go test ./tests/integration -cover
```

### Race Detection

Always run tests with race detector before committing:

```bash
go test -race ./tests/integration
```

### Performance

Integration tests should complete quickly:

```bash
# Check execution time
time go test ./tests/integration -run SchemaLookup

# Target: < 10 seconds total
```

## Debugging Tests

### Verbose Output

```bash
# See detailed test output
go test ./tests/integration -v -run SchemaLookup
```

### Specific Test

```bash
# Run only one test
go test ./tests/integration -run TestSchemaLookup_LookupHelper/lookup_helper_renders
```

### Print Debug Info

```go
// In tests, use t.Logf for debug output
t.Logf("Debug: vault path = %s", vaultPath)
t.Logf("Debug: note count = %d", len(notes))
```

### Golden File Diffs

```bash
# See what changed in golden files
git diff testdata/golden/
```

## Common Issues

### Golden File Mismatches

**Problem:** Test fails with "output does not match golden file"

**Solution:**
1. Check if behavior change was intentional
2. Review the diff in test output
3. If correct: `UPDATE_GOLDEN=1 go test ...`
4. Review changes: `git diff testdata/golden/`
5. Commit with explanation

### Race Conditions

**Problem:** Test fails with race detector

**Solution:**
1. Check for shared state between tests
2. Ensure mocks are thread-safe (use sync.Mutex)
3. Verify each test has isolated workspace

### Temp Directory Issues

**Problem:** Tests fail with "file not found"

**Solution:**
1. Verify using `testutils.NewWorkspace(t)`
2. Check fixture copying
3. Use relative paths, not absolute

### Slow Tests

**Problem:** Tests take too long

**Solution:**
1. Use `-parallel` flag
2. Reduce fixture data size
3. Remove unnecessary sleeps
4. Profile: `go test -cpuprofile=cpu.prof`

## Best Practices

### Do's

✅ Use `testutils.NewWorkspace(t)` for isolation
✅ Copy fixtures to temp directory
✅ Test both success and error paths
✅ Use descriptive test names
✅ Keep tests focused (one behavior per test)
✅ Run with race detector
✅ Review golden file changes

### Don'ts

❌ Use global variables
❌ Share filesystem paths between tests
❌ Commit failing tests
❌ Skip race detector
❌ Regenerate golden files without review
❌ Use absolute paths
❌ Mock domain services in integration tests

## CI Integration

Integration tests run in CI with:

```bash
# CI command
go test -race -parallel 4 -timeout 10s ./tests/integration
```

**Requirements:**
- Execution time < 10 seconds
- No race conditions
- All tests pass
- Parallel execution works

## Examples

See `schema_lookup_test.go` for comprehensive examples of:
- Test environment setup
- Fixture management
- Golden file testing
- Error handling
- Concurrent execution
- Event verification

## Getting Help

- **Testing Strategy:** See `docs/architecture/testing-strategy.md`
- **Test Utilities:** See `tests/utils/` for helper functions
- **Example Tests:** See `schema_lookup_test.go` for patterns
- **Go Testing Docs:** https://golang.org/pkg/testing/

## Contributing

When adding integration tests:

1. Follow existing patterns in `schema_lookup_test.go`
2. Use `testutils.NewWorkspace(t)` for isolation
3. Add golden files for output verification
4. Document fixtures in test comments
5. Run with `-race` flag
6. Verify tests can run in parallel
7. Keep execution time < 10s total
