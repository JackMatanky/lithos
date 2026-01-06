package template

import (
	"context"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/rs/zerolog"
)

// publishLookup publishes lookup performance events asynchronously.
func publishLookup(
	ctx context.Context,
	bus events.EventBus,
	log zerolog.Logger,
	basename string,
	resultCount int,
	duration time.Duration,
	lookupType string,
) {
	if bus == nil {
		return
	}
	event := events.MustNewLookupPerformedEvent(
		basename,
		resultCount,
		duration,
		lookupType,
		time.Now(),
	)
	events.PublishAsync(ctx, bus, log, event)
}

// publishQuery publishes query performance events asynchronously.
func publishQuery(
	ctx context.Context,
	bus events.EventBus,
	log zerolog.Logger,
	filter map[string]any,
	resultCount int,
	duration time.Duration,
) {
	if bus == nil {
		return
	}
	event := events.MustNewQueryPerformedEvent(
		filter,
		resultCount,
		duration,
		"frontmatter",
		time.Now(),
	)
	events.PublishAsync(ctx, bus, log, event)
}

// publishSchemaLookup publishes schema lookup events asynchronously.
func publishSchemaLookup(
	ctx context.Context,
	bus events.EventBus,
	log zerolog.Logger,
	noteID string,
	fileClass string,
	found bool,
	duration time.Duration,
) {
	if bus == nil {
		return
	}
	event := events.MustNewSchemaLookupEvent(
		noteID,
		fileClass,
		found,
		duration,
		time.Now(),
	)
	events.PublishAsync(ctx, bus, log, event)
}
