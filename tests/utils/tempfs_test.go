package utils

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

// TestNewWorkspaceProvidesIsolatedDirectory verifies that NewWorkspace creates
// a temporary directory and resolves paths within it.
func TestNewWorkspaceProvidesIsolatedDirectory(t *testing.T) {
	ws := NewWorkspace(t)

	require.DirExists(t, ws.Root())

	resolved := ws.Path("templates")
	require.True(
		t,
		strings.HasPrefix(filepath.Clean(resolved), filepath.Clean(ws.Root())),
	)
}

// TestResolveWorkspacePathRejectsEscape verifies that path resolution rejects
// attempts to escape the workspace using parent directory references.
func TestResolveWorkspacePathRejectsEscape(t *testing.T) {
	root := t.TempDir()
	_, err := resolveWorkspacePath(root, "..", "outside")
	require.Error(t, err)
	require.Contains(t, err.Error(), "attempts to escape")
}

// TestResolveWorkspacePathRejectsNonSnakeCase verifies that path resolution
// rejects segments that do not follow snake_case naming conventions.
func TestResolveWorkspacePathRejectsNonSnakeCase(t *testing.T) {
	root := t.TempDir()
	_, err := resolveWorkspacePath(root, "schemas", "Invalid.json")
	require.Error(t, err)
	require.Contains(t, err.Error(), "snake_case")
}

// TestResolveWorkspacePathAllowsHiddenDirectories verifies that path
// resolution allows hidden directories (starting with a dot).
func TestResolveWorkspacePathAllowsHiddenDirectories(t *testing.T) {
	root := t.TempDir()
	path, err := resolveWorkspacePath(root, ".cache", "data.txt")
	require.NoError(t, err)
	require.True(
		t,
		strings.HasPrefix(filepath.Clean(path), filepath.Clean(root)),
	)
}

// TestCopyFromTestdataCopiesFile verifies that CopyFromTestdata correctly
// copies individual files with the expected permissions.
func TestCopyFromTestdataCopiesFile(t *testing.T) {
	ws := NewWorkspace(t)

	CopyFromTestdata(
		t,
		ws,
		filepath.Join("templates", "basic_note.md"),
		"templates",
		"basic_note.md",
	)

	dest := ws.Path("templates", "basic_note.md")
	require.FileExists(t, dest)

	src := Path(t, "templates", "basic_note.md")
	expected, err := os.ReadFile(src)
	require.NoError(t, err)

	actual, err := os.ReadFile(dest)
	require.NoError(t, err)
	require.Equal(t, expected, actual)

	info, err := os.Stat(dest)
	require.NoError(t, err)
	require.True(t, info.Mode().IsRegular())
}

// TestCopyFromTestdataCopiesDirectory verifies that CopyFromTestdata
// recursively copies directories with correct permissions.
func TestCopyFromTestdataCopiesDirectory(t *testing.T) {
	ws := NewWorkspace(t)

	CopyFromTestdata(
		t,
		ws,
		filepath.Join("schemas", "valid"),
		"schemas",
		"valid",
	)

	destFile := ws.Path("schemas", "valid", "note.json")
	require.FileExists(t, destFile)

	destDir := ws.Path("schemas", "valid")
	info, err := os.Stat(destDir)
	require.NoError(t, err)
	require.True(t, info.IsDir())
}

// TestWriteFileAndMkdirAll verifies that MkdirAll and WriteFile correctly
// create directories and write files within the workspace.
func TestWriteFileAndMkdirAll(t *testing.T) {
	ws := NewWorkspace(t)

	ws.MkdirAll("cache/data", 0o750)
	ws.WriteFile("cache/data/sample.txt", []byte("workspace"), 0o600)

	dir := ws.Path("cache", "data")
	require.DirExists(t, dir)

	content, err := os.ReadFile(filepath.Join(dir, "sample.txt"))
	require.NoError(t, err)
	require.Equal(t, []byte("workspace"), content)
}
