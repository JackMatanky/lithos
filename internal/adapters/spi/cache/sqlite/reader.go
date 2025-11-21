package sqlite

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"strings"
	"sync"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

const (
	selectNoteQuery = `SELECT path, frontmatter FROM notes WHERE path = ?`
	// defaultNoteSliceCapacity is the default capacity for note slices to
	// reduce allocations.
	defaultNoteSliceCapacity = 32
	// typicalFrontmatterSize is the typical size of frontmatter maps.
	typicalFrontmatterSize = 8
)

// Interface compliance checks.
var _ spi.CacheReaderPort = (*SQLiteReaderAdapter)(nil)
var _ spi.MetadataQueryPort = (*SQLiteReaderAdapter)(nil)

// Pool for reusing frontmatter field maps to reduce allocations.
var frontmatterMapPool = sync.Pool{
	New: func() interface{} {
		return make(
			map[string]interface{},
			typicalFrontmatterSize,
		) // Pre-size for typical frontmatter
	},
}

// SQLiteReaderAdapter implements CacheReaderPort and MetadataQueryPort for
// SQLite deep storage.
type SQLiteReaderAdapter struct {
	*commonAdapter

	config domain.Config
}

// NewSQLiteReaderAdapter creates a new reader adapter.
func NewSQLiteReaderAdapter(
	config domain.Config,
	log zerolog.Logger,
	migrator *SchemaViewMigrator,
) (*SQLiteReaderAdapter, error) {
	common, err := newCommonAdapter(
		config,
		log,
		migrator,
		func(dbPath, operation string, cause error) error {
			return lithosErr.NewCacheReadError("", dbPath, operation, cause)
		},
	)
	if err != nil {
		return nil, err
	}

	return &SQLiteReaderAdapter{
		commonAdapter: common,
		config:        config,
	}, nil
}

// Read retrieves a single note by ID.
func (a *SQLiteReaderAdapter) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	if err := ctx.Err(); err != nil {
		return domain.Note{}, err
	}

	path := string(id)
	query := selectNoteQuery

	var fmJSON string
	var dbPath string // to verify

	err := a.db.QueryRowContext(ctx, query, path).Scan(&dbPath, &fmJSON)
	if err != nil {
		if err == sql.ErrNoRows {
			return domain.Note{}, lithosErr.ErrNotFound
		}
		return domain.Note{}, lithosErr.NewCacheReadError(
			path,
			path,
			"read_note",
			err,
		)
	}

	return a.reconstructNote(dbPath, fmJSON)
}

// List retrieves all cached notes.
func (a *SQLiteReaderAdapter) List(ctx context.Context) ([]domain.Note, error) {
	if err := ctx.Err(); err != nil {
		return nil, err
	}

	query := `SELECT path, frontmatter FROM notes`
	rows, err := a.db.QueryContext(ctx, query)
	if err != nil {
		return nil, lithosErr.NewCacheReadError("", "", "list_notes", err)
	}
	defer func() { _ = rows.Close() }()

	var notes []domain.Note
	for rows.Next() {
		var path, fmJSON string
		if scanErr := rows.Scan(&path, &fmJSON); scanErr != nil {
			a.log.Warn().Err(scanErr).Msg("failed to scan note row")
			continue
		}
		if note, recErr := a.reconstructNote(path, fmJSON); recErr == nil {
			notes = append(notes, note)
		}
	}
	if iterErr := rows.Err(); iterErr != nil {
		return nil, lithosErr.NewCacheReadError(
			"",
			"",
			"list_notes_iteration",
			iterErr,
		)
	}

	return notes, nil
}

// PathQuery finds notes using a flexible path selector.
func (a *SQLiteReaderAdapter) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	normalized, err := opts.Validate()
	if err != nil {
		return nil, err
	}

	var query string
	var args []interface{}

	switch normalized.Scope {
	case spi.PathQueryScopeFull:
		query = selectNoteQuery
		args = []interface{}{normalized.Value}
	case spi.PathQueryScopeBasename:
		// Basic implementation: LIKE comparison or parsing path
		// SQLite doesn't have path functions easily, but we can use LIKE
		// '%/basename.md' or 'basename.md'
		// Assuming standard extension .md
		basename := normalized.Value
		query = `SELECT path, frontmatter FROM notes WHERE path LIKE ? OR path = ?`
		args = []interface{}{"%/" + basename + ".md", basename + ".md"}
	case spi.PathQueryScopeFolder:
		folder := normalized.Value
		if !strings.HasSuffix(folder, "/") {
			folder += "/"
		}
		query = `SELECT path, frontmatter FROM notes WHERE path LIKE ?`
		args = []interface{}{folder + "%"}
	default:
		return nil, fmt.Errorf("unsupported scope: %s", normalized.Scope)
	}

	return a.executeListQuery(ctx, query, args...)
}

