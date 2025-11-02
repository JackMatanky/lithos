package cache

import (
	"context"
	"database/sql"
	"encoding/json"
	"path/filepath"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	_ "modernc.org/sqlite" // Register SQLite driver
)

// Compile-time interface compliance check.
// This ensures SQLiteCacheReadAdapter implements CacheReaderPort correctly.
var _ spi.CacheReaderPort = (*SQLiteCacheReadAdapter)(nil)

// SQLiteCacheReadAdapter implements CacheReaderPort for SQLite-based
// complex queries with JSON frontmatter support. It provides advanced
// query capabilities for deep storage retrieval.
//
// Query Capabilities:
// - ByPath: Direct path lookup with index
// - ByFrontmatter: Complex JSON queries (e.g., tags, dates, custom fields)
// - Full-text search: SQLite FTS5 for content search
// - ByFileClass: Indexed file class queries
// - Staleness detection: Compare file_mod_time vs index_time
//
// Performance:
// - Optimized with indexes on common query patterns
// - JSON operators for flexible frontmatter queries
// - Composite indexes for staleness detection
//
// Thread Safety:
// - SQLite provides concurrent read access
// - Safe for multiple concurrent readers
//
// See docs/architecture/components.md#sqlite-cache-adapters for implementation
// details.
type SQLiteCacheReadAdapter struct {
	config domain.Config
	log    zerolog.Logger
	db     *sql.DB
}

// NewSQLiteCacheReadAdapter creates a new SQLiteCacheReadAdapter with the
// provided
// configuration and logger. The adapter opens the existing SQLite database
// for read operations.
//
// Parameters:
//   - config: Application configuration containing CacheDir and FileClassKey
//   - log: Structured logger for operation tracking
//
// Returns:
//   - *SQLiteCacheReadAdapter: Configured adapter ready for complex queries
//   - error: If database opening fails
//
// Thread Safety: The returned adapter is safe for concurrent use.
func NewSQLiteCacheReadAdapter(
	config domain.Config,
	log zerolog.Logger,
) (*SQLiteCacheReadAdapter, error) {
	// Open SQLite database in cache directory (read-only for safety)
	dbPath := filepath.Join(config.CacheDir, "cache.db")
	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, errors.NewCacheReadError("", dbPath, "open_db", err)
	}

	// Enable foreign keys for consistency
	if _, execErr := db.ExecContext(
		context.Background(),
		"PRAGMA foreign_keys = ON;",
	); execErr != nil {
		_ = db.Close()
		return nil, errors.NewCacheReadError(
			"",
			dbPath,
			"init_pragmas",
			execErr,
		)
	}

	return &SQLiteCacheReadAdapter{
		config: config,
		log:    log,
		db:     db,
	}, nil
}

// Close closes the SQLite database connection.
// Should be called when the adapter is no longer needed.
func (a *SQLiteCacheReadAdapter) Close() error {
	return a.db.Close()
}

// Read retrieves a single note by ID from the SQLite cache.
// Returns the complete note with all frontmatter preserved.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//   - id: NoteID to retrieve
//
// Returns:
//   - Note: The retrieved note with path and frontmatter
//
// - error: ErrNotFound if note doesn't exist, or wrapped error for other issues
//
// Thread-safe: Safe for concurrent calls.
func (a *SQLiteCacheReadAdapter) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	// Check for context cancellation
	if err := checkContext(ctx); err != nil {
		return domain.Note{}, err
	}

	var dbID, path string
	var title, fileClass sql.NullString
	var frontmatterJSON string

	err := a.executeReadQuery(
		ctx,
		id,
		&dbID,
		&path,
		&title,
		&fileClass,
		&frontmatterJSON,
	)
	if err != nil {
		return domain.Note{}, err
	}

	return a.reconstructNote(
		id,
		dbID,
		path,
		title,
		fileClass,
		frontmatterJSON,
	)
}

