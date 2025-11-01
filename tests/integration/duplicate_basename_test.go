package integration

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache"
	vaultadapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/app/query"
	vaultindexer "github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestDuplicateBasenameHandling_Integration tests that files with identical
// basenames in different directories generate unique NoteIDs and cache entries.
func TestDuplicateBasenameHandling_Integration(t *testing.T) {
	// Setup test environment
	tempDir := t.TempDir()
	vaultDir := filepath.Join(tempDir, "vault")
	cacheDir := filepath.Join(tempDir, "cache")

	// Create test vault structure with duplicate basenames
	createDuplicateBasenameVault(t, vaultDir)

	// Setup dependencies
	config := domain.Config{
		VaultPath: vaultDir,
		CacheDir:  cacheDir,
	}
	logger := zerolog.New(nil).Level(zerolog.Disabled)

	// Create vault scanner (VaultReaderAdapter implements VaultScannerPort)
	vaultReader := vaultadapter.NewVaultReaderAdapter(config, logger)

	// Create cache writer
	cacheWriter := cache.NewJSONCacheWriter(config, logger)

	// Create indexer (without frontmatter/schema for simplicity)
	indexer := vaultindexer.NewVaultIndexer(
		vaultReader,
		cacheWriter,
		nil,
		nil,
		config,
		logger,
	)

	// Create query service
	cacheReader := cache.NewJSONCacheReader(config, logger)
	queryService := query.NewQueryService(cacheReader, logger)

	// Execute indexing
	ctx := context.Background()
	_, err := indexer.Build(ctx)
	require.NoError(t, err, "Indexing should succeed")

	// Refresh query service
	err = queryService.RefreshFromCache(ctx)
	require.NoError(t, err, "Query service refresh should succeed")

	// Verify unique NoteIDs
	note1, err := queryService.ByID(ctx, domain.NoteID("projects/meeting.md"))
	require.NoError(t, err, "Should find note with projects path")
	assert.Equal(t, domain.NoteID("projects/meeting.md"), note1.ID)

	note2, err := queryService.ByID(ctx, domain.NoteID("ideas/meeting.md"))
	require.NoError(t, err, "Should find note with ideas path")
	assert.Equal(t, domain.NoteID("ideas/meeting.md"), note2.ID)

	// Verify unique cache files exist
	cacheFile1 := filepath.Join(cacheDir, "projects-meeting.md.json")
	cacheFile2 := filepath.Join(cacheDir, "ideas-meeting.md.json")

	assert.FileExists(
		t,
		cacheFile1,
		"Cache file for projects/meeting.md should exist",
	)
	assert.FileExists(
		t,
		cacheFile2,
		"Cache file for ideas/meeting.md should exist",
	)

	// Verify byBasename returns both notes
	notes, err := queryService.ByBasename(ctx, "meeting")
	require.NoError(t, err, "ByBasename should succeed")
	assert.Len(t, notes, 2, "ByBasename should return 2 notes for 'meeting'")
}

// createDuplicateBasenameVault creates a test vault with files having identical
// basenames in different directories.
func createDuplicateBasenameVault(t *testing.T, vaultDir string) {
	t.Helper()

	// Create directory structure
	require.NoError(
		t,
		os.MkdirAll(filepath.Join(vaultDir, "projects"), 0o755),
	) // #nosec G301
	require.NoError(
		t,
		os.MkdirAll(filepath.Join(vaultDir, "ideas"), 0o755),
	) // #nosec G301

	// Create files with same basename but different content
	projectContent := `---
fileClass: meeting
---

# Project Meeting

This is content from the projects directory.
`

	ideaContent := `---
fileClass: meeting
---

# Idea Meeting

This is content from the ideas directory.
`

	require.NoError(t, os.WriteFile( // #nosec G306 - test file
		filepath.Join(vaultDir, "projects", "meeting.md"),
		[]byte(projectContent),
		0o644,
	))

	require.NoError(t, os.WriteFile( // #nosec G306 - test file
		filepath.Join(vaultDir, "ideas", "meeting.md"),
		[]byte(ideaContent),
		0o644,
	))
}
