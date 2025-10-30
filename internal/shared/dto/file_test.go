package dto

import (
	"io/fs"
	"os"
	"path/filepath"
	"testing"
	"time"
)

// mockFileInfo implements fs.FileInfo for testing.
type mockFileInfo struct {
	name    string
	size    int64
	mode    fs.FileMode
	modTime time.Time
	isDir   bool
}

// Name returns the mock file name.
func (m mockFileInfo) Name() string { return m.name }

// Size returns the mock file size.
func (m mockFileInfo) Size() int64 { return m.size }

// Mode returns the mock file mode.
func (m mockFileInfo) Mode() fs.FileMode { return m.mode }

// ModTime returns the mock modification time.
func (m mockFileInfo) ModTime() time.Time { return m.modTime }

// IsDir returns whether the mock represents a directory.
func (m mockFileInfo) IsDir() bool { return m.isDir }

// Sys returns mock system-specific data.
func (m mockFileInfo) Sys() any { return nil }

// TestNewFileMetadata tests FileMetadata construction and field computation
//
// coverage.
//
//nolint:gocognit // Test function with multiple scenarios for comprehensive
func TestNewFileMetadata(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		info     fs.FileInfo
		expected FileMetadata
	}{
		{
			name: "markdown file in subdirectory",
			path: "/vault/templates/note.md",
			info: mockFileInfo{
				name:    "note.md",
				size:    1024,
				modTime: time.Date(2025, 1, 15, 10, 30, 0, 0, time.UTC),
				isDir:   false,
			},
			expected: FileMetadata{
				Path:     "/vault/templates/note.md",
				Basename: "note",
				Folder:   "/vault/templates",
				Ext:      ".md",
				ModTime:  time.Date(2025, 1, 15, 10, 30, 0, 0, time.UTC),
				Size:     1024,
				MimeType: "text/markdown",
			},
		},
		{
			name: "json file in root",
			path: "config.json",
			info: mockFileInfo{
				name:    "config.json",
				size:    512,
				modTime: time.Date(2025, 2, 1, 14, 0, 0, 0, time.UTC),
				isDir:   false,
			},
			expected: FileMetadata{
				Path:     "config.json",
				Basename: "config",
				Folder:   ".",
				Ext:      ".json",
				ModTime:  time.Date(2025, 2, 1, 14, 0, 0, 0, time.UTC),
				Size:     512,
				MimeType: "application/json",
			},
		},
		{
			name: "file without extension",
			path: "/tmp/README",
			info: mockFileInfo{
				name:    "README",
				size:    256,
				modTime: time.Date(2025, 3, 10, 9, 15, 0, 0, time.UTC),
				isDir:   false,
			},
			expected: FileMetadata{
				Path:     "/tmp/README",
				Basename: "README",
				Folder:   "/tmp",
				Ext:      "",
				ModTime:  time.Date(2025, 3, 10, 9, 15, 0, 0, time.UTC),
				Size:     256,
				MimeType: "application/octet-stream",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := NewFileMetadata(tt.path, tt.info)

			if result.Path != tt.expected.Path {
				t.Errorf("Path = %v, want %v", result.Path, tt.expected.Path)
			}
			if result.Basename != tt.expected.Basename {
				t.Errorf(
					"Basename = %v, want %v",
					result.Basename,
					tt.expected.Basename,
				)
			}
			if result.Folder != tt.expected.Folder {
				t.Errorf(
					"Folder = %v, want %v",
					result.Folder,
					tt.expected.Folder,
				)
			}
			if result.Ext != tt.expected.Ext {
				t.Errorf("Ext = %v, want %v", result.Ext, tt.expected.Ext)
			}
			if !result.ModTime.Equal(tt.expected.ModTime) {
				t.Errorf(
					"ModTime = %v, want %v",
					result.ModTime,
					tt.expected.ModTime,
				)
			}
			if result.Size != tt.expected.Size {
				t.Errorf("Size = %v, want %v", result.Size, tt.expected.Size)
			}
			if result.MimeType != tt.expected.MimeType {
				t.Errorf(
					"MimeType = %v, want %v",
					result.MimeType,
					tt.expected.MimeType,
				)
			}
		})
	}
}

