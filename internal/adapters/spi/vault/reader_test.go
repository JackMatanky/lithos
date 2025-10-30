package vault

import (
	"context"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// setupTestVault creates a temporary directory with test files for vault
// scanning tests.
// Returns the vault path and a cleanup function.
func setupTestVault(t *testing.T) string {
	t.Helper()

	tmpDir := t.TempDir()

	// Create vault structure
	require.NoError(t, os.MkdirAll(filepath.Join(tmpDir, "notes"), 0755))
	require.NoError(t, os.MkdirAll(filepath.Join(tmpDir, "attachments"), 0755))
	require.NoError(
		t,
		os.MkdirAll(filepath.Join(tmpDir, ".lithos", "cache"), 0755),
	)

	// Write test files
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, "notes", "contact.md"),
			[]byte("# Contact\n\nJohn Doe"),
			0644,
		),
	)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, "notes", "project.md"),
			[]byte("# Project\n\nSecret project"),
			0644,
		),
	)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, "attachments", "image.png"),
			[]byte("fake png content"),
			0644,
		),
	)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, "attachments", "doc.pdf"),
			[]byte("fake pdf content"),
			0644,
		),
	)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, ".lithos", "cache", "cached.json"),
			[]byte(`{"cached": true}`),
			0644,
		),
	)

	return tmpDir
}

func TestNewVaultReaderAdapter(t *testing.T) {
	config := domain.DefaultConfig()
	log := logger.NewTest()

	adapter := NewVaultReaderAdapter(config, log)

	assert.NotNil(t, adapter)
	assert.Equal(t, config, adapter.config)
	assert.Equal(t, log, adapter.log)
}

func TestScanAll_EmptyVault(t *testing.T) {
	vaultPath := t.TempDir()
	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())

	require.NoError(t, err)
	assert.Empty(t, files)
}

func TestScanAll_WithFiles(t *testing.T) {
	vaultPath := setupTestVault(t)
	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())

	require.NoError(t, err)
	assert.Len(t, files, 4) // 2 .md + 2 attachments, excludes .lithos/cache

	// Check that all files are VaultFile instances with proper structure
	for _, vf := range files {
		assert.NotEmpty(t, vf.Path)
		assert.NotEmpty(t, vf.Basename)
		assert.NotEmpty(t, vf.Folder)
		assert.NotEmpty(t, vf.Ext)
		assert.NotZero(t, vf.ModTime)
		assert.Positive(t, vf.Size)
		assert.NotEmpty(t, vf.MimeType)
		assert.NotNil(t, vf.Content)
	}
}

func TestScanAll_IgnoresCacheDirectories(t *testing.T) {
	vaultPath := setupTestVault(t)
	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())

	require.NoError(t, err)

	// Ensure no files from .lithos/cache directory are included
	for _, vf := range files {
		assert.NotContains(
			t,
			vf.Path,
			".lithos",
			"Cache files should be excluded: %s",
			vf.Path,
		)
	}
}

func TestScanAll_ConstructsVaultFileCorrectly(t *testing.T) {
	vaultPath := t.TempDir()
	testFile := filepath.Join(vaultPath, "test.md")
	content := []byte("# Test\n\nContent")
	require.NoError(t, os.WriteFile(testFile, content, 0644))

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())

	require.NoError(t, err)
	require.Len(t, files, 1)

	vf := files[0]
	assert.Equal(t, testFile, vf.Path)
	assert.Equal(t, "test", vf.Basename)
	assert.Equal(t, vaultPath, vf.Folder)
	assert.Equal(t, ".md", vf.Ext)
	assert.Equal(t, int64(len(content)), vf.Size)
	assert.Equal(t, content, vf.Content)
	assert.Equal(t, "text/markdown", vf.MimeType)
}

