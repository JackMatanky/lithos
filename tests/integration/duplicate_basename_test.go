package integration

import (
	"context"
	"encoding/json"
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

	// Create cache reader
	cacheReader := cache.NewJSONCacheReader(config, logger)

	// Create indexer (without frontmatter/schema for simplicity)
	indexer := vaultindexer.NewVaultIndexer(
		vaultReader,
		cacheWriter,
		cacheReader,
		nil,
		nil,
		config,
		logger,
	)
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

	// Verify unique cache entries exist with correct NoteIDs
	cacheFiles, globErr := filepath.Glob(filepath.Join(cacheDir, "*.json"))
	require.NoError(t, globErr, "Should list cache files")
	assert.Len(t, cacheFiles, 2, "Cache should contain exactly two entries")

	foundIDs := make(map[domain.NoteID]bool)
	for _, path := range cacheFiles {
		content, readErr := os.ReadFile(path)
		require.NoErrorf(t, readErr, "Should read cache file %s", path)

		var note domain.Note
		require.NoErrorf(
			t,
			json.Unmarshal(content, &note),
			"Should unmarshal cache file %s",
			path,
		)
		foundIDs[note.ID] = true
	}

	assert.True(
		t,
		foundIDs[domain.NoteID("projects/meeting.md")],
		"Cache should contain projects/meeting.md",
	)
	assert.True(
		t,
		foundIDs[domain.NoteID("ideas/meeting.md")],
		"Cache should contain ideas/meeting.md",
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
