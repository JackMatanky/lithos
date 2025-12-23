package cache

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNoteFilePath tests the NoteFilePath function with various inputs.
// TestNoteFilePath tests the function.
func TestNoteFilePath(t *testing.T) {
	cacheDir := "/tmp/cache"
	tests := []string{
		"test-note",
		"projects/notes/meeting.md",
		"projects\\notes\\meeting.md",
		"deep/nested/path/to/file.txt",
	}

	for _, id := range tests {
		t.Run(id, func(t *testing.T) {
			result := NoteFilePath(cacheDir, id)

			assert.True(
				t,
				strings.HasPrefix(result, cacheDir),
				"path should start with cache dir",
			)
			assert.True(
				t,
				strings.HasSuffix(result, cacheFileExt),
				"path should end with .json",
			)

			filename := filepath.Base(result)
			// Note: The new encoding uses base64 directly without prefix
			assert.True(
				t,
				strings.HasSuffix(filename, cacheFileExt),
				"filename should end with .json",
			)

			decoded, ok := DecodePathFromFilename(filename)
			require.True(t, ok, "filename should decode using new scheme")
			assert.Equal(
				t,
				strings.ReplaceAll(id, "\\", "/"),
				decoded,
			)
		})
	}

	t.Run("empty cache dir produces filename only", func(t *testing.T) {
		result := NoteFilePath("", "note")
		assert.False(
			t,
			strings.HasPrefix(result, "/"),
			"relative path expected",
		)
		assert.True(t, strings.HasSuffix(result, cacheFileExt))
	})
}

// TestEnsureCacheDir tests the EnsureCacheDir function with various
// scenarios.
// TestEnsureCacheDir tests the function.
func TestEnsureCacheDir(t *testing.T) {
	tests := []struct {
		name        string
		cacheDir    string
		setupFunc   func(t *testing.T, cacheDir string)
		cleanupFunc func(t *testing.T, cacheDir string)
		wantErr     bool
	}{
		{
			name:     "create new directory",
			cacheDir: t.TempDir() + "/new-cache",
			wantErr:  false,
		},
		{
			name:     "directory already exists",
			cacheDir: t.TempDir() + "/existing-cache",
			setupFunc: func(t *testing.T, cacheDir string) {
				err := os.MkdirAll(cacheDir, 0o750)
				require.NoError(t, err)
			},
			wantErr: false,
		},
		{
			name:     "nested directories",
			cacheDir: t.TempDir() + "/deep/nested/cache",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.setupFunc != nil {
				tt.setupFunc(t, tt.cacheDir)
			}

			err := EnsureCacheDir(tt.cacheDir)
			if tt.wantErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				// Verify directory exists
				info, statErr := os.Stat(tt.cacheDir)
				require.NoError(t, statErr)
				assert.True(t, info.IsDir())
			}

			if tt.cleanupFunc != nil {
				tt.cleanupFunc(t, tt.cacheDir)
			}
		})
	}
}
