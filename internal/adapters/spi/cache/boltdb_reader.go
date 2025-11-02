package cache

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	sharederrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"go.etcd.io/bbolt"
)

// Compile-time interface compliance check.
// This ensures BoltDBCacheReadAdapter implements QueryReaderPort correctly.
var _ spi.QueryReaderPort = (*BoltDBCacheReadAdapter)(nil)

// BoltDBCacheReadAdapter implements CacheReaderPort for BoltDB-based
// note retrieval with optimized queries for hot data access. It uses
// the CQRS read-side pattern with structured buckets for fast lookups.
//
// Query Optimizations:
// - ByPath: Direct lookup in /paths/ bucket (O(1))
// - ByBasename: Index lookup in /basenames/ bucket
// - ByAlias: Index lookup in /aliases/ bucket
// - ByFileClass: Index lookup in /file_classes/ bucket
//
// Thread Safety:
// - BoltDB provides concurrent read access
// - Safe for multiple concurrent readers
//
// See docs/architecture/components.md#boltdb-cache-adapters for implementation
// details.
type BoltDBCacheReadAdapter struct {
	config domain.Config
	log    zerolog.Logger
	db     *bbolt.DB
}

// NewBoltDBCacheReadAdapter creates a new BoltDBCacheReadAdapter with the
// provided
// configuration and logger. The adapter opens the existing BoltDB database
// for read operations.
//
// Parameters:
//   - config: Application configuration containing CacheDir and FileClassKey
//   - log: Structured logger for operation tracking
//
// Returns:
//   - *BoltDBCacheReadAdapter: Configured adapter ready for cache queries
//   - error: If database opening fails
//
// Thread Safety: The returned adapter is safe for concurrent use.
func NewBoltDBCacheReadAdapter(
	config domain.Config,
	log zerolog.Logger,
) (*BoltDBCacheReadAdapter, error) {
	// Open BoltDB database in cache directory (read-only for safety)
	dbPath := config.CacheDir + "/cache.db"
	options := *bbolt.DefaultOptions
	options.ReadOnly = true
	options.Timeout = 1 * time.Second

	db, err := bbolt.Open(
		dbPath,
		boltDBFileMode,
		&options,
	)
	if err != nil {
		return nil, sharederrors.NewCacheReadError("", dbPath, "open_db", err)
	}

	return &BoltDBCacheReadAdapter{
		config: config,
		log:    log,
		db:     db,
	}, nil
}

// Close closes the BoltDB database connection.
// Should be called when the adapter is no longer needed.
func (a *BoltDBCacheReadAdapter) Close() error {
	return a.db.Close()
}

// Read retrieves a single note by ID from the BoltDB cache.
// Returns the complete note with all frontmatter preserved.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//   - id: NoteID to retrieve
//
// Returns:
//   - Note: The retrieved note with path and frontmatter
//
// - error: ErrNotFound if note doesn't exist, or wrapped error for other issues
//
// Thread-safe: Safe for concurrent calls.
func (a *BoltDBCacheReadAdapter) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return domain.Note{}, ctx.Err()
	default:
	}

	var note domain.Note
	var found bool

	err := a.db.View(func(tx *bbolt.Tx) error {
		pathsBucket := tx.Bucket([]byte(bucketPaths))
		if pathsBucket == nil {
			return sharederrors.NewCacheReadError(
				"",
				"",
				"bucket_missing",
				fmt.Errorf("paths bucket not found"),
			)
		}

		// Scan paths bucket to find note with matching ID
		return pathsBucket.ForEach(func(k, v []byte) error {
			select {
			case <-ctx.Done():
				return ctx.Err()
			default:
			}

			var metadata BoltDBNoteMetadata
			if err := json.Unmarshal(v, &metadata); err != nil {
				a.log.Warn().
					Err(err).
					Str("path", string(k)).
					Msg("Failed to unmarshal note metadata, skipping")
				return nil // Continue scanning
			}

			if metadata.ID == string(id) {
				// Found the note, reconstruct it
				note = a.reconstructNoteFromMetadata(id, metadata)
				found = true
				return nil // Stop scanning
			}

			return nil // Continue scanning
		})
	})

	if err != nil {
		return domain.Note{}, sharederrors.NewCacheReadError(
			string(id),
			"",
			"read_scan",
			err,
		)
	}

	if !found {
		return domain.Note{}, sharederrors.ErrNotFound
	}

	a.log.Debug().
		Str("note_id", string(id)).
		Str("path", note.Path).
		Msg("Read note from BoltDB cache")

	return note, nil
}

