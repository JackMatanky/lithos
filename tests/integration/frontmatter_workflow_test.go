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

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache"
	schemaadapter "github.com/JackMatanky/lithos/internal/adapters/spi/schema"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/query"
	schemaengine "github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/dto"
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
	)
	require.NoError(t, err)

	// Create frontmatter service
	fmService := frontmatter.NewFrontmatterService(schemaEngine, logger)

	// Create cache adapters
	cacheWriter := cache.NewJSONCacheWriter(*config, logger)
	cacheReader := cache.NewJSONCacheReader(*config, logger)

	// Mock vault scanner
	vaultScanner := &mockVaultScanner{vaultDir: vaultDir}

	// Create VaultIndexer
	indexer := vault.NewVaultIndexer(
		vaultScanner,
		cacheWriter,
		fmService,
		schemaEngine,
		*config,
		logger,
	)

	// Create QueryService
	queryService := query.NewQueryService(cacheReader, logger)

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

			relPath, relErr := filepath.Rel(m.vaultDir, path)
			if relErr != nil {
				return relErr
			}

			info, infoErr := d.Info()
			if infoErr != nil {
				return infoErr
			}

			files = append(files, dto.VaultFile{
				FileMetadata: dto.NewFileMetadata(path, info),
				Content:      content,
			})
			// Override the Path to be relative
			files[len(files)-1].Path = relPath
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

		// Step 2: Refresh query service from cache
		err = env.queryService.RefreshFromCache(ctx)
		require.NoError(t, err)

		// Step 3: Query by frontmatter fields
		notes, err := env.queryService.ByFrontmatter(
			ctx,
			"author",
			"Test Author",
		)
		require.NoError(t, err)
		assert.Len(t, notes, 1, "should find note by author")
		assert.Equal(t, "Test Note", notes[0].Frontmatter.Fields["title"])

		// Note: tags query removed as tags are now stored as string, not array

		notes, err = env.queryService.ByFrontmatter(ctx, "fileClass", "note")
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

		// Refresh query service
		err = env.queryService.RefreshFromCache(ctx)
		require.NoError(t, err)

		// Benchmark frontmatter query performance - AC10: <10% overhead vs
		// baseline query time
		start := time.Now()
		for range 100 {
			notes, queryErr := env.queryService.ByFrontmatter(
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
}
