package schema

import (
	"context"
	"errors"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
	"sync/atomic"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestSchemaLoaderAdapter_LoadSuccess verifies a full property bank and schema
// load.
func TestSchemaLoaderAdapter_LoadSuccess(t *testing.T) {
	t.Parallel()

	tempDir := t.TempDir()
	copySchemaFixture(t, tempDir, "property_bank.json")
	copySchemaFixture(t, tempDir, "base_note.json")
	copySchemaFixture(t, tempDir, "meeting_note.json")

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	adapter := NewSchemaLoaderAdapter(cfg, &log)

	schemas, bank, err := adapter.Load(context.Background())
	require.NoError(t, err)

	assert.Len(t, bank.Properties, 3)
	assert.Len(t, schemas, 2)

	names := []string{schemas[0].Name, schemas[1].Name}
	assert.ElementsMatch(t, []string{"base-note", "meeting_note"}, names)
	for _, schema := range schemas {
		assert.NotNil(t, schema.Properties)
	}
}

// TestSchemaLoaderAdapter_MissingPropertyBank asserts missing property bank
// errors.
func TestSchemaLoaderAdapter_MissingPropertyBank(t *testing.T) {
	t.Parallel()

	tempDir := t.TempDir()
	copySchemaFixture(t, tempDir, "base_note.json")

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	adapter := NewSchemaLoaderAdapter(cfg, &log)

	_, _, err := adapter.Load(context.Background())
	require.Error(t, err)

	var resourceErr *domainerrors.ResourceError
	require.ErrorAs(t, err, &resourceErr)
	const expectedMissingHint = "Create schemas/property_bank.json or configure PropertyBankFile"
	assert.Contains(t, err.Error(), expectedMissingHint)
	assert.Equal(t, "schema", resourceErr.Resource())
	assert.Equal(t, "load", resourceErr.Operation())
}

// TestSchemaLoaderAdapter_MalformedPropertyBank reports malformed JSON with
// remediation.
func TestSchemaLoaderAdapter_MalformedPropertyBank(t *testing.T) {
	t.Parallel()

	tempDir := t.TempDir()
	writeFile(
		t,
		filepath.Join(tempDir, "property_bank.json"),
		`{"properties": {`,
	)
	copySchemaFixture(t, tempDir, "base_note.json")

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	adapter := NewSchemaLoaderAdapter(cfg, &log)

	_, _, err := adapter.Load(context.Background())
	require.Error(t, err)

	var schemaErr *domainerrors.SchemaError
	require.ErrorAs(t, err, &schemaErr)
	assert.Contains(t, schemaErr.Remediation, "Check JSON syntax")
}

// TestSchemaLoaderAdapter_MalformedSchemaJSON reports malformed schema JSON.
func TestSchemaLoaderAdapter_MalformedSchemaJSON(t *testing.T) {
	t.Parallel()

	tempDir := t.TempDir()
	copySchemaFixture(t, tempDir, "property_bank.json")
	copySchemaFixture(t, tempDir, "invalid.json")

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	adapter := NewSchemaLoaderAdapter(cfg, &log)

	_, _, err := adapter.Load(context.Background())
	require.Error(t, err)

	var schemaErr *domainerrors.SchemaError
	require.ErrorAs(t, err, &schemaErr)
	assert.Contains(t, schemaErr.Remediation, "Check JSON syntax")
}

// TestSchemaLoaderAdapter_EmptySchemasDirectory returns an empty slice when no
// schemas exist.
func TestSchemaLoaderAdapter_EmptySchemasDirectory(t *testing.T) {
	t.Parallel()

	tempDir := t.TempDir()
	copySchemaFixture(t, tempDir, "property_bank.json")

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	adapter := NewSchemaLoaderAdapter(cfg, &log)

	schemas, bank, err := adapter.Load(context.Background())
	require.NoError(t, err)

	assert.Empty(t, schemas)
	assert.NotNil(t, bank.Properties)
}

// TestSchemaLoaderAdapter_PropertyBankReadOnce ensures the property bank file
// is read once.
func TestSchemaLoaderAdapter_PropertyBankReadOnce(t *testing.T) {
	t.Parallel()

	tempDir := t.TempDir()
	copySchemaFixture(t, tempDir, "property_bank.json")
	copySchemaFixture(t, tempDir, "base_note.json")

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	adapter := NewSchemaLoaderAdapter(cfg, &log)

	var propertyReads atomic.Int32
	adapter.readFile = func(path string) ([]byte, error) {
		if strings.HasSuffix(path, "property_bank.json") {
			propertyReads.Add(1)
		}
		return os.ReadFile(path)
	}

	_, _, err := adapter.Load(context.Background())
	require.NoError(t, err)
	assert.Equal(t, int32(1), propertyReads.Load())
}

// TestSchemaLoaderAdapter_RespectsConfigPropertyBankPath honors custom property
// bank filenames.
func TestSchemaLoaderAdapter_RespectsConfigPropertyBankPath(t *testing.T) {
	t.Parallel()

	tempDir := t.TempDir()
	writeFile(t, filepath.Join(tempDir, "custom_bank.json"), `{
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
    },
    "iso_date": {
      "type": "string",
      "format": "date-time",
      "required": true,
      "array": false
    }
  }
}`)
	copySchemaFixture(t, tempDir, "base_note.json")

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "custom_bank.json",
	}
	log := logger.NewTest()
	adapter := NewSchemaLoaderAdapter(cfg, &log)

	_, _, err := adapter.Load(context.Background())
	require.NoError(t, err)
}

