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

	path := note.Path
	payload, modTime, size, err := a.preparePersistData(note)
	if err != nil {
		return err
	}

	return a.executePersist(ctx, path, payload, modTime, indexTime, size)
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
	path string,
) error {
	if err := ctx.Err(); err != nil {
		return err
	}

	_, err := a.db.ExecContext(ctx, "DELETE FROM notes WHERE path = ?", path)
	if err != nil {
		return lithosErr.NewCacheDeleteError(
			path,
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
) (payload string, modTime time.Time, fileSize int64, err error) {
	fields := ensureFileClassField(
		note.Frontmatter.Fields,
		note.Frontmatter.FileClass(),
		a.config.FileClassKey,
	)
	bytes, err := json.Marshal(fields)
	if err != nil {
		return "", time.Time{}, 0, lithosErr.NewCacheWriteError(
			note.Path,
			note.Path,
			"marshal_frontmatter",
			err,
		)
	}

	payload = string(bytes)
	modTime = cache.ExtractFileModTime(fields)
	fileSize = extractFileSize(fields)
	return
}

func (a *SQLiteWriterAdapter) executePersist(
	ctx context.Context,
	path string,
	payload string,
	modTime time.Time,
	indexTime time.Time,
	size int64,
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
		size,
	); err != nil {
		return lithosErr.NewCacheWriteError(path, path, "insert_note", err)
	}

	if err = tx.Commit(); err != nil {
		return lithosErr.NewCacheWriteError(path, path, "commit_tx", err)
	}
	return nil
}

func extractFileSize(fields map[string]interface{}) int64 {
	if fields == nil {
		return 0
	}
	if size, ok := fields["file_size"]; ok {
		switch v := size.(type) {
		case int64:
			return v
		case int:
			return int64(v)
		case float64:
			return int64(v)
		}
	}
	if size, ok := fields["size"]; ok {
		switch v := size.(type) {
		case int64:
			return v
		case int:
			return int64(v)
		case float64:
			return int64(v)
		}
	}
	return 0
}
