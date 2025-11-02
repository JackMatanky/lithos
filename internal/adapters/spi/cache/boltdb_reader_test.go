package cache

import (
	"context"
	"encoding/json"
	"testing"
	"time"

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
		pathsBucket, err := tx.CreateBucketIfNotExists([]byte(bucketPaths))
		if err != nil {
			return err
		}
		stalenessBucket, err := tx.CreateBucketIfNotExists(
			[]byte(bucketStaleness),
		)
		if err != nil {
			return err
		}

		// Insert test note metadata
		metadata := BoltDBNoteMetadata{
			Path:        "/notes/test1.md",
			ID:          "test1",
			Title:       "Test Note 1",
			Aliases:     []string{"alias1"},
			FileClass:   "contact",
			FileModTime: time.Now().Add(-time.Hour),
			IndexTime:   time.Now(),
		}

		data, err := json.Marshal(metadata)
		if err != nil {
			return err
		}

		if putErr := pathsBucket.Put([]byte(metadata.Path), data); putErr != nil {
			return putErr
		}

		// Insert staleness data
		stalenessData, _ := json.Marshal(map[string]time.Time{
			"file_mod_time": metadata.FileModTime,
			"index_time":    metadata.IndexTime,
		})
		if putErr := stalenessBucket.Put([]byte(metadata.ID), stalenessData); putErr != nil {
			return putErr
		}

		// Insert second test note
		metadata2 := BoltDBNoteMetadata{
			Path:        "/notes/test2.md",
			ID:          "test2",
			Title:       "Test Note 2",
			Aliases:     []string{"alias2", "alias2b"},
			FileClass:   "project",
			FileModTime: time.Now().Add(-2 * time.Hour),
			IndexTime:   time.Now().Add(-time.Minute),
		}

		data2, err := json.Marshal(metadata2)
		if err != nil {
			return err
		}

		if putErr := pathsBucket.Put([]byte(metadata2.Path), data2); putErr != nil {
			return putErr
		}

		stalenessData2, _ := json.Marshal(map[string]time.Time{
			"file_mod_time": metadata2.FileModTime,
			"index_time":    metadata2.IndexTime,
		})
		if putErr := stalenessBucket.Put([]byte(metadata2.ID), stalenessData2); putErr != nil {
			return putErr
		}

		return nil
	})
	require.NoError(t, err)
}

// TestBoltDBCacheReadAdapter_NewBoltDBCacheReadAdapter tests the
// NewBoltDBCacheReadAdapter constructor.
// TestBoltDBCacheReadAdapter_NewBoltDBCacheReadAdapter tests the function.
func TestBoltDBCacheReadAdapter_NewBoltDBCacheReadAdapter(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	// First create a writer to set up the database
	writer, err := NewBoltDBCacheWriter(config, log)
	require.NoError(t, err)
	_ = writer.Close()

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
		{
			name: "error - database doesn't exist",
			config: domain.Config{
				CacheDir:     "/nonexistent/path",
				FileClassKey: "file_class",
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			adapter, adapterErr := NewBoltDBCacheReadAdapter(tt.config, log)

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

// TestBoltDBCacheReadAdapter_Read tests the function.
func TestBoltDBCacheReadAdapter_Read(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	// Setup test data
	writer, err := NewBoltDBCacheWriter(config, log)
	require.NoError(t, err)
	setupBoltDBTestData(t, writer.db)
	_ = writer.Close()

	// Create reader
	reader, err := NewBoltDBCacheReadAdapter(config, log)
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
			noteID:  domain.NewNoteID("test1"),
			wantErr: false,
			expected: domain.Note{
				ID:   domain.NewNoteID("test1"),
				Path: "/notes/test1.md",
				Frontmatter: domain.Frontmatter{
					FileClass: "contact",
					Fields: map[string]interface{}{
						"title":      "Test Note 1",
						"aliases":    []interface{}{"alias1"},
						"file_class": "contact",
					},
				},
			},
		},
		{
			name:    "success - reads second note",
			noteID:  domain.NewNoteID("test2"),
			wantErr: false,
			expected: domain.Note{
				ID:   domain.NewNoteID("test2"),
				Path: "/notes/test2.md",
				Frontmatter: domain.Frontmatter{
					FileClass: "project",
					Fields: map[string]interface{}{
						"title":      "Test Note 2",
						"aliases":    []interface{}{"alias2", "alias2b"},
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
			assert.Equal(t, tt.expected.Path, note.Path)
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

// TestBoltDBCacheReadAdapter_List tests the function.
func TestBoltDBCacheReadAdapter_List(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	// Setup test data
	writer, err := NewBoltDBCacheWriter(config, log)
	require.NoError(t, err)
	setupBoltDBTestData(t, writer.db)
	_ = writer.Close()

	// Create reader
	reader, err := NewBoltDBCacheReadAdapter(config, log)
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
		assert.True(t, noteIDs["test1"])
		assert.True(t, noteIDs["test2"])

		// Verify frontmatter reconstruction
		for _, note := range notes {
			assert.NotEmpty(t, note.Frontmatter.Fields)
			assert.Contains(t, note.Frontmatter.Fields, "title")
			assert.Contains(t, note.Frontmatter.Fields, config.FileClassKey)
		}
	})

	t.Run("success - empty database returns empty slice", func(t *testing.T) {
		// Create a new empty database
		emptyCacheDir := t.TempDir()
		emptyConfig := domain.Config{
			CacheDir:     emptyCacheDir,
			FileClassKey: "file_class",
		}

		emptyWriter, emptyWriterErr := NewBoltDBCacheWriter(emptyConfig, log)
		require.NoError(t, emptyWriterErr)
		_ = emptyWriter.Close()

		emptyReader, emptyReaderErr := NewBoltDBCacheReadAdapter(
			emptyConfig,
			log,
		)
		require.NoError(t, emptyReaderErr)
		defer func() { _ = emptyReader.Close() }()

		ctx := context.Background()
		notes, emptyListErr := emptyReader.List(ctx)

		require.NoError(t, emptyListErr)
		assert.Empty(t, notes)
	})
}

// TestBoltDBCacheReadAdapter_IsStale tests the function.
func TestBoltDBCacheReadAdapter_IsStale(t *testing.T) {
	// Note: IsStale method doesn't exist in the current implementation
	// This test would be added if/when the method is implemented
	t.Skip("IsStale method not yet implemented in BoltDBCacheReadAdapter")
}

// TestBoltDBCacheReadAdapter_Close tests the function.
func TestBoltDBCacheReadAdapter_Close(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	// Create writer first to set up database
	writer, err := NewBoltDBCacheWriter(config, log)
	require.NoError(t, err)
	_ = writer.Close()

	// Create reader
	reader, err := NewBoltDBCacheReadAdapter(config, log)
	require.NoError(t, err)

	// Verify database is open
	assert.NotNil(t, reader.db)

	// Close it
	err = reader.Close()
	require.NoError(t, err)

	// Note: We can't easily test that the database is closed without internal
	// access
	// The Close method should be idempotent
	err = reader.Close()
	assert.NoError(t, err)
}
