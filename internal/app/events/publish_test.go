package events_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
)

// MockEventBus is a mock implementation of the EventBus interface.
type MockEventBus struct {
	mock.Mock
}

// MockEvent is a mock implementation of the DomainEvent interface.
type MockEvent struct {
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

func (m *MockEvent) EventType() string {
	args := m.Called()
	return args.String(0)
}

func (m *MockEvent) OccurredAt() time.Time {
	args := m.Called()
	return args.Get(0).(time.Time)
}

func (m *MockEvent) AggregateID() string {
	args := m.Called()
	return args.String(0)
}

func TestPublishSync(t *testing.T) {
	ctx := context.Background()
	bus := new(MockEventBus)
	event := new(MockEvent)

	t.Run("Success", func(t *testing.T) {
		bus.On("Publish", ctx, event).Return(nil).Once()
		err := events.PublishSync(ctx, bus, event)
		require.NoError(t, err)
		bus.AssertExpectations(t)
	})

	t.Run("Failure", func(t *testing.T) {
		testErr := errors.New("publish failed")
		bus.On("Publish", ctx, event).Return(testErr).Once()
		err := events.PublishSync(ctx, bus, event)
		require.ErrorIs(t, err, testErr)
		bus.AssertExpectations(t)
	})
}

func TestPublishAsync(t *testing.T) {
	ctx := context.Background()
	bus := new(MockEventBus)
	event := new(MockEvent)
	log := zerolog.Nop()

	t.Run("DispatchesToGoroutine", func(t *testing.T) {
		// We use a channel to wait for the async call to complete
		done := make(chan struct{})
		bus.On("Publish", ctx, event).Run(func(args mock.Arguments) {
			close(done)
		}).Return(nil).Once()

		events.PublishAsync(ctx, bus, log, event)

		select {
		case <-done:
			// Success
		case <-time.After(time.Second):
			t.Fatal("timeout waiting for async publish")
		}
		bus.AssertExpectations(t)
	})
}
