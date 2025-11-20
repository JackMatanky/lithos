package boltdb

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"go.etcd.io/bbolt"
)

// Compile-time interface compliance check.
var _ spi.CacheReaderPort = (*BoltDBCacheReadAdapter)(nil)
var _ spi.MetadataQueryPort = (*BoltDBCacheReadAdapter)(nil)

// BoltDBCacheReadAdapter implements CacheReaderPort and MetadataQueryPort for
// BoltDB. It uses structured buckets for O(1) lookups and supports hot path
// query optimization.
type BoltDBCacheReadAdapter struct {
	config domain.Config
	log    zerolog.Logger
	db     *bbolt.DB
}

// NewBoltDBCacheReadAdapter creates a new BoltDBCacheReadAdapter.
func NewBoltDBCacheReadAdapter(
	config domain.Config,
	log zerolog.Logger,
) (*BoltDBCacheReadAdapter, error) {
	dbPath := config.CacheDir + "/hot.db"
	options := *bbolt.DefaultOptions
	options.ReadOnly = true
	options.Timeout = 1 * time.Second // TODO: Use config or constant

	db, err := bbolt.Open(
		dbPath,
		boltDBFileMode,
		&options,
	)
	if err != nil {
		return nil, lithosErr.NewCacheReadError("", dbPath, "open_db", err)
	}

	return &BoltDBCacheReadAdapter{
		config: config,
		log:    log,
		db:     db,
	}, nil
}

// Close closes the BoltDB database connection.
func (a *BoltDBCacheReadAdapter) Close() error {
	return a.db.Close()
}

// Read retrieves a single note by ID (Path) from the BoltDB cache.
func (a *BoltDBCacheReadAdapter) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	select {
	case <-ctx.Done():
		return domain.Note{}, ctx.Err()
	default:
	}

	path := string(id)
	var note domain.Note

	err := a.db.View(func(tx *bbolt.Tx) error {
		bucket := tx.Bucket([]byte(BucketNotes))
		if bucket == nil {
			return lithosErr.ErrNotFound
		}

		data := bucket.Get([]byte(path))
		if data == nil {
			return lithosErr.ErrNotFound
		}

		var cached CachedNote
		if err := json.Unmarshal(data, &cached); err != nil {
			return fmt.Errorf("failed to unmarshal cached note: %w", err)
		}

		note = a.reconstructNote(cached)
		return nil
	})

	if err != nil {
		if errors.Is(err, lithosErr.ErrNotFound) {
			return domain.Note{}, lithosErr.ErrNotFound
		}
		return domain.Note{}, lithosErr.NewCacheReadError(
			path,
			path,
			"read_scan",
			err,
		)
	}

	return note, nil
}

// List retrieves all cached notes from the BoltDB cache.
func (a *BoltDBCacheReadAdapter) List(
	ctx context.Context,
) ([]domain.Note, error) {
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	var notes []domain.Note

	err := a.db.View(func(tx *bbolt.Tx) error {
		bucket := tx.Bucket([]byte(BucketNotes))
		if bucket == nil {
			return nil // Empty cache is valid
		}

		return bucket.ForEach(func(k, v []byte) error {
			select {
			case <-ctx.Done():
				return ctx.Err()
			default:
			}

			var cached CachedNote
			if err := json.Unmarshal(v, &cached); err != nil {
				a.log.Warn().
					Err(err).
					Str("key", string(k)).
					Msg("skipping invalid cache entry")
				return nil
			}

			notes = append(notes, a.reconstructNote(cached))
			return nil
		})
	})

	if err != nil {
		return nil, lithosErr.NewCacheReadError(
			"",
			"",
			"list_scan",
			err,
		)
	}

	return notes, nil
}

// ByBasename finds notes by filename without extension.
func (a *BoltDBCacheReadAdapter) ByBasename(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	return a.lookupIndex(ctx, BucketIndexByBasename, basename)
}

// ByAlias finds notes by frontmatter alias values.
func (a *BoltDBCacheReadAdapter) ByAlias(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	return a.lookupIndex(ctx, BucketIndexByAlias, alias)
}

// ByFileClass finds notes by schema fileClass value.
func (a *BoltDBCacheReadAdapter) ByFileClass(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	return a.lookupIndex(ctx, BucketIndexByFileClass, fileClass)
}

// PathQuery finds notes using a flexible path selector.
func (a *BoltDBCacheReadAdapter) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	normalized, err := opts.Validate()
	if err != nil {
		return nil, err
	}

	switch normalized.Scope {
	case spi.PathQueryScopeFull:
		// Direct lookup by path
		id := domain.NewNoteID(normalized.Value)
		note, readErr := a.Read(ctx, id)
		if readErr != nil {
			if errors.Is(readErr, lithosErr.ErrNotFound) {
				return []domain.Note{}, nil
			}
			return nil, readErr
		}
		return []domain.Note{note}, nil

	case spi.PathQueryScopeBasename:
		return a.ByBasename(ctx, normalized.Value)

	case spi.PathQueryScopeFolder:
		return a.lookupIndex(ctx, BucketIndexByFolder, normalized.Value)

	default:
		return nil, fmt.Errorf("unsupported scope: %s", normalized.Scope)
	}
}

