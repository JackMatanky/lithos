package vault

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestCachingService_HandleCacheRequested tests successful note caching.
func TestCachingService_HandleCacheRequested(t *testing.T) {
	boltWriter := utils.NewMockCacheWriterPort()
	sqliteWriter := utils.NewMockCacheWriterPort()
	eventBus := utils.NewMockEventBus()

	service := NewCachingService(
		boltWriter,
		sqliteWriter,
		eventBus,
		zerolog.Nop(),
	)

	// Create a note
	note, err := domain.NewNote("test.md", domain.NewFrontmatter(map[string]any{
		"fileClass": "note",
		"title":     "Test Note",
	}), nil, nil, nil, nil)
	require.NoError(t, err)

	// Create a cache request event
	event, err := domain.NewNoteCacheRequestedEvent(note, time.Now())
	require.NoError(t, err)

	ctx := context.Background()
	err = service.handleCacheRequested(ctx, event)

	require.NoError(t, err)
}

// TestCachingService_Creation tests that the service can be created.
func TestCachingService_Creation(t *testing.T) {
	boltWriter := utils.NewMockCacheWriterPort()
	sqliteWriter := utils.NewMockCacheWriterPort()
	eventBus := utils.NewMockEventBus()
	logger := zerolog.Nop()

	service := NewCachingService(boltWriter, sqliteWriter, eventBus, logger)

	assert.NotNil(t, service)
	assert.Equal(t, boltWriter, service.boltWriter)
	assert.Equal(t, sqliteWriter, service.sqliteWriter)
	assert.Equal(t, eventBus, service.eventBus)
	assert.Equal(t, logger, service.log)
}
