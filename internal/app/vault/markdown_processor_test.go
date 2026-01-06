package vault

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestMarkdownProcessingService_HandleFileParseRequested tests markdown file
// parsing.
func TestMarkdownProcessingService_HandleFileParseRequested(t *testing.T) {
	markdownParser := utils.NewMockMarkdownParserPort()
	markdownParser.SetParseResult(map[string]any{
		"fileClass": "note",
		"title":     "Test Note",
	}, nil)

	eventBus := utils.NewMockEventBus()
	service := NewMarkdownProcessingService(
		markdownParser,
		eventBus,
		zerolog.Nop(),
	)

	// Create a parse request event
	event, err := events.NewFileParseRequestedEvent(
		"/tmp/test/note.md",
		[]byte("---\nfileClass: note\ntitle: Test Note\n---\n\nContent"),
		time.Now(),
	)
	require.NoError(t, err)

	ctx := context.Background()
	err = service.handleFileParseRequested(ctx, event)

	require.NoError(t, err)

	// Check that NoteParsedEvent was published
	publishedEvents := eventBus.GetPublishedEvents()
	assert.NotEmpty(t, publishedEvents)

	foundNoteParsed := false
	for _, publishedEvent := range publishedEvents {
		if publishedEvent.EventType() == "NoteParsed" {
			foundNoteParsed = true
			break
		}
	}
	assert.True(t, foundNoteParsed, "Should publish NoteParsedEvent")
}

// TestMarkdownProcessingService_HandleFileParseRequested_ParseError tests
// handling of parse errors.
func TestMarkdownProcessingService_HandleFileParseRequested_ParseError(
	t *testing.T,
) {
	markdownParser := utils.NewMockMarkdownParserPort()
	markdownParser.SetParseResult(nil, assert.AnError)

	eventBus := utils.NewMockEventBus()
	service := NewMarkdownProcessingService(
		markdownParser,
		eventBus,
		zerolog.Nop(),
	)

	// Create a parse request event
	event, err := events.NewFileParseRequestedEvent(
		"/tmp/test/note.md",
		[]byte("invalid content"),
		time.Now(),
	)
	require.NoError(t, err)

	ctx := context.Background()
	err = service.handleFileParseRequested(ctx, event)

	require.Error(t, err)
}

// TestMarkdownProcessingService_HandleFileParseRequested_NonMarkdownFile tests
// filtering of non-markdown files.
func TestMarkdownProcessingService_HandleFileParseRequested_NonMarkdownFile(
	t *testing.T,
) {
	markdownParser := utils.NewMockMarkdownParserPort()
	eventBus := utils.NewMockEventBus()
	service := NewMarkdownProcessingService(
		markdownParser,
		eventBus,
		zerolog.Nop(),
	)

	// Create a parse request event for non-markdown file
	event, err := events.NewFileParseRequestedEvent(
		"/tmp/test/note.txt",
		[]byte("some content"),
		time.Now(),
	)
	require.NoError(t, err)

	ctx := context.Background()
	err = service.handleFileParseRequested(ctx, event)

	// Should not error, but should not publish any events
	require.NoError(t, err)
	assert.Empty(
		t,
		eventBus.GetPublishedEvents(),
		"Should not publish events for non-markdown files",
	)
}
