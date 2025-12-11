// Package dto provides data transfer objects used by SPI adapters.
// These DTOs handle infrastructure-specific concerns and should never
// be exposed to the domain layer.
//
// This package contains VaultFile DTO for vault scanning and file operations,
// using fs.FileInfo delegation and vault-relative paths for cross-platform
// compatibility.
package dto

import (
	"io/fs"
	"path/filepath"
	"strings"
	"time"
)

// VaultFile represents a data transfer object for vault file scanning.
// Returns vault-relative file metadata with content for use by
// VaultReaderAdapter internally.
//
// Architecture Layer: SPI Adapter (Transport DTO)
// Location: internal/adapters/spi/dto/vault_file.go
//
// VaultFile is a lean DTO that delegates to Go stdlib fs.FileInfo instead of
// duplicating fields. Uses vault-relative paths for portability. VaultFile
// is internal to adapters - VaultReaderAdapter constructs Note domain models
// from VaultFile and returns Notes to application layer.
//
// Reference: docs/architecture/data-models.md#vaultfile.
type VaultFile struct {
	// Path is vault-relative path with forward slashes (e.g.,
	// "notes/meeting.md").
	// Portable across platforms. Normalized using filepath.ToSlash().
	Path string

	// Info delegates to Go stdlib for ModTime, Size, Mode, IsDir.
	// No duplication.
	Info fs.FileInfo

	// Content is raw file content loaded on-demand.
	// For MVP: markdown text from .md files.
	Content []byte
}

// NewVaultFile creates VaultFile from absolute path and fs.FileInfo.
func NewVaultFile(
	absPath, vaultRoot string,
	info fs.FileInfo,
	content []byte,
) (VaultFile, error) {
	relPath, err := NormalizePath(absPath, vaultRoot)
	if err != nil {
		return VaultFile{}, err
	}

	return VaultFile{
		Path:    relPath,
		Info:    info,
		Content: content,
	}, nil
}

// Basename returns filename without extension.
func (v VaultFile) Basename() string {
	base := filepath.Base(v.Path)
	return strings.TrimSuffix(base, filepath.Ext(base))
}

// Folder returns parent directory path.
func (v VaultFile) Folder() string {
	return filepath.Dir(v.Path)
}

// Ext returns file extension with dot.
func (v VaultFile) Ext() string {
	return filepath.Ext(v.Path)
}

// ModifiedAt delegates to fs.FileInfo.
func (v VaultFile) ModifiedAt() time.Time {
	return v.Info.ModTime()
}

// Size delegates to fs.FileInfo.
func (v VaultFile) Size() int64 {
	return v.Info.Size()
}

// AbsolutePath helper for I/O operations.
func (v VaultFile) AbsolutePath(vaultRoot string) string {
	return filepath.Join(vaultRoot, filepath.FromSlash(v.Path))
}

// NormalizePath converts absolute path to vault-relative with forward slashes.
func NormalizePath(absPath, vaultRoot string) (string, error) {
	if vaultRoot == "" {
		return "", filepath.ErrBadPattern
	}

	relPath, err := filepath.Rel(vaultRoot, absPath)
	if err != nil {
		return "", err
	}

	relPath = filepath.Clean(relPath)
	relPath = filepath.ToSlash(relPath)

	if relPath == ".." || strings.HasPrefix(relPath, "../") {
		return "", filepath.ErrBadPattern
	}

	return relPath, nil
}
