package vault

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestDataProcessingOrchestrator_Start tests that the orchestrator starts and
// subscribes to events.
func TestDataProcessingOrchestrator_Start(t *testing.T) {
	eventBus := utils.NewMockEventBus()
	logger := zerolog.Nop()

	orchestrator := NewDataProcessingOrchestrator(eventBus, logger)

	ctx := context.Background()
	err := orchestrator.Start(ctx)

	require.NoError(t, err)
	subscribedTypes := eventBus.GetSubscribedTypes()
	assert.Contains(t, subscribedTypes, "FileDiscovered")
	assert.Contains(t, subscribedTypes, "NoteParsed")
	assert.Contains(t, subscribedTypes, "FrontmatterValidated")
}

// TestDataProcessingOrchestrator_HandleFileDiscovered tests file discovery
// event handling.
func TestDataProcessingOrchestrator_HandleFileDiscovered(t *testing.T) {
	eventBus := utils.NewMockEventBus()
	logger := zerolog.Nop()

	orchestrator := NewDataProcessingOrchestrator(eventBus, logger)
	_ = orchestrator.Start(context.Background())

	// Create a file discovered event
	testFile, err := dto.NewVaultFile(
		"/tmp/test/file.md",
		"/tmp/test",
		nil,
		[]byte("content"),
	)
	require.NoError(t, err)

	event, err := domain.NewFileDiscoveredEvent(
		testFile.Path,
		len(testFile.Content),
		testFile.Content,
		time.Now(),
	)
	require.NoError(t, err)

	// Handle the event
	err = orchestrator.handleFileDiscovered(context.Background(), event)
	require.NoError(t, err)

	// Check that a parse request event was published
	publishedEvents := eventBus.GetPublishedEvents()
	assert.NotEmpty(t, publishedEvents)

	foundParseRequest := false
	for _, publishedEvent := range publishedEvents {
		if publishedEvent.EventType() == "FileParseRequested" {
			foundParseRequest = true
			break
		}
	}
	assert.True(t, foundParseRequest, "Should publish FileParseRequestedEvent")
}

// TestDataProcessingOrchestrator_HandleNoteParsed tests note parsed event
// handling.
func TestDataProcessingOrchestrator_HandleNoteParsed(t *testing.T) {
	eventBus := utils.NewMockEventBus()
	logger := zerolog.Nop()

	orchestrator := NewDataProcessingOrchestrator(eventBus, logger)
	_ = orchestrator.Start(context.Background())

	// Create a note
	note, err := domain.NewNote("test.md", domain.NewFrontmatter(map[string]any{
		"fileClass": "note",
		"title":     "Test",
	}), nil, nil, nil, nil)
	require.NoError(t, err)

	event, err := domain.NewNoteParsedEvent(note, time.Now())
	require.NoError(t, err)

	// Handle the event
	err = orchestrator.handleNoteParsed(context.Background(), event)
	require.NoError(t, err)

	// Check that a validation request event was published
	publishedEvents := eventBus.GetPublishedEvents()
	assert.NotEmpty(t, publishedEvents)

	foundValidationRequest := false
	for _, publishedEvent := range publishedEvents {
		if publishedEvent.EventType() == "FrontmatterValidationRequested" {
			foundValidationRequest = true
			break
		}
	}
	assert.True(
		t,
		foundValidationRequest,
		"Should publish FrontmatterValidationRequestedEvent",
	)
}

// TestDataProcessingOrchestrator_HandleFrontmatterValidated tests frontmatter
// validated event handling.
func TestDataProcessingOrchestrator_HandleFrontmatterValidated(t *testing.T) {
	eventBus := utils.NewMockEventBus()
	logger := zerolog.Nop()

	orchestrator := NewDataProcessingOrchestrator(eventBus, logger)
	_ = orchestrator.Start(context.Background())

	// Create a note
	note, err := domain.NewNote("test.md", domain.NewFrontmatter(map[string]any{
		"fileClass": "note",
		"title":     "Test",
	}), nil, nil, nil, nil)
	require.NoError(t, err)

	event, err := domain.NewFrontmatterValidatedEvent(
		note,
		"note",
		true,
		[]string{},
		time.Now(),
	)
	require.NoError(t, err)

	// Handle the event
	err = orchestrator.handleFrontmatterValidated(context.Background(), event)
	require.NoError(t, err)

	// Check that a cache request event was published
	publishedEvents := eventBus.GetPublishedEvents()
	assert.NotEmpty(t, publishedEvents)

	foundCacheRequest := false
	for _, publishedEvent := range publishedEvents {
		if publishedEvent.EventType() == "NoteCacheRequested" {
			foundCacheRequest = true
			break
		}
	}
	assert.True(t, foundCacheRequest, "Should publish NoteCacheRequestedEvent")
}
