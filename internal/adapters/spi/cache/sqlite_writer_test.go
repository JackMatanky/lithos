package cache

import (
	"context"
	"database/sql"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	_ "modernc.org/sqlite" // Register SQLite driver
)

// TestSQLiteCacheWriteAdapter_NewSQLiteCacheWriteAdapter tests the function.
func TestSQLiteCacheWriteAdapter_NewSQLiteCacheWriteAdapter(t *testing.T) {
	tests := []struct {
		name      string
		config    domain.Config
		wantErr   bool
		setupFunc func(t *testing.T, cacheDir string)
	}{
		{
			name: "success - creates database and schema",
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
			adapter, err := NewSQLiteCacheWriteAdapter(tt.config, log)

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

			// Verify schema was created
			ctx := context.Background()
			var tableCount int
			tableCountErr := adapter.db.QueryRowContext(
				ctx,
				"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='notes'",
			).Scan(&tableCount)
			require.NoError(t, tableCountErr)
			assert.Equal(t, 1, tableCount)

			// Verify indexes were created
			var indexCount int
			indexCountErr := adapter.db.QueryRowContext(
				ctx,
				"SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_notes%'",
			).Scan(&indexCount)
			require.NoError(t, indexCountErr)
			assert.Equal(
				t,
				5,
				indexCount,
			) // path, file_class, file_mod_time, index_time, staleness (composite)

			// Cleanup
			_ = adapter.Close()
		})
	}
}

// TestSQLiteCacheWriteAdapter_Persist tests the function.
func TestSQLiteCacheWriteAdapter_Persist(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	adapter, err := NewSQLiteCacheWriteAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = adapter.Close() }()

	const testUpdatedTitle = "Updated Again"

	tests := []struct {
		name         string
		note         domain.Note
		wantErr      bool
		validateFunc func(t *testing.T, adapter *SQLiteCacheWriteAdapter, note domain.Note)
	}{
		{
			name: "success - persists note metadata",
			note: domain.Note{
				ID:   domain.NewNoteID("test-note"),
				Path: "/path/to/test-note.md",
				Frontmatter: domain.Frontmatter{
					FileClass: "contact",
					Fields: map[string]interface{}{
						"title":        "Test Note",
						"aliases":      []interface{}{"alias1", "alias2"},
						"file_class":   "contact",
						"custom_field": "preserved",
					},
				},
			},
			wantErr: false,
			validateFunc: func(t *testing.T, adapter *SQLiteCacheWriteAdapter, note domain.Note) {
				ctx := context.Background()

				var count int
				queryErr := adapter.db.QueryRowContext(
					ctx,
					"SELECT COUNT(*) FROM notes WHERE id = ?",
					string(note.ID),
				).Scan(&count)
				require.NoError(t, queryErr)
				assert.Equal(t, 1, count)

				var (
					id,
					path,
					title,
					fileClass,
					frontmatterJSON string
				)
				detailsErr := adapter.db.QueryRowContext(
					ctx,
					"SELECT id, path, title, file_class, frontmatter FROM notes WHERE id = ?",
					string(note.ID),
				).Scan(&id, &path, &title, &fileClass, &frontmatterJSON)
				require.NoError(t, detailsErr)
				assert.Equal(t, string(note.ID), id)
				assert.Equal(t, note.Path, path)
				assert.Equal(t, "Test Note", title)
				assert.Equal(t, "contact", fileClass)

				// Verify frontmatter JSON contains all fields
				assert.Contains(t, frontmatterJSON, "title")
				assert.Contains(t, frontmatterJSON, "aliases")
				assert.Contains(t, frontmatterJSON, "file_class")
				assert.Contains(t, frontmatterJSON, "custom_field")
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
			validateFunc: func(t *testing.T, adapter *SQLiteCacheWriteAdapter, note domain.Note) {
				// First persist
				ctx := context.Background()
				persistErr := adapter.Persist(ctx, note)
				require.NoError(t, persistErr)

				// Update and persist again
				note.Frontmatter.Fields["title"] = testUpdatedTitle
				persistErr = adapter.Persist(ctx, note)
				require.NoError(t, persistErr)

				// Verify update
				var title string
				queryErr := adapter.db.QueryRowContext(
					ctx,
					"SELECT title FROM notes WHERE id = ?",
					string(note.ID),
				).Scan(&title)
				require.NoError(t, queryErr)
				assert.Equal(t, testUpdatedTitle, title)
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
			validateFunc: func(t *testing.T, adapter *SQLiteCacheWriteAdapter, note domain.Note) {
				ctx := context.Background()
				persistErr := adapter.Persist(ctx, note)
				require.NoError(t, persistErr)

				var count int
				queryErr := adapter.db.QueryRowContext(
					ctx,
					"SELECT COUNT(*) FROM notes WHERE id = ?",
					string(note.ID),
				).Scan(&count)
				require.NoError(t, queryErr)
				assert.Equal(t, 1, count)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			persistErr := adapter.Persist(ctx, tt.note)

			if tt.wantErr {
				require.Error(t, persistErr)
			} else {
				require.NoError(t, persistErr)
			}

			if tt.validateFunc != nil {
				tt.validateFunc(t, adapter, tt.note)
			}
		})
	}
}

// TestSQLiteCacheWriteAdapter_Delete tests the function.
func TestSQLiteCacheWriteAdapter_Delete(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	adapter, err := NewSQLiteCacheWriteAdapter(config, log)
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
		validateFunc func(t *testing.T, adapter *SQLiteCacheWriteAdapter, noteID domain.NoteID)
	}{
		{
			name:    "success - deletes existing note",
			noteID:  note.ID,
			wantErr: false,
			validateFunc: func(t *testing.T, adapter *SQLiteCacheWriteAdapter, noteID domain.NoteID) {
				validationCtx := context.Background()
				var count int
				queryErr := adapter.db.QueryRowContext(
					validationCtx,
					"SELECT COUNT(*) FROM notes WHERE id = ?",
					string(noteID),
				).Scan(&count)
				require.NoError(t, queryErr)
				assert.Equal(t, 0, count)
			},
		},
		{
			name:    "success - idempotent for non-existent note",
			noteID:  domain.NewNoteID("non-existent"),
			wantErr: false,
		},
		{
			name:    testContextCanceled,
			noteID:  domain.NewNoteID("canceled-delete"),
			wantErr: true,
			validateFunc: func(t *testing.T, adapter *SQLiteCacheWriteAdapter, noteID domain.NoteID) {
				cancelCtx, cancel := context.WithCancel(context.Background())
				cancel()

				deleteErr := adapter.Delete(cancelCtx, noteID)
				require.Error(t, deleteErr)
				assert.Equal(t, context.Canceled, deleteErr)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			baseCtx := context.Background()
			runCtx := baseCtx
			if tt.name == testContextCanceled {
				cancelCtx, cancel := context.WithCancel(context.Background())
				cancel()
				runCtx = cancelCtx
			}
			deleteErr := adapter.Delete(runCtx, tt.noteID)

			if tt.wantErr {
				require.Error(t, deleteErr)
			} else {
				require.NoError(t, deleteErr)
			}

			if tt.validateFunc != nil {
				tt.validateFunc(t, adapter, tt.noteID)
			}
		})
	}
}

