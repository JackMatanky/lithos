// Package vault provides filesystem-based adapters for vault reading
// operations.
// This package implements the VaultReaderPort interface using standard Go
// filesystem operations with proper error handling and security measures.
//
// The adapter provides both full vault scanning and incremental scanning
// capabilities, with built-in cache directory filtering and path traversal
// prevention.
package vault

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/dto"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

const (
	// maxFileSizeBytes is the maximum size of a file that will be loaded into
	// memory.
	// Files larger than this will be skipped to prevent memory exhaustion.
	// 10MB should be sufficient for most markdown files while preventing abuse.
	maxFileSizeBytes = 10 * 1024 * 1024 // 10MB
)

// Compile-time checks to ensure VaultReaderAdapter implements both interfaces.
var _ spi.VaultScannerPort = (*VaultReaderAdapter)(nil)
var _ spi.VaultReaderPort = (*VaultReaderAdapter)(nil)

// FilterFunc defines a function type for filtering files during vault scanning.
// Returns true if the file should be included in the scan results.
type FilterFunc func(path string, info os.FileInfo) bool

// VaultReaderAdapter implements both VaultScannerPort and VaultReaderPort using
// filesystem operations. It provides vault scanning and single file reading
// capabilities with proper error handling, cache directory filtering, and
// security measures against path traversal.
type VaultReaderAdapter struct {
	config   domain.Config
	log      zerolog.Logger
	readFile func(string) ([]byte, error)
	walkDir  func(string, filepath.WalkFunc) error
	stat     func(string) (os.FileInfo, error)
}

// NewVaultReaderAdapter creates a new VaultReaderAdapter with the given config
// and logger.
// The adapter uses the vault path from config for all file operations.
func NewVaultReaderAdapter(
	config domain.Config,
	log zerolog.Logger,
) *VaultReaderAdapter {
	return &VaultReaderAdapter{
		config:   config,
		log:      log,
		readFile: os.ReadFile,
		walkDir:  filepath.Walk,
		stat:     os.Stat,
	}
}

// ScanAll performs a full vault scan, returning all files as VaultFile DTOs.
// Filters files by extension (.md, .markdown) before loading content to prevent
// memory issues with large binary files. Ignores cache directories (.lithos/)
// and skips directories during traversal.
// Errors are logged but don't stop the scan; partial results are returned.
func (a *VaultReaderAdapter) ScanAll(
	ctx context.Context,
) ([]dto.VaultFile, error) {
	startTime := time.Now()

	// Filter: include only markdown files, exclude directories and cache
	// directories
	filter := func(path string, info os.FileInfo) bool {
		if info.IsDir() {
			return false
		}
		if a.isCacheDirectory(path) {
			return false
		}
		return a.isMarkdownFile(path)
	}

	files, err := a.scanVault(ctx, a.config.VaultPath, filter)
	if err != nil {
		return nil, err
	}

	duration := time.Since(startTime)
	a.log.Debug().
		Int("files_scanned", len(files)).
		Dur("duration", duration).
		Msg("vault scan completed")

	return files, nil
}

// ScanModified scans vault files that have been modified after the given
// timestamp.
// Uses the same scanning logic as ScanAll but filters by modification time.
// Enables NFR4 performance optimization for large vaults.
func (a *VaultReaderAdapter) ScanModified(
	ctx context.Context,
	since time.Time,
) ([]dto.VaultFile, error) {
	startTime := time.Now()

	// Filter: include files modified after since, exclude directories and cache
	filter := func(path string, info os.FileInfo) bool {
		if info.IsDir() {
			return false
		}
		if a.isCacheDirectory(path) {
			return false
		}
		if !a.isMarkdownFile(path) {
			return false
		}
		return filterByModTime(info, since)
	}

	files, err := a.scanVault(ctx, a.config.VaultPath, filter)
	if err != nil {
		return nil, err
	}

	duration := time.Since(startTime)
	a.log.Debug().
		Int("files_scanned", len(files)).
		Time("since", since).
		Dur("duration", duration).
		Msg("incremental vault scan completed")

	return files, nil
}

// Read performs single file read with path validation and security checks.
// Validates path is within vault to prevent directory traversal attacks.
// Returns VaultFile DTO with metadata and content.
func (a *VaultReaderAdapter) Read(
	ctx context.Context,
	path string,
) (dto.VaultFile, error) {
	// Check for context cancellation
	if ctx.Err() != nil {
		return dto.VaultFile{}, ctx.Err()
	}

	// Validate path is within vault (prevent directory traversal)
	if err := a.validatePathInVault(path); err != nil {
		return dto.VaultFile{}, err
	}

	// Read file and construct VaultFile
	return a.readFileWithMetadata(path)
}

