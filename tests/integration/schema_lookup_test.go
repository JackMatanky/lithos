package integration

import (
	"context"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"
	"time"

	cachejson "github.com/JackMatanky/lithos/internal/adapters/spi/cache/json"
	templateAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/template"
	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	queryService "github.com/JackMatanky/lithos/internal/app/query"
	templateService "github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	testutils "github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// schemaLookupTestEnv wires the real adapters (vault parser, cache, query
// service, template engine) using test fixtures copied into a temporary
// workspace. This enables true end-to-end template rendering against actual
// notes without mocks.
type schemaLookupTestEnv struct {
	t              *testing.T
	ctx            context.Context
	ws             *testutils.Workspace
	logger         zerolog.Logger
	config         *domain.Config
	templateEngine *templateService.TemplateEngine
	queryService   *queryService.QueryService
}

func newSchemaLookupTestEnv(t *testing.T) *schemaLookupTestEnv {
	t.Helper()
	ws := testutils.NewWorkspace(t)
	ctx := context.Background()
	logger := zerolog.New(zerolog.NewTestWriter(t)).With().Timestamp().Logger()
	env := &schemaLookupTestEnv{
		t:      t,
		ctx:    ctx,
		ws:     ws,
		logger: logger,
	}
	env.setup()
	return env
}

func (env *schemaLookupTestEnv) setup() {
	env.copyFixtures()
	env.config = &domain.Config{
		VaultPath:        env.ws.Path("vault"),
		SchemasDir:       env.ws.Path("schemas"),
		CacheDir:         env.ws.Path("cache"),
		TemplatesDir:     env.ws.Path("templates"),
		PropertyBankFile: "property_bank.json",
		FileClassKey:     "file_class",
	}
	env.indexFixtures()
	env.initTemplateEngine()
}

func (env *schemaLookupTestEnv) copyFixtures() {
	env.t.Helper()

	fixtures := []struct {
		dest string
		src  []string
	}{
		{
			"schemas/valid/contact.json",
			[]string{"schemas", "valid", "contact.json"},
		},
		{
			"schemas/properties/property_bank.json",
			[]string{"schemas", "properties", "property_bank.json"},
		},
		{
			"vault/contacts/jane_smith.md",
			[]string{"notes", "contacts", "jane_smith.md"},
		},
		{
			"vault/contacts/john_doe.md",
			[]string{"notes", "contacts", "john_doe.md"},
		},
		{
			"vault/projects/project_alpha.md",
			[]string{"notes", "projects", "project_alpha.md"},
		},
		{
			"templates/project_with_contacts.md",
			[]string{"templates", "project_with_contacts.md"},
		},
	}

	for _, fx := range fixtures {
		testutils.CopyFromTestdata(env.t, env.ws, fx.dest, fx.src...)
	}

	// Ensure cache directory exists
	env.ws.MkdirAll("cache", 0o750)
}

func (env *schemaLookupTestEnv) indexFixtures() {
	env.t.Helper()

	writer := cachejson.NewJSONCacheWriter(
		domain.Config{CacheDir: env.ws.Path("cache")},
		env.logger,
	)
	parser := vaultAdapter.NewMarkdownParserAdapter(env.logger)

	notePaths := []string{
		filepath.Join("contacts", "john_doe.md"),
		filepath.Join("contacts", "jane_smith.md"),
		filepath.Join("projects", "project_alpha.md"),
	}

	for _, rel := range notePaths {
		abs := env.ws.Path("vault", rel)
		data, err := os.ReadFile(abs)
		require.NoError(env.t, err)

		relPath := filepath.ToSlash(rel)
		note, err := parser.ParseNote(env.ctx, relPath, data)
		require.NoError(env.t, err)

		metadata := spi.CacheWriteMetadata{
			ModifiedAt: time.Now(),
			FileSize:   int64(len(data)),
			IndexTime:  time.Now(),
		}
		require.NoError(env.t, writer.Persist(env.ctx, note, metadata))
	}
}

func (env *schemaLookupTestEnv) initTemplateEngine() {
	env.t.Helper()

	reader := cachejson.NewJSONCacheReader(
		domain.Config{CacheDir: env.ws.Path("cache")},
		env.logger,
	)
	eventBus := testutils.NewMockEventBus()
	env.queryService = queryService.NewQueryService(
		reader,
		reader,
		*env.config,
		env.logger,
		eventBus,
	)

	templateLoader := templateAdapter.NewTemplateLoaderAdapter(
		env.config,
		&env.logger,
	)
	env.templateEngine = templateService.NewTemplateEngine(
		templateLoader,
		env.config,
		env.queryService,
		&env.logger,
		eventBus,
	)

	_, err := templateLoader.List(env.ctx)
	require.NoError(env.t, err)
}

func (env *schemaLookupTestEnv) renderTemplate(
	t *testing.T,
	templateID string,
) string {
	t.Helper()
	result, err := env.templateEngine.Render(
		env.ctx,
		domain.TemplateID(templateID),
	)
	require.NoError(t, err)
	return result
}

func (env *schemaLookupTestEnv) assertMatchesGolden(
	t *testing.T,
	goldenFilename string,
	actual string,
) {
	t.Helper()
	goldenPath := testutils.Path(t, "golden", goldenFilename)
	expectedBytes, err := os.ReadFile(goldenPath)
	require.NoError(t, err)
	expected := normalizeGeneratedLines(string(expectedBytes))
	actual = normalizeGeneratedLines(actual)
	assert.Equal(t, expected, actual)
}

func (env *schemaLookupTestEnv) lookupNoteByBasename(
	t *testing.T,
	basename string,
) domain.Note {
	t.Helper()
	notes, err := env.queryService.PathQuery(
		env.ctx,
		spi.PathQueryOptions{
			Scope: spi.PathQueryScopeBasename,
			Value: basename,
		},
	)
	require.NoError(t, err)
	require.Len(t, notes, 1)
	return notes[0]
}

func normalizeGeneratedLines(content string) string {
	lines := strings.Split(content, "\n")
	for i, line := range lines {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, "**Generated:**") {
			lines[i] = "**Generated:** <dynamic>"
		}
	}
	normalized := strings.Join(lines, "\n")
	for strings.Contains(normalized, "\n\n\n") {
		normalized = strings.ReplaceAll(normalized, "\n\n\n", "\n\n")
	}
	normalized = strings.TrimRight(normalized, "\n") + "\n"
	return normalized
}

