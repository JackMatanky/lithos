package cache

import (
	"context"
	"database/sql"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	_ "modernc.org/sqlite" // Register SQLite driver
)

// TestSQLiteCacheReadAdapter_NewSQLiteCacheReadAdapter tests the function.
func TestSQLiteCacheReadAdapter_NewSQLiteCacheReadAdapter(t *testing.T) {
	tests := []struct {
		name      string
		config    domain.Config
		setupFunc func(t *testing.T, cacheDir string)
		wantErr   bool
	}{
		{
			name: "success - opens existing database",
			config: domain.Config{
				CacheDir:     t.TempDir(),
				FileClassKey: "file_class",
			},
			setupFunc: func(t *testing.T, cacheDir string) {
				// Create database first
				db, err := sql.Open("sqlite", cacheDir+"/cache.db")
				require.NoError(t, err)
				defer func() { _ = db.Close() }()

				err = initSQLiteSchema(db)
				require.NoError(t, err)
			},
			wantErr: false,
		},
		{
			name: "success - opens non-existent database (SQLite creates it)",
			config: domain.Config{
				CacheDir:     t.TempDir(),
				FileClassKey: "file_class",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.setupFunc != nil {
				tt.setupFunc(t, tt.config.CacheDir)
			}

			log := zerolog.New(zerolog.NewTestWriter(t))
			adapter, err := NewSQLiteCacheReadAdapter(tt.config, log)

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

			// Cleanup
			_ = adapter.Close()
		})
	}
}

// TestSQLiteCacheReadAdapter_Read tests the function.
func TestSQLiteCacheReadAdapter_Read(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	const testContextCancelled = "error - context canceled"

	// Setup: create writer adapter and persist some notes
	writer, err := NewSQLiteCacheWriteAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	note1 := domain.Note{
		ID: domain.NewNoteID("test-note-1"),
		Frontmatter: domain.Frontmatter{
			FileClass: "contact",
			Fields: map[string]interface{}{
				"title":      "John Doe",
				"aliases":    []interface{}{"JD", "Johnny"},
				"file_class": "contact",
				"custom":     "field",
			},
		},
	}

	ctx := context.Background()
	err = writer.Persist(ctx, note1)
	require.NoError(t, err)

	// Now test reader
	reader, err := NewSQLiteCacheReadAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	tests := []struct {
		name         string
		noteID       domain.NoteID
		wantErr      bool
		expectedNote *domain.Note
	}{
		{
			name:    "success - reads existing note",
			noteID:  note1.ID,
			wantErr: false,
			expectedNote: &domain.Note{
				ID: note1.ID,
				Frontmatter: domain.Frontmatter{
					FileClass: "contact",
					Fields: map[string]interface{}{
						"title":      "John Doe",
						"aliases":    []interface{}{"JD", "Johnny"},
						"file_class": "contact",
						"custom":     "field",
					},
				},
			},
		},
		{
			name:    "error - note not found",
			noteID:  domain.NewNoteID("non-existent"),
			wantErr: true,
		},
		{
			name:    "error - context canceled",
			noteID:  domain.NewNoteID("canceled"),
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.name == testContextCancelled {
				cancelCtx, cancel := context.WithCancel(context.Background())
				cancel()
				_, readErr := reader.Read(cancelCtx, tt.noteID)
				require.Error(t, readErr)
				return
			}

			readCtx := context.Background()
			note, readErr := reader.Read(readCtx, tt.noteID)
			if tt.wantErr {
				require.Error(t, readErr)
				return
			}

			require.NoError(t, readErr)
			assert.Equal(t, tt.expectedNote.ID, note.ID)
			assert.Equal(
				t,
				tt.expectedNote.Frontmatter.FileClass,
				note.Frontmatter.FileClass,
			)
			assert.Equal(
				t,
				tt.expectedNote.Frontmatter.Fields["title"],
				note.Frontmatter.Fields["title"],
			)
			assert.Contains(
				t,
				note.Frontmatter.Fields,
				"custom",
			) // Unknown field preserved
		})
	}
}

// TestSQLiteCacheReadAdapter_List tests the function.
func TestSQLiteCacheReadAdapter_List(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	// Setup: create writer adapter and persist some notes
	writer, err := NewSQLiteCacheWriteAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	notes := []domain.Note{
		{
			ID: domain.NewNoteID("note1"),
			Frontmatter: domain.Frontmatter{
				FileClass: "contact",
				Fields: map[string]interface{}{
					"title": "Note 1",
				},
			},
		},
		{
			ID: domain.NewNoteID("note2"),
			Frontmatter: domain.Frontmatter{
				FileClass: "project",
				Fields: map[string]interface{}{
					"title": "Note 2",
				},
			},
		},
	}

	ctx := context.Background()
	for _, note := range notes {
		err = writer.Persist(ctx, note)
		require.NoError(t, err)
	}

	// Now test reader
	reader, err := NewSQLiteCacheReadAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	tests := []struct {
		name          string
		wantErr       bool
		expectedCount int
		contextCancel bool
	}{
		{
			name:          "success - lists all notes",
			wantErr:       false,
			expectedCount: 2,
		},
		{
			name:          "success - empty database returns empty slice",
			wantErr:       false,
			expectedCount: 0,
		},
		{
			name:          "error - context canceled",
			wantErr:       true,
			contextCancel: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			testCtx := context.Background()

			// Handle context cancellation test
			if tt.contextCancel {
				cancelCtx, cancel := context.WithCancel(context.Background())
				cancel()
				_, listErr := reader.List(cancelCtx)
				require.Error(t, listErr)
				assert.Equal(t, context.Canceled, listErr)
				return
			}

			// Handle empty database test
			if tt.expectedCount == 0 {
				emptyCacheDir := t.TempDir()
				emptyConfig := domain.Config{
					CacheDir:     emptyCacheDir,
					FileClassKey: "file_class",
				}
				emptyWriter, emptyWriterErr := NewSQLiteCacheWriteAdapter(
					emptyConfig,
					log,
				)
				require.NoError(t, emptyWriterErr)
				emptyReader, emptyReaderErr := NewSQLiteCacheReadAdapter(
					emptyConfig,
					log,
				)
				require.NoError(t, emptyReaderErr)
				defer func() { _ = emptyWriter.Close() }()
				defer func() { _ = emptyReader.Close() }()

				result, listErr := emptyReader.List(testCtx)
				require.NoError(t, listErr)
				assert.Empty(t, result)
				return
			}

			// Handle normal test cases
			result, listErr := reader.List(testCtx)
			require.NoError(t, listErr)
			assert.Len(t, result, tt.expectedCount)

			// Verify notes are returned
			noteIDs := make(map[string]bool)
			for _, note := range result {
				noteIDs[string(note.ID)] = true
			}
			assert.True(t, noteIDs["note1"])
			assert.True(t, noteIDs["note2"])
		})
	}
}

