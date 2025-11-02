package cache

import (
	"context"
	"database/sql"
	"encoding/json"
	"path/filepath"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	_ "modernc.org/sqlite" // Register SQLite driver
)

// SQLite schema for deep storage.
const (
	createNotesTable = `
		CREATE TABLE IF NOT EXISTS notes (
			id TEXT PRIMARY KEY,
			path TEXT NOT NULL,
			title TEXT,
			file_class TEXT,
			frontmatter TEXT, -- JSON blob
			file_mod_time DATETIME,
			index_time DATETIME,
			UNIQUE(path)
		)`

	createIndexes = `
		CREATE INDEX IF NOT EXISTS idx_notes_path ON notes(path);
		CREATE INDEX IF NOT EXISTS idx_notes_file_class ON notes(file_class);
		CREATE INDEX IF NOT EXISTS idx_notes_file_mod_time ON notes(file_mod_time);
		CREATE INDEX IF NOT EXISTS idx_notes_index_time ON notes(index_time);
		CREATE INDEX IF NOT EXISTS idx_notes_staleness ON notes(file_mod_time, index_time);`
)

// Compile-time interface compliance check.
// This ensures SQLiteCacheWriteAdapter implements CacheWriterPort correctly.
var _ spi.CacheWriterPort = (*SQLiteCacheWriteAdapter)(nil)

// SQLiteCacheWriteAdapter implements CacheWriterPort for SQLite-based
// deep storage with complex query capabilities. It uses relational
// storage for comprehensive note indexing and metadata storage.
//
// Schema:
// - notes table: id, path, title, file_class, frontmatter (JSON),
// file_mod_time, index_time - Indexes: path, file_class, file_mod_time,
// index_time, composite staleness index
//
// Query Capabilities:
// - Complex frontmatter queries with JSON operators
// - Full-text search capabilities
// - Relational operations and aggregations
// - Staleness detection with composite indexes
//
// Thread Safety:
// - SQLite provides ACID transactions
// - Safe for concurrent operations with proper connection handling
//
// See docs/architecture/components.md#sqlite-cache-adapters for implementation
// details.
type SQLiteCacheWriteAdapter struct {
	config domain.Config
	log    zerolog.Logger
	db     *sql.DB
}

// NewSQLiteCacheWriteAdapter creates a new SQLiteCacheWriteAdapter with the
// provided
// configuration and logger. The adapter opens/creates the SQLite database
// and initializes the schema.
//
// Parameters:
//   - config: Application configuration containing CacheDir and FileClassKey
//   - log: Structured logger for operation tracking
//
// Returns:
//   - *SQLiteCacheWriteAdapter: Configured adapter ready for cache operations
//   - error: If database initialization fails
//
// Thread Safety: The returned adapter is safe for concurrent use.
func NewSQLiteCacheWriteAdapter(
	config domain.Config,
	log zerolog.Logger,
) (*SQLiteCacheWriteAdapter, error) {
	// Open SQLite database in cache directory
	dbPath := filepath.Join(config.CacheDir, "cache.db")
	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, errors.NewCacheWriteError("", dbPath, "open_db", err)
	}

	// Enable foreign keys and WAL mode for better concurrency
	if _, execErr := db.ExecContext(
		context.Background(),
		"PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;",
	); execErr != nil {
		_ = db.Close()
		return nil, errors.NewCacheWriteError(
			"",
			dbPath,
			"init_pragmas",
			execErr,
		)
	}

	// Create schema
	if schemaErr := initSQLiteSchema(db); schemaErr != nil {
		_ = db.Close()
		return nil, errors.NewCacheWriteError(
			"",
			dbPath,
			"init_schema",
			schemaErr,
		)
	}

	return &SQLiteCacheWriteAdapter{
		config: config,
		log:    log,
		db:     db,
	}, nil
}

// initSQLiteSchema creates the database schema and indexes.
func initSQLiteSchema(db *sql.DB) error {
	// Create tables
	if _, err := db.ExecContext(context.Background(), createNotesTable); err != nil {
		return err
	}

	// Create indexes
	if _, err := db.ExecContext(context.Background(), createIndexes); err != nil {
		return err
	}

	return nil
}

// Close closes the SQLite database connection.
// Should be called when the adapter is no longer needed.
func (a *SQLiteCacheWriteAdapter) Close() error {
	return a.db.Close()
}

