package cache

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNewJSONCacheWriter tests the JSONCacheWriteAdapter constructor.
func TestNewJSONCacheWriter(t *testing.T) {
	log := logger.New(os.Stdout, "debug")
	config := domain.Config{CacheDir: "/tmp/cache"}

	adapter := NewJSONCacheWriter(config, log)

	assert.NotNil(t, adapter)
	assert.NotNil(t, adapter.config)
	assert.NotNil(t, adapter.log)
	assert.Equal(t, config.CacheDir, adapter.config.CacheDir)

	// Verify interface compliance
	var _ spi.CacheWriterPort = adapter
}

// TestPersist tests the Persist method with various scenarios.
//
//nolint:gocognit // Complex test function with multiple scenarios is acceptable
func TestPersist(t *testing.T) {
	tests := []struct {
		name      string
		note      domain.Note
		setupFunc func(t *testing.T, cacheDir string)
		wantErr   bool
		errMsg    string
	}{
		{
			name: "success - creates directory and writes JSON",
			note: domain.NewNote(
				domain.NewNoteID("test-note"),
				domain.NewFrontmatter(map[string]interface{}{
					"fileClass": "contact",
					"title":     "Test Note",
				}),
			),
			wantErr: false,
		},
		{
			name: "success - serializes Note to JSON with proper structure",
			note: domain.NewNote(
				domain.NewNoteID("json-test"),
				domain.NewFrontmatter(map[string]interface{}{
					"fileClass": "meeting",
					"title":     "JSON Test",
					"tags":      []string{"test", "json"},
				}),
			),
			wantErr: false,
		},
		{
			name: "success - uses atomic write (temp file + rename)",
			note: domain.NewNote(
				domain.NewNoteID("atomic-test"),
				domain.NewFrontmatter(map[string]interface{}{
					"fileClass": "contact",
					"title":     "Atomic Test",
				}),
			),
			wantErr: false,
		},
		{
			name: "success - overwrites existing file atomically",
			note: domain.NewNote(
				domain.NewNoteID("overwrite-test"),
				domain.NewFrontmatter(map[string]interface{}{
					"fileClass": "contact",
					"title":     "Overwrite Test",
				}),
			),
			setupFunc: func(t *testing.T, cacheDir string) {
				// Pre-create a file to test overwrite
				path := filepath.Join(cacheDir, "overwrite-test.json")
				err := os.MkdirAll(cacheDir, 0o750)
				require.NoError(t, err)
				err = os.WriteFile(path, []byte(`{"old": "content"}`), 0o600)
				require.NoError(t, err)
			},
			wantErr: false,
		},
		{
			name: "error - wraps errors with context",
			note: domain.NewNote(
				domain.NewNoteID("error-test"),
				domain.NewFrontmatter(map[string]interface{}{
					"fileClass": "contact",
					"title":     "Error Test",
				}),
			),
			setupFunc: func(t *testing.T, cacheDir string) {
				// Make cache directory read-only to trigger error
				err := os.MkdirAll(cacheDir, 0o750)
				require.NoError(t, err)
				//nolint:gosec // Intentional for testing permission error
				// scenarios
				err = os.Chmod(cacheDir, 0o444) // Read-only

				require.NoError(t, err)
			},
			wantErr: true,
			errMsg:  "cache write failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup temp directory
			cacheDir := t.TempDir()
			log := logger.New(os.Stdout, "debug")
			config := domain.Config{CacheDir: cacheDir}
			adapter := NewJSONCacheWriter(config, log)

			// Run setup function if provided
			if tt.setupFunc != nil {
				tt.setupFunc(t, cacheDir)
			}

			// Execute Persist
			err := adapter.Persist(context.Background(), tt.note)

			// Assert error expectation
			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
				return
			}

			require.NoError(t, err)

			// Verify file was created
			expectedPath := noteFilePath(cacheDir, tt.note.ID)
			assert.FileExists(t, expectedPath)

			// Verify JSON content
			content, err := os.ReadFile(expectedPath)
			require.NoError(t, err)

			// Parse JSON and verify structure
			var jsonData map[string]interface{}
			err = json.Unmarshal(content, &jsonData)
			require.NoError(t, err)

			// Verify ID field
			assert.Equal(t, string(tt.note.ID), jsonData["ID"])

			// Verify Frontmatter structure
			frontmatter, ok := jsonData["Frontmatter"].(map[string]interface{})
			require.True(t, ok, "Frontmatter should be an object")

			// Verify Fields
			fields, ok := frontmatter["Fields"].(map[string]interface{})
			require.True(t, ok, "Fields should be an object")

			// Verify specific fields from test note
			for key, expectedValue := range tt.note.Frontmatter.Fields {
				actualValue, exists := fields[key]
				assert.True(t, exists, "Field %s should exist", key)
				// Handle JSON unmarshaling type conversion ([]string becomes
				// []interface{})
				//nolint:nestif // Acceptable complexity for test type
				// assertion logic
				if expectedSlice, expectedOk := expectedValue.([]string); expectedOk {
					if actualSlice, actualOk := actualValue.([]interface{}); actualOk {
						assert.Len(
							t,
							actualSlice,
							len(expectedSlice),
							"Slice length should match for %s",
							key,
						)
						for i, v := range expectedSlice {
							assert.Equal(
								t,
								v,
								actualSlice[i],
								"Slice element %d should match for %s",
								i,
								key,
							)
						}
					} else {
						t.Errorf("Expected []string but got %T for field %s", actualValue, key)
					}
				} else {
					assert.Equal(t, expectedValue, actualValue, "Field %s should match", key)
				}
			}

			// Verify no temp files remain (atomic write cleanup)
			files, err := os.ReadDir(cacheDir)
			require.NoError(t, err)
			for _, file := range files {
				assert.False(
					t,
					strings.HasPrefix(file.Name(), ".tmp"),
					"No temp files should remain: %s",
					file.Name(),
				)
			}
		})
	}
}

