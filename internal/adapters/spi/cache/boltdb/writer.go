package boltdb

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache"
	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"go.etcd.io/bbolt"
)

// Compile-time interface compliance check.
var _ spi.CacheWriterPort = (*BoltDBCacheWriteAdapter)(nil)

// CachedNote represents the metadata stored for each note in BoltDB.
// It matches the structure required for fast lookups and staleness detection.
type CachedNote struct {
	Path      string           `json:"path"`
	ID        string           `json:"id"` // Kept for backward compatibility
	Title     string           `json:"title"`
	Aliases   []string         `json:"aliases,omitempty"`
	FileClass string           `json:"file_class,omitempty"`
	FileDates dto.FileDatesDTO `json:"file_dates"`
}

// BoltDBCacheWriteAdapter implements CacheWriterPort for BoltDB-based
// note persistence with optimized indexing for hot data queries.
type BoltDBCacheWriteAdapter struct {
	config domain.Config
	log    zerolog.Logger
	db     *bbolt.DB
}

// NewBoltDBCacheWriter creates a new BoltDBCacheWriteAdapter.
func NewBoltDBCacheWriter(
	config domain.Config,
	log zerolog.Logger,
) (*BoltDBCacheWriteAdapter, error) {
	// Open BoltDB database in cache directory
	dbPath := config.CacheDir + "/hot.db"
	options := *bbolt.DefaultOptions
	options.Timeout = 1 * time.Second

	db, err := bbolt.Open(
		dbPath,
		boltDBFileMode,
		&options,
	)
	if err != nil {
		return nil, lithosErr.NewCacheWriteError("", dbPath, "open_db", err)
	}

	// Initialize buckets
	err = db.Update(func(tx *bbolt.Tx) error {
		// Primary bucket: notes
		if _, bucketErr := tx.CreateBucketIfNotExists([]byte(BucketNotes)); bucketErr != nil {
			return bucketErr
		}

		// Parent indices bucket
		indices, bucketErr := tx.CreateBucketIfNotExists([]byte(BucketIndices))
		if bucketErr != nil {
			return bucketErr
		}

		// Secondary index buckets
		subBuckets := []string{
			BucketIndexByBasename,
			BucketIndexByAlias,
			BucketIndexByFileClass,
			BucketIndexByFolder,
		}
		for _, name := range subBuckets {
			if _, subBucketErr := indices.CreateBucketIfNotExists([]byte(name)); subBucketErr != nil {
				return subBucketErr
			}
		}
		return nil
	})

	if err != nil {
		_ = db.Close()
		return nil, lithosErr.NewCacheWriteError(
			"",
			dbPath,
			"init_buckets",
			err,
		)
	}

	return &BoltDBCacheWriteAdapter{
		config: config,
		log:    log,
		db:     db,
	}, nil
}

// Close closes the BoltDB database connection.
func (a *BoltDBCacheWriteAdapter) Close() error {
	return a.db.Close()
}

// extractBasename extracts the basename from a file path.
func extractBasename(path string) string {
	parts := strings.Split(path, "/")
	filename := parts[len(parts)-1]
	if dotIndex := strings.LastIndex(filename, "."); dotIndex > 0 {
		return filename[:dotIndex]
	}
	return filename
}

// extractDirectory extracts the directory path from a file path.
func extractDirectory(path string) string {
	if lastSlash := strings.LastIndex(path, "/"); lastSlash > 0 {
		return path[:lastSlash]
	}
	return ""
}

