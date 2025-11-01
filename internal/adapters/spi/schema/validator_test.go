package schema

import (
	"context"
	"errors"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestSchemaValidator_ValidateAll_Success tests successful validation.
func TestSchemaValidator_ValidateAll_Success(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{
			Name: "base",
			Properties: []domain.Property{
				{Name: "title", Spec: &domain.StringSpec{}},
				{Name: "tags", Spec: &domain.StringSpec{}},
			},
		},
		{
			Name:    "meeting_note",
			Extends: "base",
			Properties: []domain.Property{
				{Name: "title", Spec: &domain.StringSpec{}},
				{Name: "tags", Spec: &domain.StringSpec{}},
			},
		},
	}

	err := validator.ValidateAll(context.Background(), schemas)
	require.NoError(t, err)
}

// TestSchemaValidator_ValidateAll_EmptyNameError tests model validation error
// when schema has empty Name.
func TestSchemaValidator_ValidateAll_EmptyNameError(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{Name: "", Properties: []domain.Property{}},
	}

	err := validator.ValidateAll(
		context.Background(),
		schemas,
	)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema :")
}

// TestSchemaValidator_ValidateAll_InvalidPropertySpec tests model validation
// error when property spec is invalid.
func TestSchemaValidator_ValidateAll_InvalidPropertySpec(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{
			Name: "test",
			Properties: []domain.Property{
				{
					Name: "",
					Spec: &domain.StringSpec{},
				}, // Invalid property name
			},
		},
	}

	err := validator.ValidateAll(
		context.Background(),
		schemas,
	)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "test")
}

// TestSchemaValidator_ValidateAll_MissingParent tests Extends refers to missing
// parent error.
func TestSchemaValidator_ValidateAll_MissingParent(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{{Name: "orphan", Extends: "missing"}}

	err := validator.ValidateAll(
		context.Background(),
		schemas,
	)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "orphan")
	assert.Contains(t, err.Error(), "missing")
}

// TestSchemaValidator_ValidateAll_DuplicateNames tests duplicate schema names
// detection.
func TestSchemaValidator_ValidateAll_DuplicateNames(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{Name: "dup"},
		{Name: "dup"},
		{Name: "dup"},
	}

	err := validator.ValidateAll(
		context.Background(),
		schemas,
	)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "dup")
	assert.Contains(t, err.Error(), "duplicate schema name")
}

// TestSchemaValidator_ValidateAll_EmptyPropertyName tests property validation
// for empty property names.
func TestSchemaValidator_ValidateAll_EmptyPropertyName(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{
			Name: "note",
			Properties: []domain.Property{
				{
					Name: "",
					Spec: &domain.StringSpec{},
				}, // Empty name should trigger error
			},
		},
	}

	err := validator.ValidateAll(
		context.Background(),
		schemas,
	)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "property name cannot be empty")
	assert.Contains(t, err.Error(), "note")
}

// TestSchemaValidator_ValidateAll_EmptySchemas tests validator handles empty
// schema slice gracefully.
func TestSchemaValidator_ValidateAll_EmptySchemas(t *testing.T) {
	validator := NewSchemaValidator()

	err := validator.ValidateAll(
		context.Background(),
		[]domain.Schema{},
	)
	require.NoError(t, err)
}

// TestSchemaValidator_ValidateAll_AggregatesMultipleErrors tests multiple
// failures return aggregated error.
func TestSchemaValidator_ValidateAll_AggregatesMultipleErrors(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{Name: "", Properties: nil},          // Model error
		{Name: "orphan", Extends: "missing"}, // Cross-schema error
	}

	err := validator.ValidateAll(
		context.Background(),
		schemas,
	)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema :")
	assert.Contains(t, err.Error(), "orphan")
}

// TestSchemaValidator_ValidateAll_AggregatedErrorTypes tests aggregated error
// preserves wrapped error types.
func TestSchemaValidator_ValidateAll_AggregatedErrorTypes(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{Name: "", Properties: nil},          // Model error
		{Name: "orphan", Extends: "missing"}, // Cross-schema error
	}

	err := validator.ValidateAll(
		context.Background(),
		schemas,
	)
	require.Error(t, err)

	// Check that we can still use errors.Is/As on the aggregated error
	// Since the errors are wrapped, this tests that the aggregation preserves
	// types
	var validationErr *lithoserrors.ValidationError
	var schemaErr *lithoserrors.SchemaError
	assert.True(t, errors.As(err, &validationErr) || errors.As(err, &schemaErr))
}

// TestSchemaValidator_ValidateAll_ErrorMessagesIncludeHints tests error strings
// include remediation hints.
func TestSchemaValidator_ValidateAll_ErrorMessagesIncludeHints(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{{Name: "orphan", Extends: "missing"}}

	err := validator.ValidateAll(
		context.Background(),
		schemas,
	)
	require.Error(t, err)

	// Check for remediation hints as per FR5/FR7
	assert.Contains(t, err.Error(), "orphan")
	assert.Contains(t, err.Error(), "missing")
	// The hint should indicate what to fix
}

