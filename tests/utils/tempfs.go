package utils

import (
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

// Workspace provides a managed temporary directory for filesystem-heavy tests.
type Workspace struct {
	t    *testing.T
	root string
}

// NewWorkspace constructs a new workspace rooted at t.TempDir and registers
// cleanup to ensure the directory is removed even if tests fail.
func NewWorkspace(t *testing.T) *Workspace {
	t.Helper()

	root := t.TempDir()
	ws := &Workspace{t: t, root: root}

	t.Cleanup(func() {
		require.NoError(t, os.RemoveAll(root))
	})

	return ws
}

// Root returns the absolute path to the workspace root.
func (w *Workspace) Root() string {
	return w.root
}

// Path resolves a relative path within the workspace, failing the test if the
// path attempts to escape the workspace or contains invalid segments.
func (w *Workspace) Path(rel ...string) string {
	w.t.Helper()

	path, err := resolveWorkspacePath(w.root, rel...)
	require.NoError(w.t, err)
	return path
}

// MkdirAll creates a directory (and parents) inside the workspace.
func (w *Workspace) MkdirAll(rel string, perm fs.FileMode) {
	w.t.Helper()

	path := w.Path(rel)
	require.NoError(w.t, os.MkdirAll(path, perm))
}

// WriteFile writes data to a file inside the workspace, creating directories as
// needed and applying the provided permissions.
func (w *Workspace) WriteFile(rel string, data []byte, perm fs.FileMode) {
	w.t.Helper()

	path := w.Path(rel)
	require.NoError(w.t, os.MkdirAll(filepath.Dir(path), 0o750))
	require.NoError(w.t, os.WriteFile(path, data, perm))
}

// CopyFromTestdata copies a fixture (file or directory) from testdata into the
// workspace at the provided destination path.
func CopyFromTestdata(
	t *testing.T,
	workspace *Workspace,
	relDest string,
	fixturePath ...string,
) {
	t.Helper()

	require.NotNil(t, workspace, "workspace must not be nil")

	dest, err := resolveWorkspacePath(workspace.root, relDest)
	require.NoError(t, err)

	src := Path(t, fixturePath...)
	info, err := os.Stat(src)
	require.NoError(t, err)

	if info.IsDir() {
		copyDir(t, src, dest)
		return
	}

	copyFile(t, src, dest, info.Mode().Perm())
}

func resolveWorkspacePath(root string, segments ...string) (string, error) {
	if len(segments) == 0 {
		return "", fmt.Errorf("workspace path segments must not be empty")
	}

	cleaned := make([]string, 0, len(segments))
	for _, segment := range segments {
		if err := validateSegment(segment, &cleaned); err != nil {
			return "", err
		}
	}

	candidate := filepath.Join(append([]string{root}, cleaned...)...)
	rel, err := filepath.Rel(root, candidate)
	if err != nil {
		return "", fmt.Errorf("failed to resolve workspace path: %w", err)
	}
	if strings.HasPrefix(rel, "..") {
		return "", fmt.Errorf(
			"workspace path %q escapes workspace root",
			candidate,
		)
	}

	return candidate, nil
}

func validateSegment(segment string, cleaned *[]string) error {
	if segment == "" {
		return nil
	}
	if filepath.IsAbs(segment) {
		return fmt.Errorf("workspace path %q must be relative", segment)
	}

	parts := strings.Split(filepath.ToSlash(segment), "/")
	for _, part := range parts {
		if err := validatePart(part, segment); err != nil {
			return err
		}
	}

	*cleaned = append(*cleaned, segment)
	return nil
}

func validatePart(part, segment string) error {
	if part == "" || part == "." {
		return nil
	}
	if part == ".." {
		return fmt.Errorf(
			"workspace path %q attempts to escape the workspace",
			segment,
		)
	}
	name := normalizePart(part)
	if name != "" && !isSnakeCase(name) {
		return fmt.Errorf("workspace segment %q must be snake_case", part)
	}
	return nil
}

func normalizePart(part string) string {
	trimmed := strings.TrimPrefix(part, ".")
	if ext := filepath.Ext(trimmed); ext != "" {
		trimmed = trimmed[:len(trimmed)-len(ext)]
	}
	return trimmed
}

func copyDir(t *testing.T, src, dest string) {
	t.Helper()

	entries, err := os.ReadDir(src)
	require.NoError(t, err)

	require.NoError(t, os.MkdirAll(dest, 0o750))

	for _, entry := range entries {
		srcPath := filepath.Join(src, entry.Name())
		destPath := filepath.Join(dest, entry.Name())

		if entry.IsDir() {
			copyDir(t, srcPath, destPath)
			continue
		}

		entryInfo, entryErr := entry.Info()
		require.NoError(t, entryErr)
		copyFile(t, srcPath, destPath, entryInfo.Mode().Perm())
	}
}

func copyFile(t *testing.T, src, dest string, perm fs.FileMode) {
	t.Helper()

	require.NoError(t, os.MkdirAll(filepath.Dir(dest), 0o750))

	cleanSrc := filepath.Clean(src)
	data, err := os.ReadFile(cleanSrc)
	require.NoError(t, err)

	cleanDest := filepath.Clean(dest)
	require.NoError(t, os.WriteFile(cleanDest, data, perm))
}