// TestSQLiteCacheWriteAdapter_Close tests the function.
func TestSQLiteCacheWriteAdapter_Close(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	adapter, err := NewSQLiteCacheWriteAdapter(config, log)
	require.NoError(t, err)

	// Verify database is open
	require.NotNil(t, adapter.db)

	// Close it
	err = adapter.Close()
	require.NoError(t, err)

	// Verify we can't use it after close (should get error)
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

// Test_extractSQLiteNoteMetadata tests the function.
func Test_extractSQLiteNoteMetadata(t *testing.T) {
	tests := []struct {
		name         string
		note         domain.Note
		fileClassKey string
		checkFunc    func(t *testing.T, metadata map[string]interface{})
	}{
		{
			name: "extracts all metadata fields",
			note: domain.Note{
				ID:   domain.NewNoteID("test1"),
				Path: "/notes/contact.md",
				Frontmatter: domain.Frontmatter{
					Fields: map[string]interface{}{
						"title":      "John Doe",
						"aliases":    []interface{}{"JD", "Johnny"},
						"file_class": "contact",
						"custom":     "field",
					},
				},
			},
			fileClassKey: "file_class",
			checkFunc: func(t *testing.T, metadata map[string]interface{}) {
				assert.Equal(t, "test1", metadata["id"])
				assert.Equal(t, "/notes/contact.md", metadata["path"])
				assert.Equal(t, "John Doe", metadata["title"])
				assert.Equal(t, "contact", metadata["file_class"])
				assert.Contains(t, metadata, "frontmatter")
				assert.Contains(t, metadata, "file_mod_time")
				assert.Contains(t, metadata, "index_time")
			},
		},
		{
			name: "handles missing fields gracefully",
			note: domain.Note{
				ID:   domain.NewNoteID("minimal"),
				Path: "/notes/minimal.md",
				Frontmatter: domain.Frontmatter{
					Fields: map[string]interface{}{},
				},
			},
			fileClassKey: "file_class",
			checkFunc: func(t *testing.T, metadata map[string]interface{}) {
				assert.Equal(t, "minimal", metadata["id"])
				assert.Equal(t, "/notes/minimal.md", metadata["path"])
				assert.NotContains(t, metadata, "title")
				assert.NotContains(t, metadata, "file_class")
			},
		},
		{
			name: "uses configurable file class key",
			note: domain.Note{
				ID:   domain.NewNoteID("custom"),
				Path: "/notes/custom.md",
				Frontmatter: domain.Frontmatter{
					Fields: map[string]interface{}{
						"type": "project",
					},
				},
			},
			fileClassKey: "type",
			checkFunc: func(t *testing.T, metadata map[string]interface{}) {
				assert.Equal(t, "project", metadata["file_class"])
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			metadata, err := extractSQLiteNoteMetadata(tt.note, tt.fileClassKey)
			require.NoError(t, err)
			tt.checkFunc(t, metadata)
		})
	}
}

// Test_initSQLiteSchema tests the function.
func Test_initSQLiteSchema(t *testing.T) {
	db, err := sql.Open("sqlite", ":memory:")
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	err = initSQLiteSchema(db)
	require.NoError(t, err)

	// Verify tables
	var tableCount int
	tableCountErr := db.QueryRowContext(
		context.Background(),
		"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='notes'",
	).Scan(&tableCount)
	require.NoError(t, tableCountErr)
	assert.Equal(t, 1, tableCount)

	// Verify indexes
	var indexCount int
	indexCountErr := db.QueryRowContext(
		context.Background(),
		"SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_notes%'",
	).Scan(&indexCount)
	require.NoError(t, indexCountErr)
	assert.Equal(t, 5, indexCount)
}
