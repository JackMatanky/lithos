package errors

import (
	"errors"
	"testing"
)

// TestWrap tests the Wrap helper function.
func TestWrap(t *testing.T) {
	t.Run("wrap with message", func(t *testing.T) {
		cause := errors.New("original error")
		wrapped := Wrap(cause, "additional context")
		expected := "additional context: original error"
		if wrapped.Error() != expected {
			t.Errorf("expected '%s', got '%s'", expected, wrapped.Error())
		}
		if !errors.Is(wrapped, cause) {
			t.Error("wrapped error should contain original cause")
		}
	})

	t.Run("wrap nil error", func(t *testing.T) {
		wrapped := Wrap(nil, "context")
		if wrapped != nil {
			t.Errorf("expected nil, got %v", wrapped)
		}
	})

	t.Run("unwrap preserves original", func(t *testing.T) {
		cause := errors.New("root cause")
		wrapped := Wrap(cause, "context")
		if !errors.Is(wrapped, cause) {
			t.Error("errors.Is should find root cause")
		}
	})
}

// TestWrapWithContext tests the WrapWithContext helper function.
func TestWrapWithContext(t *testing.T) {
	t.Run("wrap with formatted context", func(t *testing.T) {
		cause := errors.New("file not found")
		wrapped := WrapWithContext(cause, "failed to read %s", "config.yaml")
		expected := "failed to read config.yaml: file not found"
		if wrapped.Error() != expected {
			t.Errorf("expected '%s', got '%s'", expected, wrapped.Error())
		}
		if !errors.Is(wrapped, cause) {
			t.Error("wrapped error should contain original cause")
		}
	})

	t.Run("wrap with multiple args", func(t *testing.T) {
		cause := errors.New("connection failed")
		wrapped := WrapWithContext(
			cause,
			"database %s on %s failed",
			"query",
			"localhost",
		)
		expected := "database query on localhost failed: connection failed"
		if wrapped.Error() != expected {
			t.Errorf("expected '%s', got '%s'", expected, wrapped.Error())
		}
	})

	t.Run("wrap nil error with context", func(t *testing.T) {
		wrapped := WrapWithContext(nil, "context %s", "value")
		if wrapped != nil {
			t.Errorf("expected nil, got %v", wrapped)
		}
	})
}

// TestErrorsJoin tests errors.Join usage patterns.
func TestErrorsJoin(t *testing.T) {
	t.Run("join multiple errors", func(t *testing.T) {
		err1 := errors.New("first error")
		err2 := errors.New("second error")
		err3 := errors.New("third error")

		joined := errors.Join(err1, err2, err3)
		if joined == nil {
			t.Fatal("expected joined error, got nil")
		}

		// errors.Join creates a joined error that contains all errors
		// We can check if individual errors are contained
		if !errors.Is(joined, err1) {
			t.Error("joined error should contain first error")
		}
		if !errors.Is(joined, err2) {
			t.Error("joined error should contain second error")
		}
		if !errors.Is(joined, err3) {
			t.Error("joined error should contain third error")
		}
	})

	t.Run("join with nil errors", func(t *testing.T) {
		err1 := errors.New("real error")
		joined := errors.Join(err1, nil, nil)
		if joined == nil {
			t.Fatal("expected joined error, got nil")
		}
		if !errors.Is(joined, err1) {
			t.Error("joined error should contain real error")
		}
	})

	t.Run("join all nil", func(t *testing.T) {
		joined := errors.Join(nil, nil, nil)
		if joined != nil {
			t.Errorf("expected nil, got %v", joined)
		}
	})
}
