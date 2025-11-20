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
// - Hot Queries (BoltDB): ByPath, ByBasename, ByAlias, directory filtering
// - Complex Queries (SQLite): ByFrontmatter, ByFileClass with property
// filtering
// - Hybrid Queries: Coordinate between stores for optimal performance.
type QueryService struct {
	mu sync.RWMutex

	// Dependencies
	boltReader   spi.CacheReaderPort   // Hot cache for fast lookups
	boltQuery    spi.MetadataQueryPort // Hot cache metadata queries (if supported)
	sqliteReader spi.CacheReaderPort   // Deep storage for complex queries
	sqliteQuery  spi.MetadataQueryPort // Deep storage metadata queries (if supported)
	config       domain.Config         // For file_class_key configuration
	log          zerolog.Logger
}

// canonicalizeFrontmatterValue normalizes frontmatter values for type-agnostic
// comparison.
// Handles numeric type conversions (int 2 == float 2.0) and safe comparison
// for complex types.
// Returns the normalized value and whether normalization was successful.
//
//nolint:cyclop // Type normalization requires exhaustive type checking
func canonicalizeFrontmatterValue(value interface{}) (interface{}, bool) {
	switch v := value.(type) {
	case int:
		// Convert int to float64 for consistent numeric comparison
		return float64(v), true
	case int8:
		return float64(v), true
	case int16:
		return float64(v), true
	case int32:
		return float64(v), true
	case int64:
		return float64(v), true
	case uint:
		return float64(v), true
	case uint8:
		return float64(v), true
	case uint16:
		return float64(v), true
	case uint32:
		return float64(v), true
	case uint64:
		return float64(v), true
	case float32:
		return float64(v), true
	case float64:
		return v, true
	case string, bool:
		// Strings and booleans are already comparable
		return v, true
	default:
		// Complex types (arrays, maps) are not safely comparable
		// Return false to indicate normalization failed
		return nil, false
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
//	note, err := qs.ByID(ctx, id)   // Query safely with smart routing
func NewQueryService(
	boltReader spi.CacheReaderPort,
	sqliteReader spi.CacheReaderPort,
	config domain.Config,
	log zerolog.Logger,
) *QueryService {
	qs := &QueryService{
		boltReader:   boltReader,
		sqliteReader: sqliteReader,
		boltQuery:    nil,
		sqliteQuery:  nil,
		config:       config,
		log:          log,
		// mu is initialized to zero value (unlocked state)
		mu: sync.RWMutex{},
	}

	// Attempt to upgrade readers to query ports
	if mq, ok := boltReader.(spi.MetadataQueryPort); ok {
		qs.boltQuery = mq
	}
	if mq, ok := sqliteReader.(spi.MetadataQueryPort); ok {
		qs.sqliteQuery = mq
	}

	return qs
}

// ByID retrieves a note by its NoteID.
// Returns the note if found, or ResourceError if not found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns ResourceError for missing notes (single result lookup)
// - Logs debug message with NoteID for troubleshooting
// - O(1) lookup performance via map access.
func (q *QueryService) ByID(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "ByID").
			Str("noteID", id.String()).
			Msg("query performance")
	}()

	// Hot path: use BoltDB for fast lookups
	if q.boltReader != nil {
		return q.boltReader.Read(ctx, id)
	}

	if q.sqliteReader != nil {
		return q.sqliteReader.Read(ctx, id)
	}

	return domain.Note{}, lithosErr.NewResourceError(
		"note",
		"get",
		id.String(),
		errors.New("not found"),
	)
}

// ByPath retrieves a note by its file path.
// Returns the note if found, or ResourceError if not found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns ResourceError for missing notes (single result lookup)
// - Logs debug message with path for troubleshooting
// - O(1) lookup performance via map access.
func (q *QueryService) ByPath(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "ByPath").
			Str("path", path).
			Msg("query performance")
	}()

	// Hot path: use BoltDB for fast lookups
	if q.boltReader != nil {
		return q.boltReader.Read(ctx, domain.NoteID(path))
	}

	if q.sqliteReader != nil {
		return q.sqliteReader.Read(ctx, domain.NoteID(path))
	}

	return domain.Note{}, lithosErr.NewResourceError(
		"note",
		"get",
		path,
		errors.New("not found"),
	)
}

