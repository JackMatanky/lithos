package cache

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"go.etcd.io/bbolt"
)

// Compile-time interface compliance check.
// This ensures BoltDBCacheWriteAdapter implements CacheWriterPort correctly.
var _ spi.CacheWriterPort = (*BoltDBCacheWriteAdapter)(nil)

// BoltDBNoteMetadata represents the metadata stored for each note in BoltDB.
// This enables fast queries across different dimensions.
type BoltDBNoteMetadata struct {
	Path        string    `json:"path"`
	ID          string    `json:"id"`
	Title       string    `json:"title"`
	Aliases     []string  `json:"aliases,omitempty"`
	FileClass   string    `json:"file_class,omitempty"`
	FileModTime time.Time `json:"file_mod_time"`
	IndexTime   time.Time `json:"index_time"`
}

// BoltDBCacheWriteAdapter implements CacheWriterPort for BoltDB-based
// note persistence with optimized indexing for hot data queries. It uses
// the CQRS write-side pattern with structured buckets for fast lookups.
//
// Bucket Structure:
// - /paths/: Key=path, Value=BoltDBNoteMetadata (primary storage)
// - /basenames/: Key=basename, Value=[]NoteID (secondary index)
// - /aliases/: Key=alias, Value=[]NoteID (secondary index)
// - /file_classes/: Key=file_class, Value=[]NoteID (secondary index)
// - /directories/: Key=directory, Value=[]NoteID (secondary index)
// - /staleness/: Key=noteID, Value={file_mod_time, index_time} (staleness
// detection)
//
// Thread Safety:
// - BoltDB provides ACID transactions and concurrent read/write access
// - Safe for concurrent operations from multiple indexers
//
// Staleness Detection:
// - Compares FileMetadata.ModTime vs stored index_time
// - Enables incremental indexing and cache invalidation
//
// See docs/architecture/components.md#boltdb-cache-adapters for implementation
// details.
type BoltDBCacheWriteAdapter struct {
	config domain.Config
	log    zerolog.Logger
	db     *bbolt.DB
}

// NewBoltDBCacheWriter creates a new BoltDBCacheWriteAdapter with the provided
// configuration and logger. The adapter opens/creates the BoltDB database
// and initializes the required buckets.
//
// Parameters:
//   - config: Application configuration containing CacheDir and FileClassKey
//   - log: Structured logger for operation tracking
//
// Returns:
//   - *BoltDBCacheWriteAdapter: Configured adapter ready for cache operations
//   - error: If database initialization fails
//
// Thread Safety: The returned adapter is safe for concurrent use.
func NewBoltDBCacheWriter(
	config domain.Config,
	log zerolog.Logger,
) (*BoltDBCacheWriteAdapter, error) {
	// Open BoltDB database in cache directory
	dbPath := config.CacheDir + "/cache.db"
	options := *bbolt.DefaultOptions
	options.Timeout = 1 * time.Second

	db, err := bbolt.Open(
		dbPath,
		boltDBFileMode,
		&options,
	)
	if err != nil {
		return nil, errors.NewCacheWriteError("", dbPath, "open_db", err)
	}

	// Initialize buckets
	err = db.Update(func(tx *bbolt.Tx) error {
		buckets := []string{
			bucketPaths,
			bucketBasenames,
			bucketAliases,
			bucketFileClasses,
			bucketDirectories,
			bucketStaleness,
		}
		for _, bucket := range buckets {
			_, bucketErr := tx.CreateBucketIfNotExists([]byte(bucket))
			if bucketErr != nil {
				return bucketErr
			}
		}
		return nil
	})
	if err != nil {
		_ = db.Close()
		return nil, errors.NewCacheWriteError("", dbPath, "init_buckets", err)
	}

	return &BoltDBCacheWriteAdapter{
		config: config,
		log:    log,
		db:     db,
	}, nil
}

// Close closes the BoltDB database connection.
// Should be called when the adapter is no longer needed.
func (a *BoltDBCacheWriteAdapter) Close() error {
	return a.db.Close()
}

// extractFileModTime extracts the file modification time from frontmatter
// fields.
// Looks for common field names like "file_mod_time", "modified", "updated".
// Falls back to current time if not found.
func extractFileModTime(fields map[string]interface{}) time.Time {
	if modTime, ok := fields["file_mod_time"].(time.Time); ok {
		return modTime
	}
	if modTime, ok := fields["modified"].(time.Time); ok {
		return modTime
	}
	if modTime, ok := fields["updated"].(time.Time); ok {
		return modTime
	}
	// Fallback to current time
	return time.Now()
}

