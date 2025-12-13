package errors

import (
	"errors"
	"testing"
)

const (
	testProperty = "prop"
	testResource = "file"
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
			t.Errorf("expected nil unwrap, got %v", err.Unwrap())
		}
	})

	t.Run("construction with cause", func(t *testing.T) {
		cause := errors.New("root cause")
		err := NewBaseError("wrapped message", cause)
		if err.Error() != "wrapped message: root cause" {
			t.Errorf("expected 'wrapped message: root cause', got '%s'",
				err.Error())
		}
		if !errors.Is(err.Unwrap(), cause) {
			t.Errorf("expected cause to be unwrapped, got %v", err.Unwrap())
		}
	})

	t.Run("errors.Is compatibility", func(t *testing.T) {
		baseErr := NewBaseError("base", nil)
		wrappedErr := NewBaseError("wrapped", baseErr)

		if !errors.Is(wrappedErr, baseErr) {
			t.Error("errors.Is should find base error in wrapped error")
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
	t.Run("construction", func(t *testing.T) {
		err := NewValidationError(testProperty, "reason", "val", nil)
		if err.Property() != testProperty {
			t.Errorf(
				"expected property '%s', got '%s'",
				testProperty,
				err.Property(),
			)
		}
		if err.Reason() != "reason" {
			t.Errorf("expected reason 'reason', got '%s'", err.Reason())
		}
		if err.Value() != "val" {
			t.Errorf("expected value 'val', got '%v'", err.Value())
		}
	})

	t.Run("with cause", func(t *testing.T) {
		baseErr := NewBaseError("base", nil)
		err := NewValidationError(
			"email",
			"invalid format",
			"invalid@",
			baseErr,
		)

		if err.Property() != "email" {
			t.Errorf("expected property 'email', got '%s'", err.Property())
		}
		if !errors.Is(err, baseErr) {
			t.Error("should unwrap to base error")
		}
	})

	t.Run("errors.As extraction", func(t *testing.T) {
		baseErr := NewBaseError("base", nil)
		valErr := NewValidationError(testProperty, "reason", "value", baseErr)

		var target *ValidationError
		if !errors.As(valErr, &target) {
			t.Error("errors.As should extract ValidationError")
		}
		if target.Property() != testProperty {
			t.Errorf("extracted property incorrect: %s", target.Property())
		}
	})
}

// TestResourceErrorConstruction tests basic ResourceError construction and
// accessors.
func TestResourceErrorConstruction(t *testing.T) {
	err := NewResourceError(
		testResource,
		"read",
		"/path/to/file",
		errors.New("permission denied"),
	)
	if err.Resource() != testResource {
		t.Errorf(
			"expected resource '%s', got '%s'",
			testResource,
			err.Resource(),
		)
	}
	if err.Operation() != "read" {
		t.Errorf("expected operation 'read', got '%s'", err.Operation())
	}
	if err.Target() != "/path/to/file" {
		t.Errorf("expected target '/path/to/file', got '%s'", err.Target())
	}
}

// TestResourceErrorWithDatabaseConnection tests ResourceError with database
// connection.
func TestResourceErrorWithDatabaseConnection(t *testing.T) {
	err := NewResourceError("database", "connect", "localhost:5432", nil)
	if err.Resource() != "database" {
		t.Errorf("expected resource 'database', got '%s'", err.Resource())
	}
	if err.Operation() != "connect" {
		t.Errorf("expected operation 'connect', got '%s'", err.Operation())
	}
}

// TestResourceErrorAsExtraction tests errors.As extraction for ResourceError.
func TestResourceErrorAsExtraction(t *testing.T) {
	baseErr := NewBaseError("base", nil)
	resErr := NewResourceError(testResource, "write", "/tmp/test", baseErr)

	var target *ResourceError
	if !errors.As(resErr, &target) {
		t.Error("errors.As should extract ResourceError")
	}
	if target.Resource() != testResource {
		t.Errorf("extracted resource incorrect: %s", target.Resource())
	}
	if target.Operation() != "write" {
		t.Errorf("extracted operation incorrect: %s", target.Operation())
	}
	if target.Target() != "/tmp/test" {
		t.Errorf("extracted target incorrect: %s", target.Target())
	}
}

// TestResourceErrorAPIEndpoint tests ResourceError for API endpoint operations.
func TestResourceErrorAPIEndpoint(t *testing.T) {
	err := NewResourceError("api", "call", "endpoint", nil)
	if err.Resource() != "api" {
		t.Errorf("expected resource 'api', got '%s'", err.Resource())
	}
	if err.Operation() != "call" {
		t.Errorf("expected operation 'call', got '%s'", err.Operation())
	}
	if err.Target() != "endpoint" {
		t.Errorf("expected target 'endpoint', got '%s'", err.Target())
	}
}

// TestErrorWrapping tests error wrapping and unwrapping across error types.
func TestErrorWrapping(t *testing.T) {
	rootCause := errors.New("connection refused")
	baseErr := NewBaseError("base level", rootCause)
	valErr := NewValidationError(testProperty, "reason", "value", baseErr)
	resErr := NewResourceError(testResource, "read", "/path", valErr)

	// Test that we can unwrap all the way to the root cause
	if !errors.Is(resErr, rootCause) {
		t.Error("should be able to find root cause through error chain")
	}

	// Test extraction of different error types from the chain
	var extractedVal *ValidationError
	if !errors.As(resErr, &extractedVal) {
		t.Error("should be able to extract ValidationError from chain")
	}
	if extractedVal.Property() != testProperty {
		t.Errorf(
			"extracted validation property incorrect: %s",
			extractedVal.Property(),
		)
	}

	var extractedRes *ResourceError
	if !errors.As(resErr, &extractedRes) {
		t.Error("should be able to extract ResourceError from chain")
	}
	if extractedRes.Resource() != testResource {
		t.Errorf("extracted resource incorrect: %s", extractedRes.Resource())
	}
}
