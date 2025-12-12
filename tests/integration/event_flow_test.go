package integration

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	sharedlogger "github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestEventFlow_Integration tests the complete event-driven architecture
// end-to-end flow. This validates:
// - Event publishing and subscription
// - Concurrent event processing with multiple subscribers
// - Error isolation (failed handlers don't block others)
// - Event overhead performance (< 5ms per event).
func TestEventFlow_Integration(t *testing.T) {
	// Initialize logger
	log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())

	// Create EventBus
	eventBus := events.NewInMemoryEventBus(log)
	defer func() {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		defer cancel()
		require.NoError(t, eventBus.Shutdown(ctx))
	}()

	// Test event flow: publish events → multiple subscribers process
	ctx := context.Background()

	// Capture events for verification
	var eventsReceived []domain.DomainEvent
	var eventsMutex sync.Mutex

	// Subscribe to events for testing
	testHandler := func(ctx context.Context, event domain.DomainEvent) error {
		eventsMutex.Lock()
		defer eventsMutex.Unlock()
		eventsReceived = append(eventsReceived, event)
		return nil
	}

	require.NoError(t, eventBus.Subscribe("NoteIndexed", testHandler))
	require.NoError(t, eventBus.Subscribe("VaultIndexingComplete", testHandler))
	require.NoError(t, eventBus.Subscribe("FrontmatterValidated", testHandler))

	// Create and publish test events
	frontmatter := domain.NewFrontmatter(map[string]interface{}{
		"file_class": "test",
	})
	note, err := domain.NewNote(
		"/test/note.md",
		frontmatter,
		nil,
		nil,
		nil,
		nil,
	)
	require.NoError(t, err)

	noteEvent := domain.MustNewNoteIndexedEvent(note, time.Now())
	completeEvent := domain.MustNewVaultIndexingCompleteEvent(
		domain.VaultIndexingSummary{
			ScannedCount:        1,
			IndexedCount:        1,
			ParseFailures:       0,
			CacheFailures:       0,
			ValidationSuccesses: 1,
			ValidationFailures:  0,
		},
		100*time.Millisecond,
		time.Now(),
	)
	validationEvent := domain.MustNewFrontmatterValidatedEvent(
		"/test/note.md",
		"test-schema",
		true,
		[]string{},
		time.Now(),
	)

	// Measure performance: publish events
	start := time.Now()
	require.NoError(t, eventBus.Publish(ctx, noteEvent))
	require.NoError(t, eventBus.Publish(ctx, completeEvent))
	require.NoError(t, eventBus.Publish(ctx, validationEvent))
	duration := time.Since(start)

	// Wait for async event processing
	time.Sleep(100 * time.Millisecond)

	// Verify events were published and received
	eventsMutex.Lock()
	defer eventsMutex.Unlock()

	assert.Len(t, eventsReceived, 3, "Should receive all three events")

	// Check for expected event types
	hasNoteIndexed := false
	hasVaultComplete := false
	hasFrontmatterValidated := false

	for _, event := range eventsReceived {
		switch event.EventType() {
		case "NoteIndexed":
			hasNoteIndexed = true
			noteEvt := event.(*domain.NoteIndexedEvent)
			assert.Equal(t, "/test/note.md", noteEvt.Path(),
				"NoteIndexed should have correct path")
		case "VaultIndexingComplete":
			hasVaultComplete = true
			completeEvt := event.(*domain.VaultIndexingCompleteEvent)
			assert.Equal(
				t,
				1,
				completeEvt.NotesIndexed(),
				"Should have indexed one note",
			)
		case "FrontmatterValidated":
			hasFrontmatterValidated = true
			validationEvt := event.(*domain.FrontmatterValidatedEvent)
			assert.Equal(
				t,
				"test-schema",
				validationEvt.SchemaName(),
				"Validation should have correct schema name",
			)
			assert.True(
				t,
				validationEvt.IsValid(),
				"Validation should be valid",
			)
		}
	}

	assert.True(t, hasNoteIndexed, "Should receive NoteIndexed event")
	assert.True(
		t,
		hasVaultComplete,
		"Should receive VaultIndexingComplete event",
	)
	assert.True(
		t,
		hasFrontmatterValidated,
		"Should receive FrontmatterValidated event",
	)

	// Basic event flow test completed successfully
	// Additional concurrency and error isolation tests would be added here
	// but are omitted for this initial implementation to focus on core
	// functionality

	// Performance check: event overhead should be < 5ms per event
	// With 3 events, total overhead should be < 15ms
	assert.Less(
		t,
		duration,
		15*time.Millisecond,
		"Event overhead should be < 5ms per event",
	)
}