func TestScanAll_WithPermissionErrors(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping permission test in short mode")
	}

	vaultPath := t.TempDir()
	testFile := filepath.Join(vaultPath, "readable.md")
	require.NoError(t, os.WriteFile(testFile, []byte("content"), 0644))

	// Create a file with no read permissions (if possible)
	unreadableFile := filepath.Join(vaultPath, "unreadable.md")
	require.NoError(t, os.WriteFile(unreadableFile, []byte("secret"), 0000))
	defer os.Chmod(unreadableFile, 0644) // Cleanup

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())

	// Should not fail completely, should return readable files
	require.NoError(t, err)
	assert.Len(t, files, 1) // Only the readable file
	assert.Equal(t, testFile, files[0].Path)
}

func TestScanModified_WithRecentFiles(t *testing.T) {
	vaultPath := setupTestVault(t)

	// Set modification time of one file to be old
	oldFile := filepath.Join(vaultPath, "notes", "contact.md")
	oldTime := time.Now().Add(-24 * time.Hour)
	require.NoError(t, os.Chtimes(oldFile, oldTime, oldTime))

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	since := time.Now().Add(-1 * time.Hour) // 1 hour ago
	files, err := adapter.ScanModified(context.Background(), since)

	require.NoError(t, err)
	assert.Len(t, files, 3) // 3 files modified within last hour

	// Ensure old file is not included
	for _, vf := range files {
		assert.NotEqual(t, oldFile, vf.Path, "Old file should be excluded")
	}
}

func TestScanModified_WithNoMatches(t *testing.T) {
	vaultPath := setupTestVault(t)
	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	since := time.Now().Add(1 * time.Hour) // Future time
	files, err := adapter.ScanModified(context.Background(), since)

	require.NoError(t, err)
	assert.Empty(t, files)
}

func TestRead_ValidFile(t *testing.T) {
	vaultPath := t.TempDir()
	testFile := filepath.Join(vaultPath, "test.md")
	content := []byte("# Test\n\nContent")
	require.NoError(t, os.WriteFile(testFile, content, 0644))

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	vf, err := adapter.Read(context.Background(), testFile)

	require.NoError(t, err)
	assert.Equal(t, testFile, vf.Path)
	assert.Equal(t, "test", vf.Basename)
	assert.Equal(t, vaultPath, vf.Folder)
	assert.Equal(t, ".md", vf.Ext)
	assert.Equal(t, content, vf.Content)
	assert.Equal(t, "text/markdown", vf.MimeType)
}

func TestRead_MissingFile(t *testing.T) {
	vaultPath := t.TempDir()
	missingFile := filepath.Join(vaultPath, "missing.md")

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	_, err := adapter.Read(context.Background(), missingFile)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "file not found")
}

func TestRead_PathTraversalPrevention(t *testing.T) {
	vaultPath := t.TempDir()
	// Create a file outside vault
	outsideFile := filepath.Join(t.TempDir(), "outside.md")
	require.NoError(t, os.WriteFile(outsideFile, []byte("outside"), 0644))

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	// Try to read file outside vault using path traversal
	traversalPath := filepath.Join(vaultPath, "..", filepath.Base(outsideFile))

	_, err := adapter.Read(context.Background(), traversalPath)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "path outside vault")
}

func TestRead_WithPermissionError(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping permission test in short mode")
	}

	vaultPath := t.TempDir()
	unreadableFile := filepath.Join(vaultPath, "unreadable.md")
	require.NoError(t, os.WriteFile(unreadableFile, []byte("secret"), 0000))
	defer os.Chmod(unreadableFile, 0644) // Cleanup

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	_, err := adapter.Read(context.Background(), unreadableFile)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "resource operation failed")
}

func TestVaultFile_NewVaultFile(t *testing.T) {
	metadata := spi.FileMetadata{
		Path:     "/vault/test.md",
		Basename: "test",
		Folder:   "/vault",
		Ext:      ".md",
		ModTime:  time.Now(),
		Size:     100,
		MimeType: "text/markdown",
	}
	content := []byte("# Test")

	vf := spi.NewVaultFile(metadata, content)

	assert.Equal(t, metadata, vf.FileMetadata)
	assert.Equal(t, content, vf.Content)
}
