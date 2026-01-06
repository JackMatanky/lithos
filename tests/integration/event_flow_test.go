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

const (
	eventTypeNoteIndexed           = "NoteIndexed"
	eventTypeVaultIndexingComplete = "VaultIndexingComplete"
	eventTypeFrontmatterValidated  = "FrontmatterValidated"
)

// capturedEvents provides thread-safe event capture for testing.
type capturedEvents struct {
	eventBus events.EventBus
	received []domain.DomainEvent
	mutex    sync.Mutex
}

// threadSafeCounters provides thread-safe counter operations.
type threadSafeCounters struct {
	counts []int
	mutex  sync.Mutex
}

// isolationCounters tracks success and failure counts for error isolation
// testing.
type isolationCounters struct {
	successCount int
	failureCount int
	mutex        sync.Mutex
}

// setupTestEventBus creates a standard event bus for testing.
func setupTestEventBus(t *testing.T) events.EventBus {
	t.Helper()
	log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())
	return events.NewInMemoryEventBus(log)
}

// shutdownEventBus gracefully shuts down the event bus.
func shutdownEventBus(t *testing.T, eventBus events.EventBus) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	require.NoError(t, eventBus.Shutdown(ctx))
}

// setupEventCapture creates a thread-safe event capture mechanism.
func setupEventCapture(eventBus events.EventBus) *capturedEvents {
	return &capturedEvents{
		eventBus: eventBus,
	}
}

// addEvent safely adds an event to the captured list.
func (ce *capturedEvents) addEvent(event domain.DomainEvent) {
	ce.mutex.Lock()
	defer ce.mutex.Unlock()
	ce.received = append(ce.received, event)
}

// getEvents safely returns a copy of captured events.
func (ce *capturedEvents) getEvents() []domain.DomainEvent {
	ce.mutex.Lock()
	defer ce.mutex.Unlock()
	// Return a copy to prevent external modification
	eventList := make([]domain.DomainEvent, len(ce.received))
	copy(eventList, ce.received)
	return eventList
}

// subscribeToTestEvents subscribes test handlers to capture events.
func subscribeToTestEvents(
	t *testing.T,
	eventBus events.EventBus,
	capture *capturedEvents,
) {
	t.Helper()

	testHandler := func(ctx context.Context, event domain.DomainEvent) error {
		capture.addEvent(event)
		return nil
	}

	require.NoError(t, eventBus.Subscribe("NoteIndexed", testHandler))
	require.NoError(t, eventBus.Subscribe("VaultIndexingComplete", testHandler))
	require.NoError(t, eventBus.Subscribe("FrontmatterValidated", testHandler))
}

// createTestEvents creates a set of test events for verification.
func createTestEvents(t *testing.T) []domain.DomainEvent {
	t.Helper()

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
	now := time.Now()

	noteEvent := domain.MustNewNoteIndexedEvent(note, now)
	completeEvent := events.MustNewVaultIndexingCompleteEvent(
		events.VaultIndexingSummary{
			ScannedCount:        1,
			IndexedCount:        1,
			ParseFailures:       0,
			CacheFailures:       0,
			ValidationSuccesses: 1,
			ValidationFailures:  0,
		},
		100*time.Millisecond,
		now,
	)
	validationEvent := domain.MustNewFrontmatterValidatedEvent(
		domain.Note{Path: "/test/note.md"},
		"test-schema",
		true,
		[]string{},
		now,
	)

	return []domain.DomainEvent{noteEvent, completeEvent, validationEvent}
}

// publishTestEvents publishes events and measures performance.
func publishTestEvents(
	t *testing.T,
	ctx context.Context,
	eventBus events.EventBus,
	eventList []domain.DomainEvent,
) {
	t.Helper()
	for _, event := range eventList {
		require.NoError(t, eventBus.Publish(ctx, event))
	}
}

// timeEventPublishing measures the time to publish events.
func timeEventPublishing(
	t *testing.T,
	ctx context.Context,
	eventBus events.EventBus,
	eventList []domain.DomainEvent,
) time.Duration {
	t.Helper()
	start := time.Now()
	publishTestEvents(t, ctx, eventBus, eventList)
	return time.Since(start)
}

// verifyEventsReceived checks that all expected events were captured.
func verifyEventsReceived(
	t *testing.T,
	capture *capturedEvents,
	expectedEvents []domain.DomainEvent,
) {
	t.Helper()

	// Wait for async processing
	time.Sleep(100 * time.Millisecond)

	eventsReceived := capture.getEvents()
	assert.Len(
		t,
		eventsReceived,
		len(expectedEvents),
		"Should receive all expected events",
	)

	// Verify event types are present
	eventTypes := make(map[string]bool)
	for _, event := range eventsReceived {
		eventTypes[event.EventType()] = true
	}

	assert.True(
		t,
		eventTypes[eventTypeNoteIndexed],
		"Should receive NoteIndexed event",
	)
	assert.True(t, eventTypes[eventTypeVaultIndexingComplete],
		"Should receive VaultIndexingComplete event")
	assert.True(t, eventTypes[eventTypeFrontmatterValidated],
		"Should receive FrontmatterValidated event")

	// Verify specific event content
	verifyEventContent(t, eventsReceived)
}

