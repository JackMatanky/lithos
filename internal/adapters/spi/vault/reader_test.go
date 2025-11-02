package vault

import (
	"context"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/dto"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockFileInfo implements fs.FileInfo for testing.
type mockFileInfo struct {
	name    string
	size    int64
	modTime time.Time
	isDir   bool
}

// Name returns the name of the file.
func (m *mockFileInfo) Name() string { return m.name }

// Size returns the size of the file.
func (m *mockFileInfo) Size() int64 { return m.size }

// Mode returns the file mode.
func (m *mockFileInfo) Mode() os.FileMode { return 0o644 }

// ModTime returns the modification time.
func (m *mockFileInfo) ModTime() time.Time { return m.modTime }

// IsDir returns whether the file is a directory.
func (m *mockFileInfo) IsDir() bool { return m.isDir }

// Sys returns the underlying data source.
func (m *mockFileInfo) Sys() any { return nil }

// setupTestVault creates a temporary directory with test files for vault
// scanning tests.
// Returns the vault path and a cleanup function.
func setupTestVault(t *testing.T) string {
	t.Helper()

	tmpDir := t.TempDir()

	// Create vault structure
	require.NoError(t, os.MkdirAll(filepath.Join(tmpDir, "notes"), 0o750))
	require.NoError(t, os.MkdirAll(filepath.Join(tmpDir, "attachments"), 0o750))
	require.NoError(
		t,
		os.MkdirAll(filepath.Join(tmpDir, ".lithos", "cache"), 0o750),
	)

	// Write test files
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, "notes", "contact.md"),
			[]byte("# Contact\n\nJohn Doe"),
			0o600,
		),
	)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, "notes", "project.md"),
			[]byte("# Project\n\nSecret project"),
			0o600,
		),
	)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, "attachments", "image.png"),
			[]byte("fake png content"),
			0o600,
		),
	)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, "attachments", "doc.pdf"),
			[]byte("fake pdf content"),
			0o600,
		),
	)
	require.NoError(
		t,
		os.WriteFile(
			filepath.Join(tmpDir, ".lithos", "cache", "cached.json"),
			[]byte(`{"cached": true}`),
			0o600,
		),
	)

	return tmpDir
}

// TestNewVaultReaderAdapter tests the creation of a new VaultReaderAdapter.
func TestNewVaultReaderAdapter(t *testing.T) {
	config := domain.DefaultConfig()
	log := logger.NewTest()

	adapter := NewVaultReaderAdapter(config, log)

	assert.NotNil(t, adapter)
	assert.Equal(t, config, adapter.config)
	assert.Equal(t, log, adapter.log)
}

// TestScanAll_EmptyVault tests scanning an empty vault directory.
func TestScanAll_EmptyVault(t *testing.T) {
	vaultPath := t.TempDir()
	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())

	require.NoError(t, err)
	assert.Empty(t, files)
}

// TestScanAll_WithFiles tests scanning a vault with existing files.
func TestScanAll_WithFiles(t *testing.T) {
	vaultPath := setupTestVault(t)
	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())

	require.NoError(t, err)
	assert.Len(
		t,
		files,
		2,
	) // Only .md files, excludes attachments and .lithos/cache

	// Check that all files are VaultFile instances with proper structure
	for _, vf := range files {
		assert.NotEmpty(t, vf.Path)
		assert.NotEmpty(t, vf.Basename)
		assert.NotEmpty(t, vf.Folder)
		assert.Equal(t, ".md", vf.Ext)
		assert.NotZero(t, vf.ModTime)
		assert.Positive(t, vf.Size)
		assert.Equal(t, "text/markdown", vf.MimeType)
		assert.NotNil(t, vf.Content)
	}
}

// TestScanAll_IgnoresCacheDirectories tests that cache directories are excluded
// from scanning.
func TestScanAll_IgnoresCacheDirectories(t *testing.T) {
	vaultPath := setupTestVault(t)
	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
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

// TestScanAll_ConstructsVaultFileCorrectly tests that VaultFile objects are
// constructed correctly.
func TestScanAll_ConstructsVaultFileCorrectly(t *testing.T) {
	vaultPath := t.TempDir()
	testFile := filepath.Join(vaultPath, "test.md")
	content := []byte("# Test\n\nContent")
	require.NoError(t, os.WriteFile(testFile, content, 0o600))

	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
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

// TestScanAll_WithPermissionErrors tests handling of files with permission
// errors.
func TestScanAll_WithPermissionErrors(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping permission test in short mode")
	}

	vaultPath := t.TempDir()
	testFile := filepath.Join(vaultPath, "readable.md")
	require.NoError(t, os.WriteFile(testFile, []byte("content"), 0o600))

	// Create a file with no read permissions (if possible)
	unreadableFile := filepath.Join(vaultPath, "unreadable.md")
	require.NoError(t, os.WriteFile(unreadableFile, []byte("secret"), 0o000))
	defer func() {
		_ = os.Chmod(unreadableFile, 0o644) // #nosec G302 - cleanup operation
	}()

	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())

	// Should not fail completely, should return readable files
	require.NoError(t, err)
	assert.Len(t, files, 1) // Only the readable file
	assert.Equal(t, testFile, files[0].Path)
}

