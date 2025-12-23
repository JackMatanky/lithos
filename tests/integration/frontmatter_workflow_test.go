// Package integration provides end-to-end tests for frontmatter processing
package integration

import (
	"context"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/json"
	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	schemaadapter "github.com/JackMatanky/lithos/internal/adapters/spi/schema"
	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/query"
	schemaengine "github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// frontmatterTestEnv holds test environment state.
type frontmatterTestEnv struct {
	vaultDir     string
	indexer      *vault.VaultIndexer
	queryService *query.QueryService
	logger       zerolog.Logger
	cleanup      func()
}

// mockVaultScanner implements VaultScannerPort for testing.
type mockVaultScanner struct {
	vaultDir string
}

// setupFrontmatterTestEnvironment creates a test environment with
// frontmatter-enabled components..
func setupFrontmatterTestEnvironment(t *testing.T) *frontmatterTestEnv {
	t.Helper()

	// Create temporary vault directory
	vaultDir, err := os.MkdirTemp("", "frontmatter-test-vault-*")
	require.NoError(t, err)

	// Create test markdown file with frontmatter
	testFile := filepath.Join(vaultDir, "test-note.md")
	testContent := `---
fileClass: "note"
title: "Test Note"
author: "Test Author"
tags: "test, integration"
---

# Test Note

This is a test note with frontmatter.
`
	err = os.WriteFile(testFile, []byte(testContent), 0o600)
	require.NoError(t, err)

	// Create schema file
	schemaDir := filepath.Join(vaultDir, "schemas")
	err = os.MkdirAll(schemaDir, 0o750)
	require.NoError(t, err)

	schemaFile := filepath.Join(schemaDir, "note.json")
	schemaContent := `{
		"name": "note",
		"properties": {
			"fileClass": {
				"type": "string",
				"required": true,
				"description": "The class of the file"
			},
			"title": {
				"type": "string",
				"required": true,
				"description": "The title of the note"
			},
			"author": {
				"type": "string",
				"required": false,
				"description": "The author of the note"
			},
			"tags": {
				"type": "string",
				"required": false,
				"description": "Tags for the note"
			}
		}
	}`
	err = os.WriteFile(schemaFile, []byte(schemaContent), 0o600)
	require.NoError(t, err)

	// Create property bank file
	propertyFile := filepath.Join(schemaDir, "property_bank.json")
	propertyContent := `{
		"commonTags": {
			"type": "string",
			"description": "Common tags for notes"
		}
	}`
	err = os.WriteFile(propertyFile, []byte(propertyContent), 0o600)
	require.NoError(t, err)

	// Create cache directory
	cacheDir := filepath.Join(vaultDir, "cache")
	err = os.MkdirAll(cacheDir, 0o750)
	require.NoError(t, err)

	// Setup components
	logger := zerolog.New(zerolog.NewTestWriter(t)).With().Timestamp().Logger()

	// Create config
	config := &domain.Config{
		SchemasDir:       schemaDir,
		PropertyBankFile: "property_bank.json",
		CacheDir:         cacheDir,
	}

	// Create schema loader and registry
	schemaLoader := schemaadapter.NewSchemaLoaderAdapter(config, &logger)
	schemaRegistry := schemaadapter.NewSchemaRegistryAdapter(logger)

	// Create schema engine
	schemaEngine, err := schemaengine.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		logger,
		nil,
	)
	require.NoError(t, err)

	// Create markdown parser adapter
	markdownParser := vaultAdapter.NewMarkdownParserAdapter(logger)

	// Create frontmatter service
	fmService := frontmatter.NewFrontmatterService(
		schemaEngine,
		logger,
		nil,
	)

	// Create cache adapters
	boltCacheDir := filepath.Join(cacheDir, "bolt")
	sqliteCacheDir := filepath.Join(cacheDir, "sqlite")
	boltConfig := domain.Config{CacheDir: boltCacheDir}
	sqliteConfig := domain.Config{CacheDir: sqliteCacheDir}

	boltWriter := json.NewJSONCacheWriter(boltConfig, logger)
	sqliteWriter := json.NewJSONCacheWriter(sqliteConfig, logger)
	cacheReader := json.NewJSONCacheReader(boltConfig, logger)

	// Mock vault scanner
	vaultScanner := &mockVaultScanner{vaultDir: vaultDir}

	// Create VaultIndexer
	indexer := vault.NewVaultIndexer(
		vaultScanner,
		boltWriter,
		sqliteWriter,
		cacheReader,
		markdownParser,
		fmService,
		schemaEngine,
		*config,
		logger,
		nil,
	)

	// Create QueryService
	queryService := query.NewQueryService(
		cacheReader,
		cacheReader,
		*config,
		logger,
		nil,
	)

	return &frontmatterTestEnv{

		vaultDir:     vaultDir,
		indexer:      indexer,
		queryService: queryService,
		logger:       logger,
		cleanup: func() {
			_ = os.RemoveAll(vaultDir)
		},
	}
}

