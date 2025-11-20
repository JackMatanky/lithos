package integration

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	schemaadapter "github.com/JackMatanky/lithos/internal/adapters/spi/schema"
	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	schemaengine "github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestComplexDataIntegration(t *testing.T) {
	// 1. Setup Workspace
	ws := utils.NewWorkspace(t)
	root := ws.Root()

	schemasDir := filepath.Join(root, "schemas")
	vaultDir := filepath.Join(root, "vault")
	ws.MkdirAll("schemas", 0o750)
	ws.MkdirAll("vault", 0o750)

	// 2. Copy Complex Data
	// Schemas
	schemaFiles := []string{"dir.json", "dir_contact.json", "task.json"}
	for _, f := range schemaFiles {
		utils.CopyFromTestdata(
			t,
			ws,
			filepath.Join("schemas", f),
			"vault",
			"schemas",
			f,
		)
	}
	// Property Bank
	utils.CopyFromTestdata(
		t,
		ws,
		"schemas/property_bank.json",
		"vault",
		"schemas",
		"property_bank.json",
	)

	// Vault Notes
	utils.CopyFromTestdata(
		t,
		ws,
		"vault/contacts/jane_smith.md",
		"vault",
		"contacts",
		"jane_smith.md",
	)
	utils.CopyFromTestdata(
		t,
		ws,
		"vault/tasks/project_setup.md",
		"vault",
		"tasks",
		"project_setup.md",
	)
	utils.CopyFromTestdata(
		t,
		ws,
		"vault/organizations/tech_corp.md",
		"vault",
		"organizations",
		"tech_corp.md",
	)

	// 3. Initialize Components
	logger := zerolog.New(zerolog.NewTestWriter(t)).With().Timestamp().Logger()
	config := &domain.Config{
		SchemasDir:       schemasDir,
		PropertyBankFile: "property_bank.json",
	}

	schemaLoader := schemaadapter.NewSchemaLoaderAdapter(config, &logger)
	schemaRegistry := schemaadapter.NewSchemaRegistryAdapter(logger)
	schemaEngine, err := schemaengine.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		logger,
	)
	require.NoError(t, err)

	// 4. Load Schemas
	ctx := context.Background()
	err = schemaEngine.Load(ctx)
	require.NoError(t, err, "Schema loading should succeed with realistic data")

	// 5. Verify Schema Inheritance and Resolution
	t.Run("VerifySchemaResolution", func(t *testing.T) {
		// Check dir_contact schema
		contactSchema, getSchemaErr := schemaRegistry.GetSchema(
			ctx,
			"dir_contact",
		)
		require.NoError(t, getSchemaErr)

		// Should have 'name_first' (own property)
		hasNameFirst := false
		// Should have 'title' (inherited from dir)
		hasTitle := false
		// Should have 'uuid' (from property bank)
		hasUUID := false

		for _, p := range contactSchema.ResolvedProperties {
			if p.Name == "name_first" {
				hasNameFirst = true
			}
			if p.Name == "title" {
				hasTitle = true
			}
			if p.Name == "uuid" {
				hasUUID = true
			}
		}

		assert.True(t, hasNameFirst, "dir_contact should have name_first")
		assert.True(t, hasTitle, "dir_contact should inherit title from dir")
		assert.True(t, hasUUID, "dir_contact should have uuid from bank")
	})

	// 6. Initialize Frontmatter Service for Validation
	markdownParser := vaultAdapter.NewMarkdownParserAdapter(logger)
	fmService := frontmatter.NewFrontmatterService(
		schemaEngine,
		markdownParser,
		logger,
	)

	// 7. Validate Realistic Notes
	t.Run("ValidateRealisticNotes", func(t *testing.T) {
		// Validate Contact
		contactPath := filepath.Join(vaultDir, "contacts", "jane_smith.md")
		contactContent, readErr := os.ReadFile(contactPath)
		require.NoError(t, readErr)

		// Parse frontmatter using the adapter (returns map[string]any)
		contactFields, parseErr := markdownParser.ParseFrontmatter(
			ctx,
			contactContent,
		)
		require.NoError(t, parseErr)

		// Create domain object
		contactFm := domain.NewFrontmatter(contactFields)

		// Validate
		validateErr := fmService.IsSchemaCompliant(ctx, contactFm)
		require.NoError(
			t,
			validateErr,
			"Jane Smith contact note should be valid",
		)

		// Validate Task
		taskPath := filepath.Join(vaultDir, "tasks", "project_setup.md")
		taskContent, taskReadErr := os.ReadFile(taskPath)
		require.NoError(t, taskReadErr)

		taskFields, taskParseErr := markdownParser.ParseFrontmatter(
			ctx,
			taskContent,
		)
		require.NoError(t, taskParseErr)

		taskFm := domain.NewFrontmatter(taskFields)

		taskValidateErr := fmService.IsSchemaCompliant(ctx, taskFm)
		assert.NoError(
			t,
			taskValidateErr,
			"Project Setup task note should be valid",
		)
	})
}