// TestScanModified_WithRecentFiles tests scanning for files modified after a
// specific time.
func TestScanModified_WithRecentFiles(t *testing.T) {
	vaultPath := setupTestVault(t)

	// Set modification time of one file to be old
	oldFile := filepath.Join(vaultPath, "notes", "contact.md")
	oldTime := time.Now().Add(-24 * time.Hour)
	require.NoError(t, os.Chtimes(oldFile, oldTime, oldTime))

	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	since := time.Now().Add(-1 * time.Hour) // 1 hour ago
	files, err := adapter.ScanModified(context.Background(), since)

	require.NoError(t, err)
	assert.Len(t, files, 1) // Only project.md is recent and markdown

	// Ensure old file is not included
	for _, vf := range files {
		assert.NotEqual(t, oldFile, vf.Path, "Old file should be excluded")
		assert.Equal(t, ".md", vf.Ext, "Only markdown files should be included")
	}
}

// TestScanModified_WithNoMatches tests scanning when no files match the
// modification time criteria.
func TestScanModified_WithNoMatches(t *testing.T) {
	vaultPath := setupTestVault(t)
	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	since := time.Now().Add(1 * time.Hour) // Future time
	files, err := adapter.ScanModified(context.Background(), since)

	require.NoError(t, err)
	assert.Empty(t, files)
}

// TestRead_ValidFile tests reading a valid file from the vault.
func TestRead_ValidFile(t *testing.T) {
	vaultPath := t.TempDir()
	testFile := filepath.Join(vaultPath, "test.md")
	content := []byte("# Test\n\nContent")
	require.NoError(t, os.WriteFile(testFile, content, 0o600))

	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
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

// TestRead_MissingFile tests reading a file that doesn't exist.
func TestRead_MissingFile(t *testing.T) {
	vaultPath := t.TempDir()
	missingFile := filepath.Join(vaultPath, "missing.md")

	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	_, err := adapter.Read(context.Background(), missingFile)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "file not found")
}

// TestRead_PathTraversalPrevention tests that path traversal attacks are
// prevented.
func TestRead_PathTraversalPrevention(t *testing.T) {
	vaultPath := t.TempDir()
	// Create a file outside vault
	outsideFile := filepath.Join(t.TempDir(), "outside.md")
	require.NoError(t, os.WriteFile(outsideFile, []byte("outside"), 0o600))

	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	// Try to read file outside vault using path traversal
	traversalPath := filepath.Join(vaultPath, "..", filepath.Base(outsideFile))

	_, err := adapter.Read(context.Background(), traversalPath)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "path outside vault")
}

// TestRead_WithPermissionError tests reading a file with permission errors.
func TestRead_WithPermissionError(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping permission test in short mode")
	}

	vaultPath := t.TempDir()
	unreadableFile := filepath.Join(vaultPath, "unreadable.md")
	require.NoError(t, os.WriteFile(unreadableFile, []byte("secret"), 0o000))
	defer func() {
		_ = os.Chmod(unreadableFile, 0o644) // #nosec G302 - cleanup operation
	}()

	config := domain.NewConfig(vaultPath, "", "", "", "", "", "")
	adapter := NewVaultReaderAdapter(config, logger.NewTest())

	_, err := adapter.Read(context.Background(), unreadableFile)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "resource operation failed")
}

// TestVaultFile_NewVaultFile tests creating a new VaultFile instance.
func TestVaultFile_NewVaultFile(t *testing.T) {
	content := []byte("# Test Content")
	info := &mockFileInfo{
		name:    "file.md",
		size:    100,
		modTime: time.Now(),
		isDir:   false,
	}
	metadata := dto.NewFileMetadata("/test/file.md", info)
	vf := dto.NewVaultFile(metadata, content)

	assert.Equal(t, metadata, vf.FileMetadata)
	assert.Equal(t, content, vf.Content)
}
