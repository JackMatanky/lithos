package cache

import (
	"os"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNoteFilePath tests the noteFilePath function with various inputs.
func TestNoteFilePath(t *testing.T) {
	tests := []struct {
		name     string
		cacheDir string
		id       domain.NoteID
		expected string
	}{
		{
			name:     "basic path construction",
			cacheDir: "/tmp/cache",
			id:       "test-note",
			expected: "/tmp/cache/test-note.json",
		},
		{
			name:     "empty cache dir",
			cacheDir: "",
			id:       "note",
			expected: "note.json",
		},
		{
			name:     "note ID with special characters",
			cacheDir: "/cache",
			id:       "note-with-dashes_and_underscores",
			expected: "/cache/note-with-dashes_and_underscores.json",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := noteFilePath(tt.cacheDir, tt.id)
			assert.Equal(t, tt.expected, result)
		})
	}
}

// TestEnsureCacheDir tests the ensureCacheDir function with various scenarios.
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

			err := ensureCacheDir(tt.cacheDir)
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