// TestSchemaLookup_SetupTests verifies that the test environment setup
// correctly handles test fixtures, temp directories, and component wiring.
// This is the RED phase - tests should FAIL until fixtures are created.
func TestSchemaLookup_SetupTests(t *testing.T) {
	// Test Case: Test fixtures load correctly from testdata
	t.Run("fixtures load correctly", func(t *testing.T) {
		ws := testutils.NewWorkspace(t)

		// Copy contact schema from testdata - this should FAIL until fixture is
		// created
		testutils.CopyFromTestdata(
			t, ws,
			filepath.Join("schemas", "valid", "contact.json"),
			"schemas", "valid", "contact.json",
		)

		// Verify contact schema was copied
		schemaPath := ws.Path("schemas", "valid", "contact.json")
		require.FileExists(t, schemaPath, "contact.json schema should exist")

		// Copy property bank from properties subdirectory - this should FAIL
		// until fixture is created
		testutils.CopyFromTestdata(
			t, ws,
			filepath.Join("schemas", "properties", "property_bank.json"),
			"schemas", "properties", "property_bank.json",
		)

		// Verify property bank was copied
		propertyBankPath := ws.Path(
			"schemas",
			"properties",
			"property_bank.json",
		)
		require.FileExists(
			t,
			propertyBankPath,
			"property_bank.json should exist",
		)

		// Copy contact notes - these should FAIL until fixtures are created
		testutils.CopyFromTestdata(
			t, ws,
			filepath.Join("notes", "contacts", "john_doe.md"),
			"notes", "contacts", "john_doe.md",
		)
		testutils.CopyFromTestdata(
			t, ws,
			filepath.Join("notes", "contacts", "jane_smith.md"),
			"notes", "contacts", "jane_smith.md",
		)

		// Verify contact notes exist
		require.FileExists(t, ws.Path("notes", "contacts", "john_doe.md"),
			"john_doe.md should exist")
		require.FileExists(t, ws.Path("notes", "contacts", "jane_smith.md"),
			"jane_smith.md should exist")

		// Copy project note - should FAIL until fixture is created
		testutils.CopyFromTestdata(
			t, ws,
			filepath.Join("notes", "projects", "project_alpha.md"),
			"notes", "projects", "project_alpha.md",
		)

		// Verify project note exists
		require.FileExists(
			t,
			ws.Path("notes", "projects", "project_alpha.md"),
			"project_alpha.md should exist",
		)

		// Copy template using lookup/query helpers - should FAIL until fixture
		// is created
		testutils.CopyFromTestdata(
			t, ws,
			filepath.Join("templates", "project_with_contacts.md"),
			"templates", "project_with_contacts.md",
		)

		// Verify template exists
		require.FileExists(t,
			ws.Path("templates", "project_with_contacts.md"),
			"project_with_contacts.md should exist")

		// Verify content is valid
		content, err := os.ReadFile(schemaPath)
		require.NoError(t, err)
		assert.Contains(t, string(content), `"name"`)
		assert.Contains(
			t,
			string(content),
			`"fileSpec"`,
		) // Contact schema should have FileSpec
	})

	// Test Case: Temp directory creation works correctly
	t.Run("temp directory creation", func(t *testing.T) {
		ws := testutils.NewWorkspace(t)
		root := ws.Root()

		// Verify root is a temp directory
		assert.True(
			t,
			isTempDir(root),
			"workspace root should be a temp directory: %s",
			root,
		)

		// Create subdirectories
		ws.MkdirAll("schemas", 0o750)
		ws.MkdirAll("vault", 0o750)
		ws.MkdirAll("templates", 0o750)

		// Verify directories exist
		require.DirExists(
			t,
			ws.Path("schemas"),
			"schemas directory should exist",
		)
		require.DirExists(t, ws.Path("vault"), "vault directory should exist")
		require.DirExists(
			t,
			ws.Path("templates"),
			"templates directory should exist",
		)
	})

	// Test Case: Component wiring works with TemplateEngine, QueryService, etc.
	t.Run("component wiring", func(t *testing.T) {
		ws := testutils.NewWorkspace(t)
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()

		// Set up directory structure
		ws.MkdirAll("schemas", 0o750)
		ws.MkdirAll("vault", 0o750)
		ws.MkdirAll("cache", 0o750)

		// Copy test schema files
		testutils.CopyFromTestdata(t, ws,
			filepath.Join("schemas", "valid", "note.json"),
			"schemas", "valid", "note.json")
		testutils.CopyFromTestdata(t, ws,
			filepath.Join("schemas", "property_bank.json"),
			"schemas", "property_bank.json")

		// Create config with workspace paths
		cfg := &domain.Config{
			VaultPath:        ws.Path("vault"),
			SchemasDir:       ws.Path("schemas"),
			CacheDir:         ws.Path("cache"),
			PropertyBankFile: "property_bank.json",
		}

		// Initialize logger
		log := zerolog.New(zerolog.NewTestWriter(t))

		// Verify config is properly initialized
		require.NotNil(t, cfg)
		assert.Equal(t, ws.Path("vault"), cfg.VaultPath)
		assert.Equal(t, ws.Path("schemas"), cfg.SchemasDir)
		assert.Equal(t, ws.Path("cache"), cfg.CacheDir)

		// Verify context respects timeout
		require.NotNil(t, ctx)
		require.NoError(t, ctx.Err())

		// Components should be instantiable (actual wiring verified in later
		// tests)
		assert.NotNil(t, log)
	})
}

