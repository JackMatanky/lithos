package boltdb

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.etcd.io/bbolt"
)

// TestBoltDBCacheWriteAdapter_NewBoltDBCacheWriter tests the constructor.
func TestBoltDBCacheWriteAdapter_NewBoltDBCacheWriter(t *testing.T) {
	tests := []struct {
		name      string
		config    domain.Config
		wantErr   bool
		setupFunc func(t *testing.T, cacheDir string)
	}{
		{
			name: "success - creates database and buckets",
			config: domain.Config{
				CacheDir:     t.TempDir(),
				FileClassKey: "file_class",
			},
			wantErr: false,
		},
		{
			name: "error - invalid cache directory",
			config: domain.Config{
				CacheDir:     "/invalid/path/that/does/not/exist",
				FileClassKey: "file_class",
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.setupFunc != nil {
				tt.setupFunc(t, tt.config.CacheDir)
			}

			log := zerolog.New(zerolog.NewTestWriter(t))
			adapter, err := NewBoltDBCacheWriter(tt.config, log)

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
						BucketIndexByBasename,
						BucketIndexByAlias,
						BucketIndexByFileClass,
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

	adapter, err := NewBoltDBCacheWriter(config, log)
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
			note: domain.Note{
				ID: domain.NewNoteID("test-note"),
				Frontmatter: domain.Frontmatter{
					FileClass: "contact",
					Fields: map[string]interface{}{
						"title":      "Test Note",
						"aliases":    []interface{}{"alias1", "alias2"},
						"file_class": "contact",
					},
				},
			},
			wantErr: false,
			validateFunc: func(t *testing.T, adapter *BoltDBCacheWriteAdapter, note domain.Note) {
				viewErr := adapter.db.View(func(tx *bbolt.Tx) error {
					notesBucket := tx.Bucket([]byte(BucketNotes))
					data := notesBucket.Get([]byte(string(note.ID)))
					assert.NotNil(
						t,
						data,
						"Note should be stored in notes bucket",
					)

					var cached CachedNote
					unmarshalErr := json.Unmarshal(data, &cached)
					require.NoError(t, unmarshalErr)
					assert.Equal(t, string(note.ID), cached.Path)
					assert.Equal(t, string(note.ID), cached.ID)
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
					bnBucket := indices.Bucket([]byte(BucketIndexByBasename))
					bnData := bnBucket.Get(
						[]byte("test-note"),
					) // extractBasename("test-note") == "test-note"
					assert.NotNil(t, bnData)
					var paths []string
					pathsErr := json.Unmarshal(bnData, &paths)
					require.NoError(t, pathsErr)
					assert.Contains(t, paths, string(note.ID))

					// Aliases
					aliasBucket := indices.Bucket([]byte(BucketIndexByAlias))
					aliasData := aliasBucket.Get([]byte("alias1"))
					assert.NotNil(t, aliasData)

					// FileClass
					fcBucket := indices.Bucket([]byte(BucketIndexByFileClass))
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
			persistErr := adapter.Persist(ctx, tt.note)
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

	adapter, err := NewBoltDBCacheWriter(config, log)
	require.NoError(t, err)
	defer func() { _ = adapter.Close() }()

	// Setup: persist a note first with all metadata for indices
	note := domain.Note{
		ID: domain.NewNoteID("notes/delete-test.md"),
		Frontmatter: domain.Frontmatter{
			FileClass: "test-class",
			Fields: map[string]interface{}{
				"title":      "Delete Test",
				"aliases":    []interface{}{"del-alias"},
				"file_class": "test-class",
			},
		},
	}

	ctx := context.Background()
	err = adapter.Persist(ctx, note)
	require.NoError(t, err)

	tests := []struct {
		name         string
		noteID       domain.NoteID
		wantErr      bool
		validateFunc func(t *testing.T, adapter *BoltDBCacheWriteAdapter, noteID domain.NoteID)
	}{
		{
			name:    "success - deletes existing note and cleans indices",
			noteID:  note.ID,
			wantErr: false,
			validateFunc: func(t *testing.T, adapter *BoltDBCacheWriteAdapter, noteID domain.NoteID) {
				viewErr := adapter.db.View(func(tx *bbolt.Tx) error {
					// 1. Verify primary note deleted
					notesBucket := tx.Bucket([]byte(BucketNotes))
					data := notesBucket.Get([]byte(string(noteID)))
					assert.Nil(t, data, "Note data should be deleted")

					// 2. Verify indices cleaned up
					indices := tx.Bucket([]byte(BucketIndices))

					// Basename: "delete-test" (from "notes/delete-test.md")
					bnBucket := indices.Bucket([]byte(BucketIndexByBasename))
					bnData := bnBucket.Get([]byte("delete-test"))
					assert.Nil(t, bnData, "Basename index should be cleaned up")

					// Alias: "del-alias"
					aliasBucket := indices.Bucket([]byte(BucketIndexByAlias))
					aliasData := aliasBucket.Get([]byte("del-alias"))
					assert.Nil(t, aliasData, "Alias index should be cleaned up")

					// FileClass: "test-class"
					fcBucket := indices.Bucket([]byte(BucketIndexByFileClass))
					fcData := fcBucket.Get([]byte("test-class"))
					assert.Nil(
						t,
						fcData,
						"FileClass index should be cleaned up",
					)

					// Folder: "notes"
					dirBucket := indices.Bucket([]byte(BucketIndexByFolder))
					dirData := dirBucket.Get([]byte("notes"))
					assert.Nil(t, dirData, "Folder index should be cleaned up")

					return nil
				})
				require.NoError(t, viewErr)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			deleteErr := adapter.Delete(ctx, tt.noteID)
			if tt.wantErr {
				require.Error(t, deleteErr)
				return
			}
			require.NoError(t, deleteErr)

			if tt.validateFunc != nil {
				tt.validateFunc(t, adapter, tt.noteID)
			}
		})
	}
}

// TestBoltDBCacheWriteAdapter_Delete_Rollback tests that delete is atomic.
func TestBoltDBCacheWriteAdapter_Delete_Rollback(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	adapter, err := NewBoltDBCacheWriter(config, log)
	require.NoError(t, err)
	defer func() { _ = adapter.Close() }()

	// Setup: persist a note
	note := domain.Note{
		ID: domain.NewNoteID("notes/rollback-test.md"),
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{
				"title": "Rollback Test",
			},
		},
	}

	ctx := context.Background()
	err = adapter.Persist(ctx, note)
	require.NoError(t, err)

	// Corrupt the basename index manually to force a failure during Delete
	err = adapter.db.Update(func(tx *bbolt.Tx) error {
		indices := tx.Bucket([]byte(BucketIndices))
		bnBucket := indices.Bucket([]byte(BucketIndexByBasename))
		// Write invalid JSON to force unmarshal error in removeFromIndex
		return bnBucket.Put([]byte("rollback-test"), []byte("{invalid-json"))
	})
	require.NoError(t, err)

	// Attempt Delete - should fail
	err = adapter.Delete(ctx, note.ID)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "failed to unmarshal index list")

	// Verify Note still exists (Rollback)
	viewErr := adapter.db.View(func(tx *bbolt.Tx) error {
		notesBucket := tx.Bucket([]byte(BucketNotes))
		data := notesBucket.Get([]byte(string(note.ID)))
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
			note: domain.Note{
				ID: domain.NewNoteID("/notes/contact.md"),
				Frontmatter: domain.Frontmatter{
					Fields: map[string]interface{}{
						"title":      "John Doe",
						"aliases":    []interface{}{"JD", "Johnny"},
						"file_class": "contact",
					},
				},
			},
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
			result := extractCachedNote(tt.note, tt.fileClassKey)

			assert.Equal(t, tt.expected.ID, result.ID)
			assert.Equal(t, tt.expected.Title, result.Title)
			assert.Equal(t, tt.expected.Aliases, result.Aliases)
			assert.Equal(t, tt.expected.FileClass, result.FileClass)
			assert.IsType(t, dto.FileDatesDTO{}, result.FileDates)
		})
	}
}