// TestComputeBasename tests the computeBasename helper function.
func TestComputeBasename(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		expected string
	}{
		{
			name:     "absolute path with extension",
			path:     "/vault/templates/note.md",
			expected: "note",
		},
		{
			name:     "relative path with extension",
			path:     "templates/contact.md",
			expected: "contact",
		},
		{
			name:     "file without extension",
			path:     "/tmp/README",
			expected: "README",
		},
		{
			name:     "deep nested path",
			path:     "a/b/c/d/file.txt",
			expected: "file",
		},
		{
			name:     "multiple dots in filename",
			path:     "/path/to/file.v1.2.md",
			expected: "file.v1.2",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := computeBasename(tt.path)
			if result != tt.expected {
				t.Errorf(
					"computeBasename(%q) = %q, want %q",
					tt.path,
					result,
					tt.expected,
				)
			}
		})
	}
}

// TestComputeFolder tests the computeFolder helper function.
func TestComputeFolder(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		expected string
	}{
		{
			name:     "absolute path with directory",
			path:     "/vault/templates/note.md",
			expected: "/vault/templates",
		},
		{
			name:     "relative path with directory",
			path:     "templates/contact.md",
			expected: "templates",
		},
		{
			name:     "file in current directory",
			path:     "README.md",
			expected: ".",
		},
		{
			name:     "deep nested path",
			path:     "a/b/c/d/file.txt",
			expected: "a/b/c/d",
		},
		{
			name:     "root level file",
			path:     "/README",
			expected: "/",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := computeFolder(tt.path)
			if result != tt.expected {
				t.Errorf(
					"computeFolder(%q) = %q, want %q",
					tt.path,
					result,
					tt.expected,
				)
			}
		})
	}
}

// TestComputeMimeType tests the computeMimeType helper function.
func TestComputeMimeType(t *testing.T) {
	tests := []struct {
		name     string
		ext      string
		expected string
	}{
		{
			name:     "markdown extension",
			ext:      ".md",
			expected: "text/markdown",
		},
		{
			name:     "json extension",
			ext:      ".json",
			expected: "application/json",
		},
		{
			name:     "text extension",
			ext:      ".txt",
			expected: "text/plain; charset=utf-8",
		},
		{
			name:     "unknown extension",
			ext:      ".xyz",
			expected: "chemical/x-xyz",
		},
		{
			name:     "no extension",
			ext:      "",
			expected: "application/octet-stream",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := computeMimeType(tt.ext)
			if result != tt.expected {
				t.Errorf(
					"computeMimeType(%q) = %q, want %q",
					tt.ext,
					result,
					tt.expected,
				)
			}
		})
	}
}

// TestFileMetadataIntegration tests FileMetadata creation with real filesystem.
func TestFileMetadataIntegration(t *testing.T) {
	const (
		testFileExt      = ".md"
		testFileMimeType = "text/markdown"
	)
	// Create a temporary file for integration testing
	tempDir := t.TempDir()
	tempFile := filepath.Join(tempDir, "test.md")

	content := "# Test Template\n\nThis is a test template."
	err := os.WriteFile(tempFile, []byte(content), 0o600)
	if err != nil {
		t.Fatalf("Failed to create temp file: %v", err)
	}

	// Get file info
	info, err := os.Stat(tempFile)
	if err != nil {
		t.Fatalf("Failed to stat temp file: %v", err)
	}

	// Create metadata
	metadata := NewFileMetadata(tempFile, info)

	// Verify computed fields
	expectedBasename := "test"
	if metadata.Basename != expectedBasename {
		t.Errorf("Basename = %q, want %q", metadata.Basename, expectedBasename)
	}

	expectedFolder := tempDir
	if metadata.Folder != expectedFolder {
		t.Errorf("Folder = %q, want %q", metadata.Folder, expectedFolder)
	}

	expectedExt := testFileExt
	if metadata.Ext != expectedExt {
		t.Errorf("Ext = %q, want %q", metadata.Ext, expectedExt)
	}

	expectedMimeType := testFileMimeType
	if metadata.MimeType != expectedMimeType {
		t.Errorf("MimeType = %q, want %q", metadata.MimeType, expectedMimeType)
	}

	// Verify size is reasonable (content length + some overhead)
	if metadata.Size <= 0 {
		t.Errorf("Size = %d, want > 0", metadata.Size)
	}
}
