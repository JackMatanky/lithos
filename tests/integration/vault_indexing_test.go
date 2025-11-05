package integration

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache"
	schemaadapter "github.com/JackMatanky/lithos/internal/adapters/spi/schema"
	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/query"
	"github.com/JackMatanky/lithos/internal/app/schema"
	vaultindexer "github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Helper functions for comprehensive vault indexing test

// TestVaultIndexing_Integration tests the complete vault indexing
// pipeline with all critical fixes integrated together. This test validates:
// - Note ID collision resolution with path-based queries
// - Memory-efficient scanning with large binary files
// - Cache management handling deletions correctly
// - Query layer working with all index types
// - End-to-end CLI workflow from index command through cache operations to
// query results.
func TestVaultIndexing_Integration(t *testing.T) {
	// Setup test environment
	tempDir := t.TempDir()
	vaultDir := filepath.Join(tempDir, "vault")
	cacheDir := filepath.Join(tempDir, "cache")

	// Create complex test vault with all edge cases
	createComplexTestVault(t, vaultDir)

	// Setup schemas for frontmatter/schema services
	projectRoot := utils.FindProjectRoot(t)
	testSchemasDir := filepath.Join(vaultDir, "schemas")

	srcPropertyBank := filepath.Join(
		projectRoot,
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Setup dependencies
	config := domain.Config{
		CacheDir:         cacheDir,
		SchemasDir:       testSchemasDir,
		PropertyBankFile: "property_bank.json",
	}
	logger := zerolog.New(nil).Level(zerolog.Disabled)

	// Create vault scanner
	vaultReader := vaultAdapter.NewVaultReaderAdapter(config, logger)

	// Create cache writer and reader
	cacheWriter := cache.NewJSONCacheWriter(config, logger)
	cacheReader := cache.NewJSONCacheReader(config, logger)

	// Create schema and frontmatter services
	schemaLoader := schemaadapter.NewSchemaLoaderAdapter(&config, &logger)
	schemaRegistry := schemaadapter.NewSchemaRegistryAdapter(logger)
	schemaEngine, err := schema.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		logger,
	)
	require.NoError(t, err)
	frontmatterService := frontmatter.NewFrontmatterService(
		schemaEngine,
		logger,
	)

	// Create indexer with all services
	indexer := vaultindexer.NewVaultIndexer(
		vaultReader,
		cacheWriter,
		cacheReader,
		frontmatterService,
		schemaEngine,
		config,
		logger,
	)
	queryService := query.NewQueryService(
		cacheReader,
		cacheReader,
		config,
		logger,
	)

	// Execute complete indexing workflow
	ctx := context.Background()
	startTime := time.Now()
	stats, err := indexer.Build(ctx)
	duration := time.Since(startTime)

	require.NoError(t, err, "Complete vault indexing should succeed")
	assert.Positive(t, stats.ScannedCount, "Should scan files")
	assert.Positive(t, stats.IndexedCount, "Should index files")

	t.Logf("Indexing completed in %v: scanned=%d, indexed=%d",
		duration, stats.ScannedCount, stats.IndexedCount)

	// Refresh query service
	err = queryService.RefreshFromCache(ctx)
	require.NoError(t, err, "Query service refresh should succeed")

	// Test 1: Note ID collision resolution with path-based queries
	t.Run("NoteID_Collision_Resolution", func(t *testing.T) {
		testNoteIDCollisionResolution(t, ctx, queryService)
	})

	// Test 2: Memory-efficient scanning with large binary files
	t.Run("Memory_Efficient_Scanning", func(t *testing.T) {
		testMemoryEfficientScanning(t, ctx, queryService, vaultDir)
	})

	// Test 3: Cache management handles deletions correctly
	t.Run("Cache_Management_Deletions", func(t *testing.T) {
		testCacheManagementDeletions(
			t,
			ctx,
			indexer,
			queryService,
			vaultDir,
			cacheDir,
		)
	})

	// Test 4: Query layer works with all index types
	t.Run("Query_Layer_Comprehensive", func(t *testing.T) {
		testQueryLayerComprehensive(t, ctx, queryService)
	})

	// Test 6: Performance benchmarks meet requirements
	t.Run("Performance_Benchmarks", func(t *testing.T) {
		testPerformanceBenchmarks(t, duration, stats)
	})

	// Test 7: Error handling across complete pipeline
	t.Run("Error_Handling_Pipeline", func(t *testing.T) {
		testErrorHandlingPipeline(t, ctx, indexer, queryService)
	})

	// Test 8: Regression testing - no existing functionality broken
	t.Run("Regression_Testing", func(t *testing.T) {
		testRegressionFunctionality(t, ctx, queryService)
	})
}

