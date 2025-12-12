package events_test

import (
	"context"
	"errors"
	"fmt"
	"sync/atomic"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	sharedlogger "github.com/JackMatanky/lithos/internal/shared/logger"
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

func TestEventBusPublishDeliveries(t *testing.T) {
	bus := newTestBus(t)

	var handled atomic.Int32
	handler := func(ctx context.Context, event domain.DomainEvent) error {
		handled.Add(1)
		return nil
	}

	require.NoError(t, bus.Subscribe("NoteIndexed", handler))
	event := domain.MustNewNoteIndexedEvent(
		newTestNote(t, "notes/test.md"),
		time.Now(),
	)
	require.NoError(t, bus.Publish(context.Background(), event))

	require.Eventually(t, func() bool {
		return handled.Load() == 1
	}, time.Second, 10*time.Millisecond)
}

func TestEventBusUnsubscribeStopsDelivery(t *testing.T) {
	bus := newTestBus(t)

	var handled atomic.Int32
	handler := func(ctx context.Context, event domain.DomainEvent) error {
		handled.Add(1)
		return nil
	}

	require.NoError(t, bus.Subscribe("NoteIndexed", handler))
	require.NoError(t, bus.Unsubscribe("NoteIndexed", handler))
	event := domain.MustNewNoteIndexedEvent(
		newTestNote(t, "notes/test.md"),
		time.Now(),
	)
	require.NoError(t, bus.Publish(context.Background(), event))

	time.Sleep(50 * time.Millisecond)
	require.Equal(t, int32(0), handled.Load())
}

func TestEventBusErrorIsolation(t *testing.T) {
	bus := newTestBus(t)

	var success atomic.Int32
	errFail := errors.New("handler failure")

	failingHandler := func(ctx context.Context, event domain.DomainEvent) error {
		return errFail
	}

	successHandler := func(ctx context.Context, event domain.DomainEvent) error {
		success.Add(1)
		return nil
	}

	require.NoError(t, bus.Subscribe("NoteIndexed", failingHandler))
	require.NoError(t, bus.Subscribe("NoteIndexed", successHandler))

	event := domain.MustNewNoteIndexedEvent(
		newTestNote(t, "notes/test.md"),
		time.Now(),
	)
	// AC 9: Failed handlers don't block other subscribers - Publish should
	// return nil
	err := bus.Publish(context.Background(), event)
	require.NoError(t, err, "Publish should not return handler errors")

	require.Eventually(t, func() bool {
		return success.Load() == 1
	}, time.Second, 10*time.Millisecond, "Successful handlers should still execute despite failures")
}

func TestEventBusConcurrentPublishes(t *testing.T) {
	bus := newTestBus(t)

	var handled atomic.Int32
	require.NoError(
		t,
		bus.Subscribe(
			"NoteIndexed",
			func(ctx context.Context, event domain.DomainEvent) error {
				handled.Add(1)
				return nil
			},
		),
	)

	ctx := context.Background()
	for i := range 25 {
		note := newTestNote(t, fmt.Sprintf("notes/test-%d.md", i))
		event := domain.MustNewNoteIndexedEvent(note, time.Now())
		go func(ev domain.DomainEvent) {
			_ = bus.Publish(ctx, ev)
		}(event)
	}

	require.Eventually(t, func() bool {
		return handled.Load() == 25
	}, 2*time.Second, 10*time.Millisecond)
}

func TestEventBusRejectsInvalidSubscriptions(t *testing.T) {
	bus := newTestBus(t)

	require.Error(
		t,
		bus.Subscribe(
			"",
			func(ctx context.Context, event domain.DomainEvent) error { return nil },
		),
	)
	require.Error(t, bus.Subscribe("NoteIndexed", nil))
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
