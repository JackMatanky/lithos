package errors

import (
	"errors"
	"testing"
)

// TestBaseError verifies BaseError construction, Error() method, and Unwrap()
// compatibility.
func TestBaseError(t *testing.T) {
	t.Run("construction without cause", func(t *testing.T) {
		err := NewBaseError("test message", nil)
		if err.Error() != "test message" {
			t.Errorf("expected 'test message', got '%s'", err.Error())
		}
		if err.Unwrap() != nil {
			t.Errorf("expected nil cause, got %v", err.Unwrap())
		}
	})

	t.Run("construction with cause", func(t *testing.T) {
		cause := errors.New("original error")
		err := NewBaseError("wrapped message", cause)
		expected := "wrapped message: original error"
		if err.Error() != expected {
			t.Errorf("expected '%s', got '%s'", expected, err.Error())
		}
		if !errors.Is(err.Unwrap(), cause) {
			t.Errorf("expected cause to be unwrapped correctly")
		}
	})

	t.Run("errors.Is compatibility", func(t *testing.T) {
		baseErr := NewBaseError("base", nil)
		wrappedErr := NewBaseError("wrapped", baseErr)

		if !errors.Is(wrappedErr, baseErr) {
			t.Error("errors.Is should find base error in chain")
		}
	})

	t.Run("errors.As compatibility", func(t *testing.T) {
		var target BaseError
		err := NewBaseError("test", nil)

		if !errors.As(err, &target) {
			t.Error("errors.As should extract BaseError")
		}
		if target.Error() != "test" {
			t.Errorf("extracted error message incorrect: %s", target.Error())
		}
	})
}

// TestValidationErrorConstruction tests ValidationError construction and
// accessors.
func TestValidationErrorConstruction(t *testing.T) {
	t.Run("with cause", func(t *testing.T) {
		err := NewValidationError(
			"name",
			"required",
			"John",
			errors.New("missing field"),
		)
		if err.Property() != "name" {
			t.Errorf("expected property 'name', got '%s'", err.Property())
		}
		if err.Reason() != "required" {
			t.Errorf("expected reason 'required', got '%s'", err.Reason())
		}
		if err.Value() != "John" {
			t.Errorf("expected value 'John', got '%v'", err.Value())
		}
		expectedMsg := "validation failed: missing field"
		if err.Error() != expectedMsg {
			t.Errorf("expected '%s', got '%s'", expectedMsg, err.Error())
		}
	})

	t.Run("without cause", func(t *testing.T) {
		err := NewValidationError("email", "invalid format", "invalid@", nil)
		if err.Property() != "email" {
			t.Errorf("expected property 'email', got '%s'", err.Property())
		}
		if err.Reason() != "invalid format" {
			t.Errorf("expected reason 'invalid format', got '%s'", err.Reason())
		}
		if err.Value() != "invalid@" {
			t.Errorf("expected value 'invalid@', got '%v'", err.Value())
		}
		if err.Error() != "validation failed" {
			t.Errorf("expected 'validation failed', got '%s'", err.Error())
		}
	})
}

// TestValidationErrorCompatibility tests ValidationError stdlib compatibility.
func TestValidationErrorCompatibility(t *testing.T) {
	t.Run("errors.Is compatibility", func(t *testing.T) {
		baseErr := NewBaseError("base", nil)
		valErr := NewValidationError("test", "reason", "value", baseErr)

		if !errors.Is(valErr, baseErr) {
			t.Error("errors.Is should find base error in chain")
		}
	})

	t.Run("errors.As compatibility", func(t *testing.T) {
		var target *ValidationError
		err := NewValidationError("prop", "reason", "val", nil)

		if !errors.As(err, &target) {
			t.Error("errors.As should extract ValidationError")
		}
		if target.Property() != "prop" { //nolint:goconst // test string
			t.Errorf("extracted property incorrect: %s", target.Property())
		}
	})
}

