package vault

import (
	"context"
	"strings"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
)

// MarkdownProcessingService handles markdown-specific parsing and processing.
// This service is responsible for converting raw markdown files into structured
// Note entities that can be validated and cached.
//
// Responsibilities:
//   - Parse markdown frontmatter and content
//   - Convert raw files into domain Note objects
//   - Handle markdown-specific parsing errors
//
// Architecture:
//   - Subscribes to FileParseRequestedEvent
//   - Publishes NoteParsedEvent on success
//   - Focused on markdown format handling
type MarkdownProcessingService struct {
	markdownParserPort spi.MarkdownParserPort
	eventBus           events.EventBus
	log                zerolog.Logger
}

// NewMarkdownProcessingService creates a new markdown processing service.
func NewMarkdownProcessingService(
	markdownParserPort spi.MarkdownParserPort,
	eventBus events.EventBus,
	log zerolog.Logger,
) *MarkdownProcessingService {
	service := &MarkdownProcessingService{
		markdownParserPort: markdownParserPort,
		eventBus:           eventBus,
		log:                log,
	}

	// Subscribe to parse requests for markdown files
	if eventBus != nil {
		_ = eventBus.Subscribe(
			"FileParseRequested",
			service.handleFileParseRequested,
		)
	}

	return service
}

// handleFileParseRequested processes parse requests for markdown files.
func (s *MarkdownProcessingService) handleFileParseRequested(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	parseEvent, ok := event.(*events.FileParseRequestedEvent)
	if !ok {
		return nil
	}

	// Only handle markdown files (could be extended for other formats)
	if !strings.HasSuffix(parseEvent.AggregateID(), ".md") {
		return nil // Skip non-markdown files
	}

	s.log.Debug().
		Str("path", parseEvent.AggregateID()).
		Msg("parsing markdown file")

	// Parse the markdown content into a Note
	note, err := s.markdownParserPort.ParseNote(
		ctx,
		parseEvent.AggregateID(),
		parseEvent.Content(),
	)
	if err != nil {
		s.log.Warn().
			Err(err).
			Str("path", parseEvent.AggregateID()).
			Msg("failed to parse markdown")
		return err
	}

	// Emit successful parse event
	parsedEvent := events.MustNewNoteParsedEvent(note, event.OccurredAt())
	return events.PublishSync(ctx, s.eventBus, parsedEvent)
}