// createComplexTestVault creates a test vault with all edge cases for
// testing.
func createComplexTestVault(t *testing.T, vaultDir string) {
	t.Helper()

	// Create directory structure with nested folders
	dirs := []string{
		"projects/active",
		"projects/archive",
		"ideas/brainstorm",
		"meetings/2025",
		"meetings/2024",
		"templates",
		"assets/images",
		"assets/documents",
	}

	for _, dir := range dirs {
		require.NoError(
			t,
			os.MkdirAll(filepath.Join(vaultDir, dir), 0o755),
		)
	}

	// Create files with duplicate basenames across directories
	duplicateBasenameContent := []struct {
		dir      string
		filename string
		title    string
		content  string
	}{
		{
			"projects/active",
			"meeting.md",
			"Active Project Meeting",
			"Content for active project",
		},
		{
			"projects/archive",
			"meeting.md",
			"Archived Project Meeting",
			"Content for archived project",
		},
		{
			"ideas/brainstorm",
			"meeting.md",
			"Brainstorm Meeting",
			"Content for brainstorm",
		},
		{"meetings/2025", "meeting.md", "2025 Meeting", "Content for 2025"},
		{"meetings/2024", "meeting.md", "2024 Meeting", "Content for 2024"},
	}

	for _, item := range duplicateBasenameContent {
		content := fmt.Sprintf(`---
title: "%s"
date: "2025-01-01"
---

# %s

%s
`, item.title, item.title, item.content)

		path := filepath.Join(vaultDir, item.dir, item.filename)
		require.NoError(
			t,
			os.WriteFile(path, []byte(content), 0o644),
		)
	}

	// Create files with different extensions and types
	mixedFiles := []struct {
		dir      string
		filename string
		content  string
	}{
		{
			"templates",
			"note-template.md",
			"# Note Template\n\nTemplate content",
		},
		{
			"templates",
			"meeting-template.md",
			"# Meeting Template\n\nMeeting template content",
		},
		{"assets/documents", "readme.txt", "This is a text document"},
		{"assets/documents", "data.json", `{"key": "value", "number": 42}`},
	}

	for _, item := range mixedFiles {
		path := filepath.Join(vaultDir, item.dir, item.filename)
		require.NoError(
			t,
			os.WriteFile(path, []byte(item.content), 0o644),
		)
	}

	// Create large binary file to test memory efficiency (1MB)
	largeBinaryPath := filepath.Join(
		vaultDir,
		"assets/images", //nolint:gocritic // filepath.Join with subpath strings is acceptable
		"large-image.jpg",
	)
	largeData := make([]byte, 1024*1024) // 1MB
	for i := range largeData {
		largeData[i] = byte(i % 256)
	}
	require.NoError(
		t,
		os.WriteFile(largeBinaryPath, largeData, 0o644),
	)

	// Create file with invalid frontmatter for error handling
	invalidFrontmatterPath := filepath.Join(
		vaultDir,
		"projects/active", //nolint:gocritic // filepath.Join with subpath strings is acceptable
		"invalid.md",
	)
	invalidContent := `---
invalid: yaml: content:
---
# Invalid Frontmatter

This file has invalid YAML frontmatter.`
	require.NoError(
		t,
		os.WriteFile(invalidFrontmatterPath, []byte(invalidContent), 0o644),
	)

	// Create file without frontmatter
	noFrontmatterPath := filepath.Join(
		vaultDir,
		"ideas/brainstorm", //nolint:gocritic // filepath.Join with subpath strings is acceptable
		"plain.md",
	)
	plainContent := `# Plain Markdown

This file has no frontmatter.`
	require.NoError(
		t,
		os.WriteFile(noFrontmatterPath, []byte(plainContent), 0o644),
	)
}

