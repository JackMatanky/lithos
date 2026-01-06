package vault

import (
	"context"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/persistence"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
)

// CacheWriter handles persistence of validated data to multiple storage
// backends. This component manages the dual-write operations to BoltDB (hot
// cache) and SQLite
// (cold storage), ensuring data consistency and transactional integrity.
//
// Responsibilities:
//   - Coordinate dual-write operations to multiple cache backends
//   - Ensure transactional consistency across storage systems
//   - Handle cache write failures gracefully
//   - Support both hot (fast) and cold (persistent) storage
//
// Architecture:
//   - Subscribes to NoteCacheRequestedEvent
//   - Uses Unit of Work pattern for transactional writes
//   - Supports BoltDB + SQLite dual-write strategy
//   - Generic enough to work with any data type requiring persistence
type CacheWriter struct {
	boltWriter   spi.CacheWriterPort
	sqliteWriter spi.CacheWriterPort
	eventBus     events.EventBus
	log          zerolog.Logger
}

// NewCacheWriter creates a new cache writer.
func NewCacheWriter(
	boltWriter spi.CacheWriterPort,
	sqliteWriter spi.CacheWriterPort,
	eventBus events.EventBus,
	log zerolog.Logger,
) *CacheWriter {
	writer := &CacheWriter{
		boltWriter:   boltWriter,
		sqliteWriter: sqliteWriter,
		eventBus:     eventBus,
		log:          log,
	}

	// Subscribe to cache requests
	if eventBus != nil {
		_ = eventBus.Subscribe(
			"NoteCacheRequested",
			writer.handleCacheRequested,
		)
	}

	return writer
}

// WriteBatch writes multiple notes to cache in a single transaction using
// ParallelWriter for optimal performance.
// This is the primary method for batch indexing operations.
func (c *CacheWriter) WriteBatch(
	ctx context.Context,
	notes []domain.Note,
	metadataMap map[string]spi.CacheWriteMetadata,
) error {
	if len(notes) == 0 {
		return nil
	}

	c.log.Debug().
		Int("count", len(notes)).
		Msg("batch caching notes to dual storage")

	// Create transaction for atomic writes
	strategy := &persistence.ParallelWriter{}
	tx := persistence.NewCacheTransaction(
		strategy,
		c.boltWriter,
		c.sqliteWriter,
	)

	// Stage all notes for caching
	for i := range notes {
		metadata := metadataMap[notes[i].Path]
		tx.AddWrite(notes[i], metadata)
	}

	// Commit the transaction (writes to BoltDB and SQLite in parallel)
	if err := tx.Commit(ctx); err != nil {
		c.log.Error().
			Err(err).
			Int("count", len(notes)).
			Msg("failed to commit batch cache transaction")
		return err
	}

	c.log.Debug().
		Int("count", len(notes)).
		Msg("batch cached notes successfully to dual storage")

	return nil
}

// DeleteBatch deletes multiple notes from cache in a single transaction using
// ParallelWriter for optimal performance.
// This is used for deletion reconciliation during refresh operations.
func (c *CacheWriter) DeleteBatch(ctx context.Context, paths []string) error {
	if len(paths) == 0 {
		return nil
	}

	c.log.Debug().
		Int("count", len(paths)).
		Msg("batch deleting notes from dual storage")

	// Create transaction for atomic deletes
	strategy := &persistence.ParallelWriter{}
	tx := persistence.NewCacheTransaction(
		strategy,
		c.boltWriter,
		c.sqliteWriter,
	)

	// Stage all deletions
	for i := range paths {
		tx.AddDelete(paths[i])
	}

	// Commit the transaction (deletes from BoltDB and SQLite in parallel)
	if err := tx.Commit(ctx); err != nil {
		c.log.Error().
			Err(err).
			Int("count", len(paths)).
			Msg("failed to commit batch delete transaction")
		return err
	}

	c.log.Debug().
		Int("count", len(paths)).
		Msg("batch deleted notes successfully from dual storage")

	return nil
}

// handleCacheRequested processes cache requests for validated notes.
// This is used for event-driven individual note caching.
func (c *CacheWriter) handleCacheRequested(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	cacheEvent, ok := event.(*events.NoteCacheRequestedEvent)
	if !ok {
		return nil
	}

	note := cacheEvent.Note()
	c.log.Debug().
		Str("path", note.Path).
		Msg("caching validated note to dual storage")

	// Use WriteBatch for consistency
	metadata := spi.CacheWriteMetadata{
		ModifiedAt: time.Time{},
		FileSize:   0,
		IndexTime:  time.Now().UTC(),
	}
	metadataMap := map[string]spi.CacheWriteMetadata{
		note.Path: metadata,
	}

	return c.WriteBatch(ctx, []domain.Note{note}, metadataMap)
}