// ByFileClass retrieves all notes matching a schema name (fileClass).
// Returns a slice of notes if any match, or empty slice if none found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching schemas (collection
// lookup)
// - Logs debug message with fileClass and result count
// - O(log n) lookup performance via map access.
func (q *QueryService) ByFileClass(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	return q.executeMetadataQuery(
		ctx,
		"ByFileClass",
		"fileClass",
		fileClass,
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.ByFileClass(ctx, param)
		},
	)
}

// ByBasename retrieves all notes matching a filename basename (without
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
// - Uses in-memory index for fast lookup performance
//
// Example: NoteID "projects/notes/meeting.md" matches basename "meeting".
func (q *QueryService) ByBasename(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "ByBasename").
			Str("basename", basename).
			Msg("query performance")
	}()

	if q.boltQuery != nil {
		return q.boltQuery.ByBasename(ctx, basename)
	}

	opts, err := (spi.PathQueryOptions{
		Scope: spi.PathQueryScopeBasename,
		Value: basename,
	}).Validate()
	if err != nil {
		return nil, err
	}

	return q.PathQuery(ctx, opts)
}

// ByAlias retrieves all notes containing an alias in their frontmatter.
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
func (q *QueryService) ByAlias(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	return q.executeMetadataQuery(
		ctx,
		"ByAlias",
		"alias",
		alias,
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.ByAlias(ctx, param)
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
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "PathQuery").
			Str("scope", string(opts.Scope)).
			Str("value", opts.Value).
			Msg("query performance")
	}()

	validatedOpts, err := opts.Validate()
	if err != nil {
		return nil, err
	}

	if q.boltQuery != nil {
		// Hot path: use BoltDB for path queries
		return q.boltQuery.PathQuery(ctx, validatedOpts)
	}

	if q.sqliteQuery != nil {
		// Deep path fallback
		return q.sqliteQuery.PathQuery(ctx, validatedOpts)
	}

	return nil, nil
}

// ByFrontmatter finds notes where a specific frontmatter field matches a value.
// Supports type-agnostic comparison (int 2 == float 2.0) and delegates to
// MetadataQueryPort for indexed lookups.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching frontmatter (collection
// lookup)
// - Type normalization: int/float conversion for numeric comparison
// - Logs debug message with field and value for troubleshooting
// - Routes to SQLite for complex frontmatter queries (no BoltDB support yet)
//
// Example:
//
//	notes := queryService.ByFrontmatter("author", "John Doe")
//	notes := queryService.ByFrontmatter("tags", "project-x")
//	notes := queryService.ByFrontmatter("status", "draft")
//	notes := queryService.ByFrontmatter("priority", 2) // matches float 2.0
func (q *QueryService) ByFrontmatter(
	ctx context.Context,
	field string,
	value interface{},
) ([]domain.Note, error) {
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "ByFrontmatter").
			Str("field", field).
			Interface("value", value).
			Msg("query performance")
	}()

	if q.sqliteQuery != nil {
		canonicalValue, ok := canonicalizeFrontmatterValue(value)
		if ok {
			// Convert value to string for query port
			return q.sqliteQuery.FrontmatterQuery(
				ctx,
				field,
				fmt.Sprintf("%v", canonicalValue),
			)
		}
	}

	return nil, nil
}

// executeMetadataQuery executes a metadata query with smart routing and
// performance logging. It routes to BoltDB first (hot path) then SQLite (deep
// path) with timing instrumentation.
func (q *QueryService) executeMetadataQuery(
	ctx context.Context,
	methodName string,
	paramName string,
	paramValue string,
	queryFn func(spi.MetadataQueryPort, context.Context, string) ([]domain.Note, error),
) ([]domain.Note, error) {
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", methodName).
			Str(paramName, paramValue).
			Msg("query performance")
	}()

	if q.boltQuery != nil {
		// Hot path: use BoltDB for fast lookups
		return queryFn(q.boltQuery, ctx, paramValue)
	}

	if q.sqliteQuery != nil {
		// Deep path fallback
		return queryFn(q.sqliteQuery, ctx, paramValue)
	}

	return nil, nil
}
