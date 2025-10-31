package e2e

import (
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"testing"

	testutils "github.com/JackMatanky/lithos/tests/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestLithos_SchemaLoading_Success tests full schema loading workflow from CLI
// startup.
func TestLithos_SchemaLoading_Success(t *testing.T) {
	// Setup workspace populated with schemas
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	ws.MkdirAll("schemas", 0o750)
	schemasDir := ws.Path("schemas")

	propertyBank := `{
  "properties": {
    "standard_title": {
      "type": "string",
      "required": true,
      "array": false
    },
    "standard_tags": {
      "type": "string",
      "required": false,
      "array": true
    }
  }
}`
	ws.WriteFile(
		filepath.Join("schemas", "property_bank.json"),
		[]byte(propertyBank),
		0o600,
	)

	schema := `{
  "name": "note",
  "properties": {
    "title": {"$ref": "#/properties/standard_title"},
    "tags": {"$ref": "#/properties/standard_tags"}
  }
}`
	ws.WriteFile(filepath.Join("schemas", "note.json"), []byte(schema), 0o600)

	// Create vault config
	configContent := fmt.Sprintf(`{
  "vault_path": "%s",
  "templates_dir": "templates/",
  "schemas_dir": "%s",
  "property_bank_file": "property_bank.json",
  "cache_dir": ".cache/",
  "log_level": "info"
 }`, tempDir, schemasDir)
	ws.WriteFile("lithos.json", []byte(configContent), 0o600)

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
	cmd.Env = append(
		os.Environ(),
		fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir),
		fmt.Sprintf("LITHOS_SCHEMAS_DIR=%s", schemasDir),
	)
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "command output: %s", output)

	// Verify: Exit code 0 and logs show schema loading
	outputStr := string(output)
	assert.Contains(t, outputStr, "loading schemas")
	assert.Contains(t, outputStr, "schemas registered")
	assert.Contains(t, outputStr, "schema engine ready")
}

// TestLithos_SchemaLoading_MissingPropertyBank tests error when
// property_bank.json is missing.
func TestLithos_SchemaLoading_MissingPropertyBank(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	testutils.CopyFromTestdata(t, ws, "schemas", "schemas")
	schemasDir := ws.Path("schemas")

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
	cmd.Env = append(
		os.Environ(),
		fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir),
		fmt.Sprintf("LITHOS_SCHEMAS_DIR=%s", schemasDir),
	)
	err := cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	exitErr := &exec.ExitError{}
	if errors.As(err, &exitErr) {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithos_SchemaLoading_InvalidJSON tests error when schema JSON is invalid.
func TestLithos_SchemaLoading_InvalidJSON(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	ws.MkdirAll("schemas", 0o750)
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("schemas", "property_bank.json"),
		"schemas",
		"property_bank.json",
	)

	schemasDir := ws.Path("schemas")

	// Create invalid schema JSON
	invalidSchema := `{
		"name": "invalid",
		"properties": {
			"title": "invalid json here
		}
	}`
	invalidSchemaPath := filepath.Join(schemasDir, "invalid.json")
	require.NoError(
		t,
		os.WriteFile(invalidSchemaPath, []byte(invalidSchema), 0o600),
	)

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
	cmd.Env = append(
		os.Environ(),
		fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir),
		fmt.Sprintf("LITHOS_SCHEMAS_DIR=%s", schemasDir),
	)
	err := cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	exitErr := &exec.ExitError{}
	if errors.As(err, &exitErr) {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithos_SchemaLoading_CircularInheritance tests error when schemas have
// circular inheritance.
func TestLithos_SchemaLoading_CircularInheritance(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	ws.MkdirAll("schemas", 0o750)
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("schemas", "property_bank.json"),
		"schemas",
		"property_bank.json",
	)

	schemasDir := ws.Path("schemas")

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
	cmd.Env = append(
		os.Environ(),
		fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir),
		fmt.Sprintf("LITHOS_SCHEMAS_DIR=%s", schemasDir),
	)
	err := cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	exitErr := &exec.ExitError{}
	if errors.As(err, &exitErr) {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithos_SchemaLoading_MissingRefTarget tests error when schema references
// missing property.
func TestLithos_SchemaLoading_MissingRefTarget(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	ws.MkdirAll("schemas", 0o750)
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("schemas", "property_bank.json"),
		"schemas",
		"property_bank.json",
	)

	schemasDir := ws.Path("schemas")

	// Create schema that references non-existent property
	schemaWithBadRef := `{
		"name": "bad_ref_schema",
		"properties": {
			"title": {"$ref": "non_existent_property"}
		}
	}`
	schemaPath := filepath.Join(schemasDir, "bad_ref.json")
	require.NoError(
		t,
		os.WriteFile(schemaPath, []byte(schemaWithBadRef), 0o600),
	)

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
	cmd.Env = append(
		os.Environ(),
		fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir),
		fmt.Sprintf("LITHOS_SCHEMAS_DIR=%s", schemasDir),
	)
	err := cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	exitErr := &exec.ExitError{}
	if errors.As(err, &exitErr) {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithos_SchemaLoading_DuplicateNames tests error when schemas have
// duplicate names.
func TestLithos_SchemaLoading_DuplicateNames(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	ws.MkdirAll("schemas", 0o750)
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("schemas", "property_bank.json"),
		"schemas",
		"property_bank.json",
	)

	schemasDir := ws.Path("schemas")

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
	cmd.Env = append(
		os.Environ(),
		fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir),
		fmt.Sprintf(
			"LITHOS_PROPERTY_BANK_FILE=%s",
			filepath.Join(schemasDir, "property_bank.json"),
		),
		fmt.Sprintf("LITHOS_SCHEMAS_DIR=%s", schemasDir),
	)
	err := cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	exitErr := &exec.ExitError{}
	if errors.As(err, &exitErr) {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}