// ScanAll implements VaultScannerPort.ScanAll.
func (m *mockVaultScanner) ScanAll(
	ctx context.Context,
) ([]dto.VaultFile, error) {
	var files []dto.VaultFile

	err := filepath.WalkDir(
		m.vaultDir,
		func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				return err
			}
			isMarkdownFile := !d.IsDir() && filepath.Ext(path) == ".md"
			if !isMarkdownFile {
				return nil
			}
			content, readErr := os.ReadFile(path)
			if readErr != nil {
				return readErr
			}

			info, infoErr := d.Info()
			if infoErr != nil {
				return infoErr
			}

			vaultFile, err := dto.NewVaultFile(path, m.vaultDir, info, content)
			if err != nil {
				return err
			}
			files = append(files, vaultFile)
			return nil
		},
	)

	return files, err
}

// ScanModified implements VaultScannerPort.ScanModified.
func (m *mockVaultScanner) ScanModified(
	ctx context.Context,
	since time.Time,
) ([]dto.VaultFile, error) {
	return m.ScanAll(ctx) // For simplicity, return all files
}

// TestFrontmatterWorkflow tests the complete frontmatter processing workflow.
func TestFrontmatterWorkflow(t *testing.T) {
	ctx := context.Background()
	env := setupFrontmatterTestEnvironment(t)
	defer env.cleanup()

	t.Run("complete frontmatter workflow", func(t *testing.T) {
		// Step 1: Build index (includes frontmatter processing)
		stats, err := env.indexer.Build(ctx)
		require.NoError(t, err)

		// Verify indexing stats
		assert.Positive(t, stats.ScannedCount, "should have scanned files")
		assert.Positive(t, stats.IndexedCount, "should have indexed notes")
		assert.Positive(
			t,
			stats.ValidationSuccesses,
			"should have validation successes",
		)
		assert.Equal(
			t,
			0,
			stats.ValidationFailures,
			"should have no validation failures",
		)

		// Step 2: RefreshFromCache removed (QueryService is read-only)
		// err = env.queryService.RefreshFromCache(ctx)
		// require.NoError(t, err)

		// Step 3: Query by frontmatter fields

		notes, err := env.queryService.FrontmatterQuery(
			ctx,
			"author",
			"Test Author",
		)
		require.NoError(t, err)
		assert.Len(t, notes, 1, "should find note by author")
		assert.Equal(t, "Test Note", notes[0].Frontmatter.Title())

		// Note: tags query removed as tags are now stored as string, not array

		notes, err = env.queryService.FrontmatterQuery(ctx, "fileClass", "note")
		require.NoError(t, err)
		assert.Len(t, notes, 1, "should find note by fileClass")
	})

	t.Run("frontmatter validation failure handling", func(t *testing.T) {
		// Create a file with invalid frontmatter
		invalidFile := filepath.Join(env.vaultDir, "invalid-note.md")
		invalidContent := `---
fileClass: "note"
---

# Invalid Note

Missing required title.
`
		err := os.WriteFile(invalidFile, []byte(invalidContent), 0o600)
		require.NoError(t, err)

		// Build index - should handle validation failure gracefully
		stats, err := env.indexer.Build(ctx)
		require.NoError(t, err)

		// Should have validation failure but continue indexing
		assert.Positive(
			t,
			stats.ValidationFailures,
			"should record validation failure",
		)
		assert.Positive(t, stats.IndexedCount, "should still index valid notes")
	})

	t.Run("frontmatter query performance", func(t *testing.T) {
		// Add more test notes for performance testing
		for i := range 10 {
			noteFile := filepath.Join(
				env.vaultDir,
				fmt.Sprintf("perf-note-%d.md", i),
			)
			noteContent := fmt.Sprintf(`---
fileClass: "note"
title: "Performance Note %d"
author: "Perf Author"
---

# Performance Note %d

This is performance test note %d.
`, i, i, i)
			err := os.WriteFile(noteFile, []byte(noteContent), 0o600)
			require.NoError(t, err)
		}

		// Re-index with additional notes
		stats, err := env.indexer.Build(ctx)
		require.NoError(t, err)
		assert.GreaterOrEqual(
			t,
			stats.IndexedCount,
			11,
			"should index all notes",
		)

		// RefreshFromCache removed
		// err = env.queryService.RefreshFromCache(ctx)
		// require.NoError(t, err)

		// Benchmark frontmatter query performance - AC10: <10% overhead vs

		// baseline query time
		start := time.Now()
		for range 100 {
			notes, queryErr := env.queryService.FrontmatterQuery(
				ctx,
				"author",
				"Perf Author",
			)
			require.NoError(t, queryErr)
			assert.GreaterOrEqual(
				t,
				len(notes),
				10,
				"should find performance notes",
			)
		}
		queryDuration := time.Since(start)

		// AC10: Query operations should be reasonably fast
		// Allow up to 1 second total for 100 queries (10ms per query average)
		maxAllowedDuration := 1 * time.Second
		assert.Less(
			t,
			queryDuration,
			maxAllowedDuration,
			"frontmatter queries should complete within 10ms, took %v for 100 queries",
			queryDuration,
		)

		t.Logf(
			"Query performance: %v total for 100 queries (%.2f µs per query)",
			queryDuration,
			float64(queryDuration.Nanoseconds())/100000.0,
		)
	})

	t.Run("frontmatter helper methods integration", func(t *testing.T) {
		// Create a test note with various frontmatter fields to test helpers
		testFile := filepath.Join(env.vaultDir, "helper-test-note.md")
		testContent := `---
fileClass: "note"
title: "Helper Methods Test"
aliases: ["helper test", "method test"]
tags: "test, helper, integration"
author: "Test Author"
---

# Helper Methods Test

This note tests the Frontmatter helper methods.
`
		err := os.WriteFile(testFile, []byte(testContent), 0o600)
		require.NoError(t, err)

		// Re-index to include the new note
		_, err = env.indexer.Build(ctx)
		require.NoError(t, err)

		// Query for the test note
		notes, err := env.queryService.FrontmatterQuery(
			ctx,
			"title",
			"Helper Methods Test",
		)
		require.NoError(t, err)
		require.Len(t, notes, 1, "should find the helper test note")

		note := notes[0]
		fm := note.Frontmatter

		// Verify the note was created with correct frontmatter using helper
		// methods
		assert.Equal(
			t,
			"Helper Methods Test",
			fm.Title(),
			"Title should be accessible via helper",
		)
		assert.Equal(
			t,
			[]string{"helper test", "method test"},
			fm.Aliases(),
			"Aliases should be accessible via helper",
		)

		// Test Get/Has methods
		title, exists := fm.Get("title")
		assert.True(t, exists, "title field should exist")
		assert.Equal(t, "Helper Methods Test", title)

		_, exists = fm.Get("nonexistent")
		assert.False(t, exists, "nonexistent field should not exist")

		assert.True(
			t,
			fm.Has("title"),
			"Has should return true for existing field",
		)
		assert.False(
			t,
			fm.Has("nonexistent"),
			"Has should return false for missing field",
		)

		// Test type inspectors
		assert.True(t, domain.Is[string](fm, "title"), "title should be string")
		assert.True(t, fm.IsArray("aliases"), "aliases should be array")
		assert.True(t, domain.Is[string](fm, "tags"), "tags should be string")
		assert.True(
			t,
			domain.Is[string](fm, "author"),
			"author should be string",
		)
		assert.False(
			t,
			domain.Is[map[string]any](fm, "fileClass"),
			"fileClass should not be map",
		)

		// Test delegation helpers
		assert.Equal(
			t,
			"note",
			fm.FileClass(),
			"FileClass should return fileClass",
		)
		assert.Equal(
			t,
			"Helper Methods Test",
			fm.Title(),
			"Title should return title",
		)
		assert.Equal(
			t,
			[]string{"helper test", "method test"},
			fm.Aliases(),
			"Aliases should return normalized array",
		)

		// Test with different alias formats
		testFile2 := filepath.Join(env.vaultDir, "alias-test-note.md")
		testContent2 := `---
fileClass: "note"
title: "Alias Test"
aliases: "single alias"
---

# Alias Test
`
		err = os.WriteFile(testFile2, []byte(testContent2), 0o600)
		require.NoError(t, err)

		// Re-index
		_, err = env.indexer.Build(ctx)
		require.NoError(t, err)

		// Query for the alias test note
		notes, err = env.queryService.FrontmatterQuery(
			ctx,
			"title",
			"Alias Test",
		)
		require.NoError(t, err)
		require.Len(t, notes, 1)

		note2 := notes[0]
		assert.Equal(
			t,
			[]string{"single alias"},
			note2.Frontmatter.Aliases(),
			"single string alias should be normalized to array",
		)
	})
}
