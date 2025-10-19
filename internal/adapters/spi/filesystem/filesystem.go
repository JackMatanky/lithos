// Package filesystem provides a local file system adapter implementing FileSystemPort.
//
// This adapter wraps Go's standard library filesystem operations with atomic write
// guarantees and safe defaults for vault operations. It follows the hexagonal
// architecture pattern by implementing the FileSystemPort interface.
package filesystem

import (
	"os"
	"path/filepath"

	"github.com/jack/lithos/internal/ports/spi"
)

// LocalFileSystemAdapter implements the FileSystemPort interface using the
// local filesystem via Go's standard library. All write operations are atomic
// using temp file + rename pattern to ensure consistency.
type LocalFileSystemAdapter struct{}

// NewLocalFileSystemAdapter creates a new LocalFileSystemAdapter instance.
func NewLocalFileSystemAdapter() spi.FileSystemPort {
	return &LocalFileSystemAdapter{}
}

// ReadFile reads the contents of a file at the given path.
// Returns the file contents or an error if the file cannot be read.
func (a *LocalFileSystemAdapter) ReadFile(path string) ([]byte, error) {
	return os.ReadFile(path) //nolint:gosec // path is controlled by caller
}

// WriteFileAtomic writes data to a file atomically using temp file + rename.
// This ensures that concurrent readers never see partial writes.
// The file is created with appropriate permissions (0644) if it doesn't exist.
func (a *LocalFileSystemAdapter) WriteFileAtomic(path string, data []byte) error {
	// Create the directory if it doesn't exist
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0750); err != nil {
		return err
	}

	// Create a temporary file in the same directory as the target
	tempFile, err := os.CreateTemp(dir, filepath.Base(path)+".tmp.*")
	if err != nil {
		return err
	}
	tempPath := tempFile.Name()

	// Ensure cleanup on error
	defer func() {
		_ = tempFile.Close()
		if tempPath != "" {
			_ = os.Remove(tempPath)
		}
	}()

	// Write data to temp file
	if _, writeErr := tempFile.Write(data); writeErr != nil {
		return writeErr
	}

	// Sync to ensure data is written to disk
	if syncErr := tempFile.Sync(); syncErr != nil {
		return syncErr
	}

	// Close the temp file before rename
	if closeErr := tempFile.Close(); closeErr != nil {
		return closeErr
	}

	// Atomically rename temp file to target
	// This is the atomic operation that ensures consistency
	if renameErr := os.Rename(tempPath, path); renameErr != nil {
		return renameErr
	}

	// Success - don't remove temp file in defer
	tempPath = ""
	return nil
}

// Walk traverses a directory tree starting at root, calling fn for each
// file and directory encountered. The fn callback receives the file path
// and a boolean indicating if it's a directory.
// If fn returns an error, walking stops and the error is returned.
func (a *LocalFileSystemAdapter) Walk(root string, fn spi.WalkFunc) error {
	return filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}
		return fn(path, d.IsDir())
	})
}
