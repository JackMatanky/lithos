package vault

import (
	"context"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

// NoteValidationService handles complete note entity validation.
// This service validates the entire note (including frontmatter as part of the
// note)
// against schema requirements, ensuring data integrity before caching.
//
// Responsibilities:
//   - Validate complete note entities against schema rules
//   - Handle semantic validation of note content and metadata
//   - Emit validation results for downstream processing
//
// Architecture:
//   - Subscribes to FrontmatterValidationRequestedEvent
//   - Publishes FrontmatterValidatedEvent with complete validation results
//   - Works with the note as a cohesive entity, not isolated frontmatter
type NoteValidationService struct {
	frontmatterService *frontmatter.FrontmatterService
	eventBus           events.EventBus
	log                zerolog.Logger
}

// NewNoteValidationService creates a new note validation service.
func NewNoteValidationService(
	frontmatterService *frontmatter.FrontmatterService,
	eventBus events.EventBus,
	log zerolog.Logger,
) *NoteValidationService {
	service := &NoteValidationService{
		frontmatterService: frontmatterService,
		eventBus:           eventBus,
		log:                log,
	}

	// Subscribe to validation requests
	if eventBus != nil {
		_ = eventBus.Subscribe(
			"FrontmatterValidationRequested",
			service.handleValidationRequested,
		)
	}

	return service
}

// handleValidationRequested processes validation requests for complete notes.
func (s *NoteValidationService) handleValidationRequested(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	validationEvent, ok := event.(*domain.FrontmatterValidationRequestedEvent)
	if !ok {
		return nil
	}

	note := validationEvent.Note()
	s.log.Debug().
		Str("path", note.Path).
		Msg("validating complete note entity")

	// Perform comprehensive note validation
	err := s.frontmatterService.IsSchemaCompliant(
		ctx,
		note.Path,
		note.Frontmatter,
	)

	var validationErrors []string
	isValid := err == nil
	if err != nil {
		validationErrors = []string{err.Error()}
	}

	// Emit validation result event with the complete note
	resultEvent := domain.MustNewFrontmatterValidatedEvent(
		note,
		note.FileClass(),
		isValid,
		validationErrors,
		event.OccurredAt(),
	)

	return s.eventBus.Publish(ctx, resultEvent)
}
