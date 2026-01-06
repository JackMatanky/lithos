package events

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

// PublishAsync publishes an event asynchronously without blocking the caller.
// Errors during publishing are logged but not returned.
// This is the standard pattern for fire-and-forget event notifications.
func PublishAsync(
	ctx context.Context,
	bus EventBus,
	log zerolog.Logger,
	event domain.DomainEvent,
) {
	go func() {
		if err := bus.Publish(ctx, event); err != nil {
			log.Error().
				Err(err).
				Str("event_type", event.EventType()).
				Str("aggregate_id", event.AggregateID()).
				Msg("failed to publish event asynchronously")
		}
	}()
}

// PublishSync publishes an event synchronously and returns any error.
// Use this when the caller needs to handle publishing failures.
func PublishSync(
	ctx context.Context,
	bus EventBus,
	event domain.DomainEvent,
) error {
	return bus.Publish(ctx, event)
}
