package sqlite

import (
	"context"
	"encoding/json"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// Interface compliance checks.
var _ spi.CacheWriterPort = (*SQLiteWriterAdapter)(nil)

// SQLiteWriterAdapter implements CacheWriterPort for SQLite deep storage.
type SQLiteWriterAdapter struct {
	*commonAdapter

	config domain.Config
}

// NewSQLiteWriterAdapter creates a new writer adapter.
func NewSQLiteWriterAdapter(
	config domain.Config,
	log zerolog.Logger,
) (*SQLiteWriterAdapter, error) {
	dbPath := config.CacheDir + "/cold.db" // Using cold.db to match Tech Stack

	db, err := InitializeDatabase(dbPath)
	if err != nil {
		return nil, lithosErr.NewCacheWriteError("", dbPath, "init_db", err)
	}

	return &SQLiteWriterAdapter{
		commonAdapter: &commonAdapter{db: db, log: log},
		config:        config,
	}, nil
}

// Persist writes the note to the notes table.
func (a *SQLiteWriterAdapter) Persist(
	ctx context.Context,
	note domain.Note,
	indexTime time.Time,
) error {
	if err := ctx.Err(); err != nil {
		return err
	}

	path := string(note.ID)

	// Serialize frontmatter
	fmBytes, err := json.Marshal(note.Frontmatter.Fields)
	if err != nil {
		return lithosErr.NewCacheWriteError(
			string(note.ID),
			path,
			"marshal_frontmatter",
			err,
		)
	}

	// Extract timestamps
	modTime := cache.ExtractFileModTime(note.Frontmatter.Fields)

	tx, err := a.db.BeginTx(ctx, nil)
	if err != nil {
		return lithosErr.NewCacheWriteError(
			string(note.ID),
			path,
			"begin_tx",
			err,
		)
	}
	defer func() { _ = tx.Rollback() }()

	query := `INSERT OR REPLACE INTO notes (path, frontmatter, modified_at, indexed_time, size) ` +
		`VALUES (?, ?, ?, ?, ?)`

	size := int64(
		0,
	) // Size not available in domain.Note, assume 0 for metadata cache

	_, err = tx.ExecContext(ctx, query,
		path,
		string(fmBytes),
		toUnix(modTime),
		toUnix(indexTime),
		size,
	)
	if err != nil {
		return lithosErr.NewCacheWriteError(
			string(note.ID),
			path,
			"insert_note",
			err,
		)
	}

	if err = tx.Commit(); err != nil {
		return lithosErr.NewCacheWriteError(
			string(note.ID),
			path,
			"commit_tx",
			err,
		)
	}

	return nil
}

// Delete removes the note.
func (a *SQLiteWriterAdapter) Delete(
	ctx context.Context,
	id domain.NoteID,
) error {
	if err := ctx.Err(); err != nil {
		return err
	}

	path := string(id)

	_, err := a.db.ExecContext(ctx, "DELETE FROM notes WHERE path = ?", path)
	if err != nil {
		return lithosErr.NewCacheDeleteError(
			string(id),
			path,
			"delete_note",
			err,
		)
	}
	return nil
}

// Close closes the DB.
func (a *SQLiteWriterAdapter) Close() error {
	return a.db.Close()
}