// TestDelete tests the Delete method with various scenarios.
func TestDelete(t *testing.T) {
	tests := []struct {
		name      string
		noteID    domain.NoteID
		setupFunc func(t *testing.T, cacheDir string)
		wantErr   bool
		errMsg    string
	}{
		{
			name:   "success - removes file",
			noteID: domain.NewNoteID("delete-test"),
			setupFunc: func(t *testing.T, cacheDir string) {
				// Pre-create a file to delete
				path := filepath.Join(cacheDir, "delete-test.json")
				err := os.MkdirAll(cacheDir, 0o750)
				require.NoError(t, err)
				err = os.WriteFile(path, []byte(`{"test": "data"}`), 0o600)
				require.NoError(t, err)
			},
			wantErr: false,
		},
		{
			name:   "success - idempotent (non-existent file)",
			noteID: domain.NewNoteID("non-existent"),
			setupFunc: func(t *testing.T, cacheDir string) {
				// Ensure cache directory exists but file doesn't
				err := os.MkdirAll(cacheDir, 0o750)
				require.NoError(t, err)
			},
			wantErr: false,
		},
		{
			name:   "error - context canceled",
			noteID: domain.NewNoteID("cancel-test"),
			setupFunc: func(t *testing.T, cacheDir string) {
				// Create file to delete
				path := filepath.Join(cacheDir, "cancel-test.json")
				err := os.MkdirAll(cacheDir, 0o750)
				require.NoError(t, err)
				err = os.WriteFile(path, []byte(`{"test": "data"}`), 0o600)
				require.NoError(t, err)
			},
			wantErr: true,
			errMsg:  "context canceled",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup temp directory
			cacheDir := t.TempDir()
			log := logger.New(os.Stdout, "debug")
			config := domain.Config{CacheDir: cacheDir}
			adapter := NewJSONCacheWriter(config, log)

			// Run setup function if provided
			if tt.setupFunc != nil {
				tt.setupFunc(t, cacheDir)
			}

			// Execute Delete
			ctx := context.Background()
			if tt.name == "error - context canceled" {
				var cancel context.CancelFunc
				ctx, cancel = context.WithCancel(context.Background())
				cancel() // Cancel immediately
			}
			err := adapter.Delete(ctx, tt.noteID)

			// Assert error expectation
			if tt.wantErr {
				assert.Error(t, err, "Expected error but got none")
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
				return
			}

			require.NoError(t, err)

			// Verify file was removed (if it existed)
			expectedPath := noteFilePath(cacheDir, tt.noteID)
			assert.NoFileExists(t, expectedPath)
		})
	}
}
