package boltdb

import (
	"context"
	"encoding/json"
	"fmt"
	pathpkg "path"
	"strings"

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"go.etcd.io/bbolt"
)

// Compile-time interface compliance check.
var _ spi.CacheWriterPort = (*BoltDBCacheWriteAdapter)(nil)

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
	db *bbolt.DB,
) (*BoltDBCacheWriteAdapter, error) {
	// Initialize buckets
	err := db.Update(func(tx *bbolt.Tx) error {
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
			BucketIndexBasenameQuery,
			BucketIndexAliasQuery,
			BucketIndexFileClassQuery,
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
		return nil, lithosErr.NewCacheWriteError(
			"",
			config.CacheDir,
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
	// We do NOT close DB here if it's shared.
	// But adapter interface has no Close method?
	// Wait, writer.go had Close().
	// If we pass DB in, who owns it?
	// The caller owns it.
	// So we shouldn't close it?
	// Or we keep Close() for convenience but caller must be aware.
	// For now, let's keep Close() but typically shared DB is closed by main.
	return nil // No-op, let caller close DB
}

// extractBasename extracts the basename from a file path.
func extractBasename(path string) string {
	normalized := normalizePath(path)
	if normalized == "" {
		return ""
	}

	filename := pathpkg.Base(normalized)
	extension := pathpkg.Ext(filename)
	return strings.TrimSuffix(filename, extension)
}

// extractDirectory extracts the directory path from a file path.
func extractDirectory(path string) string {
	normalized := normalizePath(path)
	if normalized == "" {
		return ""
	}

	dir := pathpkg.Dir(normalized)
	if dir == "." || dir == "/" {
		return ""
	}

	return dir
}

// extractCachedNote extracts indexing metadata from a Note.
func extractCachedNote(
	note domain.Note,
	fileClassKey string,
	metadata spi.CacheWriteMetadata,
) CachedNote {
	var cached CachedNote
	cached.Path = note.Path // Use Path as identifier per new domain model
	cached.ID = note.Path

	// FileDatesDTO
	fileDates := dto.NewFileDatesDTO(metadata.ModifiedAt)
	fileDates.IndexedAt = metadata.IndexTime
	cached.FileDates = fileDates

	// Extract title
	if domain.Is[string](note.Frontmatter, "title") {
		if title, ok := note.Frontmatter.Get("title"); ok {
			cached.Title = title.(string)
		}
	}

	// Extract aliases
	if note.Frontmatter.IsArray("aliases") {
		cached.Aliases = note.Frontmatter.Aliases()
	}

	// Extract file class
	if domain.Is[string](note.Frontmatter, fileClassKey) {
		if fileClass, ok := note.Frontmatter.Get(fileClassKey); ok {
			cached.FileClass = fileClass.(string)
		}
	}

	return cached
}

// Persist stores note metadata in BoltDB with comprehensive indexing.
func (a *BoltDBCacheWriteAdapter) Persist(
	ctx context.Context,
	note domain.Note,
	metadata spi.CacheWriteMetadata,
) error {
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	cached := extractCachedNote(note, a.config.FileClassKey, metadata)

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
	path string,
) error {
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

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

	// BasenameQuery
	if err := deleteFromBucket(BucketIndexBasenameQuery, extractBasename(cached.Path)); err != nil {
		return err
	}

	// AliasQuery
	if aliasBucket := indicesBucket.Bucket([]byte(BucketIndexAliasQuery)); aliasBucket != nil {
		for _, alias := range cached.Aliases {
			if err := removeFromIndex(aliasBucket, alias, cached.Path); err != nil {
				return err
			}
		}
	}

	// FileClassQuery
	if err := deleteFromBucket(BucketIndexFileClassQuery, cached.FileClass); err != nil {
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
	notesBucket, err := requireBucket(tx, BucketNotes)
	if err != nil {
		return err
	}

	if putErr := notesBucket.Put([]byte(cached.Path), data); putErr != nil {
		return putErr
	}

	indicesBucket, err := requireBucket(tx, BucketIndices)
	if err != nil {
		return err
	}

	if updateErr := a.updateBasenameIndex(indicesBucket, cached); updateErr != nil {
		return updateErr
	}
	if updateErr := a.updateAliasIndex(indicesBucket, cached); updateErr != nil {
		return updateErr
	}
	if updateErr := a.updateFileClassIndex(indicesBucket, cached); updateErr != nil {
		return updateErr
	}
	return a.updateFolderIndex(indicesBucket, cached)
}

func (a *BoltDBCacheWriteAdapter) updateBasenameIndex(
	indicesBucket *bbolt.Bucket,
	cached CachedNote,
) error {
	bucket, err := requireSubBucket(indicesBucket, BucketIndexBasenameQuery)
	if err != nil {
		return err
	}
	return appendToIndex(bucket, extractBasename(cached.Path), cached.Path)
}

func (a *BoltDBCacheWriteAdapter) updateAliasIndex(
	indicesBucket *bbolt.Bucket,
	cached CachedNote,
) error {
	if len(cached.Aliases) == 0 {
		return nil
	}
	bucket, err := requireSubBucket(indicesBucket, BucketIndexAliasQuery)
	if err != nil {
		return err
	}
	for _, alias := range cached.Aliases {
		if appendErr := appendToIndex(bucket, alias, cached.Path); appendErr != nil {
			return appendErr
		}
	}
	return nil
}

func (a *BoltDBCacheWriteAdapter) updateFileClassIndex(
	indicesBucket *bbolt.Bucket,
	cached CachedNote,
) error {
	if cached.FileClass == "" {
		return nil
	}
	bucket, err := requireSubBucket(indicesBucket, BucketIndexFileClassQuery)
	if err != nil {
		return err
	}
	return appendToIndex(bucket, cached.FileClass, cached.Path)
}

func (a *BoltDBCacheWriteAdapter) updateFolderIndex(
	indicesBucket *bbolt.Bucket,
	cached CachedNote,
) error {
	directory := extractDirectory(cached.Path)
	if directory == "" {
		return nil
	}
	bucket, err := requireSubBucket(indicesBucket, BucketIndexByFolder)
	if err != nil {
		return err
	}
	return appendToIndex(bucket, directory, cached.Path)
}

// appendToIndex adds a path to a JSON array in the specified bucket key.
func appendToIndex(bucket *bbolt.Bucket, key, value string) error {
	if key == "" || value == "" {
		return nil
	}

	list, err := readIndexPaths(bucket, key)
	if err != nil {
		return err
	}

	if containsPath(list, value) {
		return nil
	}

	list = append(list, value)
	return writeIndexPaths(bucket, key, list)
}

// removeFromIndex removes a path from a JSON array in the specified bucket key.
func removeFromIndex(bucket *bbolt.Bucket, key, value string) error {
	if key == "" || value == "" {
		return nil
	}

	list, err := readIndexPaths(bucket, key)
	if err != nil {
		return err
	}

	if len(list) == 0 {
		return nil
	}

	newList := make([]string, 0, len(list))
	for _, v := range list {
		if v != value {
			newList = append(newList, v)
		}
	}

	if len(newList) == len(list) {
		return nil
	}

	return writeIndexPaths(bucket, key, newList)
}

func normalizePath(path string) string {
	if path == "" {
		return ""
	}
	sanitized := strings.ReplaceAll(path, "\\", "/")
	clean := pathpkg.Clean(sanitized)
	if clean == "." {
		return ""
	}
	return clean
}

func readIndexPaths(bucket *bbolt.Bucket, key string) ([]string, error) {
	if bucket == nil {
		return nil, fmt.Errorf("bucket is nil for key %s", key)
	}
	data := bucket.Get([]byte(key))
	if data == nil {
		return nil, nil
	}
	var list []string
	if err := json.Unmarshal(data, &list); err != nil {
		return nil, fmt.Errorf(
			"failed to unmarshal index list for %s: %w",
			key,
			err,
		)
	}
	return list, nil
}

func writeIndexPaths(bucket *bbolt.Bucket, key string, paths []string) error {
	if bucket == nil {
		return fmt.Errorf("bucket is nil for key %s", key)
	}
	if len(paths) == 0 {
		return bucket.Delete([]byte(key))
	}
	updatedBytes, err := json.Marshal(paths)
	if err != nil {
		return fmt.Errorf("failed to marshal index list for %s: %w", key, err)
	}
	return bucket.Put([]byte(key), updatedBytes)
}

func containsPath(paths []string, value string) bool {
	for _, path := range paths {
		if path == value {
			return true
		}
	}
	return false
}

func requireBucket(tx *bbolt.Tx, name string) (*bbolt.Bucket, error) {
	bucket := tx.Bucket([]byte(name))
	if bucket == nil {
		return nil, fmt.Errorf("bucket %s does not exist", name)
	}
	return bucket, nil
}

func requireSubBucket(
	parent *bbolt.Bucket,
	name string,
) (*bbolt.Bucket, error) {
	if parent == nil {
		return nil, fmt.Errorf("parent bucket missing for %s", name)
	}
	bucket := parent.Bucket([]byte(name))
	if bucket == nil {
		return nil, fmt.Errorf("bucket %s does not exist", name)
	}
	return bucket, nil
}