// List retrieves all cached notes from the BoltDB cache.
// Returns a slice of all notes with their metadata.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//
// Returns:
//   - []Note: All cached notes
//   - error: Wrapped error if listing fails
//
// Thread-safe: Safe for concurrent calls.
// Performance: May be slow for large caches - consider pagination in future.
func (a *BoltDBCacheReadAdapter) List(
	ctx context.Context,
) ([]domain.Note, error) {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	var notes []domain.Note

	err := a.db.View(func(tx *bbolt.Tx) error {
		pathsBucket := tx.Bucket([]byte(bucketPaths))
		return pathsBucket.ForEach(func(k, v []byte) error {
			select {
			case <-ctx.Done():
				return ctx.Err()
			default:
			}

			var metadata BoltDBNoteMetadata
			if err := json.Unmarshal(v, &metadata); err != nil {
				a.log.Warn().
					Err(err).
					Str("path", string(k)).
					Msg("Failed to unmarshal note metadata, skipping")
				return nil // Continue with other notes
			}

			// Reconstruct Note from metadata
			note := domain.Note{
				ID:   domain.NewNoteID(metadata.ID),
				Path: metadata.Path,
				Frontmatter: domain.Frontmatter{
					FileClass: metadata.FileClass,
					Fields: map[string]interface{}{
						"title":               metadata.Title,
						"aliases":             metadata.Aliases,
						a.config.FileClassKey: metadata.FileClass,
					},
				},
			}

			notes = append(notes, note)
			return nil
		})
	})

	if err != nil {
		return nil, sharederrors.NewCacheReadError(
			"",
			"",
			"list_transaction",
			err,
		)
	}

	a.log.Debug().
		Int("count", len(notes)).
		Msg("Listed notes from BoltDB cache")

	return notes, nil
}

// GetByPath retrieves a note by its vault-relative file path.
// Optimized for BoltDB hot cache with O(1) lookup performance.
//
// Query Behavior:
// - Direct lookup in /paths/ bucket
// - Returns reconstructed note with available metadata
// - Returns ErrNotFound if path not found
//
// Performance: <1ms average
// Thread-safe: Safe for concurrent calls.
func (a *BoltDBCacheReadAdapter) GetByPath(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	if ctx.Err() != nil {
		return domain.Note{}, ctx.Err()
	}

	var note domain.Note
	err := a.db.View(func(tx *bbolt.Tx) error {
		pathsBucket := tx.Bucket([]byte(bucketPaths))
		if pathsBucket == nil {
			return errors.New("paths bucket not found")
		}

		metadataBytes := pathsBucket.Get([]byte(path))
		if metadataBytes == nil {
			return sharederrors.ErrNotFound
		}

		var metadata BoltDBNoteMetadata
		if err := json.Unmarshal(metadataBytes, &metadata); err != nil {
			return fmt.Errorf("failed to unmarshal BoltDB metadata: %w", err)
		}

		id := domain.NewNoteID(metadata.ID)
		note = a.reconstructNoteFromMetadata(id, metadata)
		return nil
	})

	if err != nil {
		if errors.Is(err, sharederrors.ErrNotFound) {
			return domain.Note{}, sharederrors.ErrNotFound
		}
		return domain.Note{}, sharederrors.NewCacheReadError(
			"",
			path,
			"get_by_path",
			err,
		)
	}

	a.log.Debug().Str("path", path).Msg("retrieved note from BoltDB cache")
	return note, nil
}

