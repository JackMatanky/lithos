package performance

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	sharedlogger "github.com/JackMatanky/lithos/internal/shared/logger"
)

// BenchmarkEventDispatchOverhead measures the performance overhead of event
// dispatch.
// Target: <5ms per event for the decoupling benefits to be worthwhile.
func BenchmarkEventDispatchOverhead(b *testing.B) {
	log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())
	eventBus := events.NewInMemoryEventBus(log)
	defer func() {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		defer cancel()
		_ = eventBus.Shutdown(ctx)
	}()

	ctx := context.Background()

	// Subscribe a handler that does minimal work
	_ = eventBus.Subscribe(
		"BenchmarkEvent",
		func(ctx context.Context, event domain.DomainEvent) error {
			// Minimal processing - just return
			return nil
		},
	)

	// Create a test event
	note := createEventBusTestNote()
	event := domain.MustNewNoteIndexedEvent(note, time.Now())

	b.ResetTimer()
	b.ReportAllocs()

	// Measure dispatch overhead
	for b.Loop() {
		err := eventBus.Publish(ctx, event)
		if err != nil {
			b.Fatal(err)
		}
	}
}

// BenchmarkConcurrentEventDispatch measures performance with multiple
// concurrent subscribers.
func BenchmarkConcurrentEventDispatch(b *testing.B) {
	log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())
	eventBus := events.NewInMemoryEventBus(log, events.WithWorkerCount(10))
	defer func() {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		defer cancel()
		_ = eventBus.Shutdown(ctx)
	}()

	ctx := context.Background()

	// Subscribe multiple handlers
	const numSubscribers = 5
	for range numSubscribers {
		_ = eventBus.Subscribe(
			"BenchmarkEvent",
			func(ctx context.Context, event domain.DomainEvent) error {
				// Simulate some processing time
				time.Sleep(100 * time.Microsecond)
				return nil
			},
		)
	}

	note := createEventBusTestNote()
	event := domain.MustNewNoteIndexedEvent(note, time.Now())

	b.ResetTimer()
	b.ReportAllocs()

	for b.Loop() {
		err := eventBus.Publish(ctx, event)
		if err != nil {
			b.Fatal(err)
		}
	}
}

// createEventBusTestNote creates a minimal note for event bus benchmarking.
func createEventBusTestNote() domain.Note {
	frontmatter := domain.NewFrontmatter(map[string]interface{}{
		"file_class": "benchmark",
	})
	note, _ := domain.NewNote(
		"/benchmark/note.md",
		frontmatter,
		nil,
		nil,
		nil,
		nil,
	)
	return note
}
