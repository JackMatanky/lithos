package utils

import (
	"context"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

// Statistics represents parsed CLI output statistics.
type Statistics struct {
	Scanned  int
	Indexed  int
	Duration string
}

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
	require.NoError(t, os.MkdirAll(filepath.Dir(dst), 0o750))
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
	cmd.Dir = vaultDir // Change working directory to vault directory so config is loaded from there
	output, err := cmd.CombinedOutput()
	return string(output), err
}

// VerifyStatistics verifies that the CLI output contains expected statistics.
func VerifyStatistics(t *testing.T, output string) {
	t.Helper()
	require.Contains(t, output, "Indexed")
	require.Contains(t, output, "files")
}

// CopyDir recursively copies a directory from src to dst.
func CopyDir(t *testing.T, src, dst string) {
	t.Helper()
	require.NoError(t, os.MkdirAll(dst, 0o755))

	entries, err := os.ReadDir(src)
	require.NoError(t, err)

	for _, entry := range entries {
		srcPath := filepath.Join(src, entry.Name())
		dstPath := filepath.Join(dst, entry.Name())

		if entry.IsDir() {
			CopyDir(t, srcPath, dstPath)
		} else {
			CopyFile(t, srcPath, dstPath)
		}
	}
}

// ParseStatistics parses the CLI output and extracts statistics.
func ParseStatistics(t *testing.T, output string) Statistics {
	t.Helper()
	lines := strings.Split(output, "\n")
	stats := Statistics{
		Scanned:  0,
		Indexed:  0,
		Duration: "",
	}

	for _, line := range lines {
		line = strings.TrimSpace(line)
		switch {
		case strings.Contains(line, "Scanned:"):
			// Extract number from "Scanned: X files"
			parts := strings.Fields(line)
			if len(parts) >= 2 {
				if scanned, err := strconv.Atoi(parts[1]); err == nil {
					stats.Scanned = scanned
				}
			}
		case strings.Contains(line, "Indexed:"):
			// Extract number from "Indexed: X files"
			parts := strings.Fields(line)
			if len(parts) >= 2 {
				if indexed, err := strconv.Atoi(parts[1]); err == nil {
					stats.Indexed = indexed
				}
			}
		case strings.Contains(line, "Duration:"):
			stats.Duration = strings.TrimPrefix(line, "Duration:")
			stats.Duration = strings.TrimSpace(stats.Duration)
		}
	}

	return stats
}
