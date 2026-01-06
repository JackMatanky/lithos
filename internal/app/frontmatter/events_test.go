package frontmatter

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

const (
	eventValidationPerformed = "ValidationPerformed"
	eventValidationFailed    = "ValidationFailed"
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

func TestFlattenErrors(t *testing.T) {
	t.Run("NilError", func(t *testing.T) {
		require.Nil(t, flattenErrors(nil))
	})

	t.Run("SingleError", func(t *testing.T) {
		err := errors.New("test error")
		require.Equal(t, []string{"test error"}, flattenErrors(err))
	})

	t.Run("JoinedErrors", func(t *testing.T) {
		err1 := errors.New("error 1")
		err2 := errors.New("error 2")
		err := errors.Join(err1, err2)
		require.Equal(t, []string{"error 1", "error 2"}, flattenErrors(err))
	})

	t.Run("NestedJoinedErrors", func(t *testing.T) {
		err1 := errors.New("error 1")
		err2 := errors.New("error 2")
		err3 := errors.New("error 3")
		err := errors.Join(errors.Join(err1, err2), err3)
		require.Equal(
			t,
			[]string{"error 1", "error 2", "error 3"},
			flattenErrors(err),
		)
	})
}

func TestGenerateRemediationHints(t *testing.T) {
	t.Run("NilError", func(t *testing.T) {
		require.Nil(t, generateRemediationHints(nil))
	})

	t.Run("KnownErrors", func(t *testing.T) {
		errs := []error{
			errors.New("required field missing: title"),
			errors.New("file not found: path/to/file"),
			errors.New("ambiguous reference: ref"),
			errors.New("field value is not an array"),
			errors.New("field value must not be an array"),
			errors.New("query service unavailable"),
		}
		err := errors.Join(errs...)
		hints := generateRemediationHints(err)
		require.Len(t, hints, 6)
		require.Contains(t, hints[0], "Add the missing required field")
		require.Contains(t, hints[1], "Verify the file exists")
		require.Contains(t, hints[2], "Use full path instead of wikilink")
		require.Contains(t, hints[3], "Change field to array format")
		require.Contains(t, hints[4], "Remove array brackets")
		require.Contains(t, hints[5], "Ensure QueryService is initialized")
	})

	t.Run("UnknownError", func(t *testing.T) {
		err := errors.New("unknown problem")
		hints := generateRemediationHints(err)
		require.Len(t, hints, 1)
		require.Contains(t, hints[0], "Review schema definition")
	})
}

func TestPublishValidation(t *testing.T) {
	ctx := context.Background()
	bus := new(MockEventBus)
	log := zerolog.Nop()
	fm := domain.NewFrontmatter(map[string]any{"file_class": "test-schema"})
	noteID := "test-note"
	duration := 100 * time.Millisecond

	t.Run("SuccessValidation", func(t *testing.T) {
		bus.On("Publish", ctx, mock.MatchedBy(func(ev domain.DomainEvent) bool {
			return ev.EventType() == eventValidationPerformed
		})).Return(nil).Once()

		publishValidation(ctx, bus, log, noteID, fm, nil, duration)
		bus.AssertExpectations(t)
	})

	t.Run("FailedValidation", func(t *testing.T) {
		valErr := errors.New("validation failed")
		bus.On("Publish", ctx, mock.MatchedBy(func(ev domain.DomainEvent) bool {
			return ev.EventType() == eventValidationPerformed
		})).Return(nil).Once()
		bus.On("Publish", ctx, mock.MatchedBy(func(ev domain.DomainEvent) bool {
			return ev.EventType() == eventValidationFailed
		})).Return(nil).Once()

		publishValidation(ctx, bus, log, noteID, fm, valErr, duration)
		bus.AssertExpectations(t)
	})
}