// IsStale checks if a note is stale by comparing filesystem mod time with
// cached IndexedAt.
// Returns true if missing in cache or if file mod time > indexed time.
func (a *BoltDBCacheReadAdapter) IsStale(
	ctx context.Context,
	path string,
) (bool, error) {
	// Check filesystem first
	absPath := filepath.Join(a.config.VaultPath, path)
	fi, err := os.Stat(absPath)
	if err != nil {
		if os.IsNotExist(err) {
			// File missing on disk. If it's in cache, it's stale (needs
			// removal).
			// But IsStale usually implies "needs re-indexing".
			// If file is gone, we can't re-index it. We should delete it.
			// Returning true (stale) might trigger re-index which fails?
			// Or maybe VaultIndexer handles deletion?
			// For now, if file missing, return true (stale) so caller decides.
			return true, nil
		}
		return false, err
	}

	fileModTime := fi.ModTime()

	// Check cache
	var cached CachedNote
	err = a.db.View(func(tx *bbolt.Tx) error {
		bucket := tx.Bucket([]byte(BucketNotes))
		if bucket == nil {
			return lithosErr.ErrNotFound
		}
		data := bucket.Get([]byte(path))
		if data == nil {
			return lithosErr.ErrNotFound
		}
		return json.Unmarshal(data, &cached)
	})

	if err != nil {
		if errors.Is(err, lithosErr.ErrNotFound) {
			return true, nil // Missing in cache -> Stale (needs indexing)
		}
		return false, err
	}

	// Create FileDatesDTO from file info to compare
	// Actually we compare:
	// 1. If file mod time is AFTER indexed time -> Stale
	// 2. If stored ModifiedAt != file mod time -> Stale (e.g. if file was
	// reverted to older timestamp? Unlikely but safe)

	// CachedNote has FileDatesDTO
	// If file modified AFTER indexed -> stale
	if fileModTime.After(cached.FileDates.IndexedAt) {
		return true, nil
	}

	// If stored ModifiedAt differs from current file ModTime -> stale
	// (e.g. if file was reverted to older timestamp? Unlikely but safe)
	if !cached.FileDates.ModifiedAt.Equal(fileModTime) {
		return true, nil
	}

	return false, nil
}

// Helper methods

//nolint:cyclop // complex function due to index lookup logic
func (a *BoltDBCacheReadAdapter) lookupIndex(
	ctx context.Context,
	bucketName string,
	key string,
) ([]domain.Note, error) {
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	var notes []domain.Note

	err := a.db.View(func(tx *bbolt.Tx) error {
		indices := tx.Bucket([]byte(BucketIndices))
		if indices == nil {
			return nil // No indices yet
		}

		indexBucket := indices.Bucket([]byte(bucketName))
		if indexBucket == nil {
			return nil // Specific index bucket missing
		}

		// Get list of paths
		data := indexBucket.Get([]byte(key))
		if data == nil {
			return nil
		}

		var paths []string
		if err := json.Unmarshal(data, &paths); err != nil {
			return fmt.Errorf("failed to unmarshal index list: %w", err)
		}

		// Batch lookup paths
		notesBucket := tx.Bucket([]byte(BucketNotes))
		if notesBucket == nil {
			return nil
		}

		for _, p := range paths {
			noteData := notesBucket.Get([]byte(p))
			if noteData != nil {
				var cached CachedNote
				if err := json.Unmarshal(noteData, &cached); err == nil {
					notes = append(notes, a.reconstructNote(cached))
				}
			}
		}
		return nil
	})

	if err != nil {
		return nil, lithosErr.NewCacheReadError(
			"",
			key,
			"index_lookup_"+bucketName,
			err,
		)
	}

	return notes, nil
}

func (a *BoltDBCacheReadAdapter) reconstructNote(
	cached CachedNote,
) domain.Note {
	// Reconstruct minimal domain.Note from cached metadata
	return domain.Note{
		ID: domain.NewNoteID(cached.Path),
		Frontmatter: domain.Frontmatter{
			FileClass: cached.FileClass,
			Fields: map[string]interface{}{
				"title":               cached.Title,
				"aliases":             cached.Aliases,
				a.config.FileClassKey: cached.FileClass,
				// Restore timestamps into fields if needed?
				// Domain note usually doesn't carry timestamps in fields,
				// but we have them in FileDates.
				// The original reader put them in map.
			},
		},
	}
}