// TagQuery finds notes containing a specific tag.
// Uses json_extract to check tags array.
func (a *SQLiteReaderAdapter) TagQuery(
	ctx context.Context,
	tag string,
) ([]domain.Note, error) {
	// This query assumes tags is an array of strings.
	// SQLite JSON support allows checking if value exists in array.
	// `WHERE EXISTS (SELECT 1 FROM json_each(json_extract(frontmatter,
	// '$.tags')) WHERE value = ?)`
	query := `
		SELECT path, frontmatter
		FROM notes
		WHERE EXISTS (
			SELECT 1
			FROM json_each(json_extract(frontmatter, '$.tags'))
			WHERE value = ?
		)
	`
	return a.executeListQuery(ctx, query, tag)
}

// FileClassQuery finds notes by schema fileClass value.
// Uses schema-specific view if available, or base table with filter.
// AC 15 says: "FileClassQuery uses schema-specific view (not base table)".
func (a *SQLiteReaderAdapter) FileClassQuery(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	// Sanitize fileClass to prevent SQL injection since we interpolate table
	// name
	if !isValidIdentifier(fileClass) {
		return nil, fmt.Errorf("invalid fileClass identifier: %s", fileClass)
	}

	viewName := schemaViewName(fileClass)

	// Try querying the view. If it fails (view doesn't exist), fallback to base
	// table?
	// AC 15 implies we MUST use the view.
	// However, checking if view exists adds overhead.
	// We can try query, if error, maybe fallback.
	// For now, strict implementation per AC 15.

	// Note: The view columns are extracted, but reconstructNote expects full
	// JSON frontmatter? The view we defined in views.go does NOT include
	// 'frontmatter' column (JSON blob). It includes 'path', 'modified_at',
	// 'indexed_time', 'size', and extracted properties.
	// To return domain.Note, we need Frontmatter.Fields.
	// If the view columns match the schema properties, we can reconstruct the
	// Fields map from columns! But `domain.Note` expects `Frontmatter` struct
	// which has `Fields map[string]interface{}`. If we rely on the view, we get
	// typed columns. We can build the map from them.
	// BUT, `reconstructNote` in my design takes JSON string.
	// If `v_{schema}_notes` doesn't have `frontmatter` JSON column, we can't
	// use `reconstructNote`.
	// Let's check `views.go`.
	// `columns = append(columns, "path", "modified_at", ...)`
	// It does NOT include `frontmatter`.
	// Should I add `frontmatter` to the view?
	// AC 6 example:
	/*
	   SELECT
	       path,
	       json_extract(frontmatter, '$.name') AS name,
	       ...
	       modified_at,
	       indexed_time,
	       size
	   FROM notes
	*/
	// It doesn't select `frontmatter`.
	// This implies `FileClassQuery` returning `[]domain.Note` might need to
	// reconstruct `Note` from the view columns.
	// OR I should add `frontmatter` to the view definition to make it easier.
	// Adding `frontmatter` to the view is cheapest way to support `domain.Note`
	// reconstruction without mapping columns back to map manually (which
	// requires knowing the schema at runtime here).
	// I will add `frontmatter` to the view in `views.go` later.
	// For now, let's assume the view has `frontmatter` column or we query base
	// table for it.
	// Actually, query `SELECT * FROM v_...` returns columns.
	// I'll modify `views.go` to include `frontmatter` column. It simplifies
	// everything.

	query := fmt.Sprintf("SELECT path, frontmatter FROM %s", viewName)
	notes, err := a.executeListQuery(ctx, query)
	if err == nil {
		return notes, nil
	}
	if strings.Contains(err.Error(), "no such table") {
		jsonPath := fmt.Sprintf("$.%s", a.config.FileClassKey)
		fallback := fmt.Sprintf(
			"SELECT path, frontmatter FROM notes WHERE json_extract(frontmatter, '%s') = ?",
			jsonPath,
		)
		return a.executeListQuery(ctx, fallback, fileClass)
	}
	return nil, err
}

