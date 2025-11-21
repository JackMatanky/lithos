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
	migrator *SchemaViewMigrator,
) (*SQLiteWriterAdapter, error) {
	common, err := newCommonAdapter(
		config,
		log,
		migrator,
		func(dbPath, operation string, cause error) error {
			return lithosErr.NewCacheWriteError("", dbPath, operation, cause)
		},
	)
	if err != nil {
		return nil, err
	}

	return &SQLiteWriterAdapter{
		commonAdapter: common,
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
	payload, modTime, err := a.preparePersistData(note)
	if err != nil {
		return err
	}

	return a.executePersist(ctx, path, payload, modTime, indexTime)
}

func ensureFileClassField(
	fields map[string]interface{},
	fileClass string,
	fileClassKey string,
) map[string]interface{} {
	if fields == nil {
		fields = map[string]interface{}{}
	}
	if fileClass == "" || fileClassKey == "" {
		return fields
	}
	if existing, ok := fields[fileClassKey]; ok && existing != "" {
		return fields
	}
	copied := make(map[string]interface{}, len(fields)+1)
	for k, v := range fields {
		copied[k] = v
	}
	copied[fileClassKey] = fileClass
	return copied
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

func (a *SQLiteWriterAdapter) preparePersistData(
	note domain.Note,
) (string, time.Time, error) {
	fields := ensureFileClassField(
		note.Frontmatter.Fields,
		note.Frontmatter.FileClass,
		a.config.FileClassKey,
	)
	bytes, err := json.Marshal(fields)
	if err != nil {
		return "", time.Time{}, lithosErr.NewCacheWriteError(
			string(note.ID),
			string(note.ID),
			"marshal_frontmatter",
			err,
		)
	}

	return string(bytes), cache.ExtractFileModTime(fields), nil
}

func (a *SQLiteWriterAdapter) executePersist(
	ctx context.Context,
	path string,
	payload string,
	modTime time.Time,
	indexTime time.Time,
) error {
	tx, err := a.db.BeginTx(ctx, nil)
	if err != nil {
		return lithosErr.NewCacheWriteError(
			path,
			path,
			"begin_tx",
			err,
		)
	}
	defer func() { _ = tx.Rollback() }()

	query := `INSERT OR REPLACE INTO notes (path, frontmatter, modified_at, indexed_time, size) ` +
		`VALUES (?, ?, ?, ?, ?)`
	if _, err = tx.ExecContext(
		ctx,
		query,
		path,
		payload,
		toUnix(modTime),
		toUnix(indexTime),
		int64(0),
	); err != nil {
		return lithosErr.NewCacheWriteError(path, path, "insert_note", err)
	}

	if err = tx.Commit(); err != nil {
		return lithosErr.NewCacheWriteError(path, path, "commit_tx", err)
	}
	return nil
}
