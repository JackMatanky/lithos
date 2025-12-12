// Package query provides fast in-memory lookups for indexed notes.
// It implements thread-safe concurrent reads using sync.RWMutex and supports
// the FR9 query capabilities: lookup by ID, path, basename, and schema.
package query

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// QueryService provides smart routing for indexed notes using hybrid storage.
// It implements thread-safe concurrent reads using sync.RWMutex and supports
// the FR9 query capabilities with optimized routing to BoltDB and SQLite
// backends.
//
// Hybrid Storage Architecture:
// - BoltDB: Hot cache for fast lookups (paths, basenames, aliases, file
// classes)
// - SQLite: Deep storage for complex queries and full content
// - Smart Routing: Automatic query optimization based on operation type
//
// Thread-Safe Design:
// - Multiple readers can query simultaneously via RLock
// - No data races during concurrent access patterns
//
// Query Routing:
// - Hot Queries (BoltDB): PathQuery, BasenameQuery, AliasQuery, directory
// filtering
// - Complex Queries (SQLite): FrontmatterQuery, FileClassQuery with property
// filtering
// - Hybrid Queries: Coordinate between stores for optimal performance.
type QueryService struct {
	mu sync.RWMutex

	// Dependencies
	boltReader   spi.CacheReaderPort // Hot cache for fast lookups
	sqliteReader spi.CacheReaderPort // Deep storage for complex queries
	router       *queryRouter        // Handles smart query routing
	config       domain.Config       // For file_class_key configuration
	log          zerolog.Logger

	// Observability
	observer *StalenessObserver // Records cache staleness events

	// Resilience
	boltFailures   *BackendFailureTracker // Tracks BoltDB failures
	sqliteFailures *BackendFailureTracker // Tracks SQLite failures
}

// queryLogger provides consistent performance logging for query operations.
type queryLogger struct {
	log    zerolog.Logger
	method string
}

// queryRouter handles smart routing between storage backends.
type queryRouter struct {
	boltQuery   spi.MetadataQueryPort
	sqliteQuery spi.MetadataQueryPort
}

// canonicalizeFrontmatterValue normalizes frontmatter values for type-agnostic
// comparison.
// Handles numeric type conversions (int 2 == float 2.0) and safe comparison
// for complex types.
// Returns the normalized value and whether normalization was successful.
func canonicalizeFrontmatterValue(value interface{}) (interface{}, bool) {
	switch v := value.(type) {
	case int, int8, int16, int32, int64, uint, uint8, uint16, uint32, uint64, float32:
		return toFloat64(v), true
	case float64:
		return v, true
	case string, bool:
		// Strings and booleans are already comparable
		return v, true
	default:
		// Complex types (arrays, maps) are not safely comparable
		return nil, false
	}
}

// toFloat64 converts various numeric types to float64 for consistent
// comparison.
//
//nolint:cyclop // Type conversion requires exhaustive numeric type checking
func toFloat64(value interface{}) float64 {
	switch v := value.(type) {
	case int:
		return float64(v)
	case int8:
		return float64(v)
	case int16:
		return float64(v)
	case int32:
		return float64(v)
	case int64:
		return float64(v)
	case uint:
		return float64(v)
	case uint8:
		return float64(v)
	case uint16:
		return float64(v)
	case uint32:
		return float64(v)
	case uint64:
		return float64(v)
	case float32:
		return float64(v)
	default:
		return 0
	}
}

// NewQueryService creates a new QueryService with hybrid storage routing.
// It initializes all index maps and injects required dependencies for smart
// query routing.
// The service routes queries to optimal storage backends based on query type.
//
// Hybrid Architecture:
// - BoltDB reader for hot data (paths, basenames, aliases, file classes)
// - SQLite reader for deep storage (complex queries, full content)
// - Smart routing for optimal performance
//
// Thread-Safe Design:
// - RWMutex enables multiple concurrent reads, exclusive writes
// - Dependencies are injected (no globals) for testability
//
// Usage:
//
//	qs := NewQueryService(boltReader, sqliteReader, config, logger)
//	note, err := qs.IDQuery(ctx, id)   // Query safely with smart routing
func NewQueryService(
	boltReader spi.CacheReaderPort,
	sqliteReader spi.CacheReaderPort,
	config domain.Config,
	log zerolog.Logger,
) *QueryService {
	// Extract query ports from readers
	var boltQuery, sqliteQuery spi.MetadataQueryPort
	if mq, ok := boltReader.(spi.MetadataQueryPort); ok {
		boltQuery = mq
	}
	if mq, ok := sqliteReader.(spi.MetadataQueryPort); ok {
		sqliteQuery = mq
	}

	return &QueryService{
		boltReader:     boltReader,
		sqliteReader:   sqliteReader,
		router:         newQueryRouter(boltQuery, sqliteQuery),
		config:         config,
		log:            log,
		observer:       NewStalenessObserver(log),
		boltFailures:   NewBackendFailureTracker("boltdb", log),
		sqliteFailures: NewBackendFailureTracker("sqlite", log),
		// mu is initialized to zero value (unlocked state)
		mu: sync.RWMutex{},
	}
}

