// Package spi defines service provider interface ports for hexagonal
// architecture.
//
// These interfaces define contracts that infrastructure adapters must implement
// to provide services to the domain layer. This package contains the driven
// ports
// that allow the domain to remain independent of external dependencies.
package spi

// WalkFunc is the callback function type used by the Walk method.
// It receives the file path and file info for each file/directory encountered.
// Return an error to stop walking and propagate the error up.
type WalkFunc func(path string, isDir bool) error

// FileSystemPort provides safe file read/write/walk operations for domain
// services.
//
// This port allows domain services to interact with the vault without importing
// the os package directly, maintaining the hexagonal architecture boundaries.
// All operations honor config-defined vault roots and use atomic write
// patterns.
type FileSystemPort interface {
	// ReadFile reads the contents of a file at the given path.
	// Returns the file contents or an error if the file cannot be read.
	ReadFile(path string) ([]byte, error)

	// WriteFileAtomic writes data to a file atomically using temp file +
	// rename.
	// This ensures that concurrent readers never see partial writes.
	// The file is created with appropriate permissions if it doesn't exist.
	WriteFileAtomic(path string, data []byte) error

	// Walk traverses a directory tree starting at root, calling fn for each
	// file and directory encountered. The fn callback receives the file path
	// and a boolean indicating if it's a directory.
	// If fn returns an error, walking stops and the error is returned.
	Walk(root string, fn WalkFunc) error
}
