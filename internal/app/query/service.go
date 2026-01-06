// Package query provides fast lookups for indexed notes with smart routing.
package query

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// QueryService provides smart routing for indexed notes using storage routing.
// It implements thread-safe concurrent reads using sync.RWMutex and supports
// the FR9 query capabilities with optimized routing to storage backends.
type QueryService struct {
	mu sync.RWMutex

	// Dependencies
	router   *StorageRouter // Handles smart query routing
	config   domain.Config  // For file_class_key configuration
	log      zerolog.Logger
	eventBus events.EventBus
}

type queryResult struct {
	notes   []domain.Note
	err     error
	backend string
}

// queryLogger provides consistent performance logging for query operations.
type queryLogger struct {
	log    zerolog.Logger
	method string
}

// NewQueryService creates a new QueryService with the provided router.
func NewQueryService(
	router *StorageRouter,
	config domain.Config,
	log zerolog.Logger,
	eventBus events.EventBus,
) *QueryService {
	service := &QueryService{
		router:   router,
		config:   config,
		log:      log,
		eventBus: eventBus,
		mu:       sync.RWMutex{},
	}
	service.registerSubscribers()
	return service
}

// IDQuery retrieves a note by its path (formerly NoteID).
func (q *QueryService) IDQuery(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	return q.PathQuerySingle(ctx, path)
}

// PathQuerySingle retrieves a note by its file path.
func (q *QueryService) PathQuerySingle(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	logger := newQueryLogger(q.log, "PathQuery")
	start := time.Now()
	defer func() {
		logger.logSingleResult(start, "path", path)
	}()

	// Use router for single-note lookups
	note, err := q.router.Read(ctx, path)
	duration := time.Since(start)

	if err != nil {
		q.publishLookupPerformed(ctx, path, 0, duration, "id")
		if errors.Is(err, lithosErr.ErrNotFound) {
			return domain.Note{}, lithosErr.NewResourceError(
				"note",
				"get",
				path,
				lithosErr.ErrNotFound,
			)
		}
		return domain.Note{}, err
	}

	q.publishLookupPerformed(ctx, path, 1, duration, "id")
	return note, nil
}

// FileClassQuery retrieves all notes matching a schema name (fileClass).
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

// BasenameQuery retrieves all notes matching a filename basename.
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

// TagQuery retrieves all notes containing a specific tag in their frontmatter.
func (q *QueryService) TagQuery(
	ctx context.Context,
	tag string,
) ([]domain.Note, error) {
	return q.executeMetadataQuery(
		ctx,
		newQueryLogger(q.log, "TagQuery"),
		"tag",
		tag,
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.TagQuery(ctx, param)
		},
	)
}

// PathQuery resolves notes using flexible selectors (full path, basename,
// folder).
func (q *QueryService) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	start := time.Now()

	validatedOpts, err := opts.Validate()
	if err != nil {
		return nil, err
	}

	resultChan := make(chan queryResult, 2)
	queryCtx, cancel := context.WithCancel(ctx)
	defer cancel()

	go q.queryHotPath(queryCtx, validatedOpts, resultChan)
	go q.queryDeepPath(queryCtx, validatedOpts, resultChan)

	return q.collectPathQueryResults(ctx, resultChan, start, validatedOpts)
}

// FrontmatterQuery finds notes where a specific frontmatter field matches a
// value.
func (q *QueryService) FrontmatterQuery(
	ctx context.Context,
	field, value string,
) ([]domain.Note, error) {
	logger := newQueryLogger(q.log, "FrontmatterQuery")
	start := time.Now()

	// Frontmatter queries are deep-path only - use SQLite directly
	var notes []domain.Note
	var err error
	if sqliteQuery := q.router.GetSQLiteQuery(); sqliteQuery != nil {
		notes, err = sqliteQuery.FrontmatterQuery(
			ctx,
			field,
			value,
		)
	}

	duration := time.Since(start)
	// Log performance after query execution
	logger.logPerformance(start, "field", field, len(notes))

	// Publish telemetry
	q.publishQueryPerformed(
		ctx,
		map[string]any{"field": field, "value": value},
		len(notes),
		duration,
		"frontmatter",
	)

	return notes, err
}