// extractCachedNote extracts indexing metadata from a Note.
func extractCachedNote(
	note domain.Note,
	fileClassKey string,
) CachedNote {
	var cached CachedNote
	cached.Path = string(note.ID) // Use ID as path per current domain model
	cached.ID = string(note.ID)

	// FileDatesDTO
	cached.FileDates = dto.NewFileDatesDTO(
		cache.ExtractFileModTime(note.Frontmatter.Fields),
	)

	// Extract title
	if title, ok := note.Frontmatter.Fields["title"].(string); ok {
		cached.Title = title
	}

	// Extract aliases
	if aliases, ok := note.Frontmatter.Fields["aliases"].([]interface{}); ok {
		for _, alias := range aliases {
			if aliasStr, isString := alias.(string); isString {
				cached.Aliases = append(cached.Aliases, aliasStr)
			}
		}
	}

	// Extract file class
	if fileClass, ok := note.Frontmatter.Fields[fileClassKey].(string); ok {
		cached.FileClass = fileClass
	}

	return cached
}

// Persist stores note metadata in BoltDB with comprehensive indexing.
func (a *BoltDBCacheWriteAdapter) Persist(
	ctx context.Context,
	note domain.Note,
) error {
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	cached := extractCachedNote(note, a.config.FileClassKey)

	// Serialize metadata
	data, err := json.Marshal(cached)
	if err != nil {
		return lithosErr.NewCacheWriteError(
			cached.Path,
			cached.Path,
			"serialize_metadata",
			err,
		)
	}

	// Atomic transaction
	err = a.db.Update(func(tx *bbolt.Tx) error {
		return a.persistNoteInTransaction(tx, cached, data)
	})

	if err != nil {
		return lithosErr.NewCacheWriteError(
			cached.Path,
			cached.Path,
			"persist_transaction",
			err,
		)
	}

	a.log.Debug().
		Str("path", cached.Path).
		Msg("Persisted note metadata to BoltDB")

	return nil
}

// Delete removes note metadata from all BoltDB buckets.
func (a *BoltDBCacheWriteAdapter) Delete(
	ctx context.Context,
	id domain.NoteID,
) error {
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	path := string(id)

	err := a.db.Update(func(tx *bbolt.Tx) error {
		// 1. Primary bucket
		notesBucket := tx.Bucket([]byte(BucketNotes))
		if notesBucket == nil {
			return nil
		}

		// We need to read the note first to clean up secondary indices.
		noteData := notesBucket.Get([]byte(path))
		if noteData == nil {
			return nil // Idempotent delete
		}

		var cached CachedNote
		if err := json.Unmarshal(noteData, &cached); err != nil {
			return fmt.Errorf("failed to unmarshal note during delete: %w", err)
		}

		// 2. Clean up secondary indices
		if err := a.deleteFromSecondaryIndices(tx, cached); err != nil {
			return err
		}

		// 3. Delete from primary bucket
		if err := notesBucket.Delete([]byte(path)); err != nil {
			return err
		}
		return nil
	})

	if err != nil {
		return lithosErr.NewCacheDeleteError(
			path,
			path,
			"delete_transaction",
			err,
		)
	}

	a.log.Debug().
		Str("path", path).
		Msg("Deleted note metadata from BoltDB")

	return nil
}

// deleteFromSecondaryIndices removes the note from all secondary index buckets.
func (a *BoltDBCacheWriteAdapter) deleteFromSecondaryIndices(
	tx *bbolt.Tx,
	cached CachedNote,
) error {
	indicesBucket := tx.Bucket([]byte(BucketIndices))
	if indicesBucket == nil {
		return nil
	}

	// Helper to simplify index deletion
	deleteFromBucket := func(bucketName string, key string) error {
		if key == "" {
			return nil
		}
		if bucket := indicesBucket.Bucket([]byte(bucketName)); bucket != nil {
			return removeFromIndex(bucket, key, cached.Path)
		}
		return nil
	}

	// ByBasename
	if err := deleteFromBucket(BucketIndexByBasename, extractBasename(cached.Path)); err != nil {
		return err
	}

	// ByAlias
	if aliasBucket := indicesBucket.Bucket([]byte(BucketIndexByAlias)); aliasBucket != nil {
		for _, alias := range cached.Aliases {
			if err := removeFromIndex(aliasBucket, alias, cached.Path); err != nil {
				return err
			}
		}
	}

	// ByFileClass
	if err := deleteFromBucket(BucketIndexByFileClass, cached.FileClass); err != nil {
		return err
	}

	// ByFolder
	if err := deleteFromBucket(BucketIndexByFolder, extractDirectory(cached.Path)); err != nil {
		return err
	}

	return nil
}