// readFileWithMetadata reads a file and constructs VaultFile with metadata.
// Assumes path validation has already been performed.
func (a *VaultReaderAdapter) readFileWithMetadata(
	path string,
) (dto.VaultFile, error) {
	// Check file exists
	info, err := a.stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			return dto.VaultFile{}, fmt.Errorf("file not found: %s", path)
		}
		return dto.VaultFile{}, a.wrapVaultError("stat", path, err)
	}

	// Read content
	content, err := a.readFile(
		path,
	) // #nosec G304 - path is validated by caller
	if err != nil {
		return dto.VaultFile{}, a.wrapVaultError("read", path, err)
	}

	// Construct metadata and VaultFile
	metadata := dto.NewFileMetadata(path, info)
	return dto.NewVaultFile(metadata, content), nil
}

// scanVault performs the core vault scanning logic with a custom filter.
// The filter function determines which files to include in the results.
func (a *VaultReaderAdapter) scanVault(
	ctx context.Context,
	vaultPath string,
	filter FilterFunc,
) ([]dto.VaultFile, error) {
	var files []dto.VaultFile

	err := a.walkDir(
		vaultPath,
		func(path string, info os.FileInfo, err error) error {
			// Check for context cancellation
			if ctx.Err() != nil {
				return ctx.Err()
			}

			// Handle walk errors
			if err != nil {
				a.log.Warn().Err(err).Str("path", path).Msg("walk error")
				return nil // Continue walking
			}

			// Apply filter
			if !filter(path, info) {
				// Skip cache directories entirely
				if info.IsDir() && a.isCacheDirectory(path) {
					return filepath.SkipDir
				}
				return nil
			}

			// Check file size to prevent memory exhaustion
			if info.Size() > maxFileSizeBytes {
				a.log.Warn().
					Str("path", path).
					Int64("size", info.Size()).
					Int64("max_size", maxFileSizeBytes).
					Msg("skipping file that exceeds maximum size limit")
				return nil // Skip large files
			}

			// Read file content
			content, err := a.readFile(
				path,
			) // #nosec G304 - path is validated by caller
			if err != nil {
				a.log.Warn().
					Err(err).
					Str("path", path).
					Msg("failed to read file")
				return nil // Continue walking
			}

			// Construct FileMetadata
			metadata := dto.NewFileMetadata(path, info)

			// Create VaultFile
			vf := dto.NewVaultFile(metadata, content)
			files = append(files, vf)

			return nil
		},
	)

	if err != nil {
		return nil, a.wrapVaultError("scan", vaultPath, err)
	}

	return files, nil
}

// wrapVaultError wraps filesystem errors with consistent context.
func (a *VaultReaderAdapter) wrapVaultError(op, path string, err error) error {
	return errors.NewFileSystemError(op, path, err)
}

// filterByModTime checks if a file was modified after the given timestamp.
// Returns true if the file should be included (modified after since).
func filterByModTime(info os.FileInfo, since time.Time) bool {
	return !info.ModTime().Before(since)
}

// validatePathInVault validates that the target path is within the vault
// directory.
// Prevents directory traversal attacks by ensuring the path doesn't escape
// the vault root using absolute path comparison.
// Returns an error if the path is outside the vault.
func (a *VaultReaderAdapter) validatePathInVault(targetPath string) error {
	absVault, err := filepath.Abs(a.config.VaultPath)
	if err != nil {
		return fmt.Errorf("failed to get vault path: %w", err)
	}
	absTarget, err := filepath.Abs(targetPath)
	if err != nil {
		return fmt.Errorf("failed to get target path: %w", err)
	}
	// Check target is subdirectory of vault
	rel, err := filepath.Rel(absVault, absTarget)
	if err != nil {
		return fmt.Errorf("failed to compute relative path: %w", err)
	}
	// Reject paths starting with ".." (directory traversal)
	if strings.HasPrefix(rel, "..") {
		return fmt.Errorf("path outside vault: %s", targetPath)
	}
	return nil
}

// isCacheDirectory checks if a path is within a cache directory (.lithos/).
// Uses proper path segment comparison instead of string contains to avoid
// false positives with filenames containing ".lithos".
func (a *VaultReaderAdapter) isCacheDirectory(path string) bool {
	// Get path relative to vault root
	relPath, err := filepath.Rel(a.config.VaultPath, path)
	if err != nil {
		// If we can't make it relative, check if it contains .lithos as
		// fallback
		return strings.Contains(path, ".lithos")
	}

	// Split path into segments and check if any segment is exactly ".lithos"
	parts := strings.Split(relPath, string(filepath.Separator))
	for _, part := range parts {
		if part == ".lithos" {
			return true
		}
	}
	return false
}

// isMarkdownFile checks if a file has a markdown extension.
// Supports both .md and .markdown extensions.
func (a *VaultReaderAdapter) isMarkdownFile(path string) bool {
	ext := strings.ToLower(filepath.Ext(path))
	return ext == ".md" || ext == ".markdown"
}
