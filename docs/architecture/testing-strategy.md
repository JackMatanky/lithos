# Testing Strategy

## Overview

Lithos uses a comprehensive testing strategy with unit tests, integration tests, and end-to-end tests to ensure reliability and maintainability.

## Test Categories

### Unit Tests

**Location:** `internal/*/` (co-located with source code)

**Purpose:** Test individual components in isolation

**Characteristics:**
- Fast execution (< 100ms per test)
- No external dependencies
- Use mocks for ports and adapters
- Test business logic and domain rules

**Example:**
```bash
go test ./internal/app/frontmatter -v
go test ./internal/app/query -v
```

### Integration Tests

**Location:** `tests/integration/`

**Purpose:** Test component interactions with real implementations

**Characteristics:**
- Test multiple components working together
- Use real adapters (filesystem, cache, parsers)
- No mocks for domain services
- Isolated test environments (temp directories)
- Test end-to-end workflows

**Example:**
```bash
go test ./tests/integration -v
go test ./tests/integration -run SchemaLookup -v
```

### Performance Tests

**Location:** `tests/performance/`

**Purpose:** Benchmark performance and identify bottlenecks

**Characteristics:**
- Large dataset tests
- Performance regression detection
- Memory and CPU profiling

**Example:**
```bash
go test ./tests/performance -bench=. -benchmem
```

## Integration Test Philosophy

Integration tests in Lithos follow these principles:

1. **Real Implementations:** Use actual adapters and services (no mocks for core domain logic)
2. **Isolated Environments:** Each test gets its own temp directory via `testutils.NewWorkspace(t)`
3. **Automatic Cleanup:** Resources cleaned up via `t.Cleanup()`
4. **Golden Files:** Expected outputs stored for regression detection
5. **Parallelizable:** Tests can run concurrently without conflicts

## Schema-Driven Lookup Integration Tests

### Purpose

The schema-driven lookup integration tests (`tests/integration/schema_lookup_test.go`) verify that:
- Template helpers (lookup, query, fileClass) work correctly
- QueryService integrates with TemplateEngine
- FileSpec validation works with real schemas
- Note creation workflow (NewNote) works end-to-end
- Error handling provides helpful user feedback

### Fixture Layout

Integration test fixtures are organized in `testdata/`:

```
testdata/
├── schemas/
│   ├── valid/
│   │   ├── contact.json         # Contact schema with FileSpec
│   │   └── project.json         # Project schema with FileSpec array
│   └── properties/
│       └── property_bank.json   # Standard properties
├── notes/
│   ├── contacts/
│   │   ├── john_doe.md          # Contact: John Doe
│   │   └── jane_smith.md        # Contact: Jane Smith
│   └── projects/
│       └── project_alpha.md     # Project referencing contacts
├── templates/
│   ├── project_with_contacts.md       # Template using lookup/query helpers
│   └── schema_lookup_new_note.md      # Template for NewNote workflow
└── golden/
    ├── project_with_contacts.md       # Expected output (lookup)
    ├── project_with_contacts_query.md # Expected output (query)
    └── project_with_contacts_fileclass.md # Expected output (fileClass)
```

### Running Tests

**Run all schema lookup tests:**
```bash
go test ./tests/integration -run SchemaLookup -v
```

**Run specific test:**
```bash
go test ./tests/integration -run SchemaLookup/LookupHelper -v
```

**Run with race detector:**
```bash
go test -race ./tests/integration -run SchemaLookup
```

**Run in parallel:**
```bash
go test ./tests/integration -run SchemaLookup -parallel 4
```

**Check execution time:**
```bash
time go test ./tests/integration -run SchemaLookup
```

### Golden File Testing

Integration tests use golden files to verify output consistency.

**What are golden files?**
- Stored expected outputs in `testdata/golden/`
- Tests compare actual output against golden file
- Protect against unintended behavior changes

**When to regenerate golden files:**
- Template rendering behavior changes intentionally
- Frontmatter validation messages change
- Query result formatting changes

**How to regenerate:**
```bash
# Regenerate all golden files
UPDATE_GOLDEN=1 go test ./tests/integration -run SchemaLookup -v

# Regenerate specific golden file
UPDATE_GOLDEN=1 go test ./tests/integration -run SchemaLookup/LookupHelper -v
```

**Important:** Always review regenerated golden files before committing to ensure changes are intentional.

### Test Isolation

Each integration test runs in complete isolation:

**Workspace Isolation:**
- Each test gets a unique temp directory via `testutils.NewWorkspace(t)`
- Automatic cleanup via `t.Cleanup()`
- No shared state between tests

**Concurrent Execution:**
- Tests can run in parallel safely
- Each test has isolated vault, cache, schemas
- No race conditions (verified with `-race` flag)

