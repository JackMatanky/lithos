package sqlite

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

// SchemaViewMigrator manages schema-driven SQLite views by comparing schema
// signatures against the persisted metadata table and rebuilding views when
// definitions change.
type SchemaViewMigrator struct {
	fileClassKey string
	schemas      []domain.Schema
	log          zerolog.Logger
}

// NewSchemaViewMigrator creates a migrator with defensive copies of schemas to
// prevent accidental mutation.
func NewSchemaViewMigrator(
	schemas []domain.Schema,
	fileClassKey string,
	log zerolog.Logger,
) *SchemaViewMigrator {
	copied := make([]domain.Schema, len(schemas))
	for i, schema := range schemas {
		copied[i] = cloneDomainSchema(schema)
	}

	if fileClassKey == "" {
		fileClassKey = defaultFileClassKey
	}

	return &SchemaViewMigrator{
		fileClassKey: fileClassKey,
		schemas:      copied,
		log:          log,
	}
}

// EnsureViews idempotently creates or updates schema views inside the provided
// database connection.
func (m *SchemaViewMigrator) EnsureViews(
	ctx context.Context,
	db *sql.DB,
) error {
	if db == nil || len(m.schemas) == 0 {
		return nil
	}

	if err := setBusyTimeout(ctx, db); err != nil {
		return err
	}

	return m.withTx(ctx, db, func(tx *sql.Tx) error {
		return m.reconcileViews(ctx, tx)
	})
}

func (m *SchemaViewMigrator) withTx(
	ctx context.Context,
	db *sql.DB,
	fn func(*sql.Tx) error,
) error {
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("begin view migration tx: %w", err)
	}
	defer func() { _ = tx.Rollback() }()

	if txErr := fn(tx); txErr != nil {
		return txErr
	}

	if commitErr := tx.Commit(); commitErr != nil {
		return fmt.Errorf("commit view migration tx: %w", commitErr)
	}
	return nil
}

func (m *SchemaViewMigrator) ensureSchemaView(
	ctx context.Context,
	tx *sql.Tx,
	schema domain.Schema,
	stored map[string]string,
) error {
	signature, err := schemaSignature(schema)
	if err != nil {
		return err
	}

	view := schemaViewName(schema.Name)
	existingSignature := stored[schema.Name]
	needsRebuild := existingSignature != signature

	if !needsRebuild {
		exists, existsErr := viewExists(ctx, tx, view)
		if existsErr != nil {
			return existsErr
		}
		needsRebuild = !exists
	}

	if !needsRebuild {
		return nil
	}

	if dropErr := dropSchemaArtifacts(ctx, tx, schema.Name); dropErr != nil {
		return dropErr
	}

	sqlStatements, viewErr := GenerateSchemaViewWithOptions(
		schema,
		ViewGenerationOptions{FileClassKey: m.fileClassKey},
	)
	if viewErr != nil {
		return viewErr
	}
	if execErr := execStatements(ctx, tx, sqlStatements); execErr != nil {
		return execErr
	}

	if upsertErr := upsertSignature(ctx, tx, schema.Name, signature); upsertErr != nil {
		return upsertErr
	}

	m.log.Info().
		Str("schema", schema.Name).
		Msg("recreated SQLite schema view")
	return nil
}

func (m *SchemaViewMigrator) dropRemovedSchemas(
	ctx context.Context,
	tx *sql.Tx,
	stored map[string]string,
	active map[string]struct{},
) error {
	for schemaName := range stored {
		if _, ok := active[schemaName]; ok {
			continue
		}
		if err := dropSchemaArtifacts(ctx, tx, schemaName); err != nil {
			return err
		}
		if _, execErr := tx.ExecContext(
			ctx,
			"DELETE FROM schema_views WHERE schema_name = ?",
			schemaName,
		); execErr != nil {
			return fmt.Errorf("delete schema metadata: %w", execErr)
		}
		m.log.Info().
			Str("schema", schemaName).
			Msg("dropped SQLite schema view for removed schema")
	}
	return nil
}

func (m *SchemaViewMigrator) reconcileViews(
	ctx context.Context,
	tx *sql.Tx,
) error {
	if err := ensureMetadataTable(ctx, tx); err != nil {
		return err
	}

	storedSignatures, err := loadStoredSignatures(ctx, tx)
	if err != nil {
		return err
	}

	active := make(map[string]struct{}, len(m.schemas))
	for _, schema := range m.schemas {
		active[schema.Name] = struct{}{}
		if ensureErr := m.ensureSchemaView(ctx, tx, schema, storedSignatures); ensureErr != nil {
			return ensureErr
		}
	}

	return m.dropRemovedSchemas(ctx, tx, storedSignatures, active)
}

func ensureMetadataTable(ctx context.Context, tx *sql.Tx) error {
	_, err := tx.ExecContext(ctx, `
		CREATE TABLE IF NOT EXISTS schema_views (
			schema_name TEXT PRIMARY KEY,
			signature   TEXT NOT NULL,
			updated_at  INTEGER NOT NULL
		);
	`)
	if err != nil {
		return fmt.Errorf("create schema_views metadata table: %w", err)
	}
	return nil
}

