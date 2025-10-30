package cache

import (
	"context"
	"encoding/json"
	"os"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
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
	config domain.Config
	log    zerolog.Logger
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
		config: config,
		log:    log,
	}
}

// marshalNote serializes a note to indented JSON bytes.
// Returns the JSON data or an error if serialization fails.
func marshalNote(note domain.Note) ([]byte, error) {
	return json.MarshalIndent(note, "", "  ")
}

// writeAtomic performs atomic file write using temp-file + rename pattern.
// Creates the file with specified permissions and ensures atomicity.
func writeAtomic(path string, data []byte, perm os.FileMode) error {
	return atomicwriter.WriteFile(path, data, perm)
}

// wrapCacheWriteError creates a standardized CacheWriteError with operation
// context.
// Includes note ID, file path, operation name, and underlying cause.
func wrapCacheWriteError(noteID, path, operation string, cause error) error {
	return errors.NewCacheWriteError(noteID, path, operation, cause)
}

// wrapCacheDeleteError creates a standardized CacheDeleteError with operation
// context.
// Includes note ID, file path, operation name, and underlying cause.
func wrapCacheDeleteError(noteID, path, operation string, cause error) error {
	return errors.NewCacheDeleteError(noteID, path, operation, cause)
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
	if err := ensureCacheDir(a.config.CacheDir); err != nil {
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
	if writeErr := writeAtomic(path, data, cacheFilePerms); writeErr != nil {
		return wrapCacheWriteError(
			string(note.ID),
			path,
			"atomic_write",
			writeErr,
		)
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
	if err := os.Remove(path); err != nil {
		// Check if file doesn't exist (idempotent behavior)
		if os.IsNotExist(err) {
			// File doesn't exist - treat as successful deletion
			a.log.Debug().
				Str("note_id", string(id)).
				Str("path", path).
				Msg("cache delete successful (file not found)")
			return nil
		}
		// Other error - wrap and return
		return wrapCacheDeleteError(
			string(id),
			path,
			"delete",
			err,
		)
	}

	// Log successful deletion
	a.log.Debug().
		Str("note_id", string(id)).
		Str("path", path).
		Msg("cache delete successful")

	return nil
}
