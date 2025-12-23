package json

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosLog "github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const testContextCanceled = "error - context canceled"

// TestNewJSONCacheWriter tests the JSONCacheWriteAdapter constructor.
// TestNewJSONCacheWriter tests the function.
func TestNewJSONCacheWriter(t *testing.T) {
	log := lithosLog.New(os.Stdout, "debug")
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
// TestPersist tests the function.
//

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
			note: func() domain.Note {
				note, _ := domain.NewNote(
					"test-note",
					domain.NewFrontmatter(map[string]interface{}{
						"fileClass": "contact",
						"title":     "Test Note",
					}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
			wantErr: false,
		},
		{
			name: "success - serializes Note to JSON with proper structure",
			note: func() domain.Note {
				note, _ := domain.NewNote(
					"json-test",
					domain.NewFrontmatter(map[string]interface{}{
						"fileClass": "meeting",
						"title":     "JSON Test",
						"tags":      []string{"test", "json"},
					}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
			wantErr: false,
		},
		{
			name: "success - uses atomic write (temp file + rename)",
			note: func() domain.Note {
				note, _ := domain.NewNote(
					"atomic-test",
					domain.NewFrontmatter(map[string]interface{}{
						"fileClass": "contact",
						"title":     "Atomic Test",
					}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
			wantErr: false,
		},
		{
			name: "success - overwrites existing file atomically",
			note: func() domain.Note {
				note, _ := domain.NewNote(
					"overwrite-test",
					domain.NewFrontmatter(map[string]interface{}{
						"fileClass": "contact",
						"title":     "Overwrite Test",
					}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
			setupFunc: func(t *testing.T, cacheDir string) {
				// Pre-create a file to test overwrite
				path := cache.NoteFilePath(
					cacheDir,
					"overwrite-test",
				)
				err := os.MkdirAll(cacheDir, 0o750)
				require.NoError(t, err)
				err = os.WriteFile(path, []byte(`{"old": "content"}`), 0o600)
				require.NoError(t, err)
			},
			wantErr: false,
		},
		{
			name: "error - context canceled",
			note: func() domain.Note {
				note, _ := domain.NewNote(
					"error-test",
					domain.NewFrontmatter(map[string]interface{}{
						"fileClass": "contact",
						"title":     "Error Test",
					}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
			wantErr: true,
			errMsg:  "context canceled",
		},
		{
			name: "error - cache write failed",
			note: func() domain.Note {
				note, _ := domain.NewNote(
					"write-fail-test",
					domain.NewFrontmatter(map[string]interface{}{
						"fileClass": "contact",
						"title":     "Write Fail Test",
					}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
			setupFunc: func(t *testing.T, cacheDir string) {
				// Make cache directory inaccessible to cause write failure
				err := os.Chmod(cacheDir, 0o000)
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
			log := lithosLog.New(os.Stdout, "debug")
			config := domain.Config{CacheDir: cacheDir}
			adapter := NewJSONCacheWriter(config, log)

			// Run setup function if provided
			if tt.setupFunc != nil {
				tt.setupFunc(t, cacheDir)
			}

			// Execute Persist
			ctx := context.Background()
			if tt.name == testContextCanceled {
				var cancel context.CancelFunc
				ctx, cancel = context.WithCancel(context.Background())
				cancel() // Cancel immediately
			}
			metadata := spi.CacheWriteMetadata{IndexTime: time.Now()}
			err := adapter.Persist(ctx, tt.note, metadata)

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
			expectedPath := cache.NoteFilePath(cacheDir, tt.note.Path)
			assert.FileExists(t, expectedPath)

			// Legacy cache filename should be gone to avoid duplicates
			legacyPath := cache.LegacyNoteFilePath(cacheDir, tt.note.Path)
			assert.False(
				t,
				fileExists(legacyPath),
				"legacy cache file should be removed if present",
			)

			// Verify JSON content
			content, err := os.ReadFile(expectedPath)
			require.NoError(t, err)

			// Parse JSON and verify structure
			var jsonData map[string]interface{}
			err = json.Unmarshal(content, &jsonData)
			require.NoError(t, err)

			// Verify Path field
			assert.Equal(t, tt.note.Path, jsonData["Path"])

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

func fileExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

// TestDelete tests the Delete method with various scenarios.
// TestDelete tests the function.
func TestDelete(t *testing.T) {
	tests := []struct {
		name      string
		notePath  string
		setupFunc func(t *testing.T, cacheDir string)
		wantErr   bool
		errMsg    string
	}{
		{
			name:     "success - removes file",
			notePath: "delete-test",
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
			name:     "success - idempotent (non-existent file)",
			notePath: "non-existent",
			setupFunc: func(t *testing.T, cacheDir string) {
				// Ensure cache directory exists but file doesn't
				err := os.MkdirAll(cacheDir, 0o750)
				require.NoError(t, err)
			},
			wantErr: false,
		},
		{
			name:     "error - context canceled",
			notePath: "cancel-test",
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
			log := lithosLog.New(os.Stdout, "debug")
			config := domain.Config{CacheDir: cacheDir}
			adapter := NewJSONCacheWriter(config, log)

			// Run setup function if provided
			if tt.setupFunc != nil {
				tt.setupFunc(t, cacheDir)
			}

			// Execute Delete
			ctx := context.Background()
			if tt.name == testContextCanceled {
				var cancel context.CancelFunc
				ctx, cancel = context.WithCancel(context.Background())
				cancel() // Cancel immediately
			}
			err := adapter.Delete(ctx, tt.notePath)

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
			expectedPath := cache.NoteFilePath(cacheDir, tt.notePath)
			assert.NoFileExists(t, expectedPath)
		})
	}
}

// BenchmarkMarshalNote benchmarks JSON serialization performance.
// Measures the performance of compact JSON marshaling.
func BenchmarkMarshalNote(b *testing.B) {
	note, _ := domain.NewNote(
		"bench",
		domain.NewFrontmatter(map[string]interface{}{
			"title": "test",
		}),
		[]domain.Link{},
		[]domain.Heading{},
		[]string{},
		[]domain.TaskItem{},
	)
	for range b.N {
		_, _ = marshalNote(note)
	}
}

// TestMarshalNoteCompact verifies that JSON output is compact (not indented).
// TestMarshalNoteCompact tests the function.
func TestMarshalNoteCompact(t *testing.T) {
	note, err := domain.NewNote(
		"compact-test",
		domain.NewFrontmatter(map[string]interface{}{
			"fileClass": "contact",
			"title":     "Compact Test",
			"tags":      []string{"test", "compact"},
		}),
		[]domain.Link{},
		[]domain.Heading{},
		[]string{},
		[]domain.TaskItem{},
	)
	require.NoError(t, err)

	data, marshalErr := marshalNote(note)
	require.NoError(t, marshalErr)

	// Verify it's valid JSON
	var result map[string]interface{}
	err = json.Unmarshal(data, &result)
	require.NoError(t, err)

	// Verify it doesn't contain indentation (no indented newlines)
	jsonStr := string(data)
	assert.NotContains(
		t,
		jsonStr,
		"\n  ",
		"JSON should not contain indented newlines",
	)
	assert.NotContains(
		t,
		jsonStr,
		"\n\t",
		"JSON should not contain tab indentation",
	)

	// Verify structure is preserved
	assert.Equal(t, "compact-test", result["Path"])
	frontmatter := result["Frontmatter"].(map[string]interface{})
	fields := frontmatter["Fields"].(map[string]interface{})
	assert.Equal(t, "Compact Test", fields["title"])
}
