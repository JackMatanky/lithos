package vault

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestVaultIndexer_Build_NoFiles tests indexing with no files.
func TestVaultIndexer_Build_NoFiles(t *testing.T) {
	// Setup mocks
	vaultScanner := utils.NewMockVaultScannerPort()
	vaultScanner.SetScanAllResult([]dto.VaultFile{}, nil)

	boltWriter := utils.NewMockCacheWriterPort()
	sqliteWriter := utils.NewMockCacheWriterPort()
	cacheReader := utils.NewMockCacheReaderPort()
	markdownParser := utils.NewMockMarkdownParserPort()
	eventBus := utils.NewMockEventBus()

	logger := zerolog.Nop()
	config := domain.Config{VaultPath: "/tmp/test"}

	// Create indexer (nil for services not being tested)
	indexer := NewVaultIndexer(
		vaultScanner,
		boltWriter,
		sqliteWriter,
		cacheReader,
		markdownParser,
		nil, // frontmatterService
		nil, // schemaEngine
		config,
		logger,
		eventBus,
	)

	// Execute
	ctx := context.Background()
	stats, err := indexer.Build(ctx)

	// Assert
	require.NoError(t, err)
	assert.Equal(t, 0, stats.ScannedCount)
	assert.Equal(t, 0, stats.IndexedCount)
	assert.Equal(t, 0, stats.ParseFailures)
	assert.Equal(t, 0, stats.CacheFailures)
	assert.Equal(t, 0, stats.ValidationSuccesses)
	assert.Equal(t, 0, stats.ValidationFailures)
	assert.Greater(t, stats.Duration, time.Duration(0))
}

// TestVaultIndexer_Build_WithFiles tests indexing with markdown files.
func TestVaultIndexer_Build_WithFiles(t *testing.T) {
	// Setup test data
	content := []byte("---\nfileClass: note\ntitle: Test\n---\n\nContent")
	testFile, err := dto.NewVaultFile(
		"/tmp/test/test.md",
		"/tmp/test",
		nil,
		content,
	)
	require.NoError(t, err)

	vaultScanner := utils.NewMockVaultScannerPort()
	vaultScanner.SetScanAllResult([]dto.VaultFile{testFile}, nil)

	markdownParser := utils.NewMockMarkdownParserPort()
	markdownParser.SetParseResult(map[string]any{
		"fileClass": "note",
		"title":     "Test",
	}, nil)

	boltWriter := utils.NewMockCacheWriterPort()
	sqliteWriter := utils.NewMockCacheWriterPort()
	cacheReader := utils.NewMockCacheReaderPort()
	eventBus := utils.NewMockEventBus()

	logger := zerolog.Nop()
	config := domain.Config{VaultPath: "/tmp/test"}

	// Create indexer
	indexer := NewVaultIndexer(
		vaultScanner,
		boltWriter,
		sqliteWriter,
		cacheReader,
		markdownParser,
		nil, // frontmatterService
		nil, // schemaEngine
		config,
		logger,
		eventBus,
	)

	// Execute
	ctx := context.Background()
	stats, err := indexer.Build(ctx)

	// Assert
	require.NoError(t, err)
	assert.Equal(t, 1, stats.ScannedCount)
	assert.Equal(t, 1, stats.IndexedCount)
	assert.Equal(t, 0, stats.ParseFailures)
	assert.Equal(t, 0, stats.CacheFailures)
	assert.Equal(
		t,
		0,
		stats.ValidationSuccesses,
	) // No validation service configured
	assert.Equal(t, 0, stats.ValidationFailures)
	assert.Greater(t, stats.Duration, time.Duration(0))
}

