package boltdb

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.etcd.io/bbolt"
)

func setupBoltDBTestData(t *testing.T, db *bbolt.DB) {
	// Insert test data
	err := db.Update(func(tx *bbolt.Tx) error {
		// Create buckets
		notesBucket, err := tx.CreateBucketIfNotExists([]byte(BucketNotes))
		if err != nil {
			return err
		}
		indicesBucket, err := tx.CreateBucketIfNotExists([]byte(BucketIndices))
		if err != nil {
			return err
		}
		subBuckets := []string{
			BucketIndexBasenameQuery,
			BucketIndexAliasQuery,
			BucketIndexFileClassQuery,
			BucketIndexByFolder,
		}
		for _, sub := range subBuckets {
			if _, subErr := indicesBucket.CreateBucketIfNotExists([]byte(sub)); subErr != nil {
				return subErr
			}
		}

		// Insert test note metadata
		cached1 := CachedNote{
			Path:      "/notes/test1.md",
			ID:        "test1",
			Title:     "Test Note 1",
			Aliases:   []string{"alias1"},
			FileClass: "contact",
			FileDates: dto.NewFileDatesDTO(time.Now().Add(-time.Hour)),
		}

		data1, err := json.Marshal(cached1)
		if err != nil {
			return err
		}

		if putErr := notesBucket.Put([]byte(cached1.Path), data1); putErr != nil {
			return putErr
		}
		// Populate indices if needed by tests (skipped for now as Read/List use
		// primary bucket)

		// Insert second test note
		cached2 := CachedNote{
			Path:      "/notes/test2.md",
			ID:        "test2",
			Title:     "Test Note 2",
			Aliases:   []string{"alias2", "alias2b"},
			FileClass: "project",
			FileDates: dto.NewFileDatesDTO(time.Now().Add(-2 * time.Hour)),
		}

		data2, err := json.Marshal(cached2)
		if err != nil {
			return err
		}

		if putErr := notesBucket.Put([]byte(cached2.Path), data2); putErr != nil {
			return putErr
		}

		return nil
	})
	require.NoError(t, err)
}

// TestBoltDBCacheReadAdapter_NewBoltDBCacheReadAdapter tests the constructor.
func TestBoltDBCacheReadAdapter_NewBoltDBCacheReadAdapter(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	db, err := Open(config)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	// First create a writer to set up the database
	_, err = NewBoltDBCacheWriter(config, log, db)
	require.NoError(t, err)

	tests := []struct {
		name    string
		config  domain.Config
		wantErr bool
	}{
		{
			name: "success - opens existing database",
			config: domain.Config{
				CacheDir:     cacheDir,
				FileClassKey: "file_class",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			adapter, adapterErr := NewBoltDBCacheReadAdapter(tt.config, log, db)

			if tt.wantErr {
				require.Error(t, adapterErr)
				assert.Nil(t, adapter)
				return
			}

			require.NoError(t, adapterErr)
			if adapter == nil {
				t.Fatal("adapter should not be nil")
			}
			require.NotNil(t, adapter.db)

			// Cleanup
			_ = adapter.Close()
		})
	}
}

// TestBoltDBCacheReadAdapter_Read tests the Read method.
func TestBoltDBCacheReadAdapter_Read(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	db, err := Open(config)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	// Setup test data
	writer, err := NewBoltDBCacheWriter(config, log, db)
	require.NoError(t, err)
	setupBoltDBTestData(t, writer.db)

	// Create reader
	reader, err := NewBoltDBCacheReadAdapter(config, log, db)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	tests := []struct {
		name     string
		noteID   domain.NoteID
		wantErr  bool
		expected domain.Note
	}{
		{
			name:    "success - reads existing note",
			noteID:  domain.NewNoteID("/notes/test1.md"), // Use path as ID
			wantErr: false,
			expected: domain.Note{
				ID: domain.NewNoteID("/notes/test1.md"),
				Frontmatter: domain.Frontmatter{
					FileClass: "contact",
					Fields: map[string]interface{}{
						"title":      "Test Note 1",
						"aliases":    []string{"alias1"},
						"file_class": "contact",
					},
				},
			},
		},
		{
			name:    "success - reads second note",
			noteID:  domain.NewNoteID("/notes/test2.md"),
			wantErr: false,
			expected: domain.Note{
				ID: domain.NewNoteID("/notes/test2.md"),
				Frontmatter: domain.Frontmatter{
					FileClass: "project",
					Fields: map[string]interface{}{
						"title":      "Test Note 2",
						"aliases":    []string{"alias2", "alias2b"},
						"file_class": "project",
					},
				},
			},
		},
		{
			name:    "error - note not found",
			noteID:  domain.NewNoteID("nonexistent"),
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			note, readErr := reader.Read(ctx, tt.noteID)

			if tt.wantErr {
				assert.Error(t, readErr)
				return
			}

			require.NoError(t, readErr)
			assert.Equal(t, tt.expected.ID, note.ID)
			assert.Equal(
				t,
				tt.expected.Frontmatter.FileClass,
				note.Frontmatter.FileClass,
			)

			// Check that fields are reconstructed correctly
			assert.Contains(t, note.Frontmatter.Fields, "title")
			assert.Contains(t, note.Frontmatter.Fields, "aliases")
			assert.Contains(t, note.Frontmatter.Fields, config.FileClassKey)
		})
	}
}

