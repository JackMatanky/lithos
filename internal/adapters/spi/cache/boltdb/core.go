package boltdb

import (
	"os"
	"path/filepath"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"go.etcd.io/bbolt"
)

// BoltDB bucket names.
const (
	// BucketNotes is the primary bucket for storing Note data.
	// Key: Vault-relative path (e.g. "notes/meeting.md")
	// Value: CachedNote JSON (including FileDatesDTO).
	BucketNotes = "notes"

	// BucketIndices is the parent bucket for secondary indices.
	BucketIndices = "indices"

	// BucketIndexByBasename is the secondary index: Basename -> []Path.
	BucketIndexByBasename = "byBasename"

	// BucketIndexByAlias is the secondary index: Alias -> []Path.
	BucketIndexByAlias = "byAlias"

	// BucketIndexByFileClass is the secondary index: FileClass -> []Path.
	BucketIndexByFileClass = "byFileClass"

	// BucketIndexByFolder is the secondary index: Folder -> []Path (for
	// folder-scoped PathQuery).
	BucketIndexByFolder = "byFolder"
)

// boltDBFileMode represents the POSIX file permissions used when creating
// BoltDB database files. Uses restrictive permissions (0600) for security.
const boltDBFileMode = 0o600

// Open creates or opens a BoltDB instance.
// It ensures the cache directory exists.
func Open(config domain.Config) (*bbolt.DB, error) {
	// Ensure cache directory exists
	if err := os.MkdirAll(config.CacheDir, 0o750); err != nil {
		return nil, lithosErr.NewCacheWriteError(
			"",
			config.CacheDir,
			"create_dir",
			err,
		)
	}

	dbPath := filepath.Join(config.CacheDir, "hot.db")
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
	return db, nil
}
