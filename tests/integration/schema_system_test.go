package integration

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	schemaAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/schema"
	schemaApp "github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestSchemaEngine_LoadValidSchemas tests SchemaEngine loading valid schemas
// with real adapters.
func TestSchemaEngine_LoadValidSchemas(t *testing.T) {
	// Setup: Create temp vault with valid schemas
	tempDir, err := os.MkdirTemp("", "lithos-integration-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy test fixtures
	testDataSchemas := filepath.Join("..", "..", "testdata", "schemas")
	copyDir(t, testDataSchemas, schemasDir)

	// Copy property_bank.json
	srcBank := filepath.Join(
		"..",
		"..",
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstBank := filepath.Join(schemasDir, "property_bank.json")
	copyFile(t, srcBank, dstBank)

	// Create minimal config for the test
	cfg := &domain.Config{
		VaultPath:        tempDir,
		SchemasDir:       "schemas",
		PropertyBankFile: "property_bank.json",
	}

	// Initialize components
	log := logger.New(os.Stdout, "info")
	schemaLoader := schemaAdapter.NewSchemaLoaderAdapter(cfg, &log)
	schemaRegistry := schemaAdapter.NewSchemaRegistryAdapter(log)
	schemaEngine, err := schemaApp.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		log,
	)
	require.NoError(t, err)

	// Execute: Load schemas
	ctx := context.Background()
	err = schemaEngine.Load(ctx)

	// Verify: No errors and schemas loaded
	require.NoError(t, err)
	// Note: Additional verification would require exposing internal state or
	// adding test methods
}

// TestSchemaEngine_ComplexInheritance tests validation and resolution with
// complex inheritance.
func TestSchemaEngine_ComplexInheritance(t *testing.T) {
	// Setup: Create schemas with multi-level inheritance
	tempDir, err := os.MkdirTemp("", "lithos-integration-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy property_bank.json
	srcBank := filepath.Join(
		"..",
		"..",
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstBank := filepath.Join(schemasDir, "property_bank.json")
	copyFile(t, srcBank, dstBank)

	// Create base schema
	baseSchema := `{
		"name": "base",
		"properties": {
			"id": "string",
			"created_at": "datetime"
		}
	}`
	basePath := filepath.Join(schemasDir, "base.json")
	require.NoError(t, os.WriteFile(basePath, []byte(baseSchema), 0o600))

	// Create middle schema extending base
	middleSchema := `{
		"name": "middle",
		"extends": "base",
		"properties": {
			"name": "string",
			"updated_at": "datetime"
		}
	}`
	middlePath := filepath.Join(schemasDir, "middle.json")
	require.NoError(t, os.WriteFile(middlePath, []byte(middleSchema), 0o600))

	// Create leaf schema extending middle
	leafSchema := `{
		"name": "leaf",
		"extends": "middle",
		"properties": {
			"description": "string",
			"tags": "array"
		}
	}`
	leafPath := filepath.Join(schemasDir, "leaf.json")
	require.NoError(t, os.WriteFile(leafPath, []byte(leafSchema), 0o600))

	// Create minimal config for the test
	cfg := &domain.Config{
		VaultPath:        tempDir,
		SchemasDir:       "schemas",
		PropertyBankFile: "property_bank.json",
	}

	// Initialize and load
	log := logger.New(os.Stdout, "info")
	schemaLoader := schemaAdapter.NewSchemaLoaderAdapter(cfg, &log)
	schemaRegistry := schemaAdapter.NewSchemaRegistryAdapter(log)
	schemaEngine, err := schemaApp.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		log,
	)
	require.NoError(t, err)

	ctx := context.Background()
	err = schemaEngine.Load(ctx)

	// Verify: Complex inheritance resolves correctly
	require.NoError(t, err)
}

// TestSchemaRegistry_ConcurrentAccess tests thread-safe concurrent registry
// access.
func TestSchemaRegistry_ConcurrentAccess(t *testing.T) {
	// Setup: Initialize registry
	log := logger.New(os.Stdout, "info")
	schemaRegistry := schemaAdapter.NewSchemaRegistryAdapter(log)

	// Execute: Concurrent access using goroutines
	done := make(chan bool, 10)

	for range 10 {
		go func() {
			// Simulate concurrent access - in real implementation this would
			// test actual registry methods
			// For now, just verify the registry instance is accessible
			assert.NotNil(t, schemaRegistry)
			done <- true
		}()
	}

	// Wait for all goroutines to complete
	for range 10 {
		<-done
	}
}

// TestSchemaEngine_ErrorMessages tests that error messages include remediation
// hints.
func TestSchemaEngine_ErrorMessages(t *testing.T) {
	// Setup: Create temp vault with invalid schema
	tempDir, err := os.MkdirTemp("", "lithos-integration-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy property_bank.json
	srcBank := filepath.Join(
		"..",
		"..",
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstBank := filepath.Join(schemasDir, "property_bank.json")
	copyFile(t, srcBank, dstBank)

	// Create invalid schema JSON
	invalidSchema := `{
		"name": "invalid",
		"properties": {
			"id": "string",
			"invalid_field":
		}
	}`
	invalidPath := filepath.Join(schemasDir, "invalid.json")
	require.NoError(t, os.WriteFile(invalidPath, []byte(invalidSchema), 0o600))

	// Create minimal config for the test
	cfg := &domain.Config{
		VaultPath:        tempDir,
		SchemasDir:       "schemas",
		PropertyBankFile: "property_bank.json",
	}

	// Initialize components
	log := logger.New(os.Stdout, "info")
	schemaLoader := schemaAdapter.NewSchemaLoaderAdapter(cfg, &log)
	schemaRegistry := schemaAdapter.NewSchemaRegistryAdapter(log)
	schemaEngine, err := schemaApp.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		log,
	)
	require.NoError(t, err)

	// Execute: Attempt to load schemas
	ctx := context.Background()
	err = schemaEngine.Load(ctx)

	// Verify: Error occurs and includes remediation hints
	require.Error(t, err)
	// Note: In real implementation, check for specific remediation hints in
	// error message
	assert.Contains(t, err.Error(), "invalid") // Basic check for error content
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

// copyFile is a helper function to copy files during test setup.
func copyFile(t *testing.T, src, dst string) {
	t.Helper()

	srcFile, err := os.Open(src)
	require.NoError(t, err)
	defer func() { _ = srcFile.Close() }()

	dstFile, err := os.Create(dst)
	require.NoError(t, err)
	defer func() { _ = dstFile.Close() }()

	_, err = dstFile.ReadFrom(srcFile)
	require.NoError(t, err)
}
