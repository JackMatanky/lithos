package cache

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	lithosLog "github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNewJSONCacheReader tests the JSONCacheReadAdapter constructor.
// TestNewJSONCacheReader tests the function.
func TestNewJSONCacheReader(t *testing.T) {
	log := lithosLog.New(os.Stdout, "debug")
	config := domain.Config{CacheDir: "/tmp/cache"}

	adapter := NewJSONCacheReader(config, log)

	assert.NotNil(t, adapter)
	assert.NotNil(t, adapter.config)
	assert.NotNil(t, adapter.log)
	assert.Equal(t, config.CacheDir, adapter.config.CacheDir)

	// Verify interface compliance
	var _ spi.CacheReaderPort = adapter
}

// TestRead tests the Read method of JSONCacheReadAdapter.
// TestRead tests the function.
func TestRead(t *testing.T) {
	// Create temporary directory for tests
	cacheDir, err := os.MkdirTemp("", "cache_test")
	require.NoError(t, err)
	defer func() {
		_ = os.RemoveAll(cacheDir) // Ignore cleanup errors in tests
	}()

	log := lithosLog.New(os.Stdout, "debug")
	config := domain.Config{CacheDir: cacheDir}
	adapter := NewJSONCacheReader(config, log)

	tests := []struct {
		name         string
		noteID       domain.NoteID
		setupFunc    func(t *testing.T, cacheDir string, noteID domain.NoteID)
		wantErr      bool
		errContains  string
		validateFunc func(t *testing.T, note domain.Note, err error)
	}{
		{
			name:   "success - deserializes valid JSON",
			noteID: "test-note",
			setupFunc: func(t *testing.T, cacheDir string, noteID domain.NoteID) {
				path := noteFilePath(cacheDir, noteID)
				jsonData := `{
					"ID": "test-note",
					"Frontmatter": {
						"FileClass": "contact",
						"Fields": {
							"fileClass": "contact",
							"title": "Test Note",
							"custom_field": "preserved_value"
						}
					}
				}`
				writeErr := os.WriteFile(path, []byte(jsonData), 0o600)
				require.NoError(t, writeErr)
			},
			wantErr: false,
			validateFunc: func(t *testing.T, note domain.Note, err error) {
				assert.Equal(t, domain.NoteID("test-note"), note.ID)
				assert.Equal(t, "contact", note.Frontmatter.FileClass)
				assert.Equal(t, "Test Note", note.Frontmatter.Fields["title"])
				assert.Equal(
					t,
					"preserved_value",
					note.Frontmatter.Fields["custom_field"],
				)
			},
		},
		{
			name:   "error - returns ErrNotFound for missing file",
			noteID: "missing-note",
			setupFunc: func(t *testing.T, cacheDir string, noteID domain.NoteID) {
				// No setup - file doesn't exist
			},
			wantErr:     true,
			errContains: "not found",
			validateFunc: func(t *testing.T, note domain.Note, err error) {
				assert.Equal(t, lithosErr.ErrNotFound, err)
			},
		},
		{
			name:   "success - reads legacy cache filename",
			noteID: "legacy/path/note.md",
			setupFunc: func(t *testing.T, cacheDir string, noteID domain.NoteID) {
				path := legacyNoteFilePath(cacheDir, noteID)
				writeErr := os.WriteFile(path, []byte(`{
						"ID": "legacy/path/note.md",
						"Frontmatter": {
							"FileClass": "legacy",
							"Fields": {"fileClass": "legacy"}
						}
					}`), 0o600)
				require.NoError(t, writeErr)
			},
			wantErr: false,
			validateFunc: func(t *testing.T, note domain.Note, err error) {
				assert.Equal(t, domain.NoteID("legacy/path/note.md"), note.ID)
				assert.Equal(t, "legacy", note.Frontmatter.FileClass)
			},
		},
		{
			name:   "success - preserves unknown fields (FR6)",
			noteID: "unknown-fields-note",
			setupFunc: func(t *testing.T, cacheDir string, noteID domain.NoteID) {
				path := noteFilePath(cacheDir, noteID)
				jsonData := `{
					"ID": "unknown-fields-note",
					"Frontmatter": {
						"FileClass": "meeting",
						"Fields": {
							"fileClass": "meeting",
							"title": "Team Meeting",
							"date": "2025-01-15",
							"participants": ["alice", "bob"],
							"custom_metadata": {
								"priority": "high",
								"tags": ["urgent", "review"]
							},
							"unknown_field_1": "value1",
							"unknown_field_2": 42,
							"unknown_field_3": true
						}
					}
				}`
				writeErr := os.WriteFile(path, []byte(jsonData), 0o600)
				require.NoError(t, writeErr)
			},
			wantErr: false,
			validateFunc: func(t *testing.T, note domain.Note, err error) {
				assert.Equal(t, domain.NoteID("unknown-fields-note"), note.ID)
				assert.Equal(t, "meeting", note.Frontmatter.FileClass)
				// Verify all unknown fields are preserved
				assert.Equal(
					t,
					"value1",
					note.Frontmatter.Fields["unknown_field_1"],
				)
				assert.InDelta(
					t,
					float64(42),
					note.Frontmatter.Fields["unknown_field_2"],
					0.001,
				)
				assert.Equal(
					t,
					true,
					note.Frontmatter.Fields["unknown_field_3"],
				)
				// Verify known fields are also present
				assert.Equal(
					t,
					"Team Meeting",
					note.Frontmatter.Fields["title"],
				)
				customMeta := note.Frontmatter.Fields["custom_metadata"].(map[string]interface{})
				assert.Equal(t, "high", customMeta["priority"])
			},
		},
		{
			name:   "error - wraps errors correctly for malformed JSON",
			noteID: "malformed-note",
			setupFunc: func(t *testing.T, cacheDir string, noteID domain.NoteID) {
				path := noteFilePath(cacheDir, noteID)
				jsonData := `{"ID": "malformed-note", "Frontmatter": {invalid json}`
				writeErr := os.WriteFile(path, []byte(jsonData), 0o600)
				require.NoError(t, writeErr)
			},
			wantErr:     true,
			errContains: "cache read failed",
			validateFunc: func(t *testing.T, note domain.Note, err error) {
				assert.Contains(t, err.Error(), "malformed-note")
				assert.Contains(t, err.Error(), "cache read failed")
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup test data
			tt.setupFunc(t, cacheDir, tt.noteID)

			// Execute Read
			note, readErr := adapter.Read(context.Background(), tt.noteID)

			// Validate results
			if tt.wantErr {
				require.Error(t, readErr)
				if tt.errContains != "" {
					assert.Contains(t, readErr.Error(), tt.errContains)
				}
			} else {
				require.NoError(t, readErr)
			}

			if tt.validateFunc != nil {
				tt.validateFunc(t, note, readErr)
			}
		})
	}
}

