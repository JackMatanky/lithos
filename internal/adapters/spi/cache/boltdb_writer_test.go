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

// TestBoltDBCacheWriteAdapter_NewBoltDBCacheWriter tests the
// NewBoltDBCacheWriter constructor.
// TestBoltDBCacheWriteAdapter_NewBoltDBCacheWriter tests the function.
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
				buckets := []string{
					bucketPaths,
					bucketBasenames,
					bucketAliases,
					bucketFileClasses,
					bucketDirectories,
					bucketStaleness,
				}
				for _, bucket := range buckets {
					b := tx.Bucket([]byte(bucket))
					assert.NotNil(t, b, "Bucket %s should exist", bucket)
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
// TestBoltDBCacheWriteAdapter_Persist tests the function.
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
				ID:   domain.NewNoteID("test-note"),
				Path: "/path/to/test-note.md",
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
					pathsBucket := tx.Bucket([]byte(bucketPaths))
					data := pathsBucket.Get([]byte(note.Path))
					assert.NotNil(
						t,
						data,
						"Note should be stored in paths bucket",
					)

					var metadata BoltDBNoteMetadata
					unmarshalErr := json.Unmarshal(data, &metadata)
					require.NoError(t, unmarshalErr)
					assert.Equal(t, note.Path, metadata.Path)
					assert.Equal(t, string(note.ID), metadata.ID)
					assert.Equal(t, "Test Note", metadata.Title)
					assert.Equal(
						t,
						[]string{"alias1", "alias2"},
						metadata.Aliases,
					)
					assert.Equal(t, "contact", metadata.FileClass)

					// Check staleness data
					stalenessBucket := tx.Bucket([]byte(bucketStaleness))
					stalenessData := stalenessBucket.Get(
						[]byte(string(note.ID)),
					)
					assert.NotNil(
						t,
						stalenessData,
						"Staleness data should be stored",
					)

					var staleness map[string]time.Time
					stalenessErr := json.Unmarshal(stalenessData, &staleness)
					require.NoError(t, stalenessErr)
					assert.Contains(t, staleness, "file_mod_time")
					assert.Contains(t, staleness, "index_time")

					return nil
				})
				require.NoError(t, viewErr)
			},
		},
		{
			name: "success - overwrites existing note",
			note: domain.Note{
				ID:   domain.NewNoteID("existing-note"),
				Path: "/path/to/existing-note.md",
				Frontmatter: domain.Frontmatter{
					FileClass: "project",
					Fields: map[string]interface{}{
						"title":      "Updated Note",
						"file_class": "project",
					},
				},
			},
			wantErr: false,
			validateFunc: func(t *testing.T, adapter *BoltDBCacheWriteAdapter, note domain.Note) {
				// First persist
				ctx := context.Background()
				persistErr := adapter.Persist(ctx, note)
				require.NoError(t, persistErr)

				// Update and persist again
				note.Frontmatter.Fields["title"] = "Updated Again"
				persistErr = adapter.Persist(ctx, note)
				require.NoError(t, persistErr)

				// Verify update
				viewErr := adapter.db.View(func(tx *bbolt.Tx) error {
					pathsBucket := tx.Bucket([]byte(bucketPaths))
					data := pathsBucket.Get([]byte(note.Path))
					assert.NotNil(t, data)

					var metadata BoltDBNoteMetadata
					unmarshalErr := json.Unmarshal(data, &metadata)
					require.NoError(t, unmarshalErr)
					assert.Equal(t, "Updated Again", metadata.Title)
					return nil
				})
				require.NoError(t, viewErr)
			},
		},
		{
			name: "success - handles missing frontmatter fields",
			note: domain.Note{
				ID:   domain.NewNoteID("minimal-note"),
				Path: "/path/to/minimal-note.md",
				Frontmatter: domain.Frontmatter{
					Fields: map[string]interface{}{},
				},
			},
			wantErr: false,
			validateFunc: func(t *testing.T, adapter *BoltDBCacheWriteAdapter, note domain.Note) {
				ctx := context.Background()
				persistErr := adapter.Persist(ctx, note)
				require.NoError(t, persistErr)

				viewErr := adapter.db.View(func(tx *bbolt.Tx) error {
					pathsBucket := tx.Bucket([]byte(bucketPaths))
					data := pathsBucket.Get([]byte(note.Path))
					assert.NotNil(t, data)

					var metadata BoltDBNoteMetadata
					unmarshalErr := json.Unmarshal(data, &metadata)
					require.NoError(t, unmarshalErr)
					assert.Empty(t, metadata.Title)
					assert.Empty(t, metadata.Aliases)
					assert.Empty(t, metadata.FileClass)
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

// TestBoltDBCacheWriteAdapter_Delete tests the function.
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

	// Setup: persist a note first
	note := domain.Note{
		ID:   domain.NewNoteID("delete-test"),
		Path: "/path/to/delete-test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{
				"title": "Delete Test",
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
			name:    "success - deletes existing note staleness data",
			noteID:  note.ID,
			wantErr: false,
			validateFunc: func(t *testing.T, adapter *BoltDBCacheWriteAdapter, noteID domain.NoteID) {
				viewErr := adapter.db.View(func(tx *bbolt.Tx) error {
					stalenessBucket := tx.Bucket([]byte(bucketStaleness))
					data := stalenessBucket.Get([]byte(string(noteID)))
					assert.Nil(t, data, "Staleness data should be deleted")
					return nil
				})
				require.NoError(t, viewErr)
			},
		},
		{
			name:    "success - idempotent for non-existent note",
			noteID:  domain.NewNoteID("non-existent"),
			wantErr: false,
		},
		{
			name:    "error - context canceled",
			noteID:  domain.NewNoteID("canceled-delete"),
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var testCtx context.Context
			if tt.name == "error - context canceled" {
				var cancel context.CancelFunc
				testCtx, cancel = context.WithCancel(context.Background())
				cancel()
			} else {
				testCtx = context.Background()
			}

			deleteErr := adapter.Delete(testCtx, tt.noteID)
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

// TestBoltDBCacheWriteAdapter_Close tests the function.
func TestBoltDBCacheWriteAdapter_Close(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	adapter, err := NewBoltDBCacheWriter(config, log)
	require.NoError(t, err)

	// Verify database is open
	require.NotNil(t, adapter.db)

	// Close it
	err = adapter.Close()
	require.NoError(t, err)

	// Verify we can't use it after close
	ctx := context.Background()
	note := domain.Note{
		ID:   domain.NewNoteID("test"),
		Path: "/test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{},
		},
	}

	err = adapter.Persist(ctx, note)
	require.Error(t, err) // Should fail after close
}

// Test_extractNoteMetadata tests the function.
func Test_extractNoteMetadata(t *testing.T) {
	tests := []struct {
		name         string
		note         domain.Note
		fileClassKey string
		expected     BoltDBNoteMetadata
	}{
		{
			name: "extracts all metadata fields",
			note: domain.Note{
				Path: "/notes/contact.md",
				Frontmatter: domain.Frontmatter{
					Fields: map[string]interface{}{
						"title":      "John Doe",
						"aliases":    []interface{}{"JD", "Johnny"},
						"file_class": "contact",
					},
				},
			},
			fileClassKey: "file_class",
			expected: BoltDBNoteMetadata{
				Path:      "/notes/contact.md",
				Title:     "John Doe",
				Aliases:   []string{"JD", "Johnny"},
				FileClass: "contact",
			},
		},
		{
			name: "handles missing fields gracefully",
			note: domain.Note{
				Path: "/notes/minimal.md",
				Frontmatter: domain.Frontmatter{
					Fields: map[string]interface{}{},
				},
			},
			fileClassKey: "file_class",
			expected: BoltDBNoteMetadata{
				Path: "/notes/minimal.md",
			},
		},
		{
			name: "uses configurable file class key",
			note: domain.Note{
				Path: "/notes/custom.md",
				Frontmatter: domain.Frontmatter{
					Fields: map[string]interface{}{
						"type": "project",
					},
				},
			},
			fileClassKey: "type",
			expected: BoltDBNoteMetadata{
				Path:      "/notes/custom.md",
				FileClass: "project",
			},
		},
		{
			name: "handles invalid aliases type",
			note: domain.Note{
				Path: "/notes/invalid.md",
				Frontmatter: domain.Frontmatter{
					Fields: map[string]interface{}{
						"aliases": "not-an-array", // Invalid type
					},
				},
			},
			fileClassKey: "file_class",
			expected: BoltDBNoteMetadata{
				Path: "/notes/invalid.md",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := extractNoteMetadata(tt.note, tt.fileClassKey)

			assert.Equal(t, tt.expected.Path, result.Path)
			assert.Equal(t, tt.expected.Title, result.Title)
			assert.Equal(t, tt.expected.Aliases, result.Aliases)
			assert.Equal(t, tt.expected.FileClass, result.FileClass)

			// Verify timestamps are set
			assert.False(t, result.FileModTime.IsZero())
			assert.False(t, result.IndexTime.IsZero())
		})
	}
}