**Verification:**
```bash
# Run tests in parallel
go test ./tests/integration -run SchemaLookup -parallel 4

# Verify no race conditions
go test -race ./tests/integration -run SchemaLookup
```

### Error Handling Tests

Integration tests verify graceful error handling:

**Error Types:**
- `ErrNotFound`: Lookup failures return typed error
- `ValidationError`: FileSpec validation includes remediation hints
- Empty results: Queries return empty slice (not error)
- Graceful degradation: Missing fileClass returns empty string

**Example:**
```go
// Verify ValidationError includes remediation (FR8)
err := frontmatterService.Validate(ctx, noteID, fm)
require.Error(t, err)
var validationErr *errors.ValidationError
require.ErrorAs(t, err, &validationErr)
assert.NotEmpty(t, validationErr.Remediation())
```

## Test Isolation Guidelines

### Using testutils.NewWorkspace

Always use `testutils.NewWorkspace(t)` for integration tests:

```go
func TestMyIntegration(t *testing.T) {
    ws := testutils.NewWorkspace(t)
    // ws.Root() returns temp directory
    // ws.Path("vault") returns path within workspace
    // Automatic cleanup via t.Cleanup()
}
```

### Avoiding Shared State

**Don't:**
- Use global variables
- Share filesystem paths between tests
- Reuse service instances across tests

**Do:**
- Create fresh services for each test
- Use temp directories
- Copy fixtures to isolated workspace

## CI Integration

Integration tests run in CI pipeline with:
- Execution time limit: < 10 seconds total
- Race detector enabled
- Parallel execution: `-parallel 4`
- Coverage reporting

**CI Command:**
```bash
go test -race -parallel 4 -timeout 10s ./tests/integration
```

## Performance Benchmarks

Performance tests ensure the system meets non-functional requirements:

**Example:**
```bash
# Run all benchmarks
go test ./tests/performance -bench=. -benchmem

# Run specific benchmark
go test ./tests/performance -bench=BenchmarkQueryService -benchmem

# Generate CPU profile
go test ./tests/performance -bench=. -cpuprofile=cpu.prof
```

## Test Coverage Goals

- **Unit Tests:** > 90% coverage for business logic
- **Integration Tests:** All critical workflows covered
- **Edge Cases:** Error handling and boundary conditions tested

**Check coverage:**
```bash
# Overall coverage
go test -cover ./...

# Detailed coverage report
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out
```

## Best Practices

### Test Organization

1. **Arrange-Act-Assert:** Structure tests clearly
2. **Descriptive Names:** Use clear test names (e.g., `TestLookup_ReturnsErrorForMissingNote`)
3. **One Assertion Focus:** Each test should verify one behavior
4. **Table-Driven Tests:** Use for multiple scenarios

### Test Data

1. **Realistic Fixtures:** Use real-world examples
2. **Minimal Data:** Only include data relevant to test
3. **Clear Names:** Use descriptive fixture filenames
4. **Version Control:** Commit fixtures and golden files

### Mocks vs Real Implementations

**Use Mocks For:**
- External services (APIs, databases)
- Infrastructure adapters in unit tests
- Slow operations in unit tests

**Use Real Implementations For:**
- Domain services
- Business logic
- Integration tests
- Workflow tests

### Thread Safety

All mocks must be thread-safe for concurrent test execution:

```go
type MockEventBus struct {
    mu sync.Mutex  // Protect concurrent access
    events []Event
}

func (m *MockEventBus) Publish(event Event) {
    m.mu.Lock()
    defer m.mu.Unlock()
    m.events = append(m.events, event)
}
```

## Troubleshooting

### Tests Fail Locally But Pass in CI

- Check for absolute paths (use relative)
- Check for race conditions (`-race` flag)
- Verify temp directory cleanup

### Golden File Mismatches

1. Check if behavior change was intentional
2. Review diff carefully
3. Regenerate if change is correct: `UPDATE_GOLDEN=1`
4. Commit updated golden files with explanation

### Race Conditions

1. Run with race detector: `go test -race`
2. Check for shared state
3. Ensure mocks are thread-safe
4. Use mutex protection for concurrent access

### Slow Integration Tests

1. Check for unnecessary sleeps
2. Reduce fixture data size
3. Use parallel execution: `-parallel 4`
4. Profile tests: `go test -cpuprofile`

## References

- [Go Testing Documentation](https://golang.org/pkg/testing/)
- [Table-Driven Tests](https://github.com/golang/go/wiki/TableDrivenTests)
- [Testify Assertions](https://github.com/stretchr/testify)
- Integration Test Examples: `tests/integration/schema_lookup_test.go`