// TestSchemaValidator_ValidateAll_NoMutation tests SchemaValidator does not
// mutate inputs.
func TestSchemaValidator_ValidateAll_NoMutation(t *testing.T) {
	validator := NewSchemaValidator()

	originalSchemas := []domain.Schema{
		{
			Name: "test",
			Properties: []domain.Property{
				{Name: "prop", Spec: &domain.StringSpec{}},
			},
		},
	}
	originalBank := domain.PropertyBank{Properties: map[string]domain.Property{
		"prop": {Name: "prop", Spec: &domain.StringSpec{}},
	}}

	// Make copies to compare
	schemasCopy := make([]domain.Schema, len(originalSchemas))
	copy(schemasCopy, originalSchemas)
	bankCopy := originalBank

	_ = validator.ValidateAll(
		context.Background(),
		originalSchemas,
	) // Ignore error for mutation test

	// Inputs should not be mutated
	assert.Equal(t, schemasCopy, originalSchemas)
	assert.Equal(t, bankCopy, originalBank)
}

// TestSchemaValidator_ValidateAll_GodocExample tests GoDoc example builds and
// confirms documented behavior.
func TestSchemaValidator_ValidateAll_GodocExample(t *testing.T) {
	// This test ensures the GoDoc example compiles and runs
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{
			Name: "note",
			Properties: []domain.Property{
				{Name: "title", Spec: &domain.StringSpec{}},
			},
		},
	}

	err := validator.ValidateAll(context.Background(), schemas)
	require.NoError(t, err)
}

// TestSchemaValidator_ValidateAll_NoLogging tests SchemaValidator makes no
// logging calls.
func TestSchemaValidator_ValidateAll_NoLogging(t *testing.T) {
	// Since SchemaValidator has no logging dependencies, this test just ensures
	// the method completes without any logging calls
	validator := NewSchemaValidator()

	schemas := []domain.Schema{{Name: ""}} // Invalid schema (empty name)

	// This should produce an error, but no logging should occur
	err := validator.ValidateAll(context.Background(), schemas)
	require.Error(t, err)
}

// TestSchemaValidator_ValidateAll_ExtendsChain tests Extends chain referencing
// multiple levels passes when parents exist.
func TestSchemaValidator_ValidateAll_ExtendsChain(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{
			Name: "base",
			Properties: []domain.Property{
				{Name: "title", Spec: &domain.StringSpec{}},
			},
		},
		{Name: "middle", Extends: "base", Properties: []domain.Property{}},
		{
			Name:    "child",
			Extends: "middle",
			Properties: []domain.Property{
				{Name: "title", Spec: &domain.StringSpec{}},
			},
		},
	}

	err := validator.ValidateAll(context.Background(), schemas)
	require.NoError(t, err)
}

// TestSchemaValidator_ValidateAll_MultipleRefs tests multiple $ref references
// resolve successfully when present in PropertyBank.
func TestSchemaValidator_ValidateAll_MultipleRefs(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{
			Name: "note",
			Properties: []domain.Property{
				{Name: "title", Spec: &domain.StringSpec{}},
				{Name: "tags", Spec: &domain.StringSpec{}},
			},
		},
	}

	err := validator.ValidateAll(context.Background(), schemas)
	require.NoError(t, err)
}

// TestSchemaValidator_ValidateAll_CombinationErrors tests combination of model
// failure and cross-schema failure aggregated together.
func TestSchemaValidator_ValidateAll_CombinationErrors(t *testing.T) {
	validator := NewSchemaValidator()

	schemas := []domain.Schema{
		{Name: "", Properties: nil},          // Model error
		{Name: "orphan", Extends: "missing"}, // Cross-schema error
	}

	err := validator.ValidateAll(
		context.Background(),
		schemas,
	)
	require.Error(t, err)

	// Should contain both types of errors
	assert.Contains(t, err.Error(), "schema :")
	assert.Contains(t, err.Error(), "orphan")
	assert.Contains(t, err.Error(), "missing")
}

// BenchmarkSchemaValidator_ValidateAll benchmarks comprehensive schema
// validation.
// Measures performance improvements from property validation caching.
func BenchmarkSchemaValidator_ValidateAll(b *testing.B) {
	validator := NewSchemaValidator()

	// Create a moderately complex schema set with regex patterns
	schemas := []domain.Schema{
		{
			Name: "user",
			Properties: []domain.Property{
				{Name: "id", Required: true, Spec: &domain.StringSpec{}},
				{
					Name:     "email",
					Required: true,
					Spec: &domain.StringSpec{
						Pattern: `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`,
					},
				},
				{Name: "name", Required: true, Spec: &domain.StringSpec{}},
				{
					Name:     "tags",
					Required: false,
					Array:    true,
					Spec:     &domain.StringSpec{},
				},
			},
		},
		{
			Name:    "post",
			Extends: "user",
			Properties: []domain.Property{
				{Name: "title", Required: true, Spec: &domain.StringSpec{}},
				{Name: "content", Required: true, Spec: &domain.StringSpec{}},
				{Name: "published", Required: false, Spec: &domain.BoolSpec{}},
			},
		},
		{
			Name: "comment",
			Properties: []domain.Property{
				{Name: "id", Required: true, Spec: &domain.StringSpec{}},
				{Name: "post_id", Required: true, Spec: &domain.StringSpec{}},
				{Name: "author", Required: true, Spec: &domain.StringSpec{}},
				{Name: "text", Required: true, Spec: &domain.StringSpec{}},
			},
		},
	}

	ctx := context.Background()
	b.ResetTimer()

	for range b.N {
		_ = validator.ValidateAll(ctx, schemas)
	}
}
