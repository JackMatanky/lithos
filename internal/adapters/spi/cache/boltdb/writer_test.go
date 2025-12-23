package boltdb

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.etcd.io/bbolt"
)

// TestBoltDBCacheWriteAdapter_NewBoltDBCacheWriter tests the constructor.
func TestBoltDBCacheWriteAdapter_NewBoltDBCacheWriter(t *testing.T) {
	tests := []struct {
		name    string
		config  domain.Config
		wantErr bool
	}{
		{
			name: "success - creates database and buckets",
			config: domain.Config{
				CacheDir:     t.TempDir(),
				FileClassKey: "file_class",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			log := zerolog.New(zerolog.NewTestWriter(t))

			db, err := Open(tt.config)
			require.NoError(t, err)
			defer func() { _ = db.Close() }()

			adapter, err := NewBoltDBCacheWriter(tt.config, log, db)

			if tt.wantErr {
				require.Error(t, err)
				assert.Nil(t, adapter)
				return
			}

			require.NoError(t, err)
			if adapter == nil {
				t.Fatal("adapter should not be nil")
			}
			require.NotNil(t, adapter.db)

			// Verify buckets were created
			err = adapter.db.View(func(tx *bbolt.Tx) error {
				// Primary bucket
				b := tx.Bucket([]byte(BucketNotes))
				assert.NotNil(t, b, "Bucket %s should exist", BucketNotes)

				// Indices bucket
				indices := tx.Bucket([]byte(BucketIndices))
				assert.NotNil(
					t,
					indices,
					"Bucket %s should exist",
					BucketIndices,
				)

				if indices != nil {
					subBuckets := []string{
						BucketIndexBasenameQuery,
						BucketIndexAliasQuery,
						BucketIndexFileClassQuery,
						BucketIndexByFolder,
					}
					for _, sub := range subBuckets {
						sb := indices.Bucket([]byte(sub))
						assert.NotNil(t, sb, "Sub-bucket %s should exist", sub)
					}
				}
				return nil
			})
			require.NoError(t, err)

			// Cleanup
			_ = adapter.Close()
		})
	}
}

