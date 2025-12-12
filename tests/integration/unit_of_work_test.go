package integration

import (
	"context"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/boltdb"
	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/sqlite"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/require"
	"go.etcd.io/bbolt"
)

func TestUnitOfWork_Integration(t *testing.T) {
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

	// 2. Test Successful Commit
	t.Run("success - commit writes to both", func(t *testing.T) {
		uow := vault.NewCacheUnitOfWork(boltWriter, sqliteWriter)
		ctx := context.Background()

		require.NoError(t, uow.Begin())

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
		require.NoError(t, err)
		indexTime := time.Now()

		require.NoError(t, uow.AddWrite(note, indexTime))
		require.NoError(t, uow.Commit(ctx))

		// Verify BoltDB
		bNote, readErr := boltReader.Read(ctx, note.Path)
		require.NoError(t, readErr)
		require.Equal(t, "Integration Test", bNote.Frontmatter.Fields["title"])

		// Verify SQLite
		sNote, readErr2 := sqliteReader.Read(ctx, note.Path)
		require.NoError(t, readErr2)
		require.Equal(t, "Integration Test", sNote.Frontmatter.Fields["title"])
	})

	// 3. Test Rollback (Simulated via failure injection? Hard with real
	// adapters unless we close DB or similar) Since we can't easily force real
	// adapters to fail mid-transaction without mocking or corrupting,
	// we rely on unit tests with mocks for failure paths.
	// But we can test manual Rollback.
	t.Run("success - manual rollback discards changes", func(t *testing.T) {
		uow := vault.NewCacheUnitOfWork(boltWriter, sqliteWriter)
		ctx := context.Background()

		require.NoError(t, uow.Begin())

		note, noteErr := domain.NewNote(
			"integration/rollback.md",
			domain.NewFrontmatter(map[string]interface{}{}),
			nil,
			nil,
			nil,
			nil,
		)
		require.NoError(t, noteErr)
		indexTime := time.Now()
		require.NoError(t, uow.AddWrite(note, indexTime))

		// Rollback
		require.NoError(t, uow.Rollback(ctx))

		// Commit empty operations (should be no-op)
		require.NoError(t, uow.Commit(ctx))

		// Verify NOT present
		_, readErr := boltReader.Read(ctx, note.Path)
		require.Error(t, readErr) // Should fail
	})
}
