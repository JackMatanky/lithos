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

var errFileStale = errors.New("file stale")

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
	db *bbolt.DB,
) (*BoltDBCacheReadAdapter, error) {
	return &BoltDBCacheReadAdapter{
		config: config,
		log:    log,
		db:     db,
	}, nil
}

// Close closes the BoltDB database connection.
func (a *BoltDBCacheReadAdapter) Close() error {
	return nil // No-op, shared DB
}

// Read retrieves a single note by path from the BoltDB cache.
func (a *BoltDBCacheReadAdapter) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	if err := ctx.Err(); err != nil {
		return domain.Note{}, err
	}

	var note domain.Note

	err := a.db.View(func(tx *bbolt.Tx) error {
		if err := ctx.Err(); err != nil {
			return err
		}

		bucket := tx.Bucket([]byte(BucketNotes))
		if bucket == nil {
			return lithosErr.ErrNotFound
		}

		data := bucket.Get([]byte(path))
		if data == nil {
			return lithosErr.ErrNotFound
		}

		cached, err := decodeCachedNote(data)
		if err != nil {
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
	if err := ctx.Err(); err != nil {
		return nil, err
	}

	var notes []domain.Note

	err := a.db.View(func(tx *bbolt.Tx) error {
		if err := ctx.Err(); err != nil {
			return err
		}

		bucket := tx.Bucket([]byte(BucketNotes))
		if bucket == nil {
			return nil // Empty cache is valid
		}

		return bucket.ForEach(func(k, v []byte) error {
			if err := ctx.Err(); err != nil {
				return err
			}

			cached, err := decodeCachedNote(v)
			if err != nil {
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

// BasenameQuery finds notes by filename without extension.
func (a *BoltDBCacheReadAdapter) BasenameQuery(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	return a.lookupIndex(ctx, BucketIndexBasenameQuery, basename)
}

// AliasQuery finds notes by frontmatter alias values.
func (a *BoltDBCacheReadAdapter) AliasQuery(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	return a.lookupIndex(ctx, BucketIndexAliasQuery, alias)
}

// FileClassQuery finds notes by schema fileClass value.
func (a *BoltDBCacheReadAdapter) FileClassQuery(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	return a.lookupIndex(ctx, BucketIndexFileClassQuery, fileClass)
}

// PathQuery finds notes using a flexible path selector.
func (a *BoltDBCacheReadAdapter) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	if err := ctx.Err(); err != nil {
		return nil, err
	}

	normalized, err := opts.Validate()
	if err != nil {
		return nil, err
	}

	switch normalized.Scope {
	case spi.PathQueryScopeFull:
		// Direct lookup by path
		note, readErr := a.Read(ctx, normalized.Value)
		if readErr != nil {
			if errors.Is(readErr, lithosErr.ErrNotFound) {
				return []domain.Note{}, nil
			}
			return nil, readErr
		}
		return []domain.Note{note}, nil

	case spi.PathQueryScopeBasename:
		return a.BasenameQuery(ctx, normalized.Value)

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
	if err := ctx.Err(); err != nil {
		return false, err
	}

	modTime, missing, err := a.statFile(path)
	if err != nil {
		return false, err
	}
	if missing {
		return true, nil
	}

	return a.cacheStaleStatus(ctx, path, modTime)
}

// TagQuery finds notes containing a specific tag.
// Not implemented in BoltDB (hot path); use SQLite (deep path) for complex
// queries.
func (a *BoltDBCacheReadAdapter) TagQuery(
	ctx context.Context,
	tag string,
) ([]domain.Note, error) {
	return nil, errors.New("TagQuery not implemented in BoltDB (use SQLite)")
}

// FrontmatterQuery finds notes where a specific frontmatter field matches a
// value. Not implemented in BoltDB (hot path); use SQLite (deep path) for
// complex queries.
func (a *BoltDBCacheReadAdapter) FrontmatterQuery(
	ctx context.Context,
	field, value string,
) ([]domain.Note, error) {
	return nil, errors.New(
		"FrontmatterQuery not implemented in BoltDB (use SQLite)",
	)
}

// Helper methods

func (a *BoltDBCacheReadAdapter) lookupIndex(
	ctx context.Context,
	bucketName string,
	key string,
) ([]domain.Note, error) {
	if err := ctx.Err(); err != nil {
		return nil, err
	}

	var notes []domain.Note

	err := a.db.View(func(tx *bbolt.Tx) error {
		if err := ctx.Err(); err != nil {
			return err
		}

		paths, err := a.indexedPaths(tx, bucketName, key)
		if err != nil {
			return err
		}
		if len(paths) == 0 {
			return nil
		}

		loaded, err := a.collectNotes(tx, paths)
		if err != nil {
			return err
		}
		notes = loaded
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
	fields := map[string]interface{}{
		"title":   cached.Title,
		"aliases": cached.Aliases,
	}
	if cached.FileClass != "" {
		fields[a.config.FileClassKey] = cached.FileClass
	}

	note, err := domain.NewNote(
		cached.Path,
		domain.Frontmatter{Fields: fields},
		nil,
		nil,
		nil,
		nil,
	)
	if err != nil {
		// This shouldn't happen for cached data, but handle gracefully
		a.log.Warn().
			Err(err).
			Str("path", cached.Path).
			Msg("failed to reconstruct note from cache")
		return domain.Note{}
	}
	return note
}

func (a *BoltDBCacheReadAdapter) statFile(
	path string,
) (time.Time, bool, error) {
	absPath := filepath.Join(a.config.VaultPath, path)
	fi, err := os.Stat(absPath)
	if err != nil {
		if os.IsNotExist(err) {
			return time.Time{}, true, nil
		}
		return time.Time{}, false, err
	}
	return fi.ModTime(), false, nil
}

func (a *BoltDBCacheReadAdapter) cacheStaleStatus(
	ctx context.Context,
	path string,
	fileModTime time.Time,
) (bool, error) {
	viewErr := a.db.View(func(tx *bbolt.Tx) error {
		return a.evaluateCacheRecord(ctx, tx, path, fileModTime)
	})

	if viewErr != nil {
		switch {
		case errors.Is(viewErr, lithosErr.ErrNotFound):
			return true, nil
		case errors.Is(viewErr, errFileStale):
			return true, nil
		default:
			return false, viewErr
		}
	}

	return false, nil
}

func (a *BoltDBCacheReadAdapter) evaluateCacheRecord(
	ctx context.Context,
	tx *bbolt.Tx,
	path string,
	fileModTime time.Time,
) error {
	if err := ctx.Err(); err != nil {
		return err
	}

	bucket := tx.Bucket([]byte(BucketNotes))
	if bucket == nil {
		return lithosErr.ErrNotFound
	}
	data := bucket.Get([]byte(path))
	if data == nil {
		return lithosErr.ErrNotFound
	}

	cached, decodeErr := decodeCachedNote(data)
	if decodeErr != nil {
		return decodeErr
	}
	if fileModTime.After(cached.FileDates.IndexedAt) {
		return errFileStale
	}
	if !cached.FileDates.ModifiedAt.Equal(fileModTime) {
		return errFileStale
	}
	return nil
}

func decodeCachedNote(data []byte) (CachedNote, error) {
	var cached CachedNote
	if err := json.Unmarshal(data, &cached); err != nil {
		return CachedNote{}, err
	}
	return cached, nil
}

func (a *BoltDBCacheReadAdapter) indexedPaths(
	tx *bbolt.Tx,
	bucketName string,
	key string,
) ([]string, error) {
	indices := tx.Bucket([]byte(BucketIndices))
	if indices == nil {
		return nil, nil
	}

	indexBucket := indices.Bucket([]byte(bucketName))
	if indexBucket == nil {
		return nil, nil
	}

	return readIndexPaths(indexBucket, key)
}

func (a *BoltDBCacheReadAdapter) collectNotes(
	tx *bbolt.Tx,
	paths []string,
) ([]domain.Note, error) {
	notesBucket := tx.Bucket([]byte(BucketNotes))
	if notesBucket == nil {
		return nil, nil
	}

	results := make([]domain.Note, 0, len(paths))
	for _, p := range paths {
		noteData := notesBucket.Get([]byte(p))
		if noteData == nil {
			continue
		}

		cached, err := decodeCachedNote(noteData)
		if err != nil {
			return nil, err
		}
		results = append(results, a.reconstructNote(cached))
	}

	return results, nil
}
