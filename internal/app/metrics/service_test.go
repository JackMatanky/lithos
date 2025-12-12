package metrics_test

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/metrics"
	"github.com/JackMatanky/lithos/internal/domain"
	sharedlogger "github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newTestBus(t *testing.T) events.EventBus {
	t.Helper()
	log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())
	bus := events.NewInMemoryEventBus(log)
	t.Cleanup(func() {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		defer cancel()
		require.NoError(t, bus.Shutdown(ctx))
	})
	return bus
}

func newTestNote(t *testing.T, path string) domain.Note {
	t.Helper()
	frontmatter := domain.NewFrontmatter(
		map[string]interface{}{"file_class": "contact"},
	)
	note, err := domain.NewNote(path, frontmatter, nil, nil, nil, nil)
	require.NoError(t, err)
	return note
}

func TestNewService(t *testing.T) {
	t.Run("creates service with nil event bus", func(t *testing.T) {
		log := sharedlogger.NewTest()
		svc := metrics.NewService(nil, log)

		require.NotNil(t, svc)
		successes, failures := svc.ValidationStats()
		assert.Equal(t, 0, successes)
		assert.Equal(t, 0, failures)
	})

	t.Run(
		"creates service and subscribes to FrontmatterValidated",
		func(t *testing.T) {
			bus := newTestBus(t)
			log := sharedlogger.NewTest()

			svc := metrics.NewService(bus, log)
			require.NotNil(t, svc)

			// Verify subscription by publishing event
			event, err := domain.NewFrontmatterValidatedEvent(
				"note1",
				"test_schema",
				true,
				nil,
				time.Now(),
			)
			require.NoError(t, err)
			err = bus.Publish(context.Background(), event)
			require.NoError(t, err)

			// Give async handler time to process
			time.Sleep(50 * time.Millisecond)

			successes, failures := svc.ValidationStats()
			assert.Equal(t, 1, successes)
			assert.Equal(t, 0, failures)
		},
	)
}

func TestService_ValidationStats(t *testing.T) {
	t.Run("returns zero counters initially", func(t *testing.T) {
		bus := newTestBus(t)
		log := sharedlogger.NewTest()

		svc := metrics.NewService(bus, log)
		successes, failures := svc.ValidationStats()

		assert.Equal(t, 0, successes)
		assert.Equal(t, 0, failures)
	})

	t.Run(
		"increments success counter for valid frontmatter",
		func(t *testing.T) {
			bus := newTestBus(t)
			log := sharedlogger.NewTest()

			svc := metrics.NewService(bus, log)

			event, err := domain.NewFrontmatterValidatedEvent(
				"note1",
				"test_schema",
				true,
				nil,
				time.Now(),
			)
			require.NoError(t, err)
			err = bus.Publish(context.Background(), event)
			require.NoError(t, err)

			time.Sleep(50 * time.Millisecond)

			successes, failures := svc.ValidationStats()
			assert.Equal(t, 1, successes)
			assert.Equal(t, 0, failures)
		},
	)

	t.Run(
		"increments failure counter for invalid frontmatter",
		func(t *testing.T) {
			bus := newTestBus(t)
			log := sharedlogger.NewTest()

			svc := metrics.NewService(bus, log)

			validationErrs := []string{"title is required", "status is invalid"}
			event, err := domain.NewFrontmatterValidatedEvent(
				"note2",
				"test_schema",
				false,
				validationErrs,
				time.Now(),
			)
			require.NoError(t, err)
			err = bus.Publish(context.Background(), event)
			require.NoError(t, err)

			time.Sleep(50 * time.Millisecond)

			successes, failures := svc.ValidationStats()
			assert.Equal(t, 0, successes)
			assert.Equal(t, 1, failures)
		},
	)

	t.Run("tracks multiple validations correctly", func(t *testing.T) {
		bus := newTestBus(t)
		log := sharedlogger.NewTest()

		svc := metrics.NewService(bus, log)

		// Publish 3 successes and 2 failures
		for range 3 {
			event, err := domain.NewFrontmatterValidatedEvent(
				"success",
				"test_schema",
				true,
				nil,
				time.Now(),
			)
			require.NoError(t, err)
			err = bus.Publish(context.Background(), event)
			require.NoError(t, err)
		}

		for range 2 {
			validationErrs := []string{"error"}
			event, err := domain.NewFrontmatterValidatedEvent(
				"failure",
				"test_schema",
				false,
				validationErrs,
				time.Now(),
			)
			require.NoError(t, err)
			err = bus.Publish(context.Background(), event)
			require.NoError(t, err)
		}

		time.Sleep(100 * time.Millisecond)

		successes, failures := svc.ValidationStats()
		assert.Equal(t, 3, successes)
		assert.Equal(t, 2, failures)
	})
}

