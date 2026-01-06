package vault

import (
	"context"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

// publishNoteParsed publishes NoteParsed events synchronously.
// This event is emitted after successfully parsing a markdown file into a Note.
func publishNoteParsed(
	ctx context.Context,
	bus events.EventBus,
	log zerolog.Logger,
	note domain.Note,
	occurredAt time.Time,
) {
	if bus == nil {
		return
	}
	event := events.MustNewNoteParsedEvent(note, occurredAt)
	if publishErr := events.PublishSync(ctx, bus, event); publishErr != nil {
		log.Warn().
			Err(publishErr).
			Str("path", note.Path).
			Msg("failed to publish note parsed event")
	}
}

// publishFrontmatterValidated publishes FrontmatterValidated events
// synchronously.
// This event is emitted after validating a note's frontmatter against its
// schema.
func publishFrontmatterValidated(
	ctx context.Context,
	bus events.EventBus,
	log zerolog.Logger,
	note domain.Note,
	isValid bool,
	validationErrors []string,
	occurredAt time.Time,
) {
	if bus == nil {
		return
	}
	event := domain.MustNewFrontmatterValidatedEvent(
		note,
		note.FileClass(),
		isValid,
		validationErrors,
		occurredAt,
	)
	if publishErr := events.PublishSync(ctx, bus, event); publishErr != nil {
		log.Warn().
			Err(publishErr).
			Str("path", note.Path).
			Msg("failed to publish frontmatter validated event")
	}
}
