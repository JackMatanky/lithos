package boltdb

import (
	"context"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

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
	config, writer, reader := newTestBoltDBAdapters(t)
	defer func() { _ = reader.Close() }()

	persistNotes(t, writer,
		domain.Note{
			ID: domain.NewNoteID("/notes/test1.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "contact",
				Fields: map[string]interface{}{
					"title":      "Test Note 1",
					"aliases":    []interface{}{"alias1"},
					"file_class": "contact",
				},
			},
		},
		domain.Note{
			ID: domain.NewNoteID("/notes/test2.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "project",
				Fields: map[string]interface{}{
					"title":      "Test Note 2",
					"aliases":    []interface{}{"alias2", "alias2b"},
					"file_class": "project",
				},
			},
		},
	)

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
	config, writer, reader := newTestBoltDBAdapters(t)
	defer func() { _ = reader.Close() }()

	persistNotes(t, writer, sampleNotes()...)

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
		_, _, emptyReader := newTestBoltDBAdapters(t)
		defer func() { _ = emptyReader.Close() }()

		ctx := context.Background()
		notes, emptyListErr := emptyReader.List(ctx)

		require.NoError(t, emptyListErr)
		assert.Empty(t, notes)
	})
}

func TestBoltDBCacheReadAdapter_MetadataQueries(t *testing.T) {
	_, writer, reader := newTestBoltDBAdapters(t)
	defer func() { _ = reader.Close() }()

	persistNotes(t, writer,
		domain.Note{
			ID: domain.NewNoteID("notes/projects/alpha.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "project",
				Fields: map[string]interface{}{
					"title":      "Alpha",
					"aliases":    []interface{}{"alpha-main"},
					"file_class": "project",
				},
			},
		},
		domain.Note{
			ID: domain.NewNoteID("notes/contacts/jane.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "contact",
				Fields: map[string]interface{}{
					"title":      "Jane",
					"aliases":    []interface{}{"jane-doe"},
					"file_class": "contact",
				},
			},
		},
	)

	ctx := context.Background()
	tests := []struct {
		name    string
		query   func(context.Context, *BoltDBCacheReadAdapter) ([]domain.Note, error)
		wantIDs []string
	}{
		{
			name: "BasenameQuery",
			query: func(ctx context.Context, adapter *BoltDBCacheReadAdapter) ([]domain.Note, error) {
				return adapter.BasenameQuery(ctx, "alpha")
			},
			wantIDs: []string{"notes/projects/alpha.md"},
		},
		{
			name: "AliasQuery",
			query: func(ctx context.Context, adapter *BoltDBCacheReadAdapter) ([]domain.Note, error) {
				return adapter.AliasQuery(ctx, "jane-doe")
			},
			wantIDs: []string{"notes/contacts/jane.md"},
		},
		{
			name: "FileClassQuery",
			query: func(ctx context.Context, adapter *BoltDBCacheReadAdapter) ([]domain.Note, error) {
				return adapter.FileClassQuery(ctx, "project")
			},
			wantIDs: []string{"notes/projects/alpha.md"},
		},
		{
			name: "no matches returns empty slice",
			query: func(ctx context.Context, adapter *BoltDBCacheReadAdapter) ([]domain.Note, error) {
				return adapter.FileClassQuery(ctx, "unknown")
			},
			wantIDs: []string{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			notes, err := tt.query(ctx, reader)
			require.NoError(t, err)
			assert.ElementsMatch(t, tt.wantIDs, toIDs(notes))
		})
	}
}

func TestBoltDBCacheReadAdapter_PathQuery(t *testing.T) {
	_, writer, reader := newTestBoltDBAdapters(t)
	defer func() { _ = reader.Close() }()

	persistNotes(t, writer,
		domain.Note{
			ID: domain.NewNoteID("notes/projects/alpha.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "project",
				Fields: map[string]interface{}{
					"title":      "Alpha",
					"aliases":    []interface{}{"project-alpha"},
					"file_class": "project",
				},
			},
		},
		domain.Note{
			ID: domain.NewNoteID("notes/projects/beta.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "project",
				Fields: map[string]interface{}{
					"title":      "Beta",
					"aliases":    []interface{}{"project-beta"},
					"file_class": "project",
				},
			},
		},
	)

	ctx := context.Background()
	tests := []struct {
		name    string
		opts    spi.PathQueryOptions
		wantIDs []string
	}{
		{
			name: "full path match",
			opts: spi.PathQueryOptions{
				Value: "notes/projects/alpha.md",
				Scope: spi.PathQueryScopeFull,
			},
			wantIDs: []string{"notes/projects/alpha.md"},
		},
		{
			name: "full path no match",
			opts: spi.PathQueryOptions{
				Value: "notes/unknown.md",
				Scope: spi.PathQueryScopeFull,
			},
			wantIDs: []string{},
		},
		{
			name: "basename scope",
			opts: spi.PathQueryOptions{
				Value: "beta",
				Scope: spi.PathQueryScopeBasename,
			},
			wantIDs: []string{"notes/projects/beta.md"},
		},
		{
			name: "folder scope",
			opts: spi.PathQueryOptions{
				Value: "notes/projects",
				Scope: spi.PathQueryScopeFolder,
			},
			wantIDs: []string{
				"notes/projects/alpha.md",
				"notes/projects/beta.md",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			notes, err := reader.PathQuery(ctx, tt.opts)
			require.NoError(t, err)
			assert.ElementsMatch(t, tt.wantIDs, toIDs(notes))
		})
	}
}