// TestBoltDBCacheReadAdapter_List tests the List method.
func TestBoltDBCacheReadAdapter_List(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	db, err := Open(config)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	// Setup test data
	writer, err := NewBoltDBCacheWriter(config, log, db)
	require.NoError(t, err)
	setupBoltDBTestData(t, writer.db)

	// Create reader
	reader, err := NewBoltDBCacheReadAdapter(config, log, db)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	t.Run("success - lists all notes", func(t *testing.T) {
		ctx := context.Background()
		notes, listErr := reader.List(ctx)

		require.NoError(t, listErr)
		assert.Len(t, notes, 2)

		// Check that both notes are present
		noteIDs := make(map[string]bool)
		for _, note := range notes {
			noteIDs[string(note.ID)] = true
		}
		assert.True(t, noteIDs["/notes/test1.md"])
		assert.True(t, noteIDs["/notes/test2.md"])

		// Verify frontmatter reconstruction
		for _, note := range notes {
			assert.NotEmpty(t, note.Frontmatter.Fields)
			assert.Contains(t, note.Frontmatter.Fields, "title")
			assert.Contains(t, note.Frontmatter.Fields, config.FileClassKey)
		}
	})

	t.Run("success - empty database returns nil slice", func(t *testing.T) {
		// Create a new empty database
		emptyCacheDir := t.TempDir()
		emptyConfig := domain.Config{
			CacheDir:     emptyCacheDir,
			FileClassKey: "file_class",
		}

		emptyDB, openErr := Open(emptyConfig)
		require.NoError(t, openErr)
		defer func() { _ = emptyDB.Close() }()

		_, emptyWriterErr := NewBoltDBCacheWriter(emptyConfig, log, emptyDB)
		require.NoError(t, emptyWriterErr)
		// No cleanup needed for writer

		emptyReader, emptyReaderErr := NewBoltDBCacheReadAdapter(
			emptyConfig,
			log,
			emptyDB,
		)
		require.NoError(t, emptyReaderErr)
		defer func() { _ = emptyReader.Close() }()

		ctx := context.Background()
		notes, emptyListErr := emptyReader.List(ctx)

		require.NoError(t, emptyListErr)
		assert.Empty(t, notes)
	})
}

// TestBoltDBCacheReadAdapter_IsStale tests the staleness detection.
func TestBoltDBCacheReadAdapter_IsStale(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
		VaultPath:    cacheDir, // Use cacheDir as fake vault root for IsStale check
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	db, err := Open(config)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	// Create writer first to set up database and persist a note
	writer, err := NewBoltDBCacheWriter(config, log, db)
	require.NoError(t, err)

	testNote := domain.Note{
		ID: domain.NewNoteID("test-note.md"), // Use filename as path
		Frontmatter: domain.Frontmatter{
			FileClass: "note",
			Fields: map[string]interface{}{
				"title":      "Test Note",
				"file_class": "note",
			},
		},
	}

	ctx := context.Background()
	err = writer.Persist(ctx, testNote, time.Now())
	require.NoError(t, err)

	// Create reader
	reader, err := NewBoltDBCacheReadAdapter(config, log, db)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	t.Run("nonexistent_note_returns_true", func(t *testing.T) {
		// Non-existent file on disk should be considered stale (or rather,
		// IsStale returns true)
		isStale, staleErr := reader.IsStale(ctx, "test-note.md")
		require.NoError(t, staleErr)
		assert.True(t, isStale, "non-existent file should be considered stale")
	})

	t.Run("nonexistent_cache_returns_true", func(t *testing.T) {
		isStale, staleErr := reader.IsStale(ctx, "other.md")
		require.NoError(t, staleErr)
		assert.True(t, isStale)
	})
}

// TestBoltDBCacheReadAdapter_Close tests the function.
func TestBoltDBCacheReadAdapter_Close(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	db, err := Open(config)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	// Create writer first to set up database
	_, err = NewBoltDBCacheWriter(config, log, db)
	require.NoError(t, err)

	// Create reader
	reader, err := NewBoltDBCacheReadAdapter(config, log, db)
	require.NoError(t, err)

	// Verify database is open
	assert.NotNil(t, reader.db)

	// Close it
	err = reader.Close()
	require.NoError(t, err)

	// Idempotent check
	err = reader.Close()
	assert.NoError(t, err)
}