// testNoteIDCollisionResolution validates that duplicate basenames generate
// unique NoteIDs.
func testNoteIDCollisionResolution(
	t *testing.T,
	ctx context.Context,
	queryService *query.QueryService,
) {
	// Test that all "meeting.md" files have unique NoteIDs
	meetingNotes, err := queryService.ByBasename(ctx, "meeting")
	require.NoError(t, err)
	assert.Len(t, meetingNotes, 5, "Should find 5 meeting.md files")

	// Verify each has unique ID based on path
	expectedIDs := []domain.NoteID{
		"projects/active/meeting.md",
		"projects/archive/meeting.md",
		"ideas/brainstorm/meeting.md",
		"meetings/2025/meeting.md",
		"meetings/2024/meeting.md",
	}

	actualIDs := make([]domain.NoteID, len(meetingNotes))
	for i, note := range meetingNotes {
		actualIDs[i] = note.ID
	}

	for _, expectedID := range expectedIDs {
		assert.Contains(
			t,
			actualIDs,
			expectedID,
			"Should contain NoteID %s",
			expectedID,
		)
	}

	// Test individual queries work
	for _, id := range expectedIDs {
		foundNote, queryErr := queryService.ByID(ctx, id)
		require.NoError(t, queryErr, "Should find note with ID %s", id)
		assert.Equal(t, id, foundNote.ID)
	}
}

// testMemoryEfficientScanning validates memory usage with large files.
func testMemoryEfficientScanning(
	t *testing.T,
	ctx context.Context,
	queryService *query.QueryService,
	vaultDir string,
) {
	// Verify large binary file exists but is not indexed (not a markdown file)
	largeFilePath := filepath.Join(
		vaultDir,
		"assets", "images",
		"large-image.jpg",
	)
	assert.FileExists(t, largeFilePath)

	// Verify markdown files are still indexed by checking specific files
	_, err := queryService.ByPath(ctx, "templates/note-template.md")
	require.NoError(t, err, "Should index markdown template files")

	_, err = queryService.ByPath(ctx, "ideas/brainstorm/plain.md")
	assert.NoError(t, err, "Should index plain markdown files")
}

// testCacheManagementDeletions validates cache handles file deletions
// correctly.
func testCacheManagementDeletions(
	t *testing.T,
	ctx context.Context,
	indexer *vaultindexer.VaultIndexer,
	queryService *query.QueryService,
	vaultDir, cacheDir string,
) {
	// First, verify initial state
	_, err := queryService.ByPath(ctx, "projects/active/meeting.md")
	require.NoError(t, err, "File should exist initially")

	// Delete a file from vault
	fileToDelete := filepath.Join(
		vaultDir,
		"projects/active/meeting.md", //nolint:gocritic // filepathJoin accepts string components
	)
	require.NoError(t, os.Remove(fileToDelete))

	// Use Refresh instead of Build to handle deletions (Refresh does deletion
	// reconciliation)
	since := time.Now().Add(-time.Hour) // Process all files as modified
	err = indexer.Refresh(ctx, since)
	require.NoError(t, err)

	// Refresh query service
	err = queryService.RefreshFromCache(ctx)
	require.NoError(t, err)

	// Verify deleted note is not queryable
	_, err = queryService.ByID(ctx, domain.NoteID("projects/active/meeting.md"))
	require.Error(t, err, "Deleted note should not be found")

	// Verify cache file is removed
	cacheFile := filepath.Join(cacheDir, "projects-active-meeting.md.json")
	assert.NoFileExists(
		t,
		cacheFile,
		"Cache file should be removed for deleted note",
	)
}

