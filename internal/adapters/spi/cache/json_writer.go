package cache

import (
	"context"
	"encoding/json"
	"os"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/moby/sys/atomicwriter"
	"github.com/rs/zerolog"
)

const (
	// cacheFilePerms defines the file permissions for cache files.
	// 0o644 = rw-r--r-- (owner read/write, group/other read-only).
	cacheFilePerms = 0o644
)

// Compile-time interface compliance check.
// This ensures JSONCacheWriteAdapter implements CacheWriterPort correctly.
// Will fail to compile if the interface contract is not satisfied.
var _ spi.CacheWriterPort = (*JSONCacheWriteAdapter)(nil)

// JSONCacheWriteAdapter implements CacheWriterPort for filesystem-based
// note persistence with atomic write guarantees. It uses the CQRS write-side
// pattern to ensure data consistency during concurrent access and system
// crashes.
//
// Atomic Write Semantics:
// - Uses temp-file + rename pattern via moby/sys/atomicwriter
// - Prevents partial reads during concurrent operations
// - Ensures cache consistency even during power failures
//
// Thread Safety:
// - Safe for concurrent writes from multiple indexers
// - Filesystem operations provide OS-level locking
// - No shared mutable state beyond configuration
//
// Error Handling:
// - All errors wrapped with operation context and resource identifiers (FR9)
// - Preserves error chains for debugging
// - Uses structured error types from internal/shared/errors
//
// See docs/architecture/components.md#jsoncachewriteadapter for implementation
// details.
type JSONCacheWriteAdapter struct {
	config     domain.Config
	log        zerolog.Logger
	writeFile  func(string, []byte, os.FileMode) error
	mkdirAll   func(string, os.FileMode) error
	removeFile func(string) error
}

// NewJSONCacheWriter creates a new JSONCacheWriteAdapter with the provided
// configuration and logger. The adapter implements atomic write semantics
// for cache persistence and is thread-safe for concurrent operations.
//
// Parameters:
//   - config: Application configuration containing CacheDir path
//   - log: Structured logger for operation tracking
//
// Returns:
//   - *JSONCacheWriteAdapter: Configured adapter ready for cache operations
//
// Thread Safety: The returned adapter is safe for concurrent use.
func NewJSONCacheWriter(
	config domain.Config,
	log zerolog.Logger,
) *JSONCacheWriteAdapter {
	return &JSONCacheWriteAdapter{
		config:     config,
		log:        log,
		writeFile:  atomicwriter.WriteFile,
		mkdirAll:   os.MkdirAll,
		removeFile: os.Remove,
	}
}

// marshalNote serializes a note to compact JSON bytes.
// Returns the JSON data or an error if serialization fails.
func marshalNote(note domain.Note) ([]byte, error) {
	return json.Marshal(note)
}

// wrapCacheWriteError creates a standardized CacheWriteError with operation
// context.
// Includes note ID, file path, operation name, and underlying cause.
func wrapCacheWriteError(noteID, path, operation string, cause error) error {
	return lithosErr.NewCacheWriteError(noteID, path, operation, cause)
}

// wrapCacheDeleteError creates a standardized CacheDeleteError with operation
// context.
// Includes note ID, file path, operation name, and underlying cause.
func wrapCacheDeleteError(noteID, path, operation string, cause error) error {
	return lithosErr.NewCacheDeleteError(noteID, path, operation, cause)
}

// Persist atomically writes note to cache using temp-file + rename pattern.
// Creates cache directory if missing. Overwrites existing cache entry.
// Returns error wrapped with operation context if write fails.
//
// Atomic Write Semantics:
// - Writes to temporary file first, then renames to final location
// - Prevents partial reads during concurrent access
// - Ensures cache consistency even during system crashes
//
// Cache Directory Behavior:
// - Creates .lithos/cache/ directory if missing
// - Uses Config.CacheDir for configurable location
// - File naming: {NoteID}.json
//
// Thread-safe: Safe for concurrent calls.
// Context: Respects ctx cancellation, returns ctx.Err() if canceled.
// Errors: Wrapped with operation context and resource identifiers (FR9).
func (a *JSONCacheWriteAdapter) Persist(
	ctx context.Context,
	note domain.Note,
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	// 1. Ensure cache directory exists
	if err := EnsureCacheDir(a.config.CacheDir); err != nil {
		return wrapCacheWriteError(
			string(note.ID),
			a.config.CacheDir,
			"ensure_cache_dir",
			err,
		)
	}

	// 2. Serialize note to JSON
	data, marshalErr := marshalNote(note)
	if marshalErr != nil {
		return wrapCacheWriteError(
			string(note.ID),
			noteFilePath(a.config.CacheDir, note.ID),
			"serialize",
			marshalErr,
		)
	}

	// 3. Atomic write
	path := noteFilePath(a.config.CacheDir, note.ID)
	if writeErr := a.writeFile(path, data, cacheFilePerms); writeErr != nil {
		return wrapCacheWriteError(
			string(note.ID),
			path,
			"atomic_write",
			writeErr,
		)
	}

	// 3b. Clean up legacy cache filename if it exists to avoid duplicates
	legacyPath := legacyNoteFilePath(a.config.CacheDir, note.ID)
	if legacyPath != path {
		if err := a.removeFile(legacyPath); err == nil {
			a.log.Debug().
				Str("note_id", string(note.ID)).
				Str("path", legacyPath).
				Msg("removed legacy cache entry")
		}
	}

	// 4. Log success
	a.log.Debug().
		Str("note_id", string(note.ID)).
		Str("path", path).
		Msg("cache write successful")

	return nil
}

// Delete removes note from cache. Idempotent: returns nil if note doesn't
// exist. Returns error wrapped with operation context if deletion fails
// (e.g., permissions).
//
// Idempotent Semantics:
// - Returns nil if note already deleted or never existed
// - No error for "note not found" - treated as successful deletion
// - Allows safe retry of delete operations
//
// Error Conditions:
// - Permission denied on cache directory
// - Filesystem errors during deletion
// - Context cancellation during operation
//
// Thread-safe: Safe for concurrent calls.
// Context: Respects ctx cancellation, returns ctx.Err() if canceled.
// Errors: Wrapped with operation context and resource identifiers (FR9).
func (a *JSONCacheWriteAdapter) Delete(
	ctx context.Context,
	id domain.NoteID,
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	// Get file path
	path := noteFilePath(a.config.CacheDir, id)

	// Attempt deletion
	removeErr := a.removeFile(path)
	if removeErr == nil {
		a.log.Debug().
			Str("note_id", string(id)).
			Str("path", path).
			Msg("cache delete successful")
		return nil
	}

	if !os.IsNotExist(removeErr) {
		return wrapCacheDeleteError(
			string(id),
			path,
			"delete",
			removeErr,
		)
	}

	legacyPath := legacyNoteFilePath(a.config.CacheDir, id)
	legacyErr := a.removeFile(legacyPath)
	switch {
	case legacyErr == nil:
		a.log.Debug().
			Str("note_id", string(id)).
			Str("path", legacyPath).
			Msg("cache delete successful (legacy)")
	case os.IsNotExist(legacyErr):
		a.log.Debug().
			Str("note_id", string(id)).
			Str("path", path).
			Msg("cache delete successful (file not found)")
	default:
		return wrapCacheDeleteError(
			string(id),
			legacyPath,
			"delete_legacy",
			legacyErr,
		)
	}

	return nil
}