// TestVaultIndexer_Build_ScanFailure tests that vault scan failures abort
// indexing.
func TestVaultIndexer_Build_ScanFailure(t *testing.T) {
	// Setup mocks
	vaultScanner := utils.NewMockVaultScannerPort()
	vaultScanner.SetScanAllResult(nil, assert.AnError) // Scan fails

	boltWriter := utils.NewMockCacheWriterPort()
	sqliteWriter := utils.NewMockCacheWriterPort()
	cacheReader := utils.NewMockCacheReaderPort()
	markdownParser := utils.NewMockMarkdownParserPort()
	eventBus := utils.NewMockEventBus()

	logger := zerolog.Nop()
	config := domain.Config{VaultPath: "/tmp/test"}

	// Create indexer
	indexer := NewVaultIndexer(
		vaultScanner,
		boltWriter,
		sqliteWriter,
		cacheReader,
		markdownParser,
		nil, // frontmatterService
		nil, // schemaEngine
		config,
		logger,
		eventBus,
	)

	// Execute
	ctx := context.Background()
	stats, err := indexer.Build(ctx)

	// Assert
	require.Error(t, err)
	assert.Equal(t, 0, stats.ScannedCount) // Should not proceed to processing
}

// TestVaultIndexer_Build_ParseFailure tests handling of markdown parsing
// failures.
func TestVaultIndexer_Build_ParseFailure(t *testing.T) {
	// Setup test data
	content := []byte("invalid content")
	testFile, err := dto.NewVaultFile(
		"/tmp/test/invalid.md",
		"/tmp/test",
		nil,
		content,
	)
	require.NoError(t, err)

	vaultScanner := utils.NewMockVaultScannerPort()
	vaultScanner.SetScanAllResult([]dto.VaultFile{testFile}, nil)

	markdownParser := utils.NewMockMarkdownParserPort()
	markdownParser.SetParseResult(nil, assert.AnError) // Parse fails

	boltWriter := utils.NewMockCacheWriterPort()
	sqliteWriter := utils.NewMockCacheWriterPort()
	cacheReader := utils.NewMockCacheReaderPort()
	eventBus := utils.NewMockEventBus()

	logger := zerolog.Nop()
	config := domain.Config{VaultPath: "/tmp/test"}

	// Create indexer
	indexer := NewVaultIndexer(
		vaultScanner,
		boltWriter,
		sqliteWriter,
		cacheReader,
		markdownParser,
		nil, // frontmatterService
		nil, // schemaEngine
		config,
		logger,
		eventBus,
	)

	// Execute
	ctx := context.Background()
	stats, err := indexer.Build(ctx)

	// Assert
	require.NoError(t, err) // Parse failures don't abort indexing
	assert.Equal(t, 1, stats.ScannedCount)
	assert.Equal(t, 0, stats.IndexedCount) // Failed to parse, so not indexed
	assert.Equal(t, 1, stats.ParseFailures)
	assert.Equal(t, 0, stats.CacheFailures)
}

// TestVaultIndexer_Build_CacheFailure tests handling of cache write failures.
func TestVaultIndexer_Build_CacheFailure(t *testing.T) {
	// Setup test data
	content := []byte("---\nfileClass: note\ntitle: Test\n---\n\nContent")
	testFile, err := dto.NewVaultFile(
		"/tmp/test/test.md",
		"/tmp/test",
		nil,
		content,
	)
	require.NoError(t, err)

	vaultScanner := utils.NewMockVaultScannerPort()
	vaultScanner.SetScanAllResult([]dto.VaultFile{testFile}, nil)

	markdownParser := utils.NewMockMarkdownParserPort()
	markdownParser.SetParseResult(map[string]any{
		"fileClass": "note",
		"title":     "Test",
	}, nil)

	// Setup cache writer to fail
	boltWriter := utils.NewMockCacheWriterPort()
	boltWriter.SetPersistResult(assert.AnError)

	sqliteWriter := utils.NewMockCacheWriterPort()
	cacheReader := utils.NewMockCacheReaderPort()
	eventBus := utils.NewMockEventBus()

	logger := zerolog.Nop()
	config := domain.Config{VaultPath: "/tmp/test"}

	// Create indexer
	indexer := NewVaultIndexer(
		vaultScanner,
		boltWriter,
		sqliteWriter,
		cacheReader,
		markdownParser,
		nil, // frontmatterService
		nil, // schemaEngine
		config,
		logger,
		eventBus,
	)

	// Execute
	ctx := context.Background()
	stats, err := indexer.Build(ctx)

	// Assert
	require.NoError(t, err) // Cache failures don't abort indexing
	assert.Equal(t, 1, stats.ScannedCount)
	assert.Equal(t, 1, stats.IndexedCount) // AddWrite succeeded, so indexed
	assert.Equal(t, 0, stats.ParseFailures)
	assert.Equal(t, 1, stats.CacheFailures) // Commit failed
}

