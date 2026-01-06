package integration

import (
	"context"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/boltdb"
	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/sqlite"
	"github.com/JackMatanky/lithos/internal/app/persistence"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/require"
	"go.etcd.io/bbolt"
)

func TestCacheTransaction_Integration(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "fileClass",
	}
	log := zerolog.Nop()

	// 0. Initialize Shared DB
	dbPath := filepath.Join(cacheDir, "lithos.db")
	db, err := bbolt.Open(dbPath, 0o600, nil)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	// 1. Initialize Real Adapters
	boltWriter, err := boltdb.NewBoltDBCacheWriter(config, log, db)
	require.NoError(t, err)
	defer func() { _ = boltWriter.Close() }()

	sqliteWriter, err := sqlite.NewSQLiteWriterAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = sqliteWriter.Close() }()

	boltReader, err := boltdb.NewBoltDBCacheReadAdapter(config, log, db)
	require.NoError(t, err)
	defer func() { _ = boltReader.Close() }()

	sqliteReader, err := sqlite.NewSQLiteReaderAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = sqliteReader.Close() }()

	strategy := &persistence.ParallelWriter{}

	// 2. Test Successful Commit
	t.Run("success - commit writes to both", func(t *testing.T) {
		tx := persistence.NewCacheTransaction(
			strategy,
			boltWriter,
			sqliteWriter,
		)
		ctx := context.Background()

		note, noteErr := domain.NewNote(
			"integration/test1.md",
			domain.NewFrontmatter(map[string]interface{}{
				"title":     "Integration Test",
				"fileClass": "test",
			}),
			nil,
			nil,
			nil,
			nil,
		)
		require.NoError(t, noteErr)
		metadata := spi.CacheWriteMetadata{IndexTime: time.Now()}

		tx.AddWrite(note, metadata)
		require.NoError(t, tx.Commit(ctx))

		// Verify BoltDB
		bNote, readErr := boltReader.Read(ctx, note.Path)
		require.NoError(t, readErr)
		require.Equal(t, "Integration Test", bNote.Frontmatter.Fields["title"])

		// Verify SQLite
		sNote, readErr2 := sqliteReader.Read(ctx, note.Path)
		require.NoError(t, readErr2)
		require.Equal(t, "Integration Test", sNote.Frontmatter.Fields["title"])
	})

	t.Run("success - rollback discards changes", func(t *testing.T) {
		tx := persistence.NewCacheTransaction(
			strategy,
			boltWriter,
			sqliteWriter,
		)
		ctx := context.Background()

		note, noteErr := domain.NewNote(
			"integration/rollback.md",
			domain.NewFrontmatter(map[string]interface{}{}),
			nil,
			nil,
			nil,
			nil,
		)
		require.NoError(t, noteErr)
		metadata := spi.CacheWriteMetadata{IndexTime: time.Now()}
		tx.AddWrite(note, metadata)

		// Rollback
		tx.Rollback()

		// Commit empty operations (should be no-op)
		require.NoError(t, tx.Commit(ctx))

		// Verify NOT present
		_, readErr := boltReader.Read(ctx, note.Path)
		require.Error(t, readErr) // Should fail
	})
}