// persistNoteInTransaction performs the atomic transaction.
func (a *BoltDBCacheWriteAdapter) persistNoteInTransaction(
	tx *bbolt.Tx,
	cached CachedNote,
	data []byte,
) error {
	pathBytes := []byte(cached.Path)

	// 1. Store primary metadata in /notes/ bucket
	notesBucket := tx.Bucket([]byte(BucketNotes))
	if err := notesBucket.Put(pathBytes, data); err != nil {
		return err
	}

	// Get indices parent bucket
	indicesBucket := tx.Bucket([]byte(BucketIndices))

	// 2. Update /indices/byBasename/
	basenameBucket := indicesBucket.Bucket([]byte(BucketIndexByBasename))
	basename := extractBasename(cached.Path)
	// Index stores []Path for duplicates
	if err := appendToIndex(basenameBucket, basename, cached.Path); err != nil {
		return err
	}

	// 3. Update /indices/byAlias/
	aliasBucket := indicesBucket.Bucket([]byte(BucketIndexByAlias))
	for _, alias := range cached.Aliases {
		if err := appendToIndex(aliasBucket, alias, cached.Path); err != nil {
			return err
		}
	}

	// 4. Update /indices/byFileClass/
	if cached.FileClass != "" {
		fcBucket := indicesBucket.Bucket([]byte(BucketIndexByFileClass))
		if err := appendToIndex(fcBucket, cached.FileClass, cached.Path); err != nil {
			return err
		}
	}

	// 5. Update /indices/byFolder/
	directory := extractDirectory(cached.Path)
	if directory != "" {
		dirBucket := indicesBucket.Bucket([]byte(BucketIndexByFolder))
		if err := appendToIndex(dirBucket, directory, cached.Path); err != nil {
			return err
		}
	}

	return nil
}

// appendToIndex adds a path to a JSON array in the specified bucket key.
func appendToIndex(bucket *bbolt.Bucket, key, value string) error {
	keyBytes := []byte(key)
	existingBytes := bucket.Get(keyBytes)
	var list []string

	if existingBytes != nil {
		if err := json.Unmarshal(existingBytes, &list); err != nil {
			return fmt.Errorf(
				"failed to unmarshal index list for %s: %w",
				key,
				err,
			)
		}
	}

	// Check for duplicates
	for _, v := range list {
		if v == value {
			return nil // Already present
		}
	}
	list = append(list, value)

	updatedBytes, err := json.Marshal(list)
	if err != nil {
		return fmt.Errorf("failed to marshal index list for %s: %w", key, err)
	}

	return bucket.Put(keyBytes, updatedBytes)
}

// removeFromIndex removes a path from a JSON array in the specified bucket key.
func removeFromIndex(bucket *bbolt.Bucket, key, value string) error {
	keyBytes := []byte(key)
	existingBytes := bucket.Get(keyBytes)
	if existingBytes == nil {
		return nil // Nothing to remove
	}

	var list []string
	if err := json.Unmarshal(existingBytes, &list); err != nil {
		return fmt.Errorf("failed to unmarshal index list for %s: %w", key, err)
	}

	// Filter out the value
	newList := make([]string, 0, len(list))
	found := false
	for _, v := range list {
		if v != value {
			newList = append(newList, v)
		} else {
			found = true
		}
	}

	if !found {
		return nil // Value not in list
	}

	if len(newList) == 0 {
		// If list is empty, delete the key
		return bucket.Delete(keyBytes)
	}

	updatedBytes, err := json.Marshal(newList)
	if err != nil {
		return fmt.Errorf("failed to marshal index list for %s: %w", key, err)
	}

	return bucket.Put(keyBytes, updatedBytes)
}