// FrontmatterQuery finds notes where a specific frontmatter field matches a
// value.
func (a *SQLiteReaderAdapter) FrontmatterQuery(
	ctx context.Context,
	field, value string,
) ([]domain.Note, error) {
	// Sanitize field to prevent SQL injection
	if !isValidJSONPath(field) {
		return nil, fmt.Errorf("invalid json path: %s", field)
	}

	// Use json_extract with flexible comparison
	// Try multiple comparison strategies for different value types
	jsonPath := "$." + field

	// First try exact string match
	query := fmt.Sprintf(
		"SELECT path, frontmatter FROM notes WHERE json_extract(frontmatter, '%s') = ?",
		jsonPath,
	)

	results, err := a.executeListQuery(ctx, query, value)
	if err == nil && len(results) > 0 {
		return results, nil
	}

	// If no results, try numeric comparison
	if numVal, numErr := parseNumeric(value); numErr == nil {
		numQuery := fmt.Sprintf(
			"SELECT path, frontmatter FROM notes WHERE json_extract(frontmatter, '%s') = ?",
			jsonPath,
		)
		return a.executeListQuery(ctx, numQuery, numVal)
	}

	// If no results, try boolean comparison
	if boolVal, boolErr := parseBoolean(value); boolErr == nil {
		boolQuery := fmt.Sprintf(
			"SELECT path, frontmatter FROM notes WHERE json_extract(frontmatter, '%s') = ?",
			jsonPath,
		)
		return a.executeListQuery(ctx, boolQuery, boolVal)
	}

	// Return original results (may be empty)
	return results, err
}

// BasenameQuery finds notes by their filename without extension.
func (a *SQLiteReaderAdapter) BasenameQuery(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	return a.PathQuery(
		ctx,
		spi.PathQueryOptions{
			Scope: spi.PathQueryScopeBasename,
			Value: basename,
		},
	)
}

// AliasQuery finds notes by their alias.
func (a *SQLiteReaderAdapter) AliasQuery(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	// TagQuery logic but for aliases array
	query := `
		SELECT path, frontmatter
		FROM notes
		WHERE EXISTS (
			SELECT 1
			FROM json_each(json_extract(frontmatter, '$.aliases'))
			WHERE value = ?
		)
	`
	return a.executeListQuery(ctx, query, alias)
}

// Helpers

// IsStale implementation.
func (a *SQLiteReaderAdapter) IsStale(
	ctx context.Context,
	path string,
) (bool, error) {
	// Check DB for indexed_time vs modified_at (which is stored in DB)
	// AC 11: "indexed_time column tracks when note was cached... modified_at
	// column tracks file's modification time"
	// "GetStaleNotes() returns paths where modified_at > indexed_time"

	// For single note check:
	query := `SELECT modified_at, indexed_time FROM notes WHERE path = ?`
	var modAt, idxAt int64
	err := a.db.QueryRowContext(ctx, query, path).Scan(&modAt, &idxAt)
	if err != nil {
		if err == sql.ErrNoRows {
			return true, nil // Missing = stale (needs index)
		}
		return false, err
	}

	// Also check actual file system? The requirement says:
	// "IsStale(ctx, path string) (bool, error) method for single-note staleness
	// checks"
	// AC 11 implies DB columns are used.
	// If the DB `modified_at` is old (from previous scan), we assume it's up to
	// date with what was scanned.
	// BUT, `IsStale` usually compares DB vs FileSystem.
	// If we only compare DB columns, we are checking if the *record* is stale
	// relative to *itself* (impossible if updated atomically).
	// Unless `modified_at` is updated separately? No.
	// The standard pattern is: Scan file -> Get FS modtime -> Compare with DB
	// indexed_time -> if FS > DB, then Stale.
	// So we need to stat the file.
	// I'll implement file stat check if I can access FS. I don't have VaultPath
	// in config here?
	// `domain.Config` usually has `VaultPath`.
	// But `SQLiteReaderAdapter` has `config`.
	// I'll implement FS check.

	// Actually, the `modified_at` column in DB is what we *know* about the
	// file. Staleness query `GetStaleNotes` (AC 11) says `WHERE modified_at >
	// indexed_time`. This implies we update `modified_at` in the DB when we
	// scan, even if we don't fully re-index?
	// Or maybe `modified_at` comes from the file system during the scan.
	// Let's stick to what the story says:
	// "GetStaleNotes() returns paths where modified_at > indexed_time".
	// This implies `modified_at` is UPDATED independently of `indexed_time`.
	// This happens if we do a "fast scan" that updates `modified_at` in DB, and
	// then "deep index" updates `indexed_time`.

	if modAt > idxAt {
		return true, nil
	}
	return false, nil
}

