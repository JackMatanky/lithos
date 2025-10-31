package utils

import (
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

// TestRootReturnsExistingDirectory verifies that Root returns an absolute path
// to an existing testdata directory.
func TestRootReturnsExistingDirectory(t *testing.T) {
	root := Root(t)

	require.DirExists(t, root)
	require.True(t, filepath.IsAbs(root), "root should be absolute")
}

// TestPathResolvesAllowedFixture verifies that Path correctly resolves paths
// within allowed root directories.
func TestPathResolvesAllowedFixture(t *testing.T) {
	path := Path(t, "schemas", "valid", "note.json")
	require.FileExists(t, path)

	base := filepath.Base(path)
	require.Equal(t, "note.json", base)
}

// TestPathAllowsTopLevelFixture verifies that Path can resolve top-level
// fixture files directly under testdata.
func TestPathAllowsTopLevelFixture(t *testing.T) {
	path := Path(t, "basic_note.md")
	require.FileExists(t, path)
}

// TestOpenReturnsReadableFile verifies that Open returns a readable file
// handle for valid fixtures.
func TestOpenReturnsReadableFile(t *testing.T) {
	file := Open(t, "templates", "static_template.md")
	t.Cleanup(func() { _ = file.Close() })

	info, err := file.Stat()
	require.NoError(t, err)
	require.Positive(t, info.Size())
}
