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

// CachingService handles persistence of validated data to multiple storage
// backends. This service manages the dual-write operations to BoltDB (hot
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
type CachingService struct {
	boltWriter   spi.CacheWriterPort
	sqliteWriter spi.CacheWriterPort
	eventBus     events.EventBus
	log          zerolog.Logger
}

// NewCachingService creates a new caching service.
func NewCachingService(
	boltWriter spi.CacheWriterPort,
	sqliteWriter spi.CacheWriterPort,
	eventBus events.EventBus,
	log zerolog.Logger,
) *CachingService {
	service := &CachingService{
		boltWriter:   boltWriter,
		sqliteWriter: sqliteWriter,
		eventBus:     eventBus,
		log:          log,
	}

	// Subscribe to cache requests
	if eventBus != nil {
		_ = eventBus.Subscribe(
			"NoteCacheRequested",
			service.handleCacheRequested,
		)
	}

	return service
}

// handleCacheRequested processes cache requests for validated notes.
func (s *CachingService) handleCacheRequested(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	cacheEvent, ok := event.(*events.NoteCacheRequestedEvent)
	if !ok {
		return nil
	}

	note := cacheEvent.Note()
	s.log.Debug().
		Str("path", note.Path).
		Msg("caching validated note to dual storage")

	// Create transaction for atomic writes
	strategy := &persistence.ParallelWriter{}
	tx := persistence.NewCacheTransaction(
		strategy,
		s.boltWriter,
		s.sqliteWriter,
	)

	// Stage the note for caching
	metadata := spi.CacheWriteMetadata{
		ModifiedAt: time.Time{},
		FileSize:   0,
		IndexTime:  time.Now().UTC(),
	}
	tx.AddWrite(note, metadata)

	// Commit the transaction
	if err := tx.Commit(ctx); err != nil {
		s.log.Error().
			Err(err).
			Str("path", note.Path).
			Msg("failed to commit cache transaction")
		return err
	}

	s.log.Debug().
		Str("path", note.Path).
		Msg("note cached successfully to dual storage")

	return nil
}
