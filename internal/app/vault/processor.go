package vault

import (
	"context"
	"strings"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
)

// MarkdownProcessor handles complete processing pipeline for markdown files:
// parsing markdown content into Note entities and validating frontmatter
// against schemas.
//
// Responsibilities:
//   - Parse markdown files into structured Note entities
//   - Validate note frontmatter against schema definitions
//   - Publish events for successful parsing and validation
//   - Handle parsing and validation errors gracefully
//
// Architecture:
// - Subscribes to FileParseRequested and FrontmatterValidationRequested events
//   - Publishes NoteParsed and FrontmatterValidated events
//   - Consolidates markdown processing and validation into a single component
type MarkdownProcessor struct {
	markdownParser spi.MarkdownParserPort
	validator      *frontmatter.FrontmatterService
	eventBus       events.EventBus
	log            zerolog.Logger
}

// NewMarkdownProcessor creates a new processor for markdown files.
func NewMarkdownProcessor(
	markdownParser spi.MarkdownParserPort,
	validator *frontmatter.FrontmatterService,
	eventBus events.EventBus,
	log zerolog.Logger,
) *MarkdownProcessor {
	processor := &MarkdownProcessor{
		markdownParser: markdownParser,
		validator:      validator,
		eventBus:       eventBus,
		log:            log,
	}

	// Subscribe to processing events
	if eventBus != nil {
		_ = eventBus.Subscribe(
			"FileParseRequested",
			processor.handleFileParseRequested,
		)
		_ = eventBus.Subscribe(
			"FrontmatterValidationRequested",
			processor.handleValidationRequested,
		)
	}

	return processor
}

// ProcessFile processes a single file through complete pipeline:
// This is a convenience method for direct (non-event-driven) processing.
func (p *MarkdownProcessor) ProcessFile(
	ctx context.Context,
	path string,
	content []byte,
) (domain.Note, error) {
	// Parse markdown content
	note, err := p.markdownParser.ParseNote(ctx, path, content)
	if err != nil {
		p.log.Warn().
			Err(err).
			Str("path", path).
			Msg("failed to parse markdown")
		return domain.Note{}, err
	}

	// Validate frontmatter
	validationErr := p.validator.IsSchemaCompliant(
		ctx,
		note.Path,
		note.Frontmatter,
	)
	if validationErr != nil {
		p.log.Warn().
			Err(validationErr).
			Str("path", note.Path).
			Msg("frontmatter validation failed")
		return note, validationErr
	}

	return note, nil
	// handleFileParseRequested processes parse requests for markdown files.
}
func (p *MarkdownProcessor) handleFileParseRequested(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	parseEvent, ok := event.(*events.FileParseRequestedEvent)
	if !ok {
		return nil
	}

	// Only handle markdown files
	if !strings.HasSuffix(parseEvent.AggregateID(), ".md") {
		return nil // Skip non-markdown files
	}

	p.log.Debug().
		Str("path", parseEvent.AggregateID()).
		Msg("parsing markdown file")

	// Parse markdown content into a Note
	note, err := p.markdownParser.ParseNote(
		ctx,
		parseEvent.AggregateID(),
		parseEvent.Content(),
	)
	if err != nil {
		p.log.Warn().
			Err(err).
			Str("path", parseEvent.AggregateID()).
			Msg("failed to parse markdown")
		return err
	}

	// Emit successful parse event
	publishNoteParsed(ctx, p.eventBus, p.log, note, event.OccurredAt())
	return nil
}

// handleValidationRequested processes validation requests for complete notes.
func (p *MarkdownProcessor) handleValidationRequested(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	validationEvent, ok := event.(*events.FrontmatterValidationRequestedEvent)
	if !ok {
		return nil
	}

	note := validationEvent.Note()
	p.log.Debug().
		Str("path", note.Path).
		Msg("validating complete note entity")

	// Perform comprehensive note validation
	err := p.validator.IsSchemaCompliant(
		ctx,
		note.Path,
		note.Frontmatter,
	)

	var validationErrors []string
	isValid := err == nil
	if err != nil {
		validationErrors = []string{err.Error()}
	}

	// Emit validation result event with complete note
	publishFrontmatterValidated(
		ctx,
		p.eventBus,
		p.log,
		note,
		isValid,
		validationErrors,
		event.OccurredAt(),
	)
	return nil
}

// ProcessFile processes a single file through complete pipeline:
// parsing and validation.
// This is a convenience method for direct (non-event-driven) processing.

// ProcessFile processes a single file through complete pipeline:
// parsing and validation.