// extractBasename extracts the basename from a file path.
// Removes the directory path and file extension.
func extractBasename(path string) string {
	// Remove directory path
	parts := strings.Split(path, "/")
	filename := parts[len(parts)-1]
	// Remove extension
	if dotIndex := strings.LastIndex(filename, "."); dotIndex > 0 {
		return filename[:dotIndex]
	}
	return filename
}

// extractDirectory extracts the directory path from a file path.
// Returns empty string for files in root directory.
func extractDirectory(path string) string {
	if lastSlash := strings.LastIndex(path, "/"); lastSlash > 0 {
		return path[:lastSlash]
	}
	return ""
}

// extractNoteMetadata extracts indexing metadata from a Note.
// This includes title, aliases, and file class for fast queries.
func extractNoteMetadata(
	note domain.Note,
	fileClassKey string,
) BoltDBNoteMetadata {
	var metadata BoltDBNoteMetadata
	metadata.Path = note.Path
	metadata.ID = string(note.ID)
	metadata.IndexTime = time.Now()

	// Extract title from frontmatter
	if title, ok := note.Frontmatter.Fields["title"].(string); ok {
		metadata.Title = title
	}

	// Extract aliases from frontmatter
	if aliases, ok := note.Frontmatter.Fields["aliases"].([]interface{}); ok {
		for _, alias := range aliases {
			if aliasStr, isString := alias.(string); isString {
				metadata.Aliases = append(metadata.Aliases, aliasStr)
			}
		}
	}

	// Extract file class using configurable key
	if fileClass, ok := note.Frontmatter.Fields[fileClassKey].(string); ok {
		metadata.FileClass = fileClass
	}

	// Extract file_mod_time from frontmatter fields
	metadata.FileModTime = extractFileModTime(note.Frontmatter.Fields)

	return metadata
}

// Persist stores note metadata in BoltDB with comprehensive indexing.
// Creates/updates entries in all relevant buckets for fast queries.
// Uses atomic transactions to ensure consistency.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//   - note: Note to persist with its metadata
//
// Returns:
//   - error: Wrapped with operation context if persistence fails
//
// Thread-safe: Safe for concurrent calls via BoltDB transactions.
// Staleness: Updates index_time for staleness detection.
func (a *BoltDBCacheWriteAdapter) Persist(
	ctx context.Context,
	note domain.Note,
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	metadata := extractNoteMetadata(note, a.config.FileClassKey)

	// Serialize metadata
	data, err := json.Marshal(metadata)
	if err != nil {
		return errors.NewCacheWriteError(
			string(note.ID),
			note.Path,
			"serialize_metadata",
			err,
		)
	}

	// Atomic transaction for all bucket updates
	err = a.db.Update(func(tx *bbolt.Tx) error {
		return a.persistNoteInTransaction(tx, note, metadata, data)
	})

	if err != nil {
		return errors.NewCacheWriteError(
			string(note.ID),
			note.Path,
			"persist_transaction",
			err,
		)
	}

	a.log.Debug().
		Str("note_id", string(note.ID)).
		Str("path", note.Path).
		Msg("Persisted note metadata to BoltDB")

	return nil
}

// Delete removes note metadata from all BoltDB buckets.
// Currently only removes from staleness bucket due to lack of reverse lookup.
// TODO: Implement full cleanup of all secondary indexes.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//   - id: NoteID to delete
//
// Returns:
//   - error: Wrapped with operation context if deletion fails
//
// Thread-safe: Safe for concurrent calls via BoltDB transactions.
func (a *BoltDBCacheWriteAdapter) Delete(
	ctx context.Context,
	id domain.NoteID,
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	noteID := []byte(string(id))

	// Clean up all secondary indexes
	// Note: This is a simplified cleanup that may leave orphaned entries
	// A full implementation would require reading metadata first
	err := a.db.Update(func(tx *bbolt.Tx) error {
		return a.deleteNoteFromBuckets(tx, noteID)
	})

	if err != nil {
		return errors.NewCacheDeleteError(
			string(id),
			"",
			"delete_transaction",
			err,
		)
	}

	a.log.Debug().
		Str("note_id", string(id)).
		Msg("Deleted note metadata from BoltDB")

	return nil
}

