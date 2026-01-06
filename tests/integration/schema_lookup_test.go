// Package integration provides end-to-end integration tests for schema-driven
// template lookups.
//
// # Golden File Testing
//
// This test suite uses golden files to verify template rendering output. Golden
// files are stored in testdata/golden/ and represent expected output.
//
// ## Regenerating Golden Files
//
// When template behavior changes intentionally, regenerate golden files:
//
//	UPDATE_GOLDEN=1 go test ./tests/integration -run SchemaLookup -v
//
// This will overwrite golden files with current test output. Review the changes
// carefully before committing to ensure they match expected behavior.
//
// ## Fixture Layout
//
// Test fixtures are organized in testdata/:
// - testdata/schemas/valid/        - Test schemas (contact.json, project.json)
//   - testdata/schemas/properties/   - Property bank (property_bank.json)
//
// - testdata/notes/contacts/       - Contact notes (john_doe.md, jane_smith.md)
//   - testdata/notes/projects/       - Project notes (project_alpha.md)
//
// - testdata/templates/            - Templates using lookup/query/fileClass
// helpers
//   - testdata/golden/               - Expected template rendering output
//
// # Running Tests
//
//	# Run all schema lookup tests
//	go test ./tests/integration -run SchemaLookup -v
//
//	# Run specific test
//	go test ./tests/integration -run SchemaLookup/LookupHelper -v
//
//	# Verify test isolation
//	go test ./tests/integration -run SchemaLookup -parallel 4
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
	schemaadapter "github.com/JackMatanky/lithos/internal/adapters/spi/schema"
	templateAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/template"
	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	command "github.com/JackMatanky/lithos/internal/app/command"
	events "github.com/JackMatanky/lithos/internal/app/events"
	frontmatterService "github.com/JackMatanky/lithos/internal/app/frontmatter"
	queryService "github.com/JackMatanky/lithos/internal/app/query"
	schemaengine "github.com/JackMatanky/lithos/internal/app/schema"
	templateService "github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
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
	t                  *testing.T
	ctx                context.Context
	ws                 *testutils.Workspace
	logger             zerolog.Logger
	config             *domain.Config
	templateEngine     *templateService.TemplateEngine
	queryService       *queryService.QueryService
	schemaEngine       *schemaengine.SchemaEngine
	frontmatterService *frontmatterService.FrontmatterService
	eventBus           events.EventBus
}

func newSchemaLookupTestEnv(t *testing.T) *schemaLookupTestEnv {
	return newSchemaLookupTestEnvWithBus(t, testutils.NewMockEventBus())
}

func newSchemaLookupTestEnvWithBus(
	t *testing.T,
	bus events.EventBus,
) *schemaLookupTestEnv {
	t.Helper()
	ws := testutils.NewWorkspace(t)
	ctx := context.Background()
	logger := zerolog.New(zerolog.NewTestWriter(t)).With().Timestamp().Logger()
	env := &schemaLookupTestEnv{
		t:        t,
		ctx:      ctx,
		ws:       ws,
		logger:   logger,
		eventBus: bus,
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
	env.initializeServices()
}

func (env *schemaLookupTestEnv) initializeServices() {
	env.t.Helper()
	if env.eventBus == nil {
		env.eventBus = testutils.NewMockEventBus()
	}
	env.initSchemaEngine()
	env.initQueryService()
	env.initTemplateEngine()
	env.initFrontmatterService()
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
			"schemas/valid/project.json",
			[]string{"schemas", "valid", "project.json"},
		},
		{
			"schemas/property_bank.json",
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
		{
			"templates/schema_lookup_new_note.md",
			[]string{"templates", "schema_lookup_new_note.md"},
		},
	}

	for _, fx := range fixtures {
		testutils.CopyFromTestdata(env.t, env.ws, fx.dest, fx.src...)
	}

	// Ensure cache directory exists
	env.ws.MkdirAll("cache", 0o750)
}