func copySchemaFixture(t *testing.T, dstDir, filename string) {
	t.Helper()

	src := filepath.Join(
		"..",
		"..",
		"..",
		"..",
		"testdata",
		"schemas",
		filename,
	)
	data, err := os.ReadFile(src)
	require.NoError(t, err)

	dest := filepath.Join(dstDir, filename)
	require.NoError(t, os.MkdirAll(filepath.Dir(dest), 0o750))
	require.NoError(t, os.WriteFile(dest, data, 0o600))
}

func writeFile(t *testing.T, path, content string) {
	t.Helper()

	require.NoError(t, os.MkdirAll(filepath.Dir(path), 0o750))
	require.NoError(t, os.WriteFile(path, []byte(content), 0o600))
}

// TestSchemaLoaderAdapter_PropertyBankBeforeSchemas verifies load order
// logging.
func TestSchemaLoaderAdapter_PropertyBankBeforeSchemas(t *testing.T) {
	t.Parallel()

	tempDir := t.TempDir()
	copySchemaFixture(t, tempDir, "property_bank.json")
	copySchemaFixture(t, tempDir, "base_note.json")

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	adapter := NewSchemaLoaderAdapter(cfg, &log)

	var order []string
	adapter.readFile = func(path string) ([]byte, error) {
		order = append(order, filepath.Base(path))
		return os.ReadFile(path)
	}

	_, _, err := adapter.Load(context.Background())
	require.NoError(t, err)
	require.NotEmpty(t, order)
	assert.Equal(t, "property_bank.json", order[0])
}

// TestSchemaLoaderAdapter_WalkError surfaces directory walk failures.
func TestSchemaLoaderAdapter_WalkError(t *testing.T) {
	t.Parallel()

	tempDir := t.TempDir()
	copySchemaFixture(t, tempDir, "property_bank.json")

	cfg := &domain.Config{
		SchemasDir:       tempDir,
		PropertyBankFile: "property_bank.json",
	}
	log := logger.NewTest()
	adapter := NewSchemaLoaderAdapter(cfg, &log)

	adapter.walkDir = func(string, fs.WalkDirFunc) error {
		return errors.New("walk failure")
	}

	_, _, err := adapter.Load(context.Background())
	require.Error(t, err)

	var resourceErr *domainerrors.ResourceError
	require.ErrorAs(t, err, &resourceErr)
	assert.Equal(t, "schema", resourceErr.Resource())
	assert.Equal(t, "scan", resourceErr.Operation())
}
