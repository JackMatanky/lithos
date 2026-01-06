package frontmatter

import (
	"context"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

// publishValidation publishes validation events to the event bus.
// It publishes both a general ValidationPerformedEvent and, if validation
// failed,
// a ValidationFailedEvent with remediation hints.
func publishValidation(
	ctx context.Context,
	bus events.EventBus,
	log zerolog.Logger,
	noteID string,
	fm domain.Frontmatter,
	validationErr error,
	duration time.Duration,
) {
	if bus == nil {
		return
	}
	schemaName := fm.FileClass()
	if strings.TrimSpace(noteID) == "" {
		noteID = "frontmatter/" + schemaName
	}
	if schemaName == "" {
		schemaName = "unknown"
	}
	messages := flattenErrors(validationErr)

	// Publish ValidationPerformedEvent (AC 4.6.2)
	event, err := events.NewValidationPerformedEvent(
		noteID,
		schemaName,
		validationErr == nil,
		duration,
		messages,
		time.Now(),
	)
	if err != nil {
		log.Error().
			Err(err).
			Msg("failed to create validation performed event")
		return
	}
	if publishErr := events.PublishSync(ctx, bus, event); publishErr != nil {
		log.Warn().
			Err(publishErr).
			Msg("failed to publish validation performed event")
	}

	// Publish ValidationFailedEvent with remediation hints (AC 4.6.10-11)
	if validationErr != nil {
		remediationHints := generateRemediationHints(validationErr)
		failedEvent, failErr := events.NewValidationFailedEvent(
			noteID,
			schemaName,
			messages,
			remediationHints,
			duration,
			time.Now(),
		)
		if failErr != nil {
			log.Error().
				Err(failErr).
				Msg("failed to create validation failed event")
			return
		}
		if publishErr := events.PublishSync(ctx, bus, failedEvent); publishErr != nil {
			log.Warn().
				Err(publishErr).
				Msg("failed to publish validation failed event")
		}
	}
}

// flattenErrors converts a potentially nested error (from errors.Join) into
// a flat slice of error messages.
func flattenErrors(err error) []string {
	if err == nil {
		return nil
	}
	type unwrapper interface{ Unwrap() []error }
	if u, ok := err.(unwrapper); ok {
		var result []string
		for _, inner := range u.Unwrap() {
			result = append(result, flattenErrors(inner)...)
		}
		return result
	}
	return []string{err.Error()}
}

// generateRemediationHints generates helpful remediation hints for validation
// errors based on common error patterns.
func generateRemediationHints(err error) []string {
	if err == nil {
		return nil
	}

	messages := flattenErrors(err)
	hints := make([]string, 0, len(messages))

	for _, msg := range messages {
		switch {
		case strings.Contains(msg, "required field missing"):
			hints = append(
				hints,
				"Add the missing required field to frontmatter",
			)
		case strings.Contains(msg, "file not found"):
			hints = append(
				hints,
				"Verify the file exists in vault or run 'lithos index' to rebuild cache",
			)
		case strings.Contains(msg, "ambiguous reference"):
			hints = append(
				hints,
				"Use full path instead of wikilink to resolve ambiguity",
			)
		case strings.Contains(msg, "field value is not an array"):
			hints = append(
				hints,
				"Change field to array format using YAML list syntax",
			)
		case strings.Contains(msg, "field value must not be an array"):
			hints = append(
				hints,
				"Remove array brackets and use single scalar value",
			)
		case strings.Contains(msg, "query service unavailable"):
			hints = append(
				hints,
				"Ensure QueryService is initialized before validation",
			)
		default:
			hints = append(
				hints,
				"Review schema definition for field constraints",
			)
		}
	}

	return hints
}