func (q *QueryService) collectPathQueryResults(
	ctx context.Context,
	resultChan <-chan queryResult,
	start time.Time,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	var hotRes, deepRes *queryResult

	// Collect up to 2 results from the channel
	for range 2 {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case res := <-resultChan:
			if res.backend == "bolt" {
				hotRes = &res
			} else {
				deepRes = &res
			}

			// Optimization: return immediately if we have results
			if res.err == nil && len(res.notes) > 0 {
				q.finishQuery(ctx, start, opts, res)
				return res.notes, nil
			}
		}
	}

	return q.resolvePathQueryFinalResult(ctx, start, opts, hotRes, deepRes)
}

func (q *QueryService) resolvePathQueryFinalResult(
	ctx context.Context,
	start time.Time,
	opts spi.PathQueryOptions,
	hotRes, deepRes *queryResult,
) ([]domain.Note, error) {
	// Select the best successful result
	selectedRes := q.selectSuccessfulResult(hotRes, deepRes)
	if selectedRes != nil {
		q.finishQuery(ctx, start, opts, *selectedRes)
		return selectedRes.notes, nil
	}

	// All failed - get the last error and report
	lastErr := q.getLastError(deepRes, hotRes)
	duration := time.Since(start)
	q.publishQueryPerformed(
		ctx,
		map[string]any{"path": opts.Value, "error": lastErr.Error()},
		0,
		duration,
		"path",
	)
	return nil, lastErr
}

// selectSuccessfulResult returns the best successful result prioritizing
// results with data.
func (q *QueryService) selectSuccessfulResult(
	hotRes, deepRes *queryResult,
) *queryResult {
	// First check for results with data
	if res := q.findResultWithData(hotRes, deepRes); res != nil {
		return res
	}

	// Then check for any successful result (prefer deep)
	if res := q.findAnySuccessResult(deepRes, hotRes); res != nil {
		return res
	}

	return nil
}

// findResultWithData returns the first result that is successful and contains
// data.
func (q *QueryService) findResultWithData(
	results ...*queryResult,
) *queryResult {
	for _, res := range results {
		if q.isSuccessWithData(res) {
			return res
		}
	}
	return nil
}

// isSuccessWithData checks if a result is successful and contains notes.
func (q *QueryService) isSuccessWithData(res *queryResult) bool {
	return res != nil && res.err == nil && len(res.notes) > 0
}

// findAnySuccessResult returns the first successful result.
func (q *QueryService) findAnySuccessResult(
	results ...*queryResult,
) *queryResult {
	for _, res := range results {
		if res != nil && res.err == nil {
			return res
		}
	}
	return nil
}

// getLastError returns the last error from the query results.
func (q *QueryService) getLastError(deepRes, hotRes *queryResult) error {
	switch {
	case deepRes != nil:
		return deepRes.err
	case hotRes != nil:
		return hotRes.err
	default:
		return errors.New("query failed: no backends responded")
	}
}

func (q *QueryService) finishQuery(
	ctx context.Context,
	start time.Time,
	opts spi.PathQueryOptions,
	res queryResult,
) {
	duration := time.Since(start)
	newQueryLogger(q.log, "PathQuery").logPerformance(
		start,
		"query",
		opts.Value,
		len(res.notes),
	)
	q.publishQueryPerformed(ctx, map[string]any{
		"path":    opts.Value,
		"scope":   string(opts.Scope),
		"backend": res.backend,
	}, len(res.notes), duration, "path")
}