// TestVaultIndexer_Build_EventPublishing tests that events are published during
// indexing.
func TestVaultIndexer_Build_EventPublishing(t *testing.T) {
	// Setup test data
	content := []byte("---\nfileClass: note\ntitle: Test\n---\n\nContent")
	testFile, err := dto.NewVaultFile(
		"/tmp/test/test.md",
		"/tmp/test",
		nil,
		content,
	)
	require.NoError(t, err)

	vaultScanner := utils.NewMockVaultScannerPort()
	vaultScanner.SetScanAllResult([]dto.VaultFile{testFile}, nil)

	markdownParser := utils.NewMockMarkdownParserPort()
	markdownParser.SetParseResult(map[string]any{
		"fileClass": "note",
		"title":     "Test",
	}, nil)

	boltWriter := utils.NewMockCacheWriterPort()
	sqliteWriter := utils.NewMockCacheWriterPort()
	cacheReader := utils.NewMockCacheReaderPort()
	eventBus := utils.NewMockEventBus()

	logger := zerolog.Nop()
	config := domain.Config{VaultPath: "/tmp/test"}

	// Create indexer
	indexer := NewVaultIndexer(
		vaultScanner,
		boltWriter,
		sqliteWriter,
		cacheReader,
		markdownParser,
		nil, // frontmatterService
		nil, // schemaEngine
		config,
		logger,
		eventBus,
	)

	// Execute
	ctx := context.Background()
	_, err = indexer.Build(ctx)

	// Assert
	require.NoError(t, err)
	publishedEvents := eventBus.GetPublishedEvents()
	assert.NotEmpty(t, publishedEvents, "Should publish events during indexing")

	// Check for NoteIndexedEvent
	foundNoteIndexed := false
	foundIndexingComplete := false
	for _, event := range publishedEvents {
		switch event.EventType() {
		case "NoteIndexed":
			foundNoteIndexed = true
		case "VaultIndexingComplete":
			foundIndexingComplete = true
		}
	}
	assert.True(t, foundNoteIndexed, "Should publish NoteIndexedEvent")
	assert.True(
		t,
		foundIndexingComplete,
		"Should publish VaultIndexingCompleteEvent",
	)
}

// TestIndexStats tests the statistics structure.
func TestIndexStats(t *testing.T) {
	stats := IndexStats{
		ScannedCount:        10,
		IndexedCount:        8,
		ParseFailures:       1,
		CacheFailures:       1,
		ValidationSuccesses: 7,
		ValidationFailures:  2,
		Duration:            1500000000, // 1.5 seconds in nanoseconds
	}

	assert.Equal(t, 10, stats.ScannedCount, "Should track scanned files")
	assert.Equal(t, 8, stats.IndexedCount, "Should track indexed files")
	assert.Equal(t, 1, stats.ParseFailures, "Should track parse failures")
	assert.Equal(t, 1, stats.CacheFailures, "Should track cache failures")
	assert.Equal(
		t,
		7,
		stats.ValidationSuccesses,
		"Should track validation successes",
	)
	assert.Equal(
		t,
		2,
		stats.ValidationFailures,
		"Should track validation failures",
	)
	assert.Positive(t, stats.Duration, "Should track duration")
}

// TestMarkdownExtConstant tests the markdown extension constant.
func TestMarkdownExtConstant(t *testing.T) {
	assert.Equal(t, ".md", markdownExt, "Markdown extension should be .md")
}
