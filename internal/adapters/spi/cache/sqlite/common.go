package sqlite

import (
	"context"
	"database/sql"
	"fmt"
	"time"

	"github.com/rs/zerolog"
	_ "modernc.org/sqlite" // Register SQLite driver
)

const (
	// Table schema.
	createNotesTable = `
		CREATE TABLE IF NOT EXISTS notes (
			path          TEXT PRIMARY KEY,
			frontmatter   TEXT,     -- JSON
			modified_at   INTEGER,  -- File's modification time (UNIX timestamp)
			indexed_time  INTEGER,  -- When we cached this (UNIX timestamp)
			size          INTEGER
		);
	`
	createIndexes = `
		CREATE INDEX IF NOT EXISTS idx_notes_modified_at ON notes(modified_at);
		CREATE INDEX IF NOT EXISTS idx_notes_indexed_time ON notes(indexed_time);
	`
)

// commonAdapter holds shared resources for SQLite operations.
type commonAdapter struct {
	db  *sql.DB
	log zerolog.Logger
}

// InitializeDatabase opens the DB and sets up the base schema.
func InitializeDatabase(dbPath string) (*sql.DB, error) {
	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, fmt.Errorf(
			"failed to open sqlite db at %s: %w",
			dbPath,
			err,
		)
	}

	// PRAGMAs for performance and consistency
	ctx := context.Background()
	if _, err = db.ExecContext(ctx, "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;"); err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("failed to set PRAGMAs: %w", err)
	}

	// Create base table and indexes
	if _, err = db.ExecContext(ctx, createNotesTable); err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("failed to create notes table: %w", err)
	}
	if _, err = db.ExecContext(ctx, createIndexes); err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("failed to create base indexes: %w", err)
	}

	return db, nil
}

// Helper to get unix timestamp from time.Time.
func toUnix(t time.Time) int64 {
	if t.IsZero() {
		return 0
	}
	return t.Unix()
}
