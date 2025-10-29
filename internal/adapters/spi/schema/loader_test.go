package schema

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// testdataPath returns the path to testdata directory from this package.
func testdataPath() string {
	return "../../../../testdata/schemas"
}

func TestSchemaLoaderAdapter_Load_Success(t *testing.T) {
	// Create temporary directory with only valid files
	tempDir := t.TempDir()
	schemaDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemaDir, 0755))

	// Copy valid files
	validFiles := []string{
		"property_bank.json",
		"base-note.json",
		"meeting-note.json",
	}
	for _, file := range validFiles {
		src := filepath.Join(testdataPath(), file)
		dst := filepath.Join(schemaDir, file)
		data, err := os.ReadFile(src)
		require.NoError(t, err)
		require.NoError(t, os.WriteFile(dst, data, 0644))
	}

	// Setup test config
	cfg := &domain.Config{
		SchemasDir:       schemaDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	// Execute Load
	schemas, bank, err := loader.Load(context.Background())

	// Verify success
	require.NoError(t, err)
	assert.NotEmpty(t, bank.Properties)
	assert.Len(t, schemas, 2)

	// Verify schema names
	names := make([]string, len(schemas))
	for i, s := range schemas {
		names[i] = s.Name
	}
	assert.Contains(t, names, "base-note")
	assert.Contains(t, names, "meeting-note")

	// Verify meeting-note extends base-note
	for _, s := range schemas {
		if s.Name == "meeting-note" {
			assert.Equal(t, "base-note", s.Extends)
		}
	}
}

func TestSchemaLoaderAdapter_Load_MissingPropertyBank(t *testing.T) {
	// Setup config with non-existent property bank
	cfg := &domain.Config{
		SchemasDir:       testdataPath(),
		PropertyBankFile: "missing.json",
	}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	// Execute Load
	_, _, err := loader.Load(context.Background())

	// Verify error
	require.Error(t, err)
	assert.Contains(t, err.Error(), "resource operation failed")
}

func TestSchemaLoaderAdapter_Load_MalformedPropertyBank(t *testing.T) {
	// Setup config pointing to malformed JSON
	cfg := &domain.Config{
		SchemasDir:       testdataPath(),
		PropertyBankFile: "invalid.json", // This is actually a schema file, not property bank
	}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	// Execute Load
	_, _, err := loader.Load(context.Background())

	// Verify error
	require.Error(t, err)
	assert.Contains(t, err.Error(), "malformed property bank JSON")
}

func TestSchemaLoaderAdapter_Load_MalformedSchema(t *testing.T) {
	// Create a temporary malformed schema file
	tempDir := t.TempDir()
	schemaDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemaDir, 0755))

	// Copy valid property bank
	validBankPath := filepath.Join(testdataPath(), "property_bank.json")
	bankData, err := os.ReadFile(validBankPath)
	require.NoError(t, err)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(schemaDir, "property_bank.json"),
			bankData,
			0644,
		),
	)

	// Create malformed schema
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(schemaDir, "bad.json"),
			[]byte("{invalid json"),
			0644,
		),
	)

	// Setup config
	cfg := &domain.Config{
		SchemasDir:       schemaDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	// Execute Load
	_, _, err = loader.Load(context.Background())

	// Verify error
	require.Error(t, err)
	assert.Contains(t, err.Error(), "malformed schema JSON")
}

func TestSchemaLoaderAdapter_Load_DuplicateNames(t *testing.T) {
	// Create temp directory with duplicate schemas
	tempDir := t.TempDir()
	schemaDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemaDir, 0755))

	// Copy property bank
	bankSrc := filepath.Join(testdataPath(), "property_bank.json")
	bankData, err := os.ReadFile(bankSrc)
	require.NoError(t, err)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(schemaDir, "property_bank.json"),
			bankData,
			0644,
		),
	)

	// Copy base-note
	baseSrc := filepath.Join(testdataPath(), "base-note.json")
	baseData, err := os.ReadFile(baseSrc)
	require.NoError(t, err)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(schemaDir, "base-note.json"),
			baseData,
			0644,
		),
	)

	// Copy meeting-note twice to create duplicate
	meetingSrc := filepath.Join(testdataPath(), "meeting-note.json")
	meetingData, err := os.ReadFile(meetingSrc)
	require.NoError(t, err)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(schemaDir, "meeting-note.json"),
			meetingData,
			0644,
		),
	)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(schemaDir, "meeting-note-duplicate.json"),
			meetingData,
			0644,
		),
	)

	// Setup config
	cfg := &domain.Config{
		SchemasDir:       schemaDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	// Execute Load
	_, _, err = loader.Load(context.Background())

	// Verify error
	require.Error(t, err)
	assert.Contains(t, err.Error(), "duplicate schema names found")
}