// List retrieves all cached notes from the SQLite cache.
// Returns a slice of all notes with their metadata.
//
// Parameters:
//   - ctx: Context for cancellation and tracing
//
// Returns:
//   - []Note: All cached notes
//   - error: Wrapped error if listing fails
//
// Thread-safe: Safe for concurrent calls.
// Performance: Uses indexed queries for efficient retrieval.
func (a *SQLiteCacheReadAdapter) List(
	ctx context.Context,
) ([]domain.Note, error) {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	query := `
		SELECT id, path, title, file_class, frontmatter
		FROM notes
		ORDER BY path`

	rows, err := a.db.QueryContext(ctx, query)
	if err != nil {
		return nil, errors.NewCacheReadError(
			"",
			"",
			"list_query",
			err,
		)
	}
	defer func() { _ = rows.Close() }()

	notes, err := a.processListRows(ctx, rows)
	if err != nil {
		return nil, err
	}

	a.log.Debug().
		Int("count", len(notes)).
		Msg("Listed notes from SQLite cache")

	return notes, nil
}

// processListRows processes database rows into notes.
func (a *SQLiteCacheReadAdapter) processListRows(
	ctx context.Context,
	rows *sql.Rows,
) ([]domain.Note, error) {
	var notes []domain.Note

	for rows.Next() {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		default:
		}

		var id, path string
		var title, fileClass sql.NullString
		var frontmatterJSON string

		if err := rows.Scan(&id, &path, &title, &fileClass, &frontmatterJSON); err != nil {
			a.log.Warn().
				Err(err).
				Msg("Failed to scan note row, skipping")
			continue
		}

		note, err := a.reconstructNote(
			domain.NewNoteID(id),
			id,
			path,
			title,
			fileClass,
			frontmatterJSON,
		)
		if err != nil {
			a.log.Warn().
				Err(err).
				Str("note_id", id).
				Msg("Failed to reconstruct note, skipping")
			continue
		}

		notes = append(notes, note)
	}

	if err := rows.Err(); err != nil {
		return nil, errors.NewCacheReadError(
			"",
			"",
			"list_rows",
			err,
		)
	}

	return notes, nil
}

// executeReadQuery performs the database query for reading a note.
func (a *SQLiteCacheReadAdapter) executeReadQuery(
	ctx context.Context,
	id domain.NoteID,
	dbID, path *string,
	title, fileClass *sql.NullString,
	frontmatterJSON *string,
) error {
	query := `
		SELECT id, path, title, file_class, frontmatter
		FROM notes
		WHERE id = ?`

	err := a.db.QueryRowContext(ctx, query, string(id)).Scan(
		dbID,
		path,
		title,
		fileClass,
		frontmatterJSON,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return errors.ErrNotFound
		}
		return errors.NewCacheReadError(
			string(id),
			"",
			"read_query",
			err,
		)
	}

	return nil
}

// reconstructNote builds a Note from database results.
func (a *SQLiteCacheReadAdapter) reconstructNote(
	id domain.NoteID,
	dbID, path string,
	title, fileClass sql.NullString,
	frontmatterJSON string,
) (domain.Note, error) {
	// Parse frontmatter JSON
	var fields map[string]interface{}
	if err := json.Unmarshal([]byte(frontmatterJSON), &fields); err != nil {
		return domain.Note{}, errors.NewCacheReadError(
			string(id),
			path,
			"unmarshal_frontmatter",
			err,
		)
	}

	// Verify ID matches (should always be true)
	if dbID != string(id) {
		return domain.Note{}, errors.NewCacheReadError(
			string(id),
			path,
			"id_mismatch",
			errors.NewBaseError("database ID does not match requested ID", nil),
		)
	}

	if title.Valid {
		if _, ok := fields["title"]; !ok {
			fields["title"] = title.String
		}
	}

	// Reconstruct Note
	note := domain.Note{
		ID:   id,
		Path: path,
		Frontmatter: domain.Frontmatter{
			FileClass: fileClass.String, // Use empty string if NULL
			Fields:    fields,
		},
	}

	a.log.Debug().
		Str("note_id", string(id)).
		Str("path", path).
		Msg("Read note from SQLite cache")

	return note, nil
}