// persistNoteInTransaction performs the atomic transaction to store note
// metadata
// in all relevant BoltDB buckets. This method handles all bucket updates within
// a single transaction to ensure consistency.
func (a *BoltDBCacheWriteAdapter) persistNoteInTransaction(
	tx *bbolt.Tx,
	note domain.Note,
	metadata BoltDBNoteMetadata,
	data []byte,
) error {
	noteID := []byte(string(note.ID))

	// 1. Store primary metadata in /paths/ bucket
	pathsBucket := tx.Bucket([]byte(bucketPaths))
	if putErr := pathsBucket.Put([]byte(note.Path), data); putErr != nil {
		return putErr
	}

	// 2. Update /basenames/ secondary index
	basenamesBucket := tx.Bucket([]byte(bucketBasenames))
	basename := extractBasename(note.Path)
	if putErr := basenamesBucket.Put([]byte(basename), noteID); putErr != nil {
		return putErr
	}

	// 3. Update /aliases/ secondary index
	aliasesBucket := tx.Bucket([]byte(bucketAliases))
	for _, alias := range metadata.Aliases {
		if putErr := aliasesBucket.Put([]byte(alias), noteID); putErr != nil {
			return putErr
		}
	}

	// 4. Update /file_classes/ secondary index
	if err := a.updateFileClassIndex(tx, metadata.FileClass, note.ID); err != nil {
		return err
	}

	// 5. Update /directories/ secondary index
	directoriesBucket := tx.Bucket([]byte(bucketDirectories))
	directory := extractDirectory(note.Path)
	if directory != "" {
		if putErr := directoriesBucket.Put([]byte(directory), noteID); putErr != nil {
			return putErr
		}
	}

	// 6. Update /staleness/ bucket
	stalenessBucket := tx.Bucket([]byte(bucketStaleness))
	stalenessData, _ := json.Marshal(map[string]time.Time{
		"file_mod_time": metadata.FileModTime,
		"index_time":    metadata.IndexTime,
	})
	if putErr := stalenessBucket.Put(noteID, stalenessData); putErr != nil {
		return putErr
	}

	return nil
}

// updateFileClassIndex updates the file class secondary index by maintaining
// a JSON array of note IDs for each file class.
func (a *BoltDBCacheWriteAdapter) updateFileClassIndex(
	tx *bbolt.Tx,
	fileClass string,
	noteID domain.NoteID,
) error {
	if fileClass == "" {
		return nil
	}

	fileClassesBucket := tx.Bucket([]byte(bucketFileClasses))

	// Get existing ID list for this file class
	existingBytes := fileClassesBucket.Get([]byte(fileClass))
	var idList []string
	if existingBytes != nil {
		if err := json.Unmarshal(existingBytes, &idList); err != nil {
			return fmt.Errorf(
				"failed to unmarshal existing ID list for file class %s: %w",
				fileClass,
				err,
			)
		}
	}

	// Add this note ID if not already present
	noteIDStr := string(noteID)
	for _, id := range idList {
		if id == noteIDStr {
			return nil // Already present
		}
	}
	idList = append(idList, noteIDStr)

	// Store updated ID list
	updatedBytes, err := json.Marshal(idList)
	if err != nil {
		return fmt.Errorf(
			"failed to marshal ID list for file class %s: %w",
			fileClass,
			err,
		)
	}

	return fileClassesBucket.Put([]byte(fileClass), updatedBytes)
}

// deleteNoteFromBuckets performs the transaction to remove note metadata
// from all BoltDB buckets. Currently only cleans primary buckets due to
// complexity of secondary index cleanup.
func (a *BoltDBCacheWriteAdapter) deleteNoteFromBuckets(
	tx *bbolt.Tx,
	noteID []byte,
) error {
	// Delete from primary paths bucket
	pathsBucket := tx.Bucket([]byte(bucketPaths))
	if delErr := pathsBucket.Delete(noteID); delErr != nil {
		return delErr
	}

	// Delete from staleness bucket
	stalenessBucket := tx.Bucket([]byte(bucketStaleness))
	if delErr := stalenessBucket.Delete(noteID); delErr != nil {
		return delErr
	}

	// Note: Secondary indexes (basenames, aliases, file_classes, directories)
	// are not cleaned up here as they require reverse lookup of the original
	// values. This is a known limitation that should be addressed in a future
	// iteration.

	return nil
}