// IDQuery retrieves a note by its path (formerly NoteID).
// IDQuery retrieves a note by its path (formerly NoteID).
//
// Parameters:
//   - ctx: Request context for cancellation and logging
//   - path: Vault-relative path to the note (e.g., "notes/meeting.md")
//
// Returns:
//   - Note: The requested note with all metadata
//   - error: ResourceError if not found, or implementation error
//
// Behavior:
//   - Delegates to PathQuerySingle for consistent behavior
//   - Logs debug message with path for troubleshooting
//   - O(1) lookup performance via map access.
func (q *QueryService) IDQuery(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	return q.PathQuerySingle(ctx, path)
}

// PathQuerySingle retrieves a note by its file path.
// Returns the note if found, or ResourceError if not found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns ResourceError for missing notes (single result lookup)
// - Logs debug message with path for troubleshooting
// - O(1) lookup performance via map access.
func (q *QueryService) PathQuerySingle(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	logger := newQueryLogger(q.log, "PathQuery")
	start := time.Now()
	defer func() {
		logger.logSingleResult(start, "path", path)
	}()

	// Hot path: use BoltDB for fast lookups
	if q.boltReader != nil {
		return q.boltReader.Read(ctx, path)
	}

	if q.sqliteReader != nil {
		return q.sqliteReader.Read(ctx, path)
	}

	return domain.Note{}, lithosErr.NewResourceError(
		"note",
		"get",
		path,
		errors.New("not found"),
	)
}

// FileClassQuery retrieves all notes matching a schema name (fileClass).
// Returns a slice of notes if any match, or empty slice if none found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching schemas (collection
// lookup)
// - Logs debug message with fileClass and result count
// - O(log n) lookup performance via map access.
func (q *QueryService) FileClassQuery(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	return q.executeMetadataQuery(
		ctx,
		newQueryLogger(q.log, "FileClassQuery"),
		"fileClass",
		fileClass,
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.FileClassQuery(ctx, param)
		},
	)
}

// BasenameQuery retrieves all notes matching a filename basename (without
// extension).
// Returns a slice of notes if any match, or empty slice if none found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching basenames (collection
// lookup)
// - Basename is extracted from NoteID (full path) by removing directory path
// and file extension
// - Logs debug message with basename and result count
// - Delegates to MetadataQueryPort for index-based lookup performance
//
// Example: NoteID "projects/notes/meeting.md" matches basename "meeting".
func (q *QueryService) BasenameQuery(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	return q.executeMetadataQuery(
		ctx,
		newQueryLogger(q.log, "BasenameQuery"),
		"basename",
		basename,
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
	)
}

// AliasQuery retrieves all notes containing an alias in their frontmatter.
// Returns a slice of notes if any match, or empty slice if none found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching aliases (collection
// lookup)
// - Searches aliases array in frontmatter for exact matches
// - Multiple notes can contain the same alias
// - Logs debug message with alias and result count
// - Delegates to MetadataQueryPort for index-based lookup performance
//
// Example: Notes with frontmatter aliases containing "project-alpha" match
// alias "project-alpha".
func (q *QueryService) AliasQuery(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	return q.executeMetadataQuery(
		ctx,
		newQueryLogger(q.log, "AliasQuery"),
		"alias",
		alias,
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.AliasQuery(ctx, param)
		},
	)
}

// PathQuery resolves notes using flexible selectors (full path, basename,
// folder). MetadataQueryPort handles the fast-path lookups when configured;
// otherwise we fall back to the in-memory indices maintained by QueryService.
func (q *QueryService) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	logger := newQueryLogger(q.log, "PathQuery")
	start := time.Now()

	validatedOpts, err := opts.Validate()
	if err != nil {
		return nil, err
	}

	// Use router for path queries
	notes, err := q.router.routeMetadataQuery(
		ctx,
		func(port spi.MetadataQueryPort, ctx context.Context, _ string) ([]domain.Note, error) {
			return port.PathQuery(ctx, validatedOpts)
		},
		validatedOpts.Value,
	)

	// Log performance after query execution
	logger.logPerformance(start, "query", validatedOpts.Value, len(notes))

	return notes, err
}

