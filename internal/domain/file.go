// Package domain contains the core business logic models for Lithos.
// These models represent domain concepts and contain no infrastructure
// dependencies.
package domain

import (
	"path/filepath"
	"strings"
	"time"
)

// File represents physical file identity and filesystem metadata.
// It separates file system concerns from content metadata, following
// the proven design pattern of separating file operations from content.
type File struct {
	Path     string    // Absolute path to note file, serves as primary key
	Basename string    // Filename without path and extension, computed from Path
	Folder   string    // Parent directory path, computed from Path
	ModTime  time.Time // File modification timestamp from os.Stat()
}

// NewFile creates File from path and filesystem metadata.
// Called by adapter during vault indexing.
func NewFile(path string, modTime time.Time) File {
	return File{
		Path:     path,
		Basename: computeBasename(path),
		Folder:   computeFolder(path),
		ModTime:  modTime,
	}
}

// computeBasename extracts basename from file path.
// Removes path and extension (e.g., "/vault/note.md" → "note").
func computeBasename(path string) string {
	base := filepath.Base(path)
	return strings.TrimSuffix(base, filepath.Ext(base))
}

// computeFolder extracts parent directory from file path.
// Returns directory path (e.g., "/vault/note.md" → "/vault").
func computeFolder(path string) string {
	return filepath.Dir(path)
}