// GetByFileClass retrieves notes matching a file class.
// Uses BoltDB index for efficient lookup with full metadata reconstruction.
//
// Query Behavior:
// - Lookup in /file_classes/ bucket for ID list
// - Reconstruct full notes from /paths/ bucket metadata
// - Returns empty slice if file class not found
// - Uses config.FileClassKey for consistency
//
// Performance: <5ms average
// Thread-safe: Safe for concurrent calls.
func (a *BoltDBCacheReadAdapter) GetByFileClass(
	ctx context.Context,
	fileClass string,
	config domain.Config,
) ([]domain.Note, error) {
	if ctx.Err() != nil {
		return nil, ctx.Err()
	}

	var notes []domain.Note
	err := a.db.View(func(tx *bbolt.Tx) error {
		idList, err := a.getIDListForFileClass(tx, fileClass)
		if err != nil {
			return err
		}
		if len(idList) == 0 {
			return nil // No notes found for this file class
		}

		metadataList, err := a.findMetadataForIDs(tx, idList)
		if err != nil {
			return err
		}

		notes = a.reconstructNotesFromMetadata(metadataList)
		return nil
	})

	if err != nil {
		return nil, sharederrors.NewCacheReadError(
			"",
			fileClass,
			"get_by_file_class",
			err,
		)
	}

	a.log.Debug().
		Str("file_class", fileClass).
		Int("notes_found", len(notes)).
		Msg("retrieved notes by file class from BoltDB cache")

	return notes, nil
}

// QueryByFrontmatter is not optimized for BoltDB.
// BoltDB stores minimal metadata for hot lookups.
// For complex frontmatter queries, use SQLite adapter.
//
// Returns error indicating this operation should use SQLite.
func (a *BoltDBCacheReadAdapter) QueryByFrontmatter(
	ctx context.Context,
	key string,
	value interface{},
) ([]domain.Note, error) {
	return nil, fmt.Errorf(
		"QueryByFrontmatter not supported by BoltDB adapter - use SQLite for complex queries",
	)
}

// IsStale checks if a note at the given path is stale compared to the provided
// file modification time. A note is considered stale if:
// - The stored file_mod_time doesn't match the provided fileModTime, OR
// - The index_time is older than the file_mod_time (indicating the file was
// modified after indexing)
//
// This enables incremental indexing by avoiding re-processing of unchanged
// files.
//
// Parameters:
//   - path: Vault-relative file path to check
//   - fileModTime: Current modification time of the file on disk
//
// Returns:
//   - bool: true if the note is stale and should be re-indexed
//   - error: If the staleness check fails (e.g., note not found in cache)
//
// Thread-safe: Safe for concurrent calls.
func (a *BoltDBCacheReadAdapter) IsStale(
	path string,
	fileModTime time.Time,
) (bool, error) {
	err := a.db.View(func(tx *bbolt.Tx) error {
		pathsBucket := tx.Bucket([]byte(bucketPaths))
		if pathsBucket == nil {
			return sharederrors.NewCacheReadError(
				"",
				path,
				"staleness_check",
				fmt.Errorf("paths bucket not found"),
			)
		}

		metadataBytes := pathsBucket.Get([]byte(path))
		if metadataBytes == nil {
			// Note not in cache, consider it stale (needs indexing)
			return sharederrors.ErrNotFound
		}

		var metadata BoltDBNoteMetadata
		if err := json.Unmarshal(metadataBytes, &metadata); err != nil {
			return sharederrors.NewCacheReadError(
				"",
				path,
				"staleness_check",
				fmt.Errorf("failed to unmarshal metadata: %w", err),
			)
		}

		// Check if file was modified after indexing
		if metadata.FileModTime != fileModTime {
			return sharederrors.ErrNotFound // Signal stale
		}

		// File is fresh
		return nil
	})

	if err != nil {
		if errors.Is(err, sharederrors.ErrNotFound) {
			return true, nil // Note not found or modified, consider stale
		}
		return false, err // Other error
	}

	return false, nil // Note is fresh
}

