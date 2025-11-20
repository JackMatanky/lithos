package integration

import (
	"context"
	"database/sql"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/sqlite"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSQLiteCacheIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir: cacheDir,
	}
	log := zerolog.Nop()

	// 1. Initialize Writer (creates DB and tables)
	writer, err := sqlite.NewSQLiteWriterAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	// 2. Define Schema and Generate View
	contactSchema := domain.Schema{
		Name: "contact",
		Properties: []domain.Property{
			{Name: "name", Spec: &domain.StringSpec{}},
			{Name: "email", Spec: &domain.StringSpec{}},
			{
				Name: "status",
				Spec: &domain.StringSpec{Enum: []string{"active", "inactive"}},
			},
		},
	}

	viewSQL, err := sqlite.GenerateSchemaView(contactSchema)
	require.NoError(t, err)

	// 3. Apply View manually (simulating migration)
	dbPath := filepath.Join(cacheDir, "cold.db")
	db, err := sql.Open("sqlite", dbPath)
	require.NoError(t, err)
	// Enable WAL mode for concurrency
	ctx := context.Background()
	_, err = db.ExecContext(ctx, "PRAGMA journal_mode=WAL;")
	require.NoError(t, err)

	_, err = db.ExecContext(ctx, viewSQL)
	require.NoError(t, err)
	require.NoError(t, db.Close())

	// 4. Persist Notes
	notes := []domain.Note{
		{
			ID: domain.NewNoteID("contacts/alice.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":     "Alice",
				"fileClass": "contact",
				"name":      "Alice Smith",
				"email":     "alice@example.com",
				"status":    "active",
			}),
		},
		{
			ID: domain.NewNoteID("contacts/bob.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":     "Bob",
				"fileClass": "contact",
				"name":      "Bob Jones",
				"email":     "bob@example.com",
				"status":    "inactive",
			}),
		},
		{
			ID: domain.NewNoteID("projects/project1.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":     "Project 1",
				"fileClass": "project",
				"status":    "active",
			}),
		},
	}

	for _, n := range notes {
		require.NoError(t, writer.Persist(ctx, n))
	}

	// 5. Initialize Reader and Query
	reader, err := sqlite.NewSQLiteReaderAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// Query ByFileClass (uses view)
	contacts, err := reader.ByFileClass(ctx, "contact")
	require.NoError(t, err)
	assert.Len(t, contacts, 2)

	// Verify extracted fields via FrontmatterQuery (uses json_extract on base
	// table, but verifies deep query works)
	activeContacts, err := reader.FrontmatterQuery(ctx, "status", "active")
	require.NoError(t, err)
	// Should match Alice (active contact) and Project 1 (active project)
	// Wait, FrontmatterQuery queries ALL notes.
	assert.Len(t, activeContacts, 2)

	// Verify specific note content
	alice, err := reader.Read(ctx, domain.NewNoteID("contacts/alice.md"))
	require.NoError(t, err)
	assert.Equal(t, "Alice Smith", alice.Frontmatter.Fields["name"])

	// Verify view-based filtering correctness (manually check if we can query
	// the view from outside) This is covered by ByFileClass which we verified
	// returns 2 (Alice and Bob) and NOT Project 1.

	// 6. Check Staleness
	// Since file doesn't exist, IsStale should check DB timestamps.
	// In our writer.go, indexTime is time.Now().
	// modified_at comes from cache.ExtractFileModTime(fields).
	// Our fields didn't have "modified_at" or similar, so it defaults to 0
	// (1970).
	// indexed_time is Now (~2025).
	// So modified_at (0) < indexed_time (2025).
	// Thus IsStale should be false?
	// Wait, ExtractFileModTime uses "modified" or "mtime" fields if present,
	// else time.Time{}.
	// 0 < Now -> Not Stale.
	stale, err := reader.IsStale(ctx, "contacts/alice.md")
	require.NoError(t, err)
	assert.False(t, stale)

	// Now simulate a file update (update DB modified_at > indexed_time)
	// We need to hack the DB to simulate this state
	db, err = sql.Open("sqlite", dbPath)
	require.NoError(t, err)
	futureMod := time.Now().Add(1 * time.Hour).Unix()
	_, err = db.ExecContext(
		ctx,
		"UPDATE notes SET modified_at = ? WHERE path = ?",
		futureMod,
		"contacts/alice.md",
	)
	require.NoError(t, err)
	require.NoError(t, db.Close())

	stale, err = reader.IsStale(ctx, "contacts/alice.md")
	require.NoError(t, err)
	assert.True(t, stale)

	staleList, err := reader.GetStaleNotes(ctx)
	require.NoError(t, err)
	assert.Contains(t, staleList, "contacts/alice.md")
}
