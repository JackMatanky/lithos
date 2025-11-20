package sqlite

import (
	"context"
	"database/sql"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSQLiteWriterAdapter_Persist(t *testing.T) {
	tmpDir := t.TempDir()
	config := domain.Config{
		CacheDir: tmpDir,
	}
	log := zerolog.Nop()

	writer, err := NewSQLiteWriterAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	// Create a test note
	noteID := domain.NoteID("test/note.md")
	fm := domain.NewFrontmatter(map[string]interface{}{
		"title":     "Test Note",
		"fileClass": "contact",
	})
	note := domain.NewNote(noteID, fm)

	ctx := context.Background()

	// Test Persist
	err = writer.Persist(ctx, note)
	require.NoError(t, err)

	// Verify directly in DB
	dbPath := filepath.Join(tmpDir, "cold.db")
	db, err := sql.Open("sqlite", dbPath)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	var path, frontmatter string
	var modTime, idxTime int64
	err = db.QueryRowContext(
		ctx,
		"SELECT path, frontmatter, modified_at, indexed_time FROM notes WHERE path = ?",
		"test/note.md",
	).Scan(&path, &frontmatter, &modTime, &idxTime)
	require.NoError(t, err)

	assert.Equal(t, "test/note.md", path)
	assert.Contains(t, frontmatter, `"title":"Test Note"`)
	assert.NotZero(t, idxTime)
}

func TestSQLiteWriterAdapter_Delete(t *testing.T) {
	tmpDir := t.TempDir()
	config := domain.Config{
		CacheDir: tmpDir,
	}
	log := zerolog.Nop()

	writer, err := NewSQLiteWriterAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	// Insert a note manually or via Persist
	noteID := domain.NoteID("test/delete.md")
	fm := domain.NewFrontmatter(map[string]interface{}{"title": "Delete Me"})
	note := domain.NewNote(noteID, fm)

	ctx := context.Background()
	err = writer.Persist(ctx, note)
	require.NoError(t, err)

	// Test Delete
	err = writer.Delete(ctx, noteID)
	require.NoError(t, err)

	// Verify deletion
	dbPath := filepath.Join(tmpDir, "cold.db")
	db, err := sql.Open("sqlite", dbPath)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	var count int
	err = db.QueryRowContext(
		ctx,
		"SELECT count(*) FROM notes WHERE path = ?",
		"test/delete.md",
	).Scan(&count)
	require.NoError(t, err)
	assert.Equal(t, 0, count)
}