// isTempDir checks if a directory appears to be a temporary directory.
func isTempDir(path string) bool {
	// Check if path is under system temp
	systemTemp := os.TempDir()
	if filepath.Dir(path) == systemTemp || path == systemTemp {
		return true
	}

	// Check for Go test temp directory pattern (e.g.,
	// TestFunctionName/random/001)
	// Go's t.TempDir() creates directories like TestFunctionName/random/001
	// We need to check the parent directories for the "Test" prefix
	dir := filepath.Dir(path)
	base := filepath.Base(path)
	// Check if this is the numbered subdirectory (001, 002, etc.) under test
	// dir
	if _, err := strconv.Atoi(base); err == nil && len(base) == 3 {
		parentBase := filepath.Base(dir)
		return strings.HasPrefix(parentBase, "Test")
	}
	// Otherwise check if path starts with "Test"
	return strings.HasPrefix(base, "Test")
}

// TestSchemaLookup_LookupHelper exercises the lookup helper using real
// adapters and verifies the rendered template output matches the golden file.
func TestSchemaLookup_LookupHelper(t *testing.T) {
	env := newSchemaLookupTestEnv(t)

	t.Run("lookup helper renders contact data", func(t *testing.T) {
		result := env.renderTemplate(t, "project_with_contacts")
		env.assertMatchesGolden(t, "project_with_contacts.md", result)
		assert.Contains(t, result, "John Doe")
		assert.Contains(t, result, "john.doe@example.com")
	})

	t.Run("lookup returns defensive copy", func(t *testing.T) {
		first := env.lookupNoteByBasename(t, "john_doe")
		first.Frontmatter.Fields["title"] = "MUTATED"

		second := env.lookupNoteByBasename(t, "john_doe")
		assert.Equal(t, "John Doe", second.Frontmatter.Fields["title"])
	})
}

// TestSchemaLookup_QueryHelper verifies the query helper returns all contacts
// using the same template rendered end-to-end.
func TestSchemaLookup_QueryHelper(t *testing.T) {
	env := newSchemaLookupTestEnv(t)

	t.Run("query helper lists all contacts", func(t *testing.T) {
		result := env.renderTemplate(t, "project_with_contacts")
		env.assertMatchesGolden(t, "project_with_contacts_query.md", result)
		assert.Contains(
			t,
			result,
			"| John Doe | john.doe@example.com | Acme Corp | Software Engineer |",
		)
		assert.Contains(
			t,
			result,
			"| Jane Smith | jane.smith@techstart.io | TechStart | Product Manager |",
		)
	})
}

// TestSchemaLookup_FileClassHelper verifies fileClass helper usage for note
// classification in templates.
func TestSchemaLookup_FileClassHelper(t *testing.T) {
	env := newSchemaLookupTestEnv(t)

	t.Run("fileClass helper marks contacts", func(t *testing.T) {
		result := env.renderTemplate(t, "project_with_contacts")
		env.assertMatchesGolden(t, "project_with_contacts_fileclass.md", result)
		assert.Contains(t, result, "Primary Contact FileClass: contact")
		assert.Contains(t, result, "Project Alpha FileClass: project")
	})
}
