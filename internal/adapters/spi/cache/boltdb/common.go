package boltdb

import (
	"os"
	"path/filepath"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"go.etcd.io/bbolt"
)

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