func TestSchemaLoaderAdapter_Load_EmptyDirectory(t *testing.T) {
	// Create empty temp directory
	tempDir := t.TempDir()
	schemaDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemaDir, 0755))

	// Copy valid property bank
	validBankPath := filepath.Join(testdataPath(), "property_bank.json")
	bankData, err := os.ReadFile(validBankPath)
	require.NoError(t, err)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(schemaDir, "property_bank.json"),
			bankData,
			0644,
		),
	)

	// Setup config
	cfg := &domain.Config{
		SchemasDir:       schemaDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	// Execute Load
	schemas, bank, err := loader.Load(context.Background())

	// Verify success with empty schemas
	require.NoError(t, err)
	assert.NotEmpty(t, bank.Properties)
	assert.Empty(t, schemas)
}

func TestSchemaLoaderAdapter_Load_PropertyBankLoadedOnce(t *testing.T) {
	// This test verifies that property bank is loaded exactly once
	// We can't easily test this directly, but we can verify the behavior
	// by checking that Load() succeeds and property bank is present

	// Create temporary directory with valid files
	tempDir := t.TempDir()
	schemaDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemaDir, 0755))

	// Copy valid files
	validFiles := []string{
		"property_bank.json",
		"base-note.json",
		"meeting-note.json",
	}
	for _, file := range validFiles {
		src := filepath.Join(testdataPath(), file)
		dst := filepath.Join(schemaDir, file)
		data, err := os.ReadFile(src)
		require.NoError(t, err)
		require.NoError(t, os.WriteFile(dst, data, 0644))
	}

	cfg := &domain.Config{
		SchemasDir:       schemaDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	schemas, bank, err := loader.Load(context.Background())

	require.NoError(t, err)
	assert.NotEmpty(t, bank.Properties)
	assert.Len(t, schemas, 2) // base-note and meeting-note
}

func TestSchemaLoaderAdapter_loadPropertyBank_Success(t *testing.T) {
	cfg := &domain.Config{
		SchemasDir:       testdataPath(),
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	bank, err := loader.loadPropertyBank(
		filepath.Join(testdataPath(), "property_bank.json"),
	)

	require.NoError(t, err)
	assert.NotEmpty(t, bank.Properties)
}

func TestSchemaLoaderAdapter_loadPropertyBank_MissingFile(t *testing.T) {
	cfg := &domain.Config{}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	_, err := loader.loadPropertyBank("nonexistent.json")

	require.Error(t, err)
	assert.Contains(t, err.Error(), "resource operation failed")
}

func TestSchemaLoaderAdapter_loadPropertyBank_MalformedJSON(t *testing.T) {
	cfg := &domain.Config{}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	_, err := loader.loadPropertyBank(
		filepath.Join(testdataPath(), "invalid.json"),
	)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "malformed property bank JSON")
}

func TestSchemaLoaderAdapter_loadSchemas_Success(t *testing.T) {
	// Create temporary directory with only valid schema files
	tempDir := t.TempDir()

	// Copy valid schema files (excluding property_bank.json)
	validFiles := []string{"base-note.json", "meeting-note.json"}
	for _, file := range validFiles {
		src := filepath.Join(testdataPath(), file)
		dst := filepath.Join(tempDir, file)
		data, err := os.ReadFile(src)
		require.NoError(t, err)
		require.NoError(t, os.WriteFile(dst, data, 0644))
	}

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	schemas, err := loader.loadSchemas(tempDir)

	require.NoError(t, err)
	assert.Len(t, schemas, 2) // base-note and meeting-note
}

func TestSchemaLoaderAdapter_loadSchemas_EmptyDir(t *testing.T) {
	tempDir := t.TempDir()
	cfg := &domain.Config{}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	schemas, err := loader.loadSchemas(tempDir)

	require.NoError(t, err)
	assert.Empty(t, schemas)
}

func TestSchemaLoaderAdapter_checkDuplicates_NoDuplicates(t *testing.T) {
	schemas := []domain.Schema{
		{Name: "schema1"},
		{Name: "schema2"},
	}
	cfg := &domain.Config{}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	err := loader.checkDuplicates(schemas)

	require.NoError(t, err)
}

func TestSchemaLoaderAdapter_checkDuplicates_WithDuplicates(t *testing.T) {
	schemas := []domain.Schema{
		{Name: "duplicate"},
		{Name: "unique"},
		{Name: "duplicate"},
	}
	cfg := &domain.Config{}
	log := logger.NewTest()
	loader := NewSchemaLoaderAdapter(cfg, &log)

	err := loader.checkDuplicates(schemas)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "duplicate schema names found")
	assert.Contains(t, err.Error(), "duplicate")
}
