package sqlite

import (
	"context"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const insertNoteQuery = `INSERT INTO notes (path, frontmatter, modified_at, indexed_time, size) ` +
	`VALUES (?, ?, ?, ?, ?)`

func TestSQLiteReaderAdapter_Read(t *testing.T) {
	tmpDir := t.TempDir()
	config := domain.Config{CacheDir: tmpDir}
	log := zerolog.Nop()

	// Setup DB with some data
	dbPath := filepath.Join(tmpDir, "cold.db")
	db, err := InitializeDatabase(dbPath)
	require.NoError(t, err)

	ctx := context.Background()
	_, err = db.ExecContext(
		ctx,
		insertNoteQuery,
		"test/read.md",
		`{"title":"Read Me","fileClass":"note"}`,
		1000,
		2000,
		0,
	)
	require.NoError(t, err)
	require.NoError(t, db.Close())

	reader, err := NewSQLiteReaderAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	notePath := "test/read.md"

	// Test Read
	note, err := reader.Read(ctx, notePath)
	require.NoError(t, err)
	assert.Equal(t, notePath, note.Path)
	assert.Equal(t, "Read Me", note.Frontmatter.Fields["title"])
}

func TestSQLiteReaderAdapter_FileClassQuery(t *testing.T) {
	tmpDir := t.TempDir()
	config := domain.Config{CacheDir: tmpDir, FileClassKey: "fileClass"}
	log := zerolog.Nop()

	// Setup DB with view
	dbPath := filepath.Join(tmpDir, "cold.db")
	db, err := InitializeDatabase(dbPath)
	require.NoError(t, err)

	ctx := context.Background()

	// Insert data
	_, err = db.ExecContext(
		ctx,
		insertNoteQuery,
		"test/alice.md",
		`{"title":"Alice","fileClass":"contact"}`,
		1000,
		2000,
		0,
	)
	require.NoError(t, err)
	_, err = db.ExecContext(
		ctx,
		insertNoteQuery,
		"test/bob.md",
		`{"title":"Bob","fileClass":"project"}`,
		1000,
		2000,
		0,
	)
	require.NoError(t, err)
	require.NoError(t, db.Close())

	testSchema := domain.Schema{
		Name: "contact",
		Properties: []domain.Property{
			{Name: "title", Spec: &domain.StringSpec{}},
		},
	}
	migrator := NewSchemaViewMigrator(
		[]domain.Schema{testSchema},
		config.FileClassKey,
		log,
	)

	reader, err := NewSQLiteReaderAdapter(config, log, migrator)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	notes, err := reader.FileClassQuery(ctx, "contact")
	require.NoError(t, err)
	assert.Len(t, notes, 1)
	assert.Equal(t, "test/alice.md", notes[0].Path)
}

func TestSQLiteReaderAdapter_PathQuery(t *testing.T) {
	tmpDir := t.TempDir()
	config := domain.Config{CacheDir: tmpDir}
	log := zerolog.Nop()

	dbPath := filepath.Join(tmpDir, "cold.db")
	db, err := InitializeDatabase(dbPath)
	require.NoError(t, err)

	ctx := context.Background()
	_, err = db.ExecContext(
		ctx,
		insertNoteQuery,
		"folder/sub/file.md",
		`{"title":"File"}`,
		0,
		0,
		0,
	)
	require.NoError(t, err)
	require.NoError(t, db.Close())

	reader, err := NewSQLiteReaderAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// Test Full
	notes, err := reader.PathQuery(
		ctx,
		spi.PathQueryOptions{
			Scope: spi.PathQueryScopeFull,
			Value: "folder/sub/file.md",
		},
	)
	require.NoError(t, err)
	assert.Len(t, notes, 1)

	// Test Folder
	notes, err = reader.PathQuery(
		ctx,
		spi.PathQueryOptions{Scope: spi.PathQueryScopeFolder, Value: "folder/"},
	)
	require.NoError(t, err)
	assert.Len(t, notes, 1)

	// Test Basename (approximate)
	notes, err = reader.PathQuery(
		ctx,
		spi.PathQueryOptions{Scope: spi.PathQueryScopeBasename, Value: "file"},
	)
	require.NoError(t, err)
	assert.Len(t, notes, 1)
}
