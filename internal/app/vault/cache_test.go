package vault

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestCacheWriter_HandleCacheRequested tests successful note caching.
func TestCacheWriter_HandleCacheRequested(t *testing.T) {
	boltWriter := utils.NewMockCacheWriterPort()
	sqliteWriter := utils.NewMockCacheWriterPort()
	eventBus := utils.NewMockEventBus()

	writer := NewCacheWriter(
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
	event, err := events.NewNoteCacheRequestedEvent(note, time.Now())
	require.NoError(t, err)

	ctx := context.Background()
	err = writer.handleCacheRequested(ctx, event)

	require.NoError(t, err)
}

// TestCacheWriter_Creation tests that the writer can be created.
func TestCacheWriter_Creation(t *testing.T) {
	boltWriter := utils.NewMockCacheWriterPort()
	sqliteWriter := utils.NewMockCacheWriterPort()
	eventBus := utils.NewMockEventBus()
	logger := zerolog.Nop()

	writer := NewCacheWriter(boltWriter, sqliteWriter, eventBus, logger)

	assert.NotNil(t, writer)
	assert.Equal(t, boltWriter, writer.boltWriter)
	assert.Equal(t, sqliteWriter, writer.sqliteWriter)
	assert.Equal(t, eventBus, writer.eventBus)
	assert.Equal(t, logger, writer.log)
}
