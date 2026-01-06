package template

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/mock"
)

const (
	testNoteID = "test-note"
)

// MockEventBus is a mock implementation of the EventBus interface.
type MockEventBus struct {
	mock.Mock
}

func (m *MockEventBus) Publish(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	args := m.Called(ctx, event)
	return args.Error(0)
}

func (m *MockEventBus) Subscribe(
	eventType string,
	handler events.EventHandler,
) error {
	args := m.Called(eventType, handler)
	return args.Error(0)
}

func (m *MockEventBus) Unsubscribe(
	eventType string,
	handler events.EventHandler,
) error {
	args := m.Called(eventType, handler)
	return args.Error(0)
}

func (m *MockEventBus) Shutdown(ctx context.Context) error {
	args := m.Called(ctx)
	return args.Error(0)
}

func TestPublishLookup(t *testing.T) {
	ctx := context.Background()
	bus := new(MockEventBus)
	log := zerolog.Nop()
	basename := testNoteID
	resultCount := 1
	duration := 100 * time.Millisecond
	lookupType := "basename"

	bus.On("Publish", ctx, mock.MatchedBy(func(ev domain.DomainEvent) bool {
		return ev.EventType() == "LookupPerformed"
	})).Return(nil).Once()

	publishLookup(ctx, bus, log, basename, resultCount, duration, lookupType)

	// Wait for async publish
	time.Sleep(50 * time.Millisecond)
	bus.AssertExpectations(t)
}

func TestPublishQuery(t *testing.T) {
	ctx := context.Background()
	bus := new(MockEventBus)
	log := zerolog.Nop()
	filter := map[string]any{"author": "John"}
	resultCount := 5
	duration := 200 * time.Millisecond

	bus.On("Publish", ctx, mock.MatchedBy(func(ev domain.DomainEvent) bool {
		return ev.EventType() == "QueryPerformed"
	})).Return(nil).Once()

	publishQuery(ctx, bus, log, filter, resultCount, duration)

	// Wait for async publish
	time.Sleep(50 * time.Millisecond)
	bus.AssertExpectations(t)
}

func TestPublishSchemaLookup(t *testing.T) {
	ctx := context.Background()
	bus := new(MockEventBus)
	log := zerolog.Nop()
	noteID := testNoteID
	fileClass := testContactClass
	found := true
	duration := 50 * time.Millisecond

	bus.On("Publish", ctx, mock.MatchedBy(func(ev domain.DomainEvent) bool {
		return ev.EventType() == "SchemaLookup"
	})).Return(nil).Once()

	publishSchemaLookup(ctx, bus, log, noteID, fileClass, found, duration)

	// Wait for async publish
	time.Sleep(50 * time.Millisecond)
	bus.AssertExpectations(t)
}