// TestConcurrentEventProcessing_Integration tests that multiple subscribers
// can process events concurrently without blocking each other.
func TestConcurrentEventProcessing_Integration(t *testing.T) {
	log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())
	eventBus := events.NewInMemoryEventBus(log, events.WithWorkerCount(10))
	defer func() {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		defer cancel()
		require.NoError(t, eventBus.Shutdown(ctx))
	}()

	ctx := context.Background()
	var wg sync.WaitGroup

	// Create multiple subscribers that process events concurrently
	const numSubscribers = 5
	const eventsPerSubscriber = 10
	totalEvents := numSubscribers * eventsPerSubscriber

	// Track events received by each subscriber
	receivedCounts := make([]int, numSubscribers)
	var countsMutex sync.Mutex

	// Subscribe multiple handlers
	for i := range numSubscribers {
		subscriberID := i
		handler := func(ctx context.Context, event domain.DomainEvent) error {
			countsMutex.Lock()
			receivedCounts[subscriberID]++
			countsMutex.Unlock()
			return nil
		}

		require.NoError(t, eventBus.Subscribe("NoteIndexed", handler))
	}

	// Publish events concurrently
	wg.Add(1)
	go func() {
		defer wg.Done()
		for range totalEvents {
			event := domain.MustNewNoteIndexedEvent(
				func() domain.Note {
					fm := domain.NewFrontmatter(
						map[string]interface{}{"file_class": "test"},
					)
					note, _ := domain.NewNote(
						"/test/note.md",
						fm,
						nil,
						nil,
						nil,
						nil,
					)
					return note
				}(),
				time.Now(),
			)
			_ = eventBus.Publish(ctx, event)
		}
	}()

	wg.Wait()

	// Wait for all events to be processed
	time.Sleep(200 * time.Millisecond)

	// Verify all subscribers received events
	countsMutex.Lock()
	defer countsMutex.Unlock()

	totalReceived := 0
	for i, count := range receivedCounts {
		assert.Positive(
			t,
			count,
			"Subscriber %d should have received events",
			i,
		)
		totalReceived += count
	}

	assert.Equal(
		t,
		totalEvents*numSubscribers,
		totalReceived,
		"All events should be processed by all subscribers",
	)
}

// TestErrorIsolation_Integration tests that failed event handlers don't
// block other subscribers from processing events.
func TestErrorIsolation_Integration(t *testing.T) {
	log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())
	eventBus := events.NewInMemoryEventBus(log)
	defer func() {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		defer cancel()
		require.NoError(t, eventBus.Shutdown(ctx))
	}()

	ctx := context.Background()

	// Track successful and failed handler executions
	var successCount, failureCount int
	var countersMutex sync.Mutex

	// Subscribe a handler that always fails
	require.NoError(
		t,
		eventBus.Subscribe(
			"NoteIndexed",
			func(ctx context.Context, event domain.DomainEvent) error {
				countersMutex.Lock()
				failureCount++
				countersMutex.Unlock()
				return assert.AnError // Always fail
			},
		),
	)

	// Subscribe handlers that succeed
	for range 3 {
		require.NoError(
			t,
			eventBus.Subscribe(
				"NoteIndexed",
				func(ctx context.Context, event domain.DomainEvent) error {
					countersMutex.Lock()
					successCount++
					countersMutex.Unlock()
					return nil
				},
			),
		)
	}

	// Publish multiple events
	for range 5 {
		event := domain.MustNewNoteIndexedEvent(
			func() domain.Note {
				fm := domain.NewFrontmatter(
					map[string]interface{}{"file_class": "test"},
				)
				note, _ := domain.NewNote(
					"/test/note.md",
					fm,
					nil,
					nil,
					nil,
					nil,
				)
				return note
			}(),
			time.Now(),
		)
		require.NoError(t, eventBus.Publish(ctx, event))
	}

	// Wait for processing
	time.Sleep(200 * time.Millisecond)

	// Verify error isolation: successful handlers should still process events
	countersMutex.Lock()
	defer countersMutex.Unlock()

	assert.Equal(
		t,
		5,
		failureCount,
		"Failing handler should be called for each event",
	)
	assert.Equal(
		t,
		15,
		successCount,
		"Successful handlers should process all events despite failures",
	)
}
