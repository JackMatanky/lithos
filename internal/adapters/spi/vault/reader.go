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
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// Compile-time check to ensure VaultReaderAdapter implements VaultReaderPort.
var _ spi.VaultReaderPort = (*VaultReaderAdapter)(nil)

// VaultReaderAdapter implements VaultReaderPort using filesystem operations.
// It provides vault scanning capabilities with proper error handling,
// cache directory filtering, and security measures against path traversal.
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
// Ignores cache directories (.lithos/) and skips directories during traversal.
// Errors are logged but don't stop the scan; partial results are returned.
func (a *VaultReaderAdapter) ScanAll(
	ctx context.Context,
) ([]spi.VaultFile, error) {
	var files []spi.VaultFile
	startTime := time.Now()

	err := a.walkDir(
		a.config.VaultPath,
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

			// Skip directories
			if info.IsDir() {
				// Skip cache directories
				if strings.Contains(path, ".lithos") {
					return filepath.SkipDir
				}
				return nil
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
			metadata := spi.NewFileMetadata(path, info)

			// Create VaultFile
			vf := spi.NewVaultFile(metadata, content)
			files = append(files, vf)

			return nil
		},
	)

	if err != nil {
		return nil, errors.NewFileSystemError("scan", a.config.VaultPath, err)
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
) ([]spi.VaultFile, error) {
	var files []spi.VaultFile
	var checked, matched int

	err := a.walkDir(
		a.config.VaultPath,
		func(path string, info os.FileInfo, err error) error {
			// Check for context cancellation
			if ctx.Err() != nil {
				return ctx.Err()
			}

			// Handle walk errors
			if err != nil {
				a.log.Warn().Err(err).Str("path", path).Msg("walk error")
				return nil
			}

			// Skip directories
			if info.IsDir() {
				if strings.Contains(path, ".lithos") {
					return filepath.SkipDir
				}
				return nil
			}

			return a.processFileIfModified(
				path,
				info,
				since,
				&files,
				&checked,
				&matched,
			)
		},
	)

	if err != nil {
		return nil, errors.NewFileSystemError(
			"scan_modified",
			a.config.VaultPath,
			err,
		)
	}

	a.log.Debug().
		Int("files_checked", checked).
		Int("files_matched", matched).
		Time("since", since).
		Msg("incremental vault scan completed")

	return files, nil
}

// Read performs single file read with path validation and security checks.
// Validates path is within vault to prevent directory traversal attacks.
// Returns VaultFile DTO with metadata and content.
func (a *VaultReaderAdapter) Read(
	ctx context.Context,
	path string,
) (spi.VaultFile, error) {
	// Check for context cancellation
	if ctx.Err() != nil {
		return spi.VaultFile{}, ctx.Err()
	}

	// Validate path is within vault (prevent directory traversal)
	if !a.isPathInVault(path) {
		return spi.VaultFile{}, fmt.Errorf("path outside vault: %s", path)
	}

	// Check file exists
	info, err := a.stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			return spi.VaultFile{}, fmt.Errorf("file not found: %s", path)
		}
		return spi.VaultFile{}, errors.NewFileSystemError("stat", path, err)
	}

	// Read content
	content, err := a.readFile(
		path,
	) // #nosec G304 - path is validated by caller
	if err != nil {
		return spi.VaultFile{}, errors.NewFileSystemError("read", path, err)
	}

	// Construct metadata and VaultFile
	metadata := spi.NewFileMetadata(path, info)
	return spi.NewVaultFile(metadata, content), nil
}

// processFileIfModified checks if a file was modified after the given time
// and processes it.
func (a *VaultReaderAdapter) processFileIfModified(
	path string,
	info os.FileInfo,
	since time.Time,
	files *[]spi.VaultFile,
	checked, matched *int,
) error {
	*checked++

	// Filter: Only include files modified after 'since' timestamp
	if info.ModTime().Before(since) {
		return nil // Skip old file
	}

	*matched++

	// Read and construct VaultFile (same as ScanAll)
	content, err := a.readFile(
		path,
	) // #nosec G304 - path is validated by caller
	if err != nil {
		a.log.Warn().
			Err(err).
			Str("path", path).
			Msg("failed to read file")
		return nil
	}

	metadata := spi.NewFileMetadata(path, info)
	*files = append(*files, spi.NewVaultFile(metadata, content))

	return nil
}

// isPathInVault validates that the target path is within the vault directory.
// Prevents directory traversal attacks by ensuring the path doesn't escape
// the vault root using absolute path comparison.
func (a *VaultReaderAdapter) isPathInVault(targetPath string) bool {
	absVault, err := filepath.Abs(a.config.VaultPath)
	if err != nil {
		return false
	}
	absTarget, err := filepath.Abs(targetPath)
	if err != nil {
		return false
	}
	// Check target is subdirectory of vault
	rel, err := filepath.Rel(absVault, absTarget)
	if err != nil {
		return false
	}
	// Reject paths starting with ".." (directory traversal)
	return !strings.HasPrefix(rel, "..")
}