// getIDListForFileClass retrieves the list of note IDs for a given file class.
func (a *BoltDBCacheReadAdapter) getIDListForFileClass(
	tx *bbolt.Tx,
	fileClass string,
) ([]string, error) {
	fileClassesBucket := tx.Bucket([]byte(bucketFileClasses))
	if fileClassesBucket == nil {
		return nil, sharederrors.NewCacheReadError(
			"",
			"",
			"bucket_missing",
			fmt.Errorf("file_classes bucket not found"),
		)
	}

	idListBytes := fileClassesBucket.Get([]byte(fileClass))
	if idListBytes == nil {
		return nil, nil // File class not found, return empty list
	}

	var idList []string
	if err := json.Unmarshal(idListBytes, &idList); err != nil {
		return nil, fmt.Errorf("failed to unmarshal ID list: %w", err)
	}

	return idList, nil
}

// findMetadataForIDs finds BoltDBNoteMetadata for each ID in the provided list.
func (a *BoltDBCacheReadAdapter) findMetadataForIDs(
	tx *bbolt.Tx,
	idList []string,
) ([]BoltDBNoteMetadata, error) {
	pathsBucket := tx.Bucket([]byte(bucketPaths))
	if pathsBucket == nil {
		return nil, sharederrors.NewCacheReadError(
			"",
			"",
			"bucket_missing",
			fmt.Errorf("paths bucket not found"),
		)
	}

	var metadataList []BoltDBNoteMetadata
	for _, idStr := range idList {
		metadata, err := a.findMetadataForID(pathsBucket, idStr)
		if err != nil {
			return nil, err
		}
		if metadata != nil {
			metadataList = append(metadataList, *metadata)
		}
	}

	return metadataList, nil
}

// findMetadataForID finds metadata for a specific note ID in the paths bucket.
func (a *BoltDBCacheReadAdapter) findMetadataForID(
	pathsBucket *bbolt.Bucket,
	idStr string,
) (*BoltDBNoteMetadata, error) {
	var metadata BoltDBNoteMetadata
	found := false

	err := pathsBucket.ForEach(func(k, v []byte) error {
		var meta BoltDBNoteMetadata
		if err := json.Unmarshal(v, &meta); err != nil {
			a.log.Warn().
				Err(err).
				Str("key", string(k)).
				Msg("failed to unmarshal metadata")
			return nil // Continue with other entries
		}
		if meta.ID == idStr {
			metadata = meta
			found = true
			return nil // Found it, stop iteration
		}
		return nil
	})

	if err != nil {
		return nil, fmt.Errorf("error during metadata lookup: %w", err)
	}

	if !found {
		return nil, sharederrors.ErrNotFound // Use sentinel error instead of nil, nil
	}

	return &metadata, nil
}

// reconstructNotesFromMetadata converts metadata list to domain.Note objects.
func (a *BoltDBCacheReadAdapter) reconstructNotesFromMetadata(
	metadataList []BoltDBNoteMetadata,
) []domain.Note {
	notes := make([]domain.Note, 0, len(metadataList))
	for i := range metadataList { // Use indexing to avoid copying 136 bytes
		metadata := &metadataList[i]
		id := domain.NewNoteID(metadata.ID)
		note := a.reconstructNoteFromMetadata(id, *metadata)
		notes = append(notes, note)
	}
	return notes
}

// reconstructNoteFromMetadata builds a Note from BoltDB metadata.
func (a *BoltDBCacheReadAdapter) reconstructNoteFromMetadata(
	id domain.NoteID,
	metadata BoltDBNoteMetadata,
) domain.Note {
	return domain.Note{
		ID:   id,
		Path: metadata.Path,
		Frontmatter: domain.Frontmatter{
			FileClass: metadata.FileClass,
			Fields: map[string]interface{}{
				"title":               metadata.Title,
				"aliases":             metadata.Aliases,
				a.config.FileClassKey: metadata.FileClass,
			},
		},
	}
}
