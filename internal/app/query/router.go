package query

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// HybridStorageRouter handles smart routing between BoltDB (hot path) and
// SQLite (deep path) storage backends. It implements the query routing strategy
// that optimizes performance by directing queries to the most appropriate
// storage backend.
//
// Routing Strategy:
// - BoltDB (Hot Path): Fast lookups for paths, basenames, aliases, file classes
// - SQLite (Deep Path): Complex queries with JSON extraction and indexed views
// - Fallback: SQLite serves as fallback when BoltDB is unavailable
//
// Performance Targets:
// - Hot Path: <1ms for common queries (BoltDB)
// - Deep Path: <50ms for complex queries (SQLite)
//
// Thread Safety: Router is read-only after construction, safe for concurrent
// use.
type HybridStorageRouter struct {
	boltQuery   spi.MetadataQueryPort
	sqliteQuery spi.MetadataQueryPort
}

// NewHybridStorageRouter creates a new router with the specified query ports.
// Both ports are optional; router will work with whichever backends are
// available.
//
// Parameters:
//   - boltQuery: Hot path query port (BoltDB) - optional
//   - sqliteQuery: Deep path query port (SQLite) - optional
//
// Returns:
//   - *HybridStorageRouter: Configured router ready for query routing
func NewHybridStorageRouter(
	boltQuery, sqliteQuery spi.MetadataQueryPort,
) *HybridStorageRouter {
	return &HybridStorageRouter{
		boltQuery:   boltQuery,
		sqliteQuery: sqliteQuery,
	}
}

// RouteMetadataQuery routes a metadata query to the appropriate backend.
// It tries BoltDB first (hot path) for optimal performance, then falls back
// to SQLite (deep path) if needed.
//
// Parameters:
//   - ctx: Request context for cancellation
//   - queryFn: Query function to execute on the selected backend
//   - param: Query parameter (basename, alias, fileClass, etc.)
//
// Returns:
//   - []domain.Note: Query results from the selected backend
//   - error: Any error from query execution
//
// Routing Logic:
// 1. Try BoltDB first (hot path) if available
// 2. Fall back to SQLite (deep path) if BoltDB unavailable or fails
// 3. Return empty slice if no backends available.
func (r *HybridStorageRouter) RouteMetadataQuery(
	ctx context.Context,
	queryFn func(spi.MetadataQueryPort, context.Context, string) ([]domain.Note, error),
	param string,
) ([]domain.Note, error) {
	// Try BoltDB first (hot path) for <1ms performance
	if r.boltQuery != nil {
		if notes, err := queryFn(r.boltQuery, ctx, param); err == nil {
			return notes, nil
		}
	}

	// Fall back to SQLite (deep path) for <50ms performance
	if r.sqliteQuery != nil {
		return queryFn(r.sqliteQuery, ctx, param)
	}

	// No backends available - return empty result
	return nil, nil
}

// GetSQLiteQuery returns the SQLite query port for deep-path operations.
// Used when queries must use SQLite-specific features (JSON extraction, etc.).
//
// Returns:
//   - spi.MetadataQueryPort: SQLite query port, or nil if not configured
func (r *HybridStorageRouter) GetSQLiteQuery() spi.MetadataQueryPort {
	return r.sqliteQuery
}

// GetBoltQuery returns the BoltDB query port for hot-path operations.
// Exposed for testing and diagnostics.
//
// Returns:
//   - spi.MetadataQueryPort: BoltDB query port, or nil if not configured
func (r *HybridStorageRouter) GetBoltQuery() spi.MetadataQueryPort {
	return r.boltQuery
}

// HasHotPath returns true if BoltDB hot path is configured.
func (r *HybridStorageRouter) HasHotPath() bool {
	return r.boltQuery != nil
}

// HasDeepPath returns true if SQLite deep path is configured.
func (r *HybridStorageRouter) HasDeepPath() bool {
	return r.sqliteQuery != nil
}
