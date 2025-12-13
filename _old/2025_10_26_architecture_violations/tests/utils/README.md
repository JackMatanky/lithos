# Test Utilities Package

The `tests/utils` package provides shared testing infrastructure and utilities for the lithos project. This centralized approach ensures consistent test data access, reliable mock implementations, and maintainable test patterns across all test suites.

## Package Overview

This package contains:

- **Test Data Management**: Centralized utilities for accessing testdata files
- **Mock Implementations**: Reusable mock objects for external dependencies
- **Path Management**: Consistent testdata directory and file path handling

## Files in this Package

- `testdata.go` - Test data loading utilities and path management
- `mocks.go` - Mock implementations for ports/adapters
- `testdata_test.go` - Tests for the test utilities themselves
- `README.md` - This documentation file

## Test Data Management

### TestDataPaths Structure

The `TestDataPaths` struct provides organized access to different categories of test data:

```go
type TestDataPaths struct {
    SchemaValid      string // "schema/valid/"
    SchemaInvalid    string // "schema/invalid/"
    SchemaProperties string // "schema/properties/"
    Golden           string // "golden/"
    Templates        string // "templates/"
    Notes            string // "notes/"
}
```

### Usage Examples

#### Basic Test Data Loading

```go
// Load any testdata file as string
data, err := LoadTestData("schema/valid/complete-user.json")
if err != nil {
    t.Fatalf("Failed to load test data: %v", err)
}

// Convenience wrapper for schema files
schemaData, err := LoadSchemaTestData("valid/complete-user.json")
if err != nil {
    t.Fatalf("Failed to load schema: %v", err)
}

// Get absolute path to testdata file
path, err := GetTestDataPath("templates/static-template.md")
if err != nil {
    t.Fatalf("Failed to get path: %v", err)
}
```

#### Using TestDataPaths for Organization

```go
func TestExample(t *testing.T) {
    paths := NewTestDataPaths()

    // Load different files from the same directory
    completeUserPath := paths.File(paths.SchemaValid, "complete-user.json")
    noteSchemaPath := paths.File(paths.SchemaValid, "note.json")

    // Load files from properties directory
    propertyBankPath := paths.File(paths.SchemaProperties, "bank.json")
    stringPropertyPath := paths.File(paths.SchemaProperties, "string.json")

    // Load invalid schemas
    malformedSchemaPath := paths.File(paths.SchemaInvalid, "malformed.json")

    // Use with LoadTestData function
    data, err := LoadTestData(completeUserPath)
    if err != nil {
        t.Fatalf("Failed to load test data: %v", err)
    }

    // Use with LoadSchemaTestData (strips "schema/" prefix automatically)
    schemaData, err := LoadSchemaTestData(paths.File("valid", "complete-user.json"))
    if err != nil {
        t.Fatalf("Failed to load schema data: %v", err)
    }
}
```

### Key Functions

#### Data Loading Functions

- `LoadTestData(filename)` - Load any testdata file as string
- `LoadSchemaTestData(filename)` - Convenience wrapper for schema files (auto-prefixes with "schema/")
- `GetTestDataPath(filename)` - Get absolute path to testdata file
- `GetSchemaTestDataPath(filename)` - Get absolute path to schema testdata file

#### Path Management Functions

- `NewTestDataPaths()` - Create TestDataPaths struct with all directory paths initialized
- `(p *TestDataPaths) File(directory, filename)` - Append filename to directory path

## Mock Implementations

### MockFileSystemPort

Provides an in-memory filesystem implementation for testing filesystem operations without touching the real filesystem.

```go
func TestWithMockFileSystem(t *testing.T) {
    mockFS := NewMockFileSystemPort()

    // Add files to the mock filesystem
    mockFS.AddFile("test-file.txt", []byte("test content"))

    // Add paths for Walk operations
    mockFS.AddWalkPath("test-file.txt")

    // Configure error injection
    mockFS.SetReadError(errors.New("simulated read error"))

    // Use in your tests
    data, err := mockFS.ReadFile("test-file.txt")
    // ... test logic
}
```

**Features:**

- In-memory file storage
- Configurable error injection for testing error scenarios
- Support for Walk operations with custom path lists
- Custom function injection for ReadFile and WriteFileAtomic operations

### MockConfigPort

Provides a mock configuration implementation for testing with different vault configurations.

```go
func TestWithMockConfig(t *testing.T) {
    mockConfig := NewMockConfigPort("/path/to/test/vault")

    config := mockConfig.Config()
    // Use config in your tests
}
```

## Best Practices

### Test Data Organization

1. **Use TestDataPaths for consistency**: Always use `NewTestDataPaths()` to access test data directories
2. **Keep test data immutable**: Don't modify files in `testdata/` during tests
3. **Use temporary directories for mutations**: Copy testdata to `t.TempDir()` when tests need to modify files
4. **Organize by validity and purpose**: Follow the existing structure (valid/, invalid/, properties/, etc.)

### Mock Usage

1. **Use mocks for external dependencies**: Mock filesystem, configuration, and any external services
2. **Configure realistic scenarios**: Set up mocks to simulate both success and error conditions
3. **Reset mock state**: Create new mock instances for each test to avoid state pollution
4. **Test error conditions**: Use error injection capabilities to test error handling paths

### Path Management

1. **Use helper functions**: Prefer `LoadTestData()` over manual path construction
2. **Handle errors properly**: Always check errors from data loading functions
3. **Use relative paths**: All testdata paths should be relative to the testdata root

## Adding New Test Data

When adding new test data files:

1. Place files in the appropriate `testdata/` subdirectory
2. Use the existing `TestDataPaths` structure to access them
3. Add tests to verify the new data loads correctly
4. Document any new data categories or structures

## Adding New Mocks

When adding new mock implementations:

1. Follow the existing patterns in `mocks.go`
2. Implement the corresponding SPI port interface
3. Include configurable error injection where appropriate
4. Add tests for the mock implementation itself
5. Document usage examples in this README

## Running Tests

Tests for the utilities themselves can be run with:

```bash
go test ./tests/utils/
```

Or as part of the full test suite:

```bash
just test
```

## Dependencies

This package depends on:

- Standard Go `testing` package
- Project SPI ports (`internal/ports/spi`)
- Project config types (`internal/adapters/spi/config`)

The package is designed to be imported by other test packages and should not have dependencies on implementation details outside of the defined ports/interfaces.