// TestResourceErrorConstruction tests ResourceError construction and accessors.
func TestResourceErrorConstruction(t *testing.T) {
	t.Run("with cause", func(t *testing.T) {
		err := NewResourceError(
			"file",
			"read",
			"/path/to/file",
			errors.New("permission denied"),
		)
		if err.Resource() != "file" { //nolint:goconst // test string
			t.Errorf("expected resource 'file', got '%s'", err.Resource())
		}
		if err.Operation() != "read" {
			t.Errorf("expected operation 'read', got '%s'", err.Operation())
		}
		if err.Target() != "/path/to/file" {
			t.Errorf("expected target '/path/to/file', got '%s'", err.Target())
		}
		expectedMsg := "resource operation failed: permission denied"
		if err.Error() != expectedMsg {
			t.Errorf("expected '%s', got '%s'", expectedMsg, err.Error())
		}
	})

	t.Run("without cause", func(t *testing.T) {
		err := NewResourceError("database", "connect", "localhost:5432", nil)
		if err.Resource() != "database" {
			t.Errorf("expected resource 'database', got '%s'", err.Resource())
		}
		if err.Operation() != "connect" {
			t.Errorf("expected operation 'connect', got '%s'", err.Operation())
		}
		if err.Target() != "localhost:5432" {
			t.Errorf("expected target 'localhost:5432', got '%s'", err.Target())
		}
		if err.Error() != "resource operation failed" {
			t.Errorf(
				"expected 'resource operation failed', got '%s'",
				err.Error(),
			)
		}
	})
}

// TestResourceErrorCompatibility tests ResourceError stdlib compatibility.
func TestResourceErrorCompatibility(t *testing.T) {
	t.Run("errors.Is compatibility", func(t *testing.T) {
		baseErr := NewBaseError("base", nil)
		resErr := NewResourceError("file", "write", "/tmp/test", baseErr)

		if !errors.Is(resErr, baseErr) {
			t.Error("errors.Is should find base error in chain")
		}
	})

	t.Run("errors.As compatibility", func(t *testing.T) {
		var target *ResourceError
		err := NewResourceError("api", "call", "endpoint", nil)

		if !errors.As(err, &target) {
			t.Error("errors.As should extract ResourceError")
		}
		if target.Resource() != "api" {
			t.Errorf("extracted resource incorrect: %s", target.Resource())
		}
	})
}

// TestErrorChainTraversal tests error chain traversal with multiple levels.
func TestErrorChainTraversal(t *testing.T) {
	t.Run("deep error chain", func(t *testing.T) {
		rootCause := errors.New("root cause")
		baseErr := NewBaseError("base level", rootCause)
		valErr := NewValidationError("prop", "reason", "value", baseErr)
		resErr := NewResourceError("file", "read", "/path", valErr)

		// Test errors.Is() traverses the entire chain
		if !errors.Is(resErr, rootCause) {
			t.Error("errors.Is should find root cause in deep chain")
		}
		if !errors.Is(resErr, baseErr) {
			t.Error("errors.Is should find base error in chain")
		}
		if !errors.Is(resErr, valErr) {
			t.Error("errors.Is should find validation error in chain")
		}

		// Test errors.As() extracts from chain
		var extractedVal *ValidationError
		if !errors.As(resErr, &extractedVal) {
			t.Error("errors.As should extract ValidationError from chain")
		}
		if extractedVal.Property() != "prop" {
			t.Errorf(
				"extracted ValidationError has wrong property: %s",
				extractedVal.Property(),
			)
		}

		var extractedRes *ResourceError
		if !errors.As(resErr, &extractedRes) {
			t.Error("errors.As should extract ResourceError from chain")
		}
		if extractedRes.Resource() != "file" {
			t.Errorf(
				"extracted ResourceError has wrong resource: %s",
				extractedRes.Resource(),
			)
		}
	})
}