// extractSQLiteNoteMetadata extracts metadata from a Note for SQLite storage.
// Includes all frontmatter as JSON for complex queries.
func extractSQLiteNoteMetadata(
	note domain.Note,
	fileClassKey string,
) (map[string]interface{}, error) {
	metadata := map[string]interface{}{
		"id":         string(note.ID),
		"path":       note.Path,
		"index_time": time.Now(),
	}

	// Extract title
	if title, ok := note.Frontmatter.Fields["title"].(string); ok {
		metadata["title"] = title
	}

	// Extract file class using configurable key
	if fileClass, ok := note.Frontmatter.Fields[fileClassKey].(string); ok {
		metadata["file_class"] = fileClass
	}

	// Store complete frontmatter as JSON
	frontmatterJSON, err := json.Marshal(note.Frontmatter.Fields)
	if err != nil {
		return nil, err
	}
	metadata["frontmatter"] = string(frontmatterJSON)

	// Extract file_mod_time from frontmatter fields
	metadata["file_mod_time"] = extractFileModTime(note.Frontmatter.Fields)

	return metadata, nil
}

// Persist stores note metadata in SQLite with comprehensive indexing.
// Supports complex queries through JSON frontmatter storage and indexes.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//   - note: Note to persist with its metadata
//
// Returns:
//   - error: Wrapped with operation context if persistence fails
//
// Thread-safe: Safe for concurrent calls via SQLite transactions.
// Staleness: Updates index_time for staleness detection.
func (a *SQLiteCacheWriteAdapter) Persist(
	ctx context.Context,
	note domain.Note,
) error {
	// Check for context cancellation
	if err := checkContext(ctx); err != nil {
		return err
	}

	metadata, err := extractSQLiteNoteMetadata(note, a.config.FileClassKey)
	if err != nil {
		return errors.NewCacheWriteError(
			string(note.ID),
			note.Path,
			"extract_metadata",
			err,
		)
	}

	return a.persistWithTransaction(ctx, note, metadata)
}

// checkContext checks if the context is canceled and returns appropriate error.
func checkContext(ctx context.Context) error {
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
		return nil
	}
}

// Delete removes note metadata from SQLite.
// Idempotent: returns nil if note doesn't exist.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//   - id: NoteID to delete
//
// Returns:
//   - error: Wrapped with operation context if deletion fails
//
// Thread-safe: Safe for concurrent calls via SQLite transactions.
func (a *SQLiteCacheWriteAdapter) Delete(
	ctx context.Context,
	id domain.NoteID,
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	// Use transaction for consistency
	tx, err := a.db.BeginTx(ctx, nil)
	if err != nil {
		return errors.NewCacheDeleteError(
			string(id),
			"",
			"begin_transaction",
			err,
		)
	}
	defer func() { _ = tx.Rollback() }()

	result, err := tx.ExecContext(
		ctx,
		"DELETE FROM notes WHERE id = ?",
		string(id),
	)
	if err != nil {
		return errors.NewCacheDeleteError(
			string(id),
			"",
			"delete_note",
			err,
		)
	}

	if commitErr := tx.Commit(); commitErr != nil {
		return errors.NewCacheDeleteError(
			string(id),
			"",
			"commit_transaction",
			commitErr,
		)
	}

	rowsAffected, _ := result.RowsAffected()
	a.log.Debug().
		Str("note_id", string(id)).
		Int64("rows_affected", rowsAffected).
		Msg("Deleted note metadata from SQLite")

	return nil
}

// persistWithTransaction handles the database transaction for persisting a
// note.
func (a *SQLiteCacheWriteAdapter) persistWithTransaction(
	ctx context.Context,
	note domain.Note,
	metadata map[string]interface{},
) error {
	// Use transaction for atomicity
	tx, err := a.db.BeginTx(ctx, nil)
	if err != nil {
		return errors.NewCacheWriteError(
			string(note.ID),
			note.Path,
			"begin_transaction",
			err,
		)
	}
	defer func() { _ = tx.Rollback() }()

	if insertErr := a.insertNoteInTransaction(ctx, tx, note, metadata); insertErr != nil {
		return insertErr
	}

	if commitErr := tx.Commit(); commitErr != nil {
		return errors.NewCacheWriteError(
			string(note.ID),
			note.Path,
			"commit_transaction",
			commitErr,
		)
	}

	a.log.Debug().
		Str("note_id", string(note.ID)).
		Str("path", note.Path).
		Msg("SQLite cache write successful")

	return nil
}

// insertNoteInTransaction inserts or replaces a note in the database
// transaction.
func (a *SQLiteCacheWriteAdapter) insertNoteInTransaction(
	ctx context.Context,
	tx *sql.Tx,
	note domain.Note,
	metadata map[string]interface{},
) error {
	query := `
		INSERT OR REPLACE INTO notes
		(id, path, title, file_class, frontmatter, file_mod_time, index_time)
		VALUES (?, ?, ?, ?, ?, ?, ?)`

	_, err := tx.ExecContext(ctx, query,
		metadata["id"],
		metadata["path"],
		metadata["title"],
		metadata["file_class"],
		metadata["frontmatter"],
		metadata["file_mod_time"],
		metadata["index_time"],
	)

	if err != nil {
		return errors.NewCacheWriteError(
			string(note.ID),
			note.Path,
			"insert_note",
			err,
		)
	}

	return nil
}
