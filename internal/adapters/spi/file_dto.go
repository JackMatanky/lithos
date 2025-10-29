// Package spi provides Service Provider Interface (SPI) adapters that implement
// domain ports by translating between domain models and infrastructure
// concerns.
// This package contains infrastructure-specific models and adapters that are
// never exposed to the domain layer.
package spi

import (
	"io/fs"
	"mime"
	"path/filepath"
	"strings"
	"time"
)

const (
	// defaultMimeType is the fallback MIME type for unknown file extensions.
	defaultMimeType = "application/octet-stream"
)

// FileMetadata represents filesystem-specific metadata used exclusively by
// filesystem storage adapters. It maps domain identifiers (NoteID/TemplateID)
// to file paths and tracks file system state. This is an infrastructure model
// that should never be exposed to the domain layer.
//
// FileMetadata is used by adapters to translate between domain identifiers
// and filesystem paths, keeping infrastructure details isolated from business
// logic.
type FileMetadata struct {
	// Path is the absolute path to the file. This serves as the primary key
	// and is immutable once set. Domain models never see filesystem paths.
	Path string
	// Basename is the filename without path and extension (computed).
	// Used by template lookup functions and wikilink resolution.
	Basename string
	// Folder is the parent directory path (computed).
	// Used for file organization queries.
	Folder string
	// Ext is the file extension including dot (computed).
	// Used for file type filtering.
	Ext string
	// ModTime is the file modification timestamp.
	// Used for staleness detection and incremental indexing.
	ModTime time.Time
	// Size is the file size in bytes.
	// Used for filtering large files.
	Size int64
	// MimeType is the MIME type detected from extension or content (computed).
	// Used for file type classification.
	MimeType string
}

// NewFileMetadata creates FileMetadata from a file path and fs.FileInfo.
// This constructor computes all derived fields from the filesystem information,
// ensuring consistency and avoiding repeated string operations.
func NewFileMetadata(path string, info fs.FileInfo) FileMetadata {
	ext := filepath.Ext(path)
	return FileMetadata{
		Path:     path,
		Basename: computeBasename(path),
		Folder:   computeFolder(path),
		Ext:      ext,
		ModTime:  info.ModTime(),
		Size:     info.Size(),
		MimeType: computeMimeType(ext),
	}
}

// computeBasename extracts the basename from a file path.
// It removes both the directory path and file extension.
// Example: "/vault/templates/note.md" → "note".
func computeBasename(path string) string {
	base := filepath.Base(path)
	return strings.TrimSuffix(base, filepath.Ext(base))
}

// computeFolder extracts the parent directory from a file path.
// Example: "/vault/templates/note.md" → "/vault/templates".
func computeFolder(path string) string {
	return filepath.Dir(path)
}

// computeMimeType detects the MIME type from a file extension.
// Returns "application/octet-stream" for unknown extensions.
func computeMimeType(ext string) string {
	mimeType := mime.TypeByExtension(ext)
	if mimeType == "" {
		return defaultMimeType
	}
	return mimeType
}
