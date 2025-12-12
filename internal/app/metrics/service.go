package metrics

import (
	"context"
	"sync"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

// Service records domain-level metrics emitted via the event bus.
type Service struct {
	eventBus events.EventBus
	log      zerolog.Logger

	mu                  sync.RWMutex
	validationSuccesses int
	validationFailures  int
}

// NewService wires event subscriptions for metrics reporting.
func NewService(bus events.EventBus, log zerolog.Logger) *Service {
	svc := &Service{
		eventBus:            bus,
		log:                 log,
		mu:                  sync.RWMutex{},
		validationSuccesses: 0,
		validationFailures:  0,
	}
	if bus != nil {
		_ = bus.Subscribe(
			"FrontmatterValidated",
			svc.handleFrontmatterValidated,
		)
	}
	return svc
}

// ValidationStats returns the current validation success/failure counters.
func (s *Service) ValidationStats() (successes, failures int) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.validationSuccesses, s.validationFailures
}

func (s *Service) handleFrontmatterValidated(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	fmEvent, ok := event.(*domain.FrontmatterValidatedEvent)
	if !ok {
		return nil
	}

	s.mu.Lock()
	if fmEvent.IsValid() {
		s.validationSuccesses++
	} else {
		s.validationFailures++
	}
	s.mu.Unlock()

	s.log.Debug().
		Str("event_type", fmEvent.EventType()).
		Str("note_id", fmEvent.AggregateID()).
		Bool("valid", fmEvent.IsValid()).
		Msg("frontmatter validation event recorded")

	return nil
}
