package query

import (
	"context"
	"fmt"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// QueryBackend defines the interface required by the router for storage
// backends. It combines the reading and querying capabilities.
type QueryBackend interface {
	spi.CacheReaderPort
	spi.MetadataQueryPort
}

// StorageRouter handles smart routing between multiple storage backends.
// It implements the query routing strategy that optimizes performance by
// directing queries to the most appropriate storage backend.
//
// Routing Strategy:
// - Hot Path (e.g., BoltDB): Fast lookups for paths, basenames, aliases
// - Deep Path (e.g., SQLite): Complex queries with JSON extraction
// - Fallback: Deep path serves as fallback when hot path is unavailable.
type StorageRouter struct {
	bolt   QueryBackend
	sqlite QueryBackend
}

// NewStorageRouter creates a new router with the specified backends.
func NewStorageRouter(
	bolt, sqlite QueryBackend,
) *StorageRouter {
	return &StorageRouter{
		bolt:   bolt,
		sqlite: sqlite,
	}
}

// RouteMetadataQuery routes a metadata query to the appropriate backend.
func (r *StorageRouter) RouteMetadataQuery(
	ctx context.Context,
	queryFn func(spi.MetadataQueryPort, context.Context, string) ([]domain.Note, error),
	param string,
) ([]domain.Note, error) {
	// Try hot path first
	if r.bolt != nil {
		if notes, err := queryFn(r.bolt, ctx, param); err == nil {
			return notes, nil
		}
	}

	// Fall back to deep path
	if r.sqlite != nil {
		return queryFn(r.sqlite, ctx, param)
	}

	return nil, nil
}

// Read retrieves a single note by its path with fallback.
func (r *StorageRouter) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	if r.bolt != nil {
		if note, err := r.bolt.Read(ctx, path); err == nil {
			return note, nil
		}
	}

	if r.sqlite != nil {
		return r.sqlite.Read(ctx, path)
	}

	return domain.Note{}, fmt.Errorf("no storage backend available")
}

// GetSQLiteQuery returns the SQLite query port for deep-path operations.
func (r *StorageRouter) GetSQLiteQuery() spi.MetadataQueryPort {
	return r.sqlite
}

// GetBoltQuery returns the BoltDB query port for hot-path operations.
func (r *StorageRouter) GetBoltQuery() spi.MetadataQueryPort {
	return r.bolt
}

// HasHotPath returns true if hot path is configured.
func (r *StorageRouter) HasHotPath() bool {
	return r.bolt != nil
}

// HasDeepPath returns true if deep path is configured.
func (r *StorageRouter) HasDeepPath() bool {
	return r.sqlite != nil
}