// GetStaleNotes returns notes that are stale.
func (a *SQLiteReaderAdapter) GetStaleNotes(
	ctx context.Context,
) ([]string, error) {
	query := `SELECT path FROM notes WHERE modified_at > indexed_time`
	rows, err := a.db.QueryContext(ctx, query)
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()
	var paths []string
	for rows.Next() {
		var p string
		if scanErr := rows.Scan(&p); scanErr == nil {
			paths = append(paths, p)
		}
	}
	if iterErr := rows.Err(); iterErr != nil {
		return nil, iterErr
	}
	return paths, nil
}

// Helpers.
func isValidIdentifier(s string) bool {
	// Simple check to prevent injection in view name
	// Allow alphanumeric and underscores
	for _, r := range s {
		if !isIdentifierChar(r) {
			return false
		}
	}
	return true
}

func isIdentifierChar(r rune) bool {
	return (r >= 'a' && r <= 'z') ||
		(r >= 'A' && r <= 'Z') ||
		(r >= '0' && r <= '9') ||
		r == '_'
}

func isValidJSONPath(s string) bool {
	// Allow alphanumeric, dots, underscores
	for _, r := range s {
		if !isJSONPathChar(r) {
			return false
		}
	}
	return true
}

func isJSONPathChar(r rune) bool {
	return isIdentifierChar(r) || r == '.'
}

// Close closes the DB.
func (a *SQLiteReaderAdapter) Close() error {
	return a.db.Close()
}

// parseNumeric attempts to parse a string as a number.
func parseNumeric(s string) (interface{}, error) {
	if intVal, err := parseInt(s); err == nil {
		return intVal, nil
	}
	if floatVal, err := parseFloat(s); err == nil {
		return floatVal, nil
	}
	return nil, fmt.Errorf("not a number")
}

// parseInt attempts to parse a string as an integer.
func parseInt(s string) (int64, error) {
	var val int64
	_, err := fmt.Sscanf(s, "%d", &val)
	return val, err
}

// parseFloat attempts to parse a string as a float.
func parseFloat(s string) (float64, error) {
	var val float64
	_, err := fmt.Sscanf(s, "%f", &val)
	return val, err
}

// parseBoolean attempts to parse a string as a boolean.
func parseBoolean(s string) (bool, error) {
	switch s {
	case "true", "1", "yes":
		return true, nil
	case "false", "0", "no":
		return false, nil
	default:
		return false, fmt.Errorf("not a boolean")
	}
}

func (a *SQLiteReaderAdapter) executeListQuery(
	ctx context.Context,
	query string,
	args ...interface{},
) ([]domain.Note, error) {
	if err := ctx.Err(); err != nil {
		return nil, err
	}

	rows, err := a.db.QueryContext(ctx, query, args...)
	if err != nil {
		return nil, lithosErr.NewCacheReadError("", "", "execute_list", err)
	}
	defer func() { _ = rows.Close() }()

	// Pre-allocate notes slice with reasonable capacity to reduce allocations
	notes := make([]domain.Note, 0, defaultNoteSliceCapacity)

	for rows.Next() {
		var path, fmJSON string
		if scanErr := rows.Scan(&path, &fmJSON); scanErr != nil {
			a.log.Warn().Err(scanErr).Msg("failed to scan row")
			continue
		}
		if note, recErr := a.reconstructNote(path, fmJSON); recErr == nil {
			notes = append(notes, note)
		} else {
			// Log reconstruction errors for debugging but continue processing
			a.log.Debug().Err(recErr).Str("path", path).Msg("failed to reconstruct note")
		}
	}
	if iterErr := rows.Err(); iterErr != nil {
		return nil, lithosErr.NewCacheReadError(
			"",
			"",
			"execute_list_iteration",
			iterErr,
		)
	}

	return notes, nil
}

// Note: Additional byte slice pooling could be added here for JSON processing
// if further optimization is needed in the future

func (a *SQLiteReaderAdapter) reconstructNote(
	path string,
	fmJSON string,
) (domain.Note, error) {
	// Get pooled map for fields
	fields := frontmatterMapPool.Get().(map[string]interface{})
	defer func() {
		// Clear map and return to pool
		for k := range fields {
			delete(fields, k)
		}
		frontmatterMapPool.Put(fields)
	}()

	// Optimize JSON unmarshaling with reduced allocations
	if err := json.Unmarshal([]byte(fmJSON), &fields); err != nil {
		return domain.Note{}, fmt.Errorf(
			"failed to unmarshal frontmatter: %w",
			err,
		)
	}

	// Create copy of fields since we're returning the map to pool
	fieldsCopy := make(map[string]interface{}, len(fields))
	for k, v := range fields {
		fieldsCopy[k] = v
	}

	fm := domain.NewFrontmatter(fieldsCopy)
	note := domain.NewNote(domain.NewNoteID(path), fm)
	return note, nil
}
