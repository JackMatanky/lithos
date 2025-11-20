package integration

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/boltdb"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestBoltDBCacheIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "fileClass",
	}
	log := zerolog.Nop()

	// 1. Create Writer and persist notes
	writer, err := boltdb.NewBoltDBCacheWriter(config, log)
	require.NoError(t, err)

	notes := []domain.Note{
		{
			ID: domain.NewNoteID("notes/alpha.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "project",
				Fields: map[string]interface{}{
					"title":     "Alpha Project",
					"fileClass": "project",
					"aliases":   []interface{}{"Alpha", "Project A"},
				},
			},
		},
		{
			ID: domain.NewNoteID("notes/beta.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "contact",
				Fields: map[string]interface{}{
					"title":     "Beta Contact",
					"fileClass": "contact",
				},
			},
		},
	}

	ctx := context.Background()
	for _, n := range notes {
		persistErr := writer.Persist(ctx, n)
		require.NoError(t, persistErr)
	}
	err = writer.Close()
	require.NoError(t, err)

	// 2. Create Reader and verify data
	reader, err := boltdb.NewBoltDBCacheReadAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// Verify Read by ID
	note, err := reader.Read(ctx, domain.NewNoteID("notes/alpha.md"))
	require.NoError(t, err)
	assert.Equal(t, "Alpha Project", note.Frontmatter.Fields["title"])

	// Verify Metadata Queries
	// ByFileClass
	projectNotes, err := reader.ByFileClass(ctx, "project")
	require.NoError(t, err)
	assert.Len(t, projectNotes, 1)
	assert.Equal(t, "notes/alpha.md", string(projectNotes[0].ID))

	// ByAlias
	aliasNotes, err := reader.ByAlias(ctx, "Project A")
	require.NoError(t, err)
	assert.Len(t, aliasNotes, 1)
	assert.Equal(t, "notes/alpha.md", string(aliasNotes[0].ID))

	// ByBasename
	basenameNotes, err := reader.ByBasename(ctx, "beta")
	require.NoError(t, err)
	assert.Len(t, basenameNotes, 1)
	assert.Equal(t, "notes/beta.md", string(basenameNotes[0].ID))

	// 3. Verify Staleness (Basic check)
	// Since file doesn't exist on disk, should be stale
	stale, err := reader.IsStale(ctx, "notes/alpha.md")
	require.NoError(t, err)
	assert.True(t, stale)
}
