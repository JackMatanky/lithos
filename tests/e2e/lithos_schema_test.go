package e2e

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestLithos_SchemaLoading_Success tests full schema loading workflow from CLI startup.
func TestLithos_SchemaLoading_Success(t *testing.T) {
	// Setup: Create temp vault with schemas and property_bank.json
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy schema fixtures from testdata/schemas/
	testDataSchemas := filepath.Join("..", "..", "testdata", "schemas")
	copyDir(t, testDataSchemas, schemasDir)

	// Copy property_bank.json from testdata/schemas/
	srcBank := filepath.Join("..", "..", "testdata", "schemas", "property_bank.json")
	dstBank := filepath.Join(schemasDir, "property_bank.json")
	copyFile(t, srcBank, dstBank)

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos version (triggers schema loading)
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"version",
	)
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "command output: %s", output)

	// Verify: Exit code 0 and logs show schema loading
	outputStr := string(output)
	assert.Contains(t, outputStr, "loading schemas")
	assert.Contains(t, outputStr, "schemas registered")
	assert.Contains(t, outputStr, "schema engine ready")
}

// TestLithos_SchemaLoading_MissingPropertyBank tests error when property_bank.json is missing.
func TestLithos_SchemaLoading_MissingPropertyBank(t *testing.T) {
	// Setup: Create temp vault with schemas but no property_bank.json
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy schema fixtures but skip property_bank.json
	testDataSchemas := filepath.Join("..", "..", "testdata", "schemas")
	copyDir(t, testDataSchemas, schemasDir)

	// Remove property_bank.json from copied files
	propertyBankPath := filepath.Join(schemasDir, "property_bank.json")
	if err := os.Remove(propertyBankPath); err != nil && !os.IsNotExist(err) {
		require.NoError(t, err)
	}

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos version (should fail with schema loading error)
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"version",
	)
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	err = cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	if exitErr, ok := err.(*exec.ExitError); ok {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithos_SchemaLoading_InvalidJSON tests error when schema JSON is invalid.
func TestLithos_SchemaLoading_InvalidJSON(t *testing.T) {
	// Setup: Create temp vault with invalid schema JSON
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy property_bank.json
	srcBank := filepath.Join("..", "..", "testdata", "schemas", "property_bank.json")
	dstBank := filepath.Join(schemasDir, "property_bank.json")
	copyFile(t, srcBank, dstBank)

	// Create invalid schema JSON
	invalidSchema := `{
		"name": "invalid",
		"properties": {
			"title": "invalid json here
		}
	}`
	invalidSchemaPath := filepath.Join(schemasDir, "invalid.json")
	require.NoError(t, os.WriteFile(invalidSchemaPath, []byte(invalidSchema), 0o600))

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos version (should fail with JSON parse error)
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"version",
	)
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	err = cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	if exitErr, ok := err.(*exec.ExitError); ok {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithos_SchemaLoading_CircularInheritance tests error when schemas have circular inheritance.
func TestLithos_SchemaLoading_CircularInheritance(t *testing.T) {
	// Setup: Create temp vault with schemas that have circular inheritance
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy property_bank.json
	srcBank := filepath.Join("..", "..", "testdata", "schemas", "property_bank.json")
	dstBank := filepath.Join(schemasDir, "property_bank.json")
	copyFile(t, srcBank, dstBank)

	// Create schema A that inherits from B
	schemaA := `{
		"name": "schema_a",
		"extends": "schema_b",
		"properties": {
			"field_a": "string"
		}
	}`
	schemaAPath := filepath.Join(schemasDir, "schema_a.json")
	require.NoError(t, os.WriteFile(schemaAPath, []byte(schemaA), 0o600))

	// Create schema B that inherits from A (circular)
	schemaB := `{
		"name": "schema_b",
		"extends": "schema_a",
		"properties": {
			"field_b": "string"
		}
	}`
	schemaBPath := filepath.Join(schemasDir, "schema_b.json")
	require.NoError(t, os.WriteFile(schemaBPath, []byte(schemaB), 0o600))

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos version (should fail with circular inheritance error)
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"version",
	)
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	err = cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	if exitErr, ok := err.(*exec.ExitError); ok {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithos_SchemaLoading_MissingRefTarget tests error when schema references missing property.
func TestLithos_SchemaLoading_MissingRefTarget(t *testing.T) {
	// Setup: Create temp vault with schema that references non-existent property
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy property_bank.json
	srcBank := filepath.Join("..", "..", "testdata", "schemas", "property_bank.json")
	dstBank := filepath.Join(schemasDir, "property_bank.json")
	copyFile(t, srcBank, dstBank)

	// Create schema that references non-existent property
	schemaWithBadRef := `{
		"name": "bad_ref_schema",
		"properties": {
			"title": {"$ref": "non_existent_property"}
		}
	}`
	schemaPath := filepath.Join(schemasDir, "bad_ref.json")
	require.NoError(t, os.WriteFile(schemaPath, []byte(schemaWithBadRef), 0o600))

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos version (should fail with missing ref error)
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"version",
	)
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	err = cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	if exitErr, ok := err.(*exec.ExitError); ok {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithos_SchemaLoading_DuplicateNames tests error when schemas have duplicate names.
func TestLithos_SchemaLoading_DuplicateNames(t *testing.T) {
	// Setup: Create temp vault with schemas that have duplicate names
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy property_bank.json
	srcBank := filepath.Join("..", "..", "testdata", "schemas", "property_bank.json")
	dstBank := filepath.Join(schemasDir, "property_bank.json")
	copyFile(t, srcBank, dstBank)

	// Create first schema with name "duplicate"
	schema1 := `{
		"name": "duplicate",
		"properties": {
			"field1": "string"
		}
	}`
	schema1Path := filepath.Join(schemasDir, "schema1.json")
	require.NoError(t, os.WriteFile(schema1Path, []byte(schema1), 0o600))

	// Create second schema with same name "duplicate"
	schema2 := `{
		"name": "duplicate",
		"properties": {
			"field2": "string"
		}
	}`
	schema2Path := filepath.Join(schemasDir, "schema2.json")
	require.NoError(t, os.WriteFile(schema2Path, []byte(schema2), 0o600))

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos version (should fail with duplicate names error)
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"version",
	)
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	err = cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	if exitErr, ok := err.(*exec.ExitError); ok {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// copyDir is a helper function to copy directories during test setup.
func copyDir(t *testing.T, src, dst string) {
	t.Helper()

	entries, err := os.ReadDir(src)
	require.NoError(t, err)

	for _, entry := range entries {
		srcPath := filepath.Join(src, entry.Name())
		dstPath := filepath.Join(dst, entry.Name())

		if entry.IsDir() {
			require.NoError(t, os.MkdirAll(dstPath, 0o750))
			copyDir(t, srcPath, dstPath)
		} else {
			copyFile(t, srcPath, dstPath)
		}
	}
}
