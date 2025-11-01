package utils

import (
	"context"
	"os"
	"os/exec"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

// FindProjectRoot finds the project root by looking for go.mod.
func FindProjectRoot(t *testing.T) string {
	t.Helper()
	dir, err := os.Getwd()
	require.NoError(t, err)
	for {
		if _, statErr := os.Stat(filepath.Join(dir, "go.mod")); statErr == nil {
			return dir
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			t.Fatal("Could not find project root (go.mod)")
		}
		dir = parent
	}
}

// CopyFile copies a file from src to dst.
func CopyFile(t *testing.T, src, dst string) {
	t.Helper()
	data, err := os.ReadFile(
		src,
	)
	require.NoError(t, err)
	require.NoError(
		t,
		os.WriteFile(
			dst,
			data,
			0o644,
		),
	)
}

// BuildLithosBinary builds the lithos binary and returns the path.
func BuildLithosBinary(t *testing.T, tempDir string) string {
	t.Helper()
	projectRoot := FindProjectRoot(t)
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-o",
		binaryPath,
		"./cmd/lithos",
	)
	cmd.Dir = projectRoot
	require.NoError(t, cmd.Run())
	return binaryPath
}

// ExecuteIndexCommand executes the lithos index command and returns output and
// error.
func ExecuteIndexCommand(binaryPath, vaultDir string) (string, error) {
	cmd := exec.CommandContext(
		context.Background(),
		binaryPath,
		"index",
		vaultDir,
	)
	output, err := cmd.CombinedOutput()
	return string(output), err
}

// VerifyStatistics verifies that the CLI output contains expected statistics.
func VerifyStatistics(t *testing.T, output string) {
	t.Helper()
	require.Contains(t, output, "Indexed")
	require.Contains(t, output, "files")
}