func (q *QueryService) queryHotPath(
	ctx context.Context,
	opts spi.PathQueryOptions,
	ch chan<- queryResult,
) {
	bolt := q.router.GetBoltQuery()
	if bolt == nil {
		ch <- queryResult{err: fmt.Errorf("hot path unavailable"), backend: "bolt", notes: nil}
		return
	}

	notes, err := bolt.PathQuery(ctx, opts)
	ch <- queryResult{notes: notes, err: err, backend: "bolt"}
}

func (q *QueryService) queryDeepPath(
	ctx context.Context,
	opts spi.PathQueryOptions,
	ch chan<- queryResult,
) {
	sqlite := q.router.GetSQLiteQuery()
	if sqlite == nil {
		ch <- queryResult{err: fmt.Errorf("deep path unavailable"), backend: "sqlite", notes: nil}
		return
	}

	notes, err := sqlite.PathQuery(ctx, opts)
	ch <- queryResult{notes: notes, err: err, backend: "sqlite"}
}

// newQueryLogger creates a new queryLogger with the specified method name.
func newQueryLogger(log zerolog.Logger, method string) *queryLogger {
	return &queryLogger{
		log:    log,
		method: method,
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

func (q *QueryService) publishLookupPerformed(
	ctx context.Context,
	noteID string,
	resultCount int,
	duration time.Duration,
	lookupType string,
) {
	if q.eventBus == nil {
		return
	}
	event := events.MustNewLookupPerformedEvent(
		noteID,
		resultCount,
		duration,
		lookupType,
		time.Now(),
	)
	events.PublishAsync(ctx, q.eventBus, q.log, event)
}

func (q *QueryService) publishQueryPerformed(
	ctx context.Context,
	filter map[string]any,
	resultCount int,
	duration time.Duration,
	queryType string,
) {
	if q.eventBus == nil {
		return
	}
	event := events.MustNewQueryPerformedEvent(
		filter,
		resultCount,
		duration,
		queryType,
		time.Now(),
	)
	events.PublishAsync(ctx, q.eventBus, q.log, event)
}

func (q *QueryService) registerSubscribers() {
	if q.eventBus == nil {
		return
	}
	_ = q.eventBus.Subscribe(
		"VaultIndexingComplete",
		q.handleVaultIndexingComplete,
	)
	_ = q.eventBus.Subscribe(
		"SchemaUpdated",
		q.handleSchemaUpdated,
	)
}

func (q *QueryService) handleVaultIndexingComplete(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	completeEvent, ok := event.(*events.VaultIndexingCompleteEvent)
	if !ok {
		return nil
	}

	q.log.Info().
		Int("indexed", completeEvent.NotesIndexed()).
		Int("scanned", completeEvent.ScannedCount()).
		Dur("duration", completeEvent.Duration()).
		Msg("query service observed vault indexing completion")

	return nil
}

// handleSchemaUpdated handles SchemaUpdatedEvent for reactive cache
// invalidation.
func (q *QueryService) handleSchemaUpdated(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	schemaEvent, ok := event.(*domain.SchemaUpdatedEvent)
	if !ok {
		return nil
	}

	// Log schema update for observability
	q.log.Info().
		Str("schema", schemaEvent.SchemaName()).
		Str("operation", schemaEvent.Operation()).
		Msg("query service observed schema update - cache invalidation triggered")

	return nil
}

// executeMetadataQuery executes a metadata query with smart routing and
// performance logging.
func (q *QueryService) executeMetadataQuery(
	ctx context.Context,
	logger *queryLogger,
	paramName string,
	paramValue string,
	queryFn func(spi.MetadataQueryPort, context.Context, string) ([]domain.Note, error),
) ([]domain.Note, error) {
	start := time.Now()

	// Use router for smart query routing
	notes, err := q.router.RouteMetadataQuery(ctx, queryFn, paramValue)
	duration := time.Since(start)

	// Log performance after query execution
	logger.logPerformance(start, paramName, paramValue, len(notes))

	// Publish telemetry
	q.publishQueryPerformed(
		ctx,
		map[string]any{paramName: paramValue},
		len(notes),
		duration,
		"metadata",
	)

	return notes, err
}
