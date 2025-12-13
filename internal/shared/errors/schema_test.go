package errors

import (
	"errors"
	"testing"
)

// TestSchemaError tests SchemaError construction and functionality.
func TestSchemaError(t *testing.T) {
	t.Run("construction with cause", func(t *testing.T) {
		cause := errors.New("invalid property type")
		err := NewSchemaError("schema validation failed", "user-schema", cause)
		if err.SchemaName != "user-schema" {
			t.Errorf(
				"expected schemaName 'user-schema', got '%s'",
				err.SchemaName,
			)
		}
		expectedMsg := "schema validation failed: invalid property type"
		if err.Error() != expectedMsg {
			t.Errorf("expected '%s', got '%s'", expectedMsg, err.Error())
		}
	})

	t.Run("construction without cause", func(t *testing.T) {
		err := NewSchemaError("schema not found", "missing-schema", nil)
		if err.SchemaName != "missing-schema" {
			t.Errorf(
				"expected schemaName 'missing-schema', got '%s'",
				err.SchemaName,
			)
		}
		if err.Error() != "schema not found" {
			t.Errorf("expected 'schema not found', got '%s'", err.Error())
		}
	})

	t.Run("errors.Is compatibility", func(t *testing.T) {
		baseErr := NewBaseError("base", nil)
		schemaErr := NewSchemaError("schema failed", "test-schema", baseErr)

		if !errors.Is(schemaErr, baseErr) {
			t.Error("errors.Is should find base error in chain")
		}
	})

	t.Run("errors.As compatibility", func(t *testing.T) {
		var target *SchemaError
		err := NewSchemaError("test", "schema-123", nil)

		if !errors.As(err, &target) {
			t.Error("errors.As should extract SchemaError")
		}
		if target.SchemaName != "schema-123" {
			t.Errorf("extracted schemaName incorrect: %s", target.SchemaName)
		}
	})
}