// TestBoltDBCacheReadAdapter_IsStale tests the staleness detection.
func TestBoltDBCacheReadAdapter_IsStale(t *testing.T) {
	config, writer, reader := newTestBoltDBAdapters(t)
	defer func() { _ = reader.Close() }()

	ctx := context.Background()
	modTime := time.Now().Add(-2 * time.Minute)
	filePath := "notes/stale-check.md"
	createVaultFile(t, config.VaultPath, filePath, modTime)

	testNote := domain.Note{
		ID: domain.NewNoteID(filePath),
		Frontmatter: domain.Frontmatter{
			FileClass: "note",
			Fields: map[string]interface{}{
				"title":          "Test Note",
				"file_class":     "note",
				"file_mod_time":  modTime,
				"modified":       modTime,
				"file_mod_epoch": modTime.Unix(),
			},
		},
	}

	indexTime := modTime.Add(time.Minute)
	require.NoError(t, writer.Persist(ctx, testNote, indexTime))

	t.Run("fresh file returns false", func(t *testing.T) {
		stale, err := reader.IsStale(ctx, filePath)
		require.NoError(t, err)
		assert.False(t, stale)
	})

	t.Run("modified file returns true", func(t *testing.T) {
		updated := modTime.Add(3 * time.Minute)
		fullPath := filepath.Join(config.VaultPath, filePath)
		require.NoError(t, os.Chtimes(fullPath, updated, updated))

		stale, err := reader.IsStale(ctx, filePath)
		require.NoError(t, err)
		assert.True(t, stale)
	})

	t.Run("missing file returns true", func(t *testing.T) {
		fullPath := filepath.Join(config.VaultPath, filePath)
		require.NoError(t, os.Remove(fullPath))

		stale, err := reader.IsStale(ctx, filePath)
		require.NoError(t, err)
		assert.True(t, stale)
	})

	t.Run("missing cache entry returns true", func(t *testing.T) {
		stale, err := reader.IsStale(ctx, "notes/missing.md")
		require.NoError(t, err)
		assert.True(t, stale)
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

func newTestBoltDBAdapters(
	t *testing.T,
) (domain.Config, *BoltDBCacheWriteAdapter, *BoltDBCacheReadAdapter) {
	t.Helper()

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
		VaultPath:    cacheDir,
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	db, err := Open(config)
	require.NoError(t, err)

	writer, err := NewBoltDBCacheWriter(config, log, db)
	require.NoError(t, err)

	reader, err := NewBoltDBCacheReadAdapter(config, log, db)
	require.NoError(t, err)

	t.Cleanup(func() {
		_ = reader.Close()
		_ = writer.Close()
		_ = db.Close()
	})

	return config, writer, reader
}

func persistNotes(
	t *testing.T,
	writer *BoltDBCacheWriteAdapter,
	notes ...domain.Note,
) {
	t.Helper()
	ctx := context.Background()
	for _, note := range notes {
		require.NoError(t, writer.Persist(ctx, note, time.Now()))
	}
}

func sampleNotes() []domain.Note {
	return []domain.Note{
		{
			ID: domain.NewNoteID("/notes/test1.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "contact",
				Fields: map[string]interface{}{
					"title":      "Test Note 1",
					"aliases":    []interface{}{"alias1"},
					"file_class": "contact",
				},
			},
		},
		{
			ID: domain.NewNoteID("/notes/test2.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "project",
				Fields: map[string]interface{}{
					"title":      "Test Note 2",
					"aliases":    []interface{}{"alias2", "alias2b"},
					"file_class": "project",
				},
			},
		},
	}
}

func toIDs(notes []domain.Note) []string {
	ids := make([]string, 0, len(notes))
	for _, note := range notes {
		ids = append(ids, string(note.ID))
	}
	return ids
}

func createVaultFile(t *testing.T, root, relPath string, modTime time.Time) {
	t.Helper()

	fullPath := filepath.Join(root, relPath)
	require.NoError(t, os.MkdirAll(filepath.Dir(fullPath), 0o750))
	require.NoError(t, os.WriteFile(fullPath, []byte("content"), 0o640))
	require.NoError(t, os.Chtimes(fullPath, modTime, modTime))
}
