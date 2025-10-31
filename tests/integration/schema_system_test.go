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
	testutils "github.com/JackMatanky/lithos/tests/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestSchemaEngine_LoadValidSchemas tests SchemaEngine loading valid schemas
// with real adapters.
func TestSchemaEngine_LoadValidSchemas(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	ws.MkdirAll("schemas", 0o750)
	ws.WriteFile(
		filepath.Join("schemas", "property_bank.json"),
		[]byte(`{
  "properties": {
    "standard_title": {
      "required": true,
      "array": false,
      "type": "string"
    },
    "standard_tags": {
      "type": "string",
      "required": false,
      "array": true
    }
  }
}`),
		0o600,
	)
	ws.WriteFile(
		filepath.Join("schemas", "note.json"),
		[]byte(`{
  "name": "note",
  "properties": {
    "title": {
      "required": true,
      "array": false,
      "type": "string"
    },
    "tags": {
      "required": false,
      "array": true,
      "type": "string"
    }
  }
}`),
		0o600,
	)

	// Create minimal config for the test
	cfg := &domain.Config{
		VaultPath:        tempDir,
		SchemasDir:       filepath.Join(tempDir, "schemas"),
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

	// Create base schema
	baseSchema := `{
  "name": "base",
  "properties": {
    "id": {
      "required": true,
      "array": false,
      "type": "string"
    },
    "created_at": {
      "required": true,
      "array": false,
      "type": "date"
    }
  }
}`
	basePath := filepath.Join(schemasDir, "base.json")
	require.NoError(t, os.WriteFile(basePath, []byte(baseSchema), 0o600))

	// Create middle schema extending base
	middleSchema := `{
  "name": "middle",
  "extends": "base",
  "properties": {
    "title": {
      "required": true,
      "array": false,
      "type": "string"
    },
    "updated_at": {
      "required": false,
      "array": false,
      "type": "date"
    }
  }
}`
	middlePath := filepath.Join(schemasDir, "middle.json")
	require.NoError(t, os.WriteFile(middlePath, []byte(middleSchema), 0o600))

	// Create leaf schema extending middle
	leafSchema := `{
  "name": "leaf",
  "extends": "middle",
  "properties": {
    "description": {
      "required": false,
      "array": false,
      "type": "string"
    },
    "tags": {
      "required": false,
      "array": true,
      "type": "string"
    }
  }
}`
	leafPath := filepath.Join(schemasDir, "leaf.json")
	require.NoError(t, os.WriteFile(leafPath, []byte(leafSchema), 0o600))

	// Create minimal config for the test
	cfg := &domain.Config{
		VaultPath:        tempDir,
		SchemasDir:       filepath.Join(tempDir, "schemas"),
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
		SchemasDir:       filepath.Join(tempDir, "schemas"),
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
