package vault

import (
	"context"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

// DataProcessingOrchestrator coordinates the event-driven data processing
// pipeline. This service orchestrates the flow: File Discovery → Parsing →
// Validation → Caching
// without being tied to specific data formats or storage implementations.
//
// Responsibilities:
//   - Coordinate the overall processing pipeline
//   - Route events between specialized processing services
//   - Handle pipeline orchestration and error recovery
//
// Architecture:
//   - Event-driven communication with processing services
//   - Loose coupling through domain events
//   - Centralized pipeline coordination
type DataProcessingOrchestrator struct {
	eventBus events.EventBus
	log      zerolog.Logger
}

// NewDataProcessingOrchestrator creates a new orchestrator for the processing
// pipeline.
func NewDataProcessingOrchestrator(
	eventBus events.EventBus,
	log zerolog.Logger,
) *DataProcessingOrchestrator {
	return &DataProcessingOrchestrator{
		eventBus: eventBus,
		log:      log,
	}
}

// Start begins listening for processing events and coordinates the pipeline.
func (o *DataProcessingOrchestrator) Start(ctx context.Context) error {
	o.log.Info().Msg("data processing orchestrator started")

	// Set up event subscriptions for pipeline coordination
	if o.eventBus != nil {
		_ = o.eventBus.Subscribe("FileDiscovered", o.handleFileDiscovered)
		_ = o.eventBus.Subscribe("NoteParsed", o.handleNoteParsed)
		_ = o.eventBus.Subscribe(
			"FrontmatterValidated",
			o.handleFrontmatterValidated,
		)
	}

	return nil
}

// handleFileDiscovered routes newly discovered files to appropriate parsers.
func (o *DataProcessingOrchestrator) handleFileDiscovered(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	fileEvent, ok := event.(*events.FileDiscoveredEvent)
	if !ok {
		return nil
	}

	o.log.Debug().
		Str("path", fileEvent.Path()).
		Int("size", fileEvent.Size()).
		Msg("routing discovered file for parsing")

	// Emit event to trigger parsing (will be handled by format-specific
	// services)
	parseEvent := events.MustNewFileParseRequestedEvent(
		fileEvent.Path(),
		fileEvent.Content(),
		event.OccurredAt(),
	)

	return events.PublishSync(ctx, o.eventBus, parseEvent)
}

// handleNoteParsed routes successfully parsed notes to validation.
func (o *DataProcessingOrchestrator) handleNoteParsed(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	parsedEvent, ok := event.(*events.NoteParsedEvent)
	if !ok {
		return nil
	}

	o.log.Debug().
		Str("path", parsedEvent.AggregateID()).
		Msg("routing parsed note for validation")

	// Emit event to trigger validation
	validationEvent := events.MustNewFrontmatterValidationRequestedEvent(
		parsedEvent.Note(),
		event.OccurredAt(),
	)

	return events.PublishSync(ctx, o.eventBus, validationEvent)
}

// handleFrontmatterValidated routes validation results to caching or error
// handling.
func (o *DataProcessingOrchestrator) handleFrontmatterValidated(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	validationEvent, ok := event.(*domain.FrontmatterValidatedEvent)
	if !ok {
		return nil
	}

	if validationEvent.IsValid() {
		o.log.Debug().
			Str("path", validationEvent.NoteID()).
			Msg("routing validated note for caching")

		// Emit event to trigger caching
		cacheEvent := events.MustNewNoteCacheRequestedEvent(
			validationEvent.Note(),
			event.OccurredAt(),
		)

		return events.PublishSync(ctx, o.eventBus, cacheEvent)
	} else {
		o.log.Warn().
			Str("path", validationEvent.NoteID()).
			Strs("errors", validationEvent.ValidationErrors()).
			Msg("note validation failed, skipping caching")

		// Could emit failure event for monitoring/metrics
		return nil
	}
}
