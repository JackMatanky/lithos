package filesystem

import (
	"bytes"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/jack/lithos/internal/ports/spi"
)

func TestLocalFileSystemAdapter_ReadFile(t *testing.T) {
	adapter := NewLocalFileSystemAdapter()

	// Create a temp directory for testing
	tempDir, err := os.MkdirTemp("", "lithos-fs-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Test reading existing file
	testFile := filepath.Join(tempDir, "test.txt")
	testContent := []byte("Hello, World!")
	if err2 := os.WriteFile(testFile, testContent, 0o600); err2 != nil {
		t.Fatalf("Failed to create test file: %v", err2)
	}

	content, err := adapter.ReadFile(testFile)
	if err != nil {
		t.Errorf("ReadFile() error = %v", err)
	}
	if !bytes.Equal(content, testContent) {
		t.Errorf("ReadFile() = %s, want %s", content, testContent)
	}

	// Test reading non-existent file
	nonExistentFile := filepath.Join(tempDir, "nonexistent.txt")
	_, err = adapter.ReadFile(nonExistentFile)
	if err == nil {
		t.Error("ReadFile() should return error for non-existent file")
	}
}

func TestLocalFileSystemAdapter_WriteFileAtomic(t *testing.T) {
	adapter := NewLocalFileSystemAdapter()

	// Create a temp directory for testing
	tempDir, err := os.MkdirTemp("", "lithos-fs-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Test writing to new file
	testFile := filepath.Join(tempDir, "write-test.txt")
	testContent := []byte("Atomic write test")

	err = adapter.WriteFileAtomic(testFile, testContent)
	if err != nil {
		t.Errorf("WriteFileAtomic() error = %v", err)
	}

	// Verify file was written correctly
	content, err := os.ReadFile(testFile)
	if err != nil {
		t.Errorf("Failed to read written file: %v", err)
	}
	if !bytes.Equal(content, testContent) {
		t.Errorf("File content = %s, want %s", content, testContent)
	}

	// Test writing to nested directory (should create directories)
	nestedFile := filepath.Join(tempDir, "subdir", "nested.txt")
	nestedContent := []byte("Nested file content")

	err = adapter.WriteFileAtomic(nestedFile, nestedContent)
	if err != nil {
		t.Errorf("WriteFileAtomic() error for nested file = %v", err)
	}

	// Verify nested file was written correctly
	content, err = os.ReadFile(nestedFile)
	if err != nil {
		t.Errorf("Failed to read nested file: %v", err)
	}
	if !bytes.Equal(content, nestedContent) {
		t.Errorf("Nested file content = %s, want %s", content, nestedContent)
	}

	// Test overwriting existing file
	newContent := []byte("Updated content")
	err = adapter.WriteFileAtomic(testFile, newContent)
	if err != nil {
		t.Errorf("WriteFileAtomic() error for overwrite = %v", err)
	}

	content, err = os.ReadFile(testFile)
	if err != nil {
		t.Errorf("Failed to read overwritten file: %v", err)
	}
	if !bytes.Equal(content, newContent) {
		t.Errorf("Overwritten file content = %s, want %s", content, newContent)
	}
}

func TestLocalFileSystemAdapter_WriteFileAtomic_Atomicity(t *testing.T) {
	adapter := NewLocalFileSystemAdapter()

	// Create a temp directory for testing
	tempDir, err := os.MkdirTemp("", "lithos-fs-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	testFile := filepath.Join(tempDir, "atomic-test.txt")
	originalContent := []byte("Original content")

	// Write initial content
	err = adapter.WriteFileAtomic(testFile, originalContent)
	if err != nil {
		t.Fatalf("Failed to write initial content: %v", err)
	}

	// Verify no temp files are left behind
	entries, err := os.ReadDir(tempDir)
	if err != nil {
		t.Fatalf("Failed to read temp dir: %v", err)
	}

	for _, entry := range entries {
		if strings.Contains(entry.Name(), ".tmp.") {
			t.Errorf("Temp file left behind: %s", entry.Name())
		}
	}

	// Verify only the target file exists
	if len(entries) != 1 || entries[0].Name() != "atomic-test.txt" {
		t.Errorf("Expected only target file, got: %v", entries)
	}
}

func TestLocalFileSystemAdapter_Walk(t *testing.T) {
	adapter := NewLocalFileSystemAdapter()

	// Create a temp directory with nested structure
	tempDir, err := os.MkdirTemp("", "lithos-fs-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Create test structure:
	// tempDir/
	//   ├── file1.txt
	//   ├── subdir1/
	//   │   ├── file2.txt
	//   │   └── subdir2/
	//   │       └── file3.txt
	//   └── file4.txt

	testFiles := []string{
		"file1.txt",
		"subdir1/file2.txt",
		"subdir1/subdir2/file3.txt",
		"file4.txt",
	}

	for _, file := range testFiles {
		fullPath := filepath.Join(tempDir, file)
		err2 := os.MkdirAll(filepath.Dir(fullPath), 0o750)
		if err2 != nil {
			t.Fatalf("Failed to create directory for %s: %v", file, err2)
		}
		err2 = os.WriteFile(fullPath, []byte("test content"), 0o600)
		if err2 != nil {
			t.Fatalf("Failed to create test file %s: %v", file, err2)
		}
	}

	// Walk the directory and collect results
	var walkResults []struct {
		path  string
		isDir bool
	}

	err = adapter.Walk(tempDir, func(path string, isDir bool) error {
		// Make path relative to tempDir for easier testing
		relPath, relErr := filepath.Rel(tempDir, path)
		if relErr != nil {
			return relErr
		}
		walkResults = append(walkResults, struct {
			path  string
			isDir bool
		}{relPath, isDir})
		return nil
	})

	if err != nil {
		t.Errorf("Walk() error = %v", err)
	}

	// Verify we found all expected files and directories
	expectedPaths := map[string]bool{
		".":                         true, // root directory
		"file1.txt":                 false,
		"file4.txt":                 false,
		"subdir1":                   true,
		"subdir1/file2.txt":         false,
		"subdir1/subdir2":           true,
		"subdir1/subdir2/file3.txt": false,
	}

	if len(walkResults) != len(expectedPaths) {
		t.Errorf(
			"Walk() found %d items, expected %d",
			len(walkResults),
			len(expectedPaths),
		)
	}

	for _, result := range walkResults {
		expectedIsDir, exists := expectedPaths[result.path]
		if !exists {
			t.Errorf("Walk() found unexpected path: %s", result.path)
			continue
		}
		if result.isDir != expectedIsDir {
			t.Errorf(
				"Walk() path %s isDir = %v, want %v",
				result.path,
				result.isDir,
				expectedIsDir,
			)
		}
	}
}

func TestLocalFileSystemAdapter_Walk_EarlyReturn(t *testing.T) {
	adapter := NewLocalFileSystemAdapter()

	// Create a temp directory with multiple files
	tempDir, err := os.MkdirTemp("", "lithos-fs-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Create test files
	for i := 1; i <= 3; i++ {
		filename := filepath.Join(tempDir, "file"+string(rune('0'+i))+".txt")
		err2 := os.WriteFile(filename, []byte("test"), 0o600)
		if err2 != nil {
			t.Fatalf("Failed to create test file: %v", err2)
		}
	}

	// Walk and return error after finding first file
	fileCount := 0
	customErr := os.ErrExist // Use a known error for testing

	err = adapter.Walk(tempDir, func(path string, isDir bool) error {
		if !isDir {
			fileCount++
			if fileCount >= 1 {
				return customErr
			}
		}
		return nil
	})

	if !errors.Is(err, customErr) {
		t.Errorf("Walk() should return custom error, got: %v", err)
	}

	if fileCount != 1 {
		t.Errorf(
			"Walk() should have stopped after 1 file, processed: %d",
			fileCount,
		)
	}
}

func TestLocalFileSystemAdapter_InterfaceCompliance(t *testing.T) {
	// Verify that LocalFileSystemAdapter implements FileSystemPort
	var _ spi.FileSystemPort = &LocalFileSystemAdapter{}
	var _ = NewLocalFileSystemAdapter()
}

func TestLocalFileSystemAdapter_Walk_NonExistentDirectory(t *testing.T) {
	adapter := NewLocalFileSystemAdapter()

	// Try to walk a non-existent directory
	err := adapter.Walk(
		"/nonexistent/directory",
		func(path string, isDir bool) error {
			return nil
		},
	)

	if err == nil {
		t.Error("Walk() should return error for non-existent directory")
	}
}