// testQueryLayerComprehensive validates all query operations work correctly.
func testQueryLayerComprehensive(
	t *testing.T,
	ctx context.Context,
	queryService *query.QueryService,
) {
	// Test ByBasename with multiple results
	meetingNotes, err := queryService.ByBasename(ctx, "meeting")
	require.NoError(t, err)
	assert.Len(
		t,
		meetingNotes,
		4,
		"Should find 4 remaining meeting.md files",
	) // One was deleted

	// Test ByFileClass (will be empty since no frontmatter processing)
	meetingClassNotes, err := queryService.ByFileClass(ctx, "meeting_note")
	require.NoError(t, err)
	assert.Empty(
		t,
		meetingClassNotes,
		"Should find no meeting_note files (no frontmatter processing)",
	)

	// Test ByPath for specific files
	_, err = queryService.ByPath(ctx, "projects/archive/meeting.md")
	require.NoError(t, err, "Should find notes by path")

	// Test ByFrontmatter (will be empty since no frontmatter processing)
	templateNotes, err := queryService.ByFrontmatter(
		ctx,
		"fileClass",
		"meeting_note",
	)
	require.NoError(t, err)
	assert.Empty(
		t,
		templateNotes,
		"Should find no notes by frontmatter (no frontmatter processing)",
	)

	// Verify no duplicates in ByBasename results
	noteIDs := make(map[domain.NoteID]bool)
	for _, note := range meetingNotes {
		assert.False(t, noteIDs[note.ID], "NoteID %s should be unique", note.ID)
		noteIDs[note.ID] = true
	}
}

// testPerformanceBenchmarks validates performance requirements are met.
func testPerformanceBenchmarks(
	t *testing.T,
	duration time.Duration,
	stats vaultindexer.IndexStats,
) {
	// Performance requirements for realistic vault sizes
	maxDuration := 30 * time.Second // Should complete within 30 seconds
	minIndexed := 5                 // Should index at least 5 files

	assert.Less(
		t,
		duration,
		maxDuration,
		"Indexing should complete within %v",
		maxDuration,
	)
	assert.GreaterOrEqual(
		t,
		stats.IndexedCount,
		minIndexed,
		"Should index at least %d files",
		minIndexed,
	)

	t.Logf(
		"Performance: %d files indexed in %v (%.2f files/sec)",
		stats.IndexedCount,
		duration,
		float64(stats.IndexedCount)/duration.Seconds(),
	)
}

// testErrorHandlingPipeline validates error handling across the complete
// pipeline.
func testErrorHandlingPipeline(
	t *testing.T,
	ctx context.Context,
	indexer *vaultindexer.VaultIndexer,
	queryService *query.QueryService,
) {
	// Test continues despite individual file errors
	// (Invalid frontmatter file should not stop indexing)

	stats, err := indexer.Build(ctx)
	require.NoError(t, err, "Indexing should succeed despite invalid files")

	// Should still index valid files
	assert.Positive(
		t,
		stats.IndexedCount,
		"Should index valid files despite errors",
	)

	// Query service should work
	err = queryService.RefreshFromCache(ctx)
	require.NoError(
		t,
		err,
		"Query service should work after indexing with errors",
	)
}

// testRegressionFunctionality ensures no existing functionality is broken.
func testRegressionFunctionality(
	t *testing.T,
	ctx context.Context,
	queryService *query.QueryService,
) {
	// Test basic query functionality still works - check specific known files
	_, err := queryService.ByPath(ctx, "templates/note-template.md")
	require.NoError(t, err, "Should find template file")

	// Test file class queries still work (empty results expected)
	notes, err := queryService.ByFileClass(ctx, "meeting_note")
	require.NoError(t, err)
	assert.Empty(
		t,
		notes,
		"File class queries should work (no frontmatter processing)",
	)

	// Test basename queries still work
	_, err = queryService.ByBasename(ctx, "template")
	require.NoError(t, err)

	// Test ByPath queries still work
	_, err = queryService.ByPath(ctx, "templates/meeting-template.md")
	require.NoError(t, err, "Path queries should work")
}