func TestService_HandleFrontmatterValidated(t *testing.T) {
	t.Run("ignores non-FrontmatterValidated events", func(t *testing.T) {
		bus := newTestBus(t)
		log := sharedlogger.NewTest()

		svc := metrics.NewService(bus, log)

		// Publish different event type
		note := newTestNote(t, "path/note1.md")
		event, err := domain.NewNoteIndexedEvent(note, time.Now())
		require.NoError(t, err)
		err = bus.Publish(context.Background(), event)
		require.NoError(t, err)

		time.Sleep(50 * time.Millisecond)

		successes, failures := svc.ValidationStats()
		assert.Equal(t, 0, successes)
		assert.Equal(t, 0, failures)
	})

	t.Run("is thread-safe with concurrent events", func(t *testing.T) {
		log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())
		bus := events.NewInMemoryEventBus(log, events.WithWorkerCount(100))
		defer func() {
			ctx, cancel := context.WithTimeout(
				context.Background(),
				time.Second,
			)
			defer cancel()
			require.NoError(t, bus.Shutdown(ctx))
		}()

		svc := metrics.NewService(bus, sharedlogger.NewTest())

		const numGoroutines = 10
		const eventsPerGoroutine = 100
		var wg sync.WaitGroup

		// Publish events concurrently
		for range numGoroutines {
			wg.Add(1)
			go func() {
				defer wg.Done()
				for j := range eventsPerGoroutine {
					isValid := j%2 == 0 // Alternating valid/invalid
					var validationErrs []string
					if !isValid {
						validationErrs = []string{"error"}
					}
					event, err := domain.NewFrontmatterValidatedEvent(
						"note",
						"test_schema",
						isValid,
						validationErrs,
						time.Now(),
					)
					if err == nil {
						_ = bus.Publish(context.Background(), event)
					}
				}
			}()
		}

		wg.Wait()
		time.Sleep(200 * time.Millisecond) // Allow handlers to process

		successes, failures := svc.ValidationStats()
		totalEvents := numGoroutines * eventsPerGoroutine
		assert.Equal(
			t,
			totalEvents,
			successes+failures,
			"total events should match",
		)
		assert.Equal(t, totalEvents/2, successes, "half should be successes")
		assert.Equal(t, totalEvents/2, failures, "half should be failures")
	})
}

func TestService_ConcurrentStatsReads(t *testing.T) {
	t.Run(
		"allows concurrent reads while processing events",
		func(t *testing.T) {
			log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())
			bus := events.NewInMemoryEventBus(log, events.WithWorkerCount(100))
			defer func() {
				ctx, cancel := context.WithTimeout(
					context.Background(),
					time.Second,
				)
				defer cancel()
				require.NoError(t, bus.Shutdown(ctx))
			}()

			svc := metrics.NewService(bus, sharedlogger.NewTest())

			var wg sync.WaitGroup

			// Concurrent writers
			wg.Add(1)
			go func() {
				defer wg.Done()
				for range 100 {
					event, err := domain.NewFrontmatterValidatedEvent(
						"note",
						"test_schema",
						true,
						nil,
						time.Now(),
					)
					if err == nil {
						_ = bus.Publish(context.Background(), event)
					}
				}
			}()

			// Concurrent readers
			for range 10 {
				wg.Add(1)
				go func() {
					defer wg.Done()
					for range 50 {
						_, _ = svc.ValidationStats()
					}
				}()
			}

			wg.Wait()
			time.Sleep(100 * time.Millisecond)

			successes, failures := svc.ValidationStats()
			assert.Equal(t, 100, successes)
			assert.Equal(t, 0, failures)
		},
	)
}