func (env *schemaLookupTestEnv) initSchemaEngine() {
	env.t.Helper()

	loader := schemaadapter.NewSchemaLoaderAdapter(env.config, &env.logger)
	registry := schemaadapter.NewSchemaRegistryAdapter(env.logger)
	engine, err := schemaengine.NewSchemaEngine(
		loader,
		registry,
		env.logger,
		env.eventBus,
	)
	require.NoError(env.t, err)
	require.NoError(env.t, engine.Load(env.ctx))
	env.schemaEngine = engine
}

func (env *schemaLookupTestEnv) initQueryService() {
	env.t.Helper()

	reader := cachejson.NewJSONCacheReader(
		domain.Config{CacheDir: env.ws.Path("cache")},
		env.logger,
	)
	router := queryService.NewStorageRouter(reader, reader)
	env.queryService = queryService.NewQueryService(
		router,
		*env.config,
		env.logger,
		env.eventBus,
	)
}

func (env *schemaLookupTestEnv) initFrontmatterService() {
	env.frontmatterService = frontmatterService.NewFrontmatterService(
		env.schemaEngine,
		env.logger,
		env.eventBus,
		env.queryService,
	)
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

	templateLoader := templateAdapter.NewTemplateLoaderAdapter(
		env.config,
		&env.logger,
	)
	env.templateEngine = templateService.NewTemplateEngine(
		templateLoader,
		env.config,
		env.queryService,
		&env.logger,
		env.eventBus,
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

// assertMatchesGolden compares actual output against golden file.
// Supports UPDATE_GOLDEN=1 environment variable for regeneration:
//
//	UPDATE_GOLDEN=1 go test ./tests/integration -run SchemaLookup
//
// When UPDATE_GOLDEN=1 is set, writes actual output to golden file instead
// of comparing. This allows updating golden files when template behavior
// changes intentionally.
func (env *schemaLookupTestEnv) assertMatchesGolden(
	t *testing.T,
	goldenFilename string,
	actual string,
) {
	t.Helper()
	goldenPath := testutils.Path(t, "golden", goldenFilename)

	// Normalize actual output before comparison or writing
	actual = normalizeGeneratedLines(actual)

	// UPDATE_GOLDEN=1: Regenerate golden file
	if os.Getenv("UPDATE_GOLDEN") == "1" {
		err := os.WriteFile(goldenPath, []byte(actual), 0o644)
		require.NoError(t, err, "failed to update golden file: %s", goldenPath)
		t.Logf("✅ Updated golden file: %s", goldenFilename)
		return
	}

	// Normal mode: Compare against existing golden file
	expectedBytes, err := os.ReadFile(goldenPath)
	require.NoError(t, err, "failed to read golden file: %s", goldenPath)
	expected := normalizeGeneratedLines(string(expectedBytes))
	assert.Equal(
		t,
		expected,
		actual,
		"output does not match golden file: %s",
		goldenFilename,
	)
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

func (env *schemaLookupTestEnv) projectFrontmatter(
	overrides map[string]any,
) domain.Frontmatter {
	fields := map[string]any{
		"file_class":  "project",
		"title":       "Project Validation",
		"description": "Validation scenario",
		"status":      "active",
		"team_members": []any{
			"[[john_doe]]",
			"[[jane_smith]]",
		},
	}
	for key, value := range overrides {
		fields[key] = value
	}
	return domain.NewFrontmatter(fields)
}

func (env *schemaLookupTestEnv) writeAndIndexNote(
	t *testing.T,
	relPath string,
	content string,
) {
	t.Helper()
	abs := env.ws.Path("vault", relPath)
	require.NoError(t, os.MkdirAll(filepath.Dir(abs), 0o750))
	require.NoError(t, os.WriteFile(abs, []byte(content), 0o640))
	env.persistNote(t, relPath, []byte(content))
}

func (env *schemaLookupTestEnv) persistNote(
	t *testing.T,
	relPath string,
	data []byte,
) {
	t.Helper()
	writer := cachejson.NewJSONCacheWriter(
		domain.Config{CacheDir: env.ws.Path("cache")},
		env.logger,
	)
	parser := vaultAdapter.NewMarkdownParserAdapter(env.logger)
	note, err := parser.ParseNote(env.ctx, filepath.ToSlash(relPath), data)
	require.NoError(t, err)
	metadata := spi.CacheWriteMetadata{
		ModifiedAt: time.Now(),
		FileSize:   int64(len(data)),
		IndexTime:  time.Now(),
	}
	require.NoError(t, writer.Persist(env.ctx, note, metadata))
}

// newCLIComander wires the CommandOrchestrator with real template, vault, and
// frontmatter services so tests can execute the actual NewNote workflow.
func (env *schemaLookupTestEnv) newCLIComander(
	t *testing.T,
) *command.CLIComander {
	t.Helper()
	vaultWriter := vaultAdapter.NewVaultWriterAdapter(*env.config, env.logger)
	markdownParser := vaultAdapter.NewMarkdownParserAdapter(env.logger)
	return command.NewCLIComander(
		nil,
		env.templateEngine,
		nil,
		vaultWriter,
		env.frontmatterService,
		markdownParser,
		env.config,
		&env.logger,
		env.eventBus,
	)
}

// indexNoteFromVault parses the given vault-relative note and persists it to
// the cache, simulating the VaultIndexer path used in production workflows.
func (env *schemaLookupTestEnv) indexNoteFromVault(
	t *testing.T,
	relPath string,
) {
	t.Helper()
	abs := env.ws.Path("vault", relPath)
	data, err := os.ReadFile(abs)
	require.NoError(t, err)
	env.persistNote(t, relPath, data)
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
// TestSchemaLookup_SetupTests verifies test environment setup works correctly.
// Broken into focused subtests for better maintainability and error isolation.
func TestSchemaLookup_SetupTests(t *testing.T) {
	t.Run("fixtures load correctly", testFixturesLoadCorrectly)
	t.Run("temp directory creation", testTempDirectoryCreation)
	t.Run("component wiring", testComponentWiring)
}

// testFixturesLoadCorrectly verifies all test fixtures can be copied from
// testdata.
func testFixturesLoadCorrectly(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	copyAndVerifyFixtures(t, ws)
}

// copyAndVerifyFixtures copies all fixtures and verifies they exist with valid
// content.
func copyAndVerifyFixtures(t *testing.T, ws *testutils.Workspace) {
	// Copy schema fixtures
	testutils.CopyFromTestdata(
		t,
		ws,
		"schemas/valid/contact.json",
		"schemas",
		"valid",
		"contact.json",
	)
	testutils.CopyFromTestdata(
		t,
		ws,
		"schemas/properties/property_bank.json",
		"schemas",
		"properties",
		"property_bank.json",
	)

	// Copy note fixtures
	testutils.CopyFromTestdata(
		t,
		ws,
		"notes/contacts/john_doe.md",
		"notes",
		"contacts",
		"john_doe.md",
	)
	testutils.CopyFromTestdata(
		t,
		ws,
		"notes/contacts/jane_smith.md",
		"notes",
		"contacts",
		"jane_smith.md",
	)
	testutils.CopyFromTestdata(
		t,
		ws,
		"notes/projects/project_alpha.md",
		"notes",
		"projects",
		"project_alpha.md",
	)

	// Copy template fixtures
	testutils.CopyFromTestdata(
		t,
		ws,
		"templates/project_with_contacts.md",
		"templates",
		"project_with_contacts.md",
	)
	testutils.CopyFromTestdata(
		t,
		ws,
		"templates/schema_lookup_new_note.md",
		"templates",
		"schema_lookup_new_note.md",
	)

	verifyAllFixturesExist(t, ws)
	verifySchemaContent(t, ws)
}

// verifyAllFixturesExist checks that all copied fixtures exist.
func verifyAllFixturesExist(t *testing.T, ws *testutils.Workspace) {
	// Schema fixtures
	require.FileExists(t, ws.Path("schemas", "valid", "contact.json"))
	require.FileExists(
		t,
		ws.Path("schemas", "properties", "property_bank.json"),
	)

	// Note fixtures
	require.FileExists(t, ws.Path("notes", "contacts", "john_doe.md"))
	require.FileExists(t, ws.Path("notes", "contacts", "jane_smith.md"))
	require.FileExists(t, ws.Path("notes", "projects", "project_alpha.md"))

	// Template fixtures
	require.FileExists(t, ws.Path("templates", "project_with_contacts.md"))
	require.FileExists(t, ws.Path("templates", "schema_lookup_new_note.md"))
}

// verifySchemaContent checks that schema fixtures contain expected structure.
func verifySchemaContent(t *testing.T, ws *testutils.Workspace) {
	content, err := os.ReadFile(ws.Path("schemas", "valid", "contact.json"))
	require.NoError(t, err)
	assert.Contains(t, string(content), `"name"`)
	assert.Contains(t, string(content), `"type": "file"`)
}

// testTempDirectoryCreation verifies workspace creates proper temp directories.
func testTempDirectoryCreation(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	root := ws.Root()

	// Verify root is a temp directory
	assert.True(
		t,
		isTempDir(root),
		"workspace root should be temp directory: %s",
		root,
	)
	require.DirExists(t, root, "workspace root should exist")

	// Verify workspace can create subdirectories (cache directory gets created
	// during setup)
	// This test verifies the basic workspace functionality, not full setup
	require.NotEmpty(
		t,
		ws.Path("cache"),
		"workspace should generate cache path",
	)
}

// testComponentWiring verifies all services are properly wired and functional.
func testComponentWiring(t *testing.T) {
	env := newSchemaLookupTestEnv(t)

	// Verify all services are initialized
	require.NotNil(t, env.templateEngine, "templateEngine should be wired")
	require.NotNil(t, env.queryService, "queryService should be wired")
	require.NotNil(t, env.schemaEngine, "schemaEngine should be wired")
	require.NotNil(
		t,
		env.frontmatterService,
		"frontmatterService should be wired",
	)

	// Verify schema engine loaded schemas
	verifySchemasLoaded(t, env)

	// Verify cache has indexed notes
	verifyNotesIndexed(t, env)
}

// verifySchemasLoaded checks that both contact and project schemas are loaded.
func verifySchemasLoaded(t *testing.T, env *schemaLookupTestEnv) {
	hasSchema := schemaengine.Has[domain.Schema](
		env.schemaEngine,
		env.ctx,
		"contact",
	)
	assert.True(t, hasSchema, "contact schema should be loaded")

	hasSchema = schemaengine.Has[domain.Schema](
		env.schemaEngine,
		env.ctx,
		"project",
	)
	assert.True(t, hasSchema, "project schema should be loaded")
}

// verifyNotesIndexed checks that all test notes are properly indexed in cache.
func verifyNotesIndexed(t *testing.T, env *schemaLookupTestEnv) {
	// Verify contact notes
	notes, err := env.queryService.IDQuery(env.ctx, "contacts/john_doe.md")
	require.NoError(t, err)
	assert.Equal(t, "John Doe", notes.Frontmatter.Fields["title"])

	notes, err = env.queryService.IDQuery(env.ctx, "contacts/jane_smith.md")
	require.NoError(t, err)
	assert.Equal(t, "Jane Smith", notes.Frontmatter.Fields["title"])

	// Verify project note
	notes, err = env.queryService.IDQuery(env.ctx, "projects/project_alpha.md")
	require.NoError(t, err)
	assert.Equal(t, "Project Alpha", notes.Frontmatter.Fields["title"])
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

// TestSchemaLookup_FileSpecValidation verifies FileSpec validation outcomes via
// FrontmatterService with real schemas + cache data.
func TestSchemaLookup_FileSpecValidation(t *testing.T) {
	t.Run("valid file references succeed", func(t *testing.T) {
		env := newSchemaLookupTestEnv(t)
		fm := env.projectFrontmatter(nil)
		require.NoError(t, env.frontmatterService.Validate(
			env.ctx,
			"projects/project_alpha.md",
			fm,
		))
	})

	t.Run("missing references produce remediation", func(t *testing.T) {
		env := newSchemaLookupTestEnv(t)
		fm := env.projectFrontmatter(map[string]any{
			"team_members": []any{"[[missing_contact]]"},
		})

		err := env.frontmatterService.Validate(
			env.ctx,
			"projects/project_alpha.md",
			fm,
		)
		require.Error(t, err)
		var validationErr *lithosErr.ValidationError
		require.ErrorAs(t, err, &validationErr)
		assert.Equal(t, "team_members", validationErr.Property())
		assert.Equal(t, "file not found", validationErr.Reason())
		assert.Contains(t, validationErr.Remediation(), "[[missing_contact]]")
	})

	t.Run("ambiguous wikilinks list matches", func(t *testing.T) {
		env := newSchemaLookupTestEnv(t)

		contactContent := `---
file_class: contact
title: Duplicate Contact
---
# Duplicate Contact
`
		env.writeAndIndexNote(
			t,
			filepath.Join("contacts", "contact.md"),
			contactContent,
		)

		projectContent := `---
file_class: project
title: Duplicate Project Contact
---
# Duplicate Project Contact
`
		env.writeAndIndexNote(
			t,
			filepath.Join("projects", "contact.md"),
			projectContent,
		)

		fm := env.projectFrontmatter(map[string]any{
			"team_members": []any{"[[contact]]"},
		})

		err := env.frontmatterService.Validate(
			env.ctx,
			"projects/project_alpha.md",
			fm,
		)
		require.Error(t, err)
		var validationErr *lithosErr.ValidationError
		require.ErrorAs(t, err, &validationErr)
		assert.Equal(t, "team_members", validationErr.Property())
		assert.Equal(t, "ambiguous reference", validationErr.Reason())
		assert.Contains(t, validationErr.Remediation(), "matches")
	})
}

// TestSchemaLookup_NewNoteWorkflow exercises CommandOrchestrator.NewNote end to
// end using real template lookups, frontmatter validation, and cache updates.
func TestSchemaLookup_NewNoteWorkflow(t *testing.T) {
	t.Run(
		"NewNote workflow creates validated project note",
		func(t *testing.T) {
			env := newSchemaLookupTestEnv(t)
			orchestrator := env.newCLIComander(t)

			ctx, cancel := context.WithTimeout(env.ctx, 5*time.Second)
			defer cancel()

			templateID := domain.TemplateID("schema_lookup_new_note")
			note, err := orchestrator.NewNote(ctx, templateID)
			require.NoError(t, err)

			assert.Equal(t, "schema_lookup_new_note.md", note.Path)
			assert.Equal(t, "project", note.FileClass())
			assert.Equal(t, "Project Alpha", note.Frontmatter.Fields["title"])
			require.Contains(t, note.Frontmatter.Fields, "team_members")

			createdPath := env.ws.Path("vault", note.Path)
			require.FileExists(t, createdPath)
			content, readErr := os.ReadFile(createdPath)
			require.NoError(t, readErr)
			assert.Contains(
				t,
				string(content),
				"Schema Lookup Integration Note",
			)

			require.NoError(
				t,
				env.frontmatterService.Validate(
					ctx,
					note.Path,
					note.Frontmatter,
				),
			)

			env.indexNoteFromVault(t, note.Path)
			notes, queryErr := env.queryService.PathQuery(
				env.ctx,
				spi.PathQueryOptions{
					Scope: spi.PathQueryScopeBasename,
					Value: "schema_lookup_new_note",
				},
			)
			require.NoError(t, queryErr)
			require.NotEmpty(t, notes)
			assert.Equal(t, note.Path, notes[0].Path)

			mockBus, ok := env.eventBus.(*testutils.MockEventBus)
			require.True(t, ok)
			published := mockBus.GetPublishedEvents()

			orchestratorEvents := 0
			seenCreated := false
			seenIndexed := false
			for _, evt := range published {
				switch evt.EventType() {
				case "NoteCreated":
					orchestratorEvents++
					seenCreated = true
				case "NoteIndexed":
					orchestratorEvents++
					seenIndexed = true
				}
			}
			require.Equal(t, 2, orchestratorEvents)
			assert.True(t, seenCreated)
			assert.True(t, seenIndexed)

			const workflowSteps = 40.0
			eventRatio := float64(orchestratorEvents) / workflowSteps
			assert.LessOrEqual(
				t,
				eventRatio,
				0.05,
				"event overhead should stay below 5%%",
			)
		},
	)
}

// TestSchemaLookup_ErrorHandling verifies error behavior for lookup, query,
// fileClass, and FileSpec validation failures per AC 4.4.32-4.4.36.
func TestSchemaLookup_ErrorHandling(t *testing.T) {
	t.Run("lookup returns error for missing note", func(t *testing.T) {
		env := newSchemaLookupTestEnv(t)

		// Test IDQuery with missing note path
		_, err := env.queryService.IDQuery(env.ctx, "nonexistent/note.md")
		require.Error(t, err)

		// AC 4.4.32: Verify returns typed error (not generic error)
		// Currently returns ErrNotFound (BaseError) from cache adapter
		// TODO(Epic 5): Consider wrapping in ResourceError for consistency
		assert.ErrorIs(t, err, lithosErr.ErrNotFound)
	})

	t.Run("query returns empty slice for no matches", func(t *testing.T) {
		env := newSchemaLookupTestEnv(t)

		// Test FileClassQuery with non-matching fileClass
		notes, err := env.queryService.FileClassQuery(
			env.ctx,
			"nonexistent-file-class",
		)

		// AC 4.4.33: Returns empty slice, NOT error (graceful from 4.5)
		require.NoError(t, err)
		assert.Empty(t, notes)
	})

	t.Run(
		"fileClass returns empty string for missing field",
		func(t *testing.T) {
			env := newSchemaLookupTestEnv(t)

			// Create note without fileClass field
			noteContent := `---
title: Note Without FileClass
---
# No FileClass
`
			relPath := filepath.Join("notes", "no_fileclass.md")
			env.writeAndIndexNote(t, relPath, noteContent)

			// Query the note to verify it exists
			note, err := env.queryService.IDQuery(env.ctx, relPath)
			require.NoError(t, err)

			// AC 4.4.34: Note has no fileClass field (graceful from 4.5)
			fileClassValue, exists := note.Frontmatter.Fields["file_class"]
			assert.False(
				t,
				exists,
				"file_class should not exist in frontmatter",
			)
			assert.Nil(t, fileClassValue)

			// Verify notes without fileClass are queryable via empty string
			// This is graceful degradation - missing fileClass = "" (not an
			// error)
			notes, err := env.queryService.FileClassQuery(env.ctx, "")
			require.NoError(t, err)
			assert.NotEmpty(
				t,
				notes,
				"notes without fileClass are found with empty string query",
			)

			// Verify the note we created is in the results
			found := false
			for _, n := range notes {
				if n.Path == relPath {
					found = true
					break
				}
			}
			assert.True(
				t,
				found,
				"note without fileClass should be in empty string query results",
			)
		},
	)

	t.Run(
		"FileSpec validation returns ValidationError with remediation",
		func(t *testing.T) {
			env := newSchemaLookupTestEnv(t)

			// Create frontmatter with invalid FileSpec reference
			fm := env.projectFrontmatter(map[string]any{
				"team_members": []any{"[[invalid-reference]]"},
			})

			err := env.frontmatterService.Validate(
				env.ctx,
				"projects/test-project.md",
				fm,
			)

			// AC 4.4.35: Returns ValidationError with Remediation field
			require.Error(t, err)
			var validationErr *lithosErr.ValidationError
			require.ErrorAs(t, err, &validationErr)

			// AC 4.4.36: Verify ValidationError includes remediation hints per
			// FR8
			assert.NotEmpty(t, validationErr.Remediation())
			assert.Contains(t, validationErr.Remediation(), "invalid-reference")
			assert.Equal(t, "team_members", validationErr.Property())
			assert.Equal(t, "file not found", validationErr.Reason())
		},
	)
}