// TestBoltDBCacheWriteAdapter_Persist tests the Persist method.
func TestBoltDBCacheWriteAdapter_Persist(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	db, err := Open(config)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	adapter, err := NewBoltDBCacheWriter(config, log, db)
	require.NoError(t, err)
	defer func() { _ = adapter.Close() }()

	tests := []struct {
		name         string
		note         domain.Note
		wantErr      bool
		validateFunc func(t *testing.T, adapter *BoltDBCacheWriteAdapter, note domain.Note)
	}{
		{
			name: "success - persists note metadata",
			note: func() domain.Note {
				note, _ := domain.NewNote(
					"test-note",
					domain.NewFrontmatter(map[string]interface{}{
						"title":      "Test Note",
						"aliases":    []interface{}{"alias1", "alias2"},
						"file_class": "contact",
					}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
			wantErr: false,
			validateFunc: func(t *testing.T, adapter *BoltDBCacheWriteAdapter, note domain.Note) {
				viewErr := adapter.db.View(func(tx *bbolt.Tx) error {
					notesBucket := tx.Bucket([]byte(BucketNotes))
					data := notesBucket.Get([]byte(note.Path))
					assert.NotNil(
						t,
						data,
						"Note should be stored in notes bucket",
					)

					var cached CachedNote
					unmarshalErr := json.Unmarshal(data, &cached)
					require.NoError(t, unmarshalErr)
					assert.Equal(t, note.Path, cached.Path)
					assert.Equal(t, "Test Note", cached.Title)
					assert.Equal(
						t,
						[]string{"alias1", "alias2"},
						cached.Aliases,
					)
					assert.Equal(t, "contact", cached.FileClass)
					assert.False(t, cached.FileDates.IndexedAt.IsZero())

					// Check indices
					indices := tx.Bucket([]byte(BucketIndices))

					// Basename
					bnBucket := indices.Bucket([]byte(BucketIndexBasenameQuery))
					bnData := bnBucket.Get(
						[]byte("test-note"),
					) // extractBasename("test-note") == "test-note"
					assert.NotNil(t, bnData)
					var paths []string
					pathsErr := json.Unmarshal(bnData, &paths)
					require.NoError(t, pathsErr)
					assert.Contains(t, paths, note.Path)

					// Aliases
					aliasBucket := indices.Bucket([]byte(BucketIndexAliasQuery))
					aliasData := aliasBucket.Get([]byte("alias1"))
					assert.NotNil(t, aliasData)

					// FileClass
					fcBucket := indices.Bucket(
						[]byte(BucketIndexFileClassQuery),
					)
					fcData := fcBucket.Get([]byte("contact"))
					assert.NotNil(t, fcData)

					return nil
				})
				require.NoError(t, viewErr)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			metadata := spi.CacheWriteMetadata{IndexTime: time.Now()}
			persistErr := adapter.Persist(ctx, tt.note, metadata)
			if tt.wantErr {
				require.Error(t, persistErr)
				return
			}
			require.NoError(t, persistErr)

			if tt.validateFunc != nil {
				tt.validateFunc(t, adapter, tt.note)
			}
		})
	}
}

// TestBoltDBCacheWriteAdapter_Delete tests the Delete method.
func TestBoltDBCacheWriteAdapter_Delete(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	db, err := Open(config)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	adapter, err := NewBoltDBCacheWriter(config, log, db)
	require.NoError(t, err)
	defer func() { _ = adapter.Close() }()

	// Setup: persist a note first with all metadata for indices
	note, _ := domain.NewNote(
		"notes/delete-test.md",
		domain.NewFrontmatter(map[string]interface{}{
			"title":      "Delete Test",
			"aliases":    []interface{}{"del-alias"},
			"file_class": "test-class",
		}),
		[]domain.Link{},
		[]domain.Heading{},
		[]string{},
		[]domain.TaskItem{},
	)

	ctx := context.Background()
	metadata := spi.CacheWriteMetadata{IndexTime: time.Now()}
	err = adapter.Persist(ctx, note, metadata)
	require.NoError(t, err)

	// Corrupt the basename index manually to force a failure during Delete
	err = adapter.db.Update(func(tx *bbolt.Tx) error {
		indices := tx.Bucket([]byte(BucketIndices))
		bnBucket := indices.Bucket([]byte(BucketIndexBasenameQuery))
		// Write invalid JSON to force unmarshal error in removeFromIndex
		// We must corrupt the key that corresponds to the note being deleted
		// ("delete-test")
		return bnBucket.Put([]byte("delete-test"), []byte("{invalid-json"))
	})
	require.NoError(t, err)

	// Attempt Delete - should fail
	err = adapter.Delete(ctx, note.Path)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "failed to unmarshal index list")

	// Verify Note still exists (Rollback)
	viewErr := adapter.db.View(func(tx *bbolt.Tx) error {
		notesBucket := tx.Bucket([]byte(BucketNotes))
		data := notesBucket.Get([]byte(note.Path))
		assert.NotNil(t, data, "Note should still exist after rollback")
		return nil
	})
	require.NoError(t, viewErr)
}

func Test_extractCachedNote(t *testing.T) {
	tests := []struct {
		name         string
		note         domain.Note
		fileClassKey string
		expected     CachedNote
	}{
		{
			name: "extracts all metadata fields",
			note: func() domain.Note {
				note, _ := domain.NewNote(
					"/notes/contact.md",
					domain.NewFrontmatter(map[string]interface{}{
						"title":      "John Doe",
						"aliases":    []interface{}{"JD", "Johnny"},
						"file_class": "contact",
					}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
			fileClassKey: "file_class",
			expected: CachedNote{
				ID:        "/notes/contact.md",
				Path:      "/notes/contact.md",
				Title:     "John Doe",
				Aliases:   []string{"JD", "Johnny"},
				FileClass: "contact",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			metadata := spi.CacheWriteMetadata{IndexTime: time.Now()}
			result := extractCachedNote(tt.note, tt.fileClassKey, metadata)

			assert.Equal(t, tt.expected.ID, result.ID)
			assert.Equal(t, tt.expected.Title, result.Title)
			assert.Equal(t, tt.expected.Aliases, result.Aliases)
			assert.Equal(t, tt.expected.FileClass, result.FileClass)
			assert.IsType(t, dto.FileDatesDTO{}, result.FileDates)
		})
	}
}

func Test_extractBasename(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name string
		path string
		want string
	}{
		{
			name: "unix path with extension",
			path: "/notes/contact.md",
			want: "contact",
		},
		{
			name: "windows path with extension",
			path: `notes\project\alpha.md`,
			want: "alpha",
		},
		{
			name: "no extension",
			path: "notes/readme",
			want: "readme",
		},
		{
			name: "empty path",
			path: "",
			want: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()
			assert.Equal(t, tt.want, extractBasename(tt.path))
		})
	}
}

func Test_extractDirectory(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name string
		path string
		want string
	}{
		{
			name: "nested directory",
			path: "/notes/projects/alpha.md",
			want: "/notes/projects",
		},
		{
			name: "single directory no leading slash",
			path: "notes/contact.md",
			want: "notes",
		},
		{
			name: "root file",
			path: "contact.md",
			want: "",
		},
		{
			name: "windows separators",
			path: `notes\contacts\john.md`,
			want: "notes/contacts",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()
			assert.Equal(t, tt.want, extractDirectory(tt.path))
		})
	}
}