// TestSQLiteCacheReadAdapter_ListStale tests the function.
func TestSQLiteCacheReadAdapter_ListStale(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	// Setup: create writer adapter and persist notes with different mod times
	writer, err := NewSQLiteCacheWriteAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	baseTime := time.Date(2024, 1, 1, 12, 0, 0, 0, time.UTC)

	notes := []domain.Note{
		{
			ID: domain.NewNoteID("fresh-note"),
			Frontmatter: domain.Frontmatter{
				FileClass: "note",
				Fields: map[string]interface{}{
					"title": "Fresh Note",
					"file_mod_time": baseTime.Add(
						2 * time.Hour,
					), // Modified 2 hours after base
				},
			},
		},
		{
			ID: domain.NewNoteID("stale-note"),
			Frontmatter: domain.Frontmatter{
				FileClass: "note",
				Fields: map[string]interface{}{
					"title": "Stale Note",
					"file_mod_time": baseTime.Add(
						-1 * time.Hour,
					), // Modified 1 hour before base
				},
			},
		},
		{
			ID: domain.NewNoteID("exact-note"),
			Frontmatter: domain.Frontmatter{
				FileClass: "note",
				Fields: map[string]interface{}{
					"title":         "Exact Note",
					"file_mod_time": baseTime, // Modified exactly at base time
				},
			},
		},
	}

	ctx := context.Background()
	for _, note := range notes {
		err = writer.Persist(ctx, note)
		require.NoError(t, err)
	}

	// Now test reader
	reader, err := NewSQLiteCacheReadAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	tests := []struct {
		name          string
		since         time.Time
		expectedIDs   []string
		contextCancel bool
	}{
		{
			name:  "success - returns notes modified after base time",
			since: baseTime,
			expectedIDs: []string{
				"fresh-note",
			}, // Only fresh-note is after baseTime
		},
		{
			name:        "success - returns all notes when since is old",
			since:       baseTime.Add(-2 * time.Hour),
			expectedIDs: []string{"fresh-note", "stale-note", "exact-note"},
		},
		{
			name:        "success - returns no notes when since is future",
			since:       baseTime.Add(3 * time.Hour),
			expectedIDs: []string{}, // No notes modified after this time
		},
		{
			name:          "error - context canceled",
			since:         baseTime,
			contextCancel: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			testCtx := context.Background()

			if tt.contextCancel {
				cancelCtx, cancel := context.WithCancel(context.Background())
				cancel()
				_, listErr := reader.ListStale(cancelCtx, tt.since)
				require.Error(t, listErr)
				assert.Equal(t, context.Canceled, listErr)
				return
			}

			staleNotes, listErr := reader.ListStale(testCtx, tt.since)
			require.NoError(t, listErr)

			actualIDs := make([]string, len(staleNotes))
			for i, note := range staleNotes {
				actualIDs[i] = string(note.ID)
			}

			assert.ElementsMatch(t, tt.expectedIDs, actualIDs)

			// Verify notes are ordered by file_mod_time ASC (basic check that
			// we have expected notes)
			for _, expectedID := range tt.expectedIDs {
				found := false
				for _, note := range staleNotes {
					if string(note.ID) == expectedID {
						found = true
						break
					}
				}
				assert.True(
					t,
					found,
					"Expected note %s not found in results",
					expectedID,
				)
			}
		})
	}
}

// TestSQLiteCacheReadAdapter_Close tests the function.
func TestSQLiteCacheReadAdapter_Close(t *testing.T) {
	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "file_class",
	}
	log := zerolog.New(zerolog.NewTestWriter(t))

	// Setup database
	writer, err := NewSQLiteCacheWriteAdapter(config, log)
	require.NoError(t, err)
	_ = writer.Close()

	reader, err := NewSQLiteCacheReadAdapter(config, log)
	require.NoError(t, err)

	// Verify database is open
	assert.NotNil(t, reader.db)

	// Close it
	err = reader.Close()
	require.NoError(t, err)

	// Verify we can't use it after close (should get error)
	ctx := context.Background()
	_, err = reader.Read(ctx, domain.NewNoteID("test"))
	require.Error(t, err) // Should fail after close
}