// verifyEventContent checks the content of received events.
func verifyEventContent(t *testing.T, eventsReceived []domain.DomainEvent) {
	t.Helper()

	for _, event := range eventsReceived {
		switch event.EventType() {
		case eventTypeNoteIndexed:
			noteEvt := event.(*domain.NoteIndexedEvent)
			assert.Equal(
				t,
				"/test/note.md",
				noteEvt.Path(),
				"NoteIndexed should have correct path",
			)
		case eventTypeVaultIndexingComplete:
			completeEvt := event.(*events.VaultIndexingCompleteEvent)
			assert.Equal(
				t,
				1,
				completeEvt.NotesIndexed(),
				"Should have indexed one note",
			)
		case eventTypeFrontmatterValidated:
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
}

// TestEventFlow_Integration tests the complete event-driven architecture
// end-to-end flow. This validates:
// - Event publishing and subscription
// - Concurrent event processing with multiple subscribers
// - Error isolation (failed handlers don't block others)
// - Event overhead performance (< 5ms per event).
func TestEventFlow_Integration(t *testing.T) {
	t.Run("event publishing and subscription", testEventPublishing)
	t.Run("performance overhead", testEventPerformance)
}

// testEventPublishing verifies basic event flow: publish → subscribe → process.
func testEventPublishing(t *testing.T) {
	eventBus := setupTestEventBus(t)
	defer shutdownEventBus(t, eventBus)
	ctx := context.Background()

	// Set up event capture
	eventsReceived := setupEventCapture(eventBus)

	// Subscribe to test events
	subscribeToTestEvents(t, eventBus, eventsReceived)

	// Create and publish test events
	testEvents := createTestEvents(t)
	publishTestEvents(t, ctx, eventBus, testEvents)

	// Wait for async processing and verify
	verifyEventsReceived(t, eventsReceived, testEvents)
}

// testEventPerformance verifies event processing performance overhead.
func testEventPerformance(t *testing.T) {
	eventBus := setupTestEventBus(t)
	defer shutdownEventBus(t, eventBus)
	ctx := context.Background()

	testEvents := createTestEvents(t)

	// Measure event publishing performance
	duration := timeEventPublishing(t, ctx, eventBus, testEvents)

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
	t.Run(
		"multiple subscribers concurrent processing",
		testConcurrentSubscribers,
	)
}

// testConcurrentSubscribers verifies concurrent event processing across
// multiple subscribers.
func testConcurrentSubscribers(t *testing.T) {
	eventBus := setupConcurrentEventBus(t)
	defer shutdownEventBus(t, eventBus)
	ctx := context.Background()

	const numSubscribers = 5
	const eventsPerSubscriber = 10
	totalEvents := numSubscribers * eventsPerSubscriber

	// Set up concurrent subscribers
	receivedCounts := setupConcurrentSubscribers(t, eventBus, numSubscribers)

	// Publish events concurrently
	publishConcurrentEvents(t, ctx, eventBus, totalEvents)

	// Wait for processing and verify results
	verifyConcurrentProcessing(t, receivedCounts, numSubscribers, totalEvents)
}

// setupConcurrentEventBus creates an event bus optimized for concurrent
// processing.
func setupConcurrentEventBus(t *testing.T) events.EventBus {
	t.Helper()
	log := sharedlogger.NewZerologAdapter(sharedlogger.NewTest())
	return events.NewInMemoryEventBus(log, events.WithWorkerCount(10))
}

// setupConcurrentSubscribers creates multiple event subscribers with counters.
func setupConcurrentSubscribers(
	t *testing.T,
	eventBus events.EventBus,
	numSubscribers int,
) *threadSafeCounters {
	t.Helper()

	counters := &threadSafeCounters{
		counts: make([]int, numSubscribers),
	}

	for i := range numSubscribers {
		subscriberID := i
		handler := func(ctx context.Context, event domain.DomainEvent) error {
			counters.increment(subscriberID)
			return nil
		}
		require.NoError(t, eventBus.Subscribe("NoteIndexed", handler))
	}

	return counters
}

// increment safely increments a counter.
func (c *threadSafeCounters) increment(index int) {
	c.mutex.Lock()
	defer c.mutex.Unlock()
	c.counts[index]++
}

// getTotal safely returns the sum of all counters.
func (c *threadSafeCounters) getTotal() int {
	c.mutex.Lock()
	defer c.mutex.Unlock()
	total := 0
	for _, count := range c.counts {
		total += count
	}
	return total
}

// getCounts safely returns a copy of all counts.
func (c *threadSafeCounters) getCounts() []int {
	c.mutex.Lock()
	defer c.mutex.Unlock()
	counts := make([]int, len(c.counts))
	copy(counts, c.counts)
	return counts
}

// publishConcurrentEvents publishes the specified number of events
// concurrently.
func publishConcurrentEvents(t *testing.T, ctx context.Context,
	eventBus events.EventBus, totalEvents int) {
	t.Helper()

	var wg sync.WaitGroup
	wg.Add(1)

	go func() {
		defer wg.Done()
		for range totalEvents {
			event := createTestNoteEvent(t)
			_ = eventBus.Publish(ctx, event)
		}
	}()

	wg.Wait()
}

// createTestNoteEvent creates a standard note event for testing.
func createTestNoteEvent(t *testing.T) domain.DomainEvent {
	t.Helper()
	fm := domain.NewFrontmatter(map[string]interface{}{"file_class": "test"})
	note, _ := domain.NewNote("/test/note.md", fm, nil, nil, nil, nil)
	return domain.MustNewNoteIndexedEvent(note, time.Now())
}

// verifyConcurrentProcessing checks that all events were processed by all
// subscribers.
func verifyConcurrentProcessing(t *testing.T, counters *threadSafeCounters,
	numSubscribers, totalEvents int) {
	t.Helper()

	// Wait for async processing
	time.Sleep(200 * time.Millisecond)

	counts := counters.getCounts()
	totalReceived := counters.getTotal()

	// Verify each subscriber received events
	for i, count := range counts {
		assert.Positive(
			t,
			count,
			"Subscriber %d should have received events",
			i,
		)
	}

	// Verify total event processing
	assert.Equal(t,
		totalEvents*numSubscribers,
		totalReceived,
		"All events should be processed by all subscribers",
	)
}

// TestErrorIsolation_Integration tests that failed event handlers don't
// block other subscribers from processing events.
func TestErrorIsolation_Integration(t *testing.T) {
	t.Run("failed handlers don't block successful ones", testErrorIsolation)
}

// testErrorIsolation verifies that failed event handlers don't prevent
// successful processing.
func testErrorIsolation(t *testing.T) {
	eventBus := setupTestEventBus(t)
	defer shutdownEventBus(t, eventBus)
	ctx := context.Background()

	// Set up mixed handlers (some fail, some succeed)
	counters := setupErrorIsolationHandlers(t, eventBus)

	// Publish events to trigger processing
	publishTestEventsForIsolation(t, ctx, eventBus, 5)

	// Verify error isolation
	verifyErrorIsolation(t, counters, 5)
}

// setupErrorIsolationHandlers creates handlers that mix failures and successes.
func setupErrorIsolationHandlers(
	t *testing.T,
	eventBus events.EventBus,
) *isolationCounters {
	t.Helper()

	counters := &isolationCounters{}

	// One handler that always fails
	require.NoError(
		t,
		eventBus.Subscribe(
			"NoteIndexed",
			func(ctx context.Context, event domain.DomainEvent) error {
				counters.incrementFailures()
				return assert.AnError
			},
		),
	)

	// Three handlers that succeed
	for range 3 {
		require.NoError(
			t,
			eventBus.Subscribe(
				"NoteIndexed",
				func(ctx context.Context, event domain.DomainEvent) error {
					counters.incrementSuccesses()
					return nil
				},
			),
		)
	}

	return counters
}

// incrementSuccesses safely increments success counter.
func (ic *isolationCounters) incrementSuccesses() {
	ic.mutex.Lock()
	defer ic.mutex.Unlock()
	ic.successCount++
}

// incrementFailures safely increments failure counter.
func (ic *isolationCounters) incrementFailures() {
	ic.mutex.Lock()
	defer ic.mutex.Unlock()
	ic.failureCount++
}

// getCounts safely returns current counts.
func (ic *isolationCounters) getCounts() (successes, failures int) {
	ic.mutex.Lock()
	defer ic.mutex.Unlock()
	return ic.successCount, ic.failureCount
}

// publishTestEventsForIsolation publishes the specified number of events for
// isolation testing.
func publishTestEventsForIsolation(t *testing.T, ctx context.Context,
	eventBus events.EventBus, numEvents int) {
	t.Helper()

	for range numEvents {
		event := createTestNoteEvent(t)
		require.NoError(t, eventBus.Publish(ctx, event))
	}
}

// verifyErrorIsolation checks that failures don't prevent successful
// processing.
func verifyErrorIsolation(
	t *testing.T,
	counters *isolationCounters,
	numEvents int,
) {
	t.Helper()

	// Wait for async processing
	time.Sleep(200 * time.Millisecond)

	successes, failures := counters.getCounts()

	assert.Equal(t,
		numEvents,
		failures,
		"Failing handler should be called for each event",
	)
	assert.Equal(t,
		numEvents*3, // 3 successful handlers
		successes,
		"Successful handlers should process all events despite failures",
	)
}
