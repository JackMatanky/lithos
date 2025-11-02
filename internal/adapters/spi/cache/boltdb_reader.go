package cache

import (
	"context"
	"encoding/json"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"go.etcd.io/bbolt"
)

// Compile-time interface compliance check.
// This ensures BoltDBCacheReadAdapter implements CacheReaderPort correctly.
var _ spi.CacheReaderPort = (*BoltDBCacheReadAdapter)(nil)

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
		return nil, errors.NewCacheReadError("", dbPath, "open_db", err)
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
			return errors.ErrNotFound
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
		return domain.Note{}, errors.NewCacheReadError(
			string(id),
			"",
			"read_scan",
			err,
		)
	}

	if !found {
		return domain.Note{}, errors.ErrNotFound
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
		return nil, errors.NewCacheReadError(
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
