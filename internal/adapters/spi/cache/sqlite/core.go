package sqlite

import (
	"context"
	"database/sql"
	"fmt"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	_ "modernc.org/sqlite" // Register SQLite driver
)

// SQLite table and view names.
const (
	// TableNotes is the primary table for storing Note data.
	// Columns: path (PRIMARY KEY), frontmatter (JSON), modified_at,
	// indexed_time, size.
	TableNotes = "notes"

	// ViewPrefix is the prefix for schema-driven views.
	// Format: v_{schema_name}_notes (e.g., v_contact_notes, v_project_notes).
	ViewPrefix = "v_"

	// ViewSuffix is the suffix for schema-driven views.
	ViewSuffix = "_notes"
)

// SQLite index names.
const (
	// IndexNotesModifiedAt indexes the modified_at column for staleness
	// queries.
	IndexNotesModifiedAt = "idx_notes_modified_at"

	// IndexNotesIndexedTime indexes the indexed_time column for staleness
	// queries.
	IndexNotesIndexedTime = "idx_notes_indexed_time"
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

func openSQLiteDatabase(
	config domain.Config,
	migrator *SchemaViewMigrator,
	wrap func(dbPath, operation string, cause error) error,
) (*sql.DB, error) {
	dbPath := config.CacheDir + "/cold.db"

	db, err := InitializeDatabase(dbPath)
	if err != nil {
		return nil, wrap(dbPath, "init_db", err)
	}

	if migrator != nil {
		if migrateErr := migrator.EnsureViews(context.Background(), db); migrateErr != nil {
			_ = db.Close()
			return nil, wrap(dbPath, "ensure_views", migrateErr)
		}
	}

	return db, nil
}

func newCommonAdapter(
	config domain.Config,
	log zerolog.Logger,
	migrator *SchemaViewMigrator,
	wrap func(dbPath, operation string, cause error) error,
) (*commonAdapter, error) {
	db, err := openSQLiteDatabase(config, migrator, wrap)
	if err != nil {
		return nil, err
	}
	return &commonAdapter{db: db, log: log}, nil
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
