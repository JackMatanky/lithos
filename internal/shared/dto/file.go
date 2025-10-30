// Package dto provides shared data transfer objects used across infrastructure
// layers. These DTOs represent infrastructure-specific models that are never
// exposed to the domain layer.
//
// This package contains reusable types that implement port interfaces,
// maintaining clean separation between interface contracts and concrete types.
package dto

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
//
// Reference: docs/architecture/data-models.md#filemetadata.
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

// VaultFile represents a data transfer object combining filesystem metadata
// with file content for vault indexing workflow. This DTO is used exclusively
// between vault scanning adapters and indexing services, never exposed to
// the domain layer.
//
// VaultFile embeds FileMetadata for filesystem information and adds raw
// content bytes needed for indexing operations like frontmatter extraction.
//
// Reference: docs/architecture/data-models.md#vaultfile.
type VaultFile struct {
	FileMetadata // Embedded filesystem metadata

	Content []byte // Raw file content for indexing
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
// Special handling for .md files as text/markdown.
func computeMimeType(ext string) string {
	// Special case for markdown files
	if ext == ".md" {
		return "text/markdown"
	}
	mimeType := mime.TypeByExtension(ext)
	if mimeType == "" {
		return defaultMimeType
	}
	return mimeType
}

// NewVaultFile creates a VaultFile DTO from FileMetadata and content.
// This constructor combines filesystem metadata with file content for
// vault indexing operations. The content may be nil for large files
// (post-MVP optimization).
func NewVaultFile(metadata FileMetadata, content []byte) VaultFile {
	return VaultFile{
		FileMetadata: metadata,
		Content:      content,
	}
}