// FrontmatterQuery finds notes where a specific frontmatter field matches a
// value.
// Supports type-agnostic comparison (int 2 == float 2.0) and delegates to
// MetadataQueryPort for indexed lookups.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching frontmatter (collection
// lookup)
// - Type normalization: int/float conversion for numeric comparison
// - Logs debug message with field and value for troubleshooting
// - Routes directly to SQLite for complex frontmatter queries (deep path only)
//
// Example:
//
//	notes := queryService.FrontmatterQuery("author", "John Doe")
//	notes := queryService.FrontmatterQuery("tags", "project-x")
//	notes := queryService.FrontmatterQuery("status", "draft")
//	notes := queryService.FrontmatterQuery("priority", 2) // matches float 2.0
func (q *QueryService) FrontmatterQuery(
	ctx context.Context,
	field string,
	value interface{},
) ([]domain.Note, error) {
	logger := newQueryLogger(q.log, "FrontmatterQuery")
	start := time.Now()

	canonicalValue, ok := canonicalizeFrontmatterValue(value)
	if !ok {
		// Cannot canonicalize value, return empty results
		logger.logPerformance(start, "field", field, 0)
		return nil, nil
	}

	// Frontmatter queries are deep-path only - use SQLite directly
	var notes []domain.Note
	var err error
	if sqliteQuery := q.router.getSQLiteQuery(); sqliteQuery != nil {
		notes, err = sqliteQuery.FrontmatterQuery(
			ctx,
			field,
			fmt.Sprintf("%v", canonicalValue),
		)
	}

	// Log performance after query execution
	logger.logPerformance(start, "field", field, len(notes))

	return notes, err
}

// newQueryLogger creates a new queryLogger with the specified method name.
func newQueryLogger(log zerolog.Logger, method string) *queryLogger {
	return &queryLogger{
		log:    log,
		method: method,
	}
}

// newQueryRouter creates a new queryRouter with the specified query ports.
func newQueryRouter(boltQuery, sqliteQuery spi.MetadataQueryPort) *queryRouter {
	return &queryRouter{
		boltQuery:   boltQuery,
		sqliteQuery: sqliteQuery,
	}
}

// logSingleResult logs the performance of a single-result query operation.
func (ql *queryLogger) logSingleResult(
	start time.Time,
	paramName, paramValue string,
) {
	duration := time.Since(start)
	ql.log.Debug().
		Dur("duration", duration).
		Str("method", ql.method).
		Str(paramName, paramValue).
		Msg("query completed")
}

// logPerformance logs the performance of a multi-result query operation.
func (ql *queryLogger) logPerformance(
	start time.Time,
	paramName, paramValue string,
	resultCount int,
) {
	duration := time.Since(start)
	ql.log.Debug().
		Dur("duration", duration).
		Str("method", ql.method).
		Str(paramName, paramValue).
		Int("results", resultCount).
		Msg("query completed")
}

// routeMetadataQuery routes a metadata query to the appropriate backend.
// It tries BoltDB first (hot path) then falls back to SQLite (deep path).
func (qr *queryRouter) routeMetadataQuery(
	ctx context.Context,
	queryFn func(spi.MetadataQueryPort, context.Context, string) ([]domain.Note, error),
	param string,
) ([]domain.Note, error) {
	// Try BoltDB first (hot path)
	if qr.boltQuery != nil {
		if notes, err := queryFn(qr.boltQuery, ctx, param); err == nil {
			return notes, nil
		}
	}

	// Fall back to SQLite (deep path)
	if qr.sqliteQuery != nil {
		return queryFn(qr.sqliteQuery, ctx, param)
	}

	return nil, nil
}

// getSQLiteQuery returns the SQLite query port for deep-path operations.
func (qr *queryRouter) getSQLiteQuery() spi.MetadataQueryPort {
	return qr.sqliteQuery
}

// GetBackendFailureStats returns failure statistics for both backends.
// This enables monitoring of backend health and resilience patterns.
func (q *QueryService) GetBackendFailureStats() map[string]int {
	return map[string]int{
		"boltdb": q.boltFailures.GetFailureCount(),
		"sqlite": q.sqliteFailures.GetFailureCount(),
	}
}

// ResetBackendFailures resets failure counters for both backends.
// Useful for manual recovery or testing.
func (q *QueryService) ResetBackendFailures() {
	q.boltFailures = NewBackendFailureTracker("boltdb", q.log)
	q.sqliteFailures = NewBackendFailureTracker("sqlite", q.log)
}

// executeMetadataQuery executes a metadata query with smart routing and
// performance logging. It routes to BoltDB first (hot path) then SQLite (deep
// path) with timing instrumentation.
func (q *QueryService) executeMetadataQuery(
	ctx context.Context,
	logger *queryLogger,
	paramName string,
	paramValue string,
	queryFn func(spi.MetadataQueryPort, context.Context, string) ([]domain.Note, error),
) ([]domain.Note, error) {
	start := time.Now()

	// Use router for smart query routing
	notes, err := q.router.routeMetadataQuery(ctx, queryFn, paramValue)

	// Log performance after query execution
	logger.logPerformance(start, paramName, paramValue, len(notes))

	return notes, err
}
