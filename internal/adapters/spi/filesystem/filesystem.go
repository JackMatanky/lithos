// Package filesystem provides a local file system adapter implementing
// FileSystemPort.
//
// This adapter wraps Go's standard library filesystem operations with atomic
// write
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
func (a *LocalFileSystemAdapter) WriteFileAtomic(
	path string,
	data []byte,
) error {
	dir := filepath.Dir(path)
	if err := a.ensureDirectory(dir); err != nil {
		return err
	}

	tempFile, tempPath, err := a.createTempFile(dir, filepath.Base(path))
	if err != nil {
		return err
	}

	// Ensure cleanup on error
	defer func() {
		_ = tempFile.Close()
		if tempPath != "" {
			_ = os.Remove(tempPath)
		}
	}()

	if writeErr := a.writeAndSync(tempFile, data); writeErr != nil {
		return writeErr
	}

	if renameErr := a.atomicRename(tempPath, path); renameErr != nil {
		return renameErr
	}

	// Success - don't remove temp file in defer
	tempPath = ""
	return nil
}

// ensureDirectory creates the directory structure if it doesn't exist.
func (a *LocalFileSystemAdapter) ensureDirectory(dir string) error {
	return os.MkdirAll(dir, 0o750)
}

// createTempFile creates a temporary file in the specified directory.
func (a *LocalFileSystemAdapter) createTempFile(
	dir, baseName string,
) (*os.File, string, error) {
	tempFile, err := os.CreateTemp(dir, baseName+".tmp.*")
	if err != nil {
		return nil, "", err
	}
	tempPath := tempFile.Name()
	return tempFile, tempPath, nil
}

// writeAndSync writes data to the file and syncs it to disk.
func (a *LocalFileSystemAdapter) writeAndSync(
	file *os.File,
	data []byte,
) error {
	if _, err := file.Write(data); err != nil {
		return err
	}
	if err := file.Sync(); err != nil {
		return err
	}
	return file.Close()
}

// atomicRename performs the atomic rename operation.
func (a *LocalFileSystemAdapter) atomicRename(
	tempPath, targetPath string,
) error {
	return os.Rename(tempPath, targetPath)
}

// Walk traverses a directory tree starting at root, calling fn for each
// file and directory encountered. The fn callback receives the file path
// and a boolean indicating if it's a directory.
// If fn returns an error, walking stops and the error is returned.
func (a *LocalFileSystemAdapter) Walk(root string, fn spi.WalkFunc) error {
	return filepath.WalkDir(
		root,
		func(path string, d os.DirEntry, err error) error {
			if err != nil {
				return err
			}
			return fn(path, d.IsDir())
		},
	)
}