func loadStoredSignatures(
	ctx context.Context,
	tx *sql.Tx,
) (map[string]string, error) {
	rows, queryErr := tx.QueryContext(
		ctx,
		"SELECT schema_name, signature FROM schema_views",
	)
	if queryErr != nil {
		return nil, fmt.Errorf("load schema view signatures: %w", queryErr)
	}
	defer func() { _ = rows.Close() }()

	signatures := map[string]string{}
	for rows.Next() {
		var name, signature string
		if scanErr := rows.Scan(&name, &signature); scanErr != nil {
			return nil, fmt.Errorf("scan schema signature: %w", scanErr)
		}
		signatures[name] = signature
	}
	if rowsErr := rows.Err(); rowsErr != nil {
		return nil, rowsErr
	}
	return signatures, nil
}

func setBusyTimeout(ctx context.Context, db *sql.DB) error {
	if _, err := db.ExecContext(ctx, "PRAGMA busy_timeout = 5000;"); err != nil {
		return fmt.Errorf("set busy timeout: %w", err)
	}
	return nil
}

func schemaSignature(schema domain.Schema) (string, error) {
	props := schema.Resolved
	if len(props) == 0 {
		props = schema.Properties
	}

	payload := struct {
		Name       string            `json:"name"`
		Properties []domain.Property `json:"properties"`
	}{
		Name:       schema.Name,
		Properties: append([]domain.Property(nil), props...),
	}
	data, err := json.Marshal(payload)
	if err != nil {
		return "", fmt.Errorf("marshal schema signature: %w", err)
	}
	sum := sha256.Sum256(data)
	return hex.EncodeToString(sum[:]), nil
}

func execStatements(
	ctx context.Context,
	tx *sql.Tx,
	sqlStatements string,
) error {
	for _, stmt := range splitStatements(sqlStatements) {
		if stmt == "" {
			continue
		}
		if _, err := tx.ExecContext(ctx, stmt); err != nil {
			return fmt.Errorf("execute statement %q: %w", stmt, err)
		}
	}
	return nil
}

func splitStatements(sqlStatements string) []string {
	lines := strings.Split(sqlStatements, "\n")
	results := make([]string, 0, len(lines))
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		results = append(results, line)
	}
	return results
}

func upsertSignature(
	ctx context.Context,
	tx *sql.Tx,
	schemaName string,
	signature string,
) error {
	_, err := tx.ExecContext(
		ctx,
		`
		INSERT INTO schema_views(schema_name, signature, updated_at)
		VALUES(?, ?, ?)
		ON CONFLICT(schema_name)
		DO UPDATE SET signature = excluded.signature, updated_at = excluded.updated_at
		`,
		schemaName,
		signature,
		time.Now().Unix(),
	)
	if err != nil {
		return fmt.Errorf("upsert schema signature: %w", err)
	}
	return nil
}

func dropSchemaArtifacts(
	ctx context.Context,
	tx *sql.Tx,
	schemaName string,
) error {
	if err := dropSchemaIndexes(ctx, tx, schemaName); err != nil {
		return err
	}
	view := schemaViewName(schemaName)
	if _, err := tx.ExecContext(
		ctx,
		fmt.Sprintf("DROP VIEW IF EXISTS %s", view),
	); err != nil {
		return fmt.Errorf("drop view %s: %w", view, err)
	}
	return nil
}

func dropSchemaIndexes(
	ctx context.Context,
	tx *sql.Tx,
	schemaName string,
) error {
	rows, err := tx.QueryContext(ctx, "PRAGMA index_list('notes')")
	if err != nil {
		return fmt.Errorf("list indexes: %w", err)
	}
	defer func() { _ = rows.Close() }()

	prefix := schemaIndexPrefix(schemaName)
	for rows.Next() {
		var seq int
		var name string
		var unique int
		var origin string
		var partial int
		if scanErr := rows.Scan(&seq, &name, &unique, &origin, &partial); scanErr != nil {
			return fmt.Errorf("scan index metadata: %w", scanErr)
		}
		if strings.HasPrefix(name, prefix) {
			if _, execErr := tx.ExecContext(
				ctx,
				fmt.Sprintf("DROP INDEX IF EXISTS %s", name),
			); execErr != nil {
				return fmt.Errorf("drop index %s: %w", name, execErr)
			}
		}
	}
	if rowsErr := rows.Err(); rowsErr != nil {
		return rowsErr
	}
	return nil
}

func viewExists(ctx context.Context, tx *sql.Tx, name string) (bool, error) {
	row := tx.QueryRowContext(
		ctx,
		"SELECT COUNT(1) FROM sqlite_master WHERE type = 'view' AND name = ?",
		name,
	)
	var count int
	if err := row.Scan(&count); err != nil {
		return false, fmt.Errorf("check view existence: %w", err)
	}
	return count > 0, nil
}

func cloneDomainSchema(src domain.Schema) domain.Schema {
	dst := src
	if len(src.Properties) > 0 {
		dst.Properties = make([]domain.Property, len(src.Properties))
		copy(dst.Properties, src.Properties)
	}
	if len(src.Resolved) > 0 {
		dst.Resolved = make(
			[]domain.Property,
			len(src.Resolved),
		)
		copy(dst.Resolved, src.Resolved)
	}
	if len(src.Excludes) > 0 {
		dst.Excludes = append([]string(nil), src.Excludes...)
	}
	return dst
}