// TestList tests the List method of JSONCacheReadAdapter.
// TestList tests the function.
func TestList(t *testing.T) {
	tests := []struct {
		name         string
		setupFunc    func(t *testing.T, cacheDir string)
		wantErr      bool
		errContains  string
		validateFunc func(t *testing.T, notes []domain.Note, err error)
	}{
		{
			name: "success - returns all notes",
			setupFunc: func(t *testing.T, cacheDir string) {
				// Create multiple valid JSON files
				note1 := `{
					"ID": "note1",
					"Frontmatter": {
						"FileClass": "contact",
						"Fields": {"title": "Note 1"}
					}
				}`
				note2 := `{
					"ID": "note2",
					"Frontmatter": {
						"FileClass": "meeting",
						"Fields": {"title": "Note 2"}
					}
				}`
				require.NoError(
					t,
					os.WriteFile(
						noteFilePath(cacheDir, domain.NoteID("note1")),
						[]byte(note1),
						0o600,
					),
				)
				require.NoError(
					t,
					os.WriteFile(
						noteFilePath(cacheDir, domain.NoteID("note2")),
						[]byte(note2),
						0o600,
					),
				)
			},
			wantErr: false,
			validateFunc: func(t *testing.T, notes []domain.Note, err error) {
				assert.Len(t, notes, 2)
				// Check that both notes are present (order may vary)
				noteIDs := make(map[domain.NoteID]bool)
				for _, note := range notes {
					noteIDs[note.ID] = true
				}
				assert.True(t, noteIDs["note1"])
				assert.True(t, noteIDs["note2"])
			},
		},
		{
			name: "success - returns empty slice for empty cache",
			setupFunc: func(t *testing.T, cacheDir string) {
				// No setup - empty directory
			},
			wantErr: false,
			validateFunc: func(t *testing.T, notes []domain.Note, err error) {
				assert.Empty(t, notes)
			},
		},
		{
			name: "success - handles partial failures (mixed valid/invalid)",
			setupFunc: func(t *testing.T, cacheDir string) {
				// Create valid JSON file
				validNote := `{
					"ID": "valid-note",
					"Frontmatter": {
						"FileClass": "contact",
						"Fields": {"title": "Valid Note"}
					}
				}`
				require.NoError(
					t,
					os.WriteFile(
						noteFilePath(cacheDir, domain.NoteID("valid-note")),
						[]byte(validNote),
						0o600,
					),
				)

				// Create invalid JSON file
				require.NoError(
					t,
					os.WriteFile(
						noteFilePath(cacheDir, domain.NoteID("invalid-note")),
						[]byte(`{"invalid": json`),
						0o600,
					),
				)
			},
			wantErr: false,
			validateFunc: func(t *testing.T, notes []domain.Note, err error) {
				// Should return only the valid note, despite invalid file
				assert.Len(t, notes, 1)
				assert.Equal(t, domain.NoteID("valid-note"), notes[0].ID)
			},
		},
		{
			name: "success - filters non-JSON files",
			setupFunc: func(t *testing.T, cacheDir string) {
				// Create valid JSON file
				validNote := `{
					"ID": "json-note",
					"Frontmatter": {
						"FileClass": "contact",
						"Fields": {"title": "JSON Note"}
					}
				}`
				require.NoError(
					t,
					os.WriteFile(
						noteFilePath(cacheDir, domain.NoteID("json-note")),
						[]byte(validNote),
						0o600,
					),
				)

				// Create non-JSON files that should be ignored
				require.NoError(
					t,
					os.WriteFile(
						filepath.Join(cacheDir, "text-file.txt"),
						[]byte("not json"),
						0o600,
					),
				)
				require.NoError(
					t,
					os.WriteFile(
						filepath.Join(cacheDir, "binary-file.dat"),
						[]byte{0x00, 0x01},
						0o600,
					),
				)
			},
			wantErr: false,
			validateFunc: func(t *testing.T, notes []domain.Note, err error) {
				// Should return only the JSON file, ignore others
				assert.Len(t, notes, 1)
				assert.Equal(t, domain.NoteID("json-note"), notes[0].ID)
			},
		},

		{
			name: "error - directory walk failure",
			setupFunc: func(t *testing.T, cacheDir string) {
				// Remove read permission from cache directory to cause walk
				// error
				require.NoError(t, os.Chmod(cacheDir, 0o000))
			},
			wantErr:     true,
			errContains: "cache read failed",
			validateFunc: func(t *testing.T, notes []domain.Note, err error) {
				assert.Empty(t, notes)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create fresh temporary directory for each test
			cacheDir, err := os.MkdirTemp("", "cache_list_test")
			require.NoError(t, err)
			defer func() {
				_ = os.RemoveAll(cacheDir) // Ignore cleanup errors in tests
			}()

			log := lithosLog.New(os.Stdout, "debug")
			config := domain.Config{CacheDir: cacheDir}
			adapter := NewJSONCacheReader(config, log)

			// Setup test data
			tt.setupFunc(t, cacheDir)

			// Execute List
			notes, listErr := adapter.List(context.Background())

			// Restore permissions for directory walk failure test
			if tt.name == "error - directory walk failure" {
				_ = os.Chmod( //nolint:gosec // Test cleanup requires directory permissions
					cacheDir,
					0o755,
				)
			}

			// Validate results
			if tt.wantErr {
				require.Error(t, listErr)
				if tt.errContains != "" {
					assert.Contains(t, listErr.Error(), tt.errContains)
				}
			} else {
				require.NoError(t, listErr)
			}

			if tt.validateFunc != nil {
				tt.validateFunc(t, notes, listErr)
			}
		})
	}
}
