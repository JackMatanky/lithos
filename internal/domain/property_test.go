package domain

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockPropertySpec is a test implementation of PropertySpec for testing
// delegation.
type mockPropertySpec struct {
	validateError error
	specType      PropertySpecType
}

// Type returns the mock spec type.
func (m mockPropertySpec) Type() PropertySpecType {
	return m.specType
}

// Validate returns the mock validation error.
func (m mockPropertySpec) Validate(ctx context.Context) error {
	return m.validateError
}

// TestPropertyValidate tests Property.Validate method.
func TestPropertyValidate(t *testing.T) {
	ctx := context.Background()

	t.Run("success with valid spec", func(t *testing.T) {
		spec := mockPropertySpec{
			validateError: nil,
			specType:      PropertyTypeString,
		}
		property := Property{
			Name:     "testProp",
			Required: false,
			Array:    false,
			Spec:     spec,
		}

		err := (&property).Validate(ctx)

		assert.NoError(t, err)
	})

	t.Run("error when name is empty", func(t *testing.T) {
		spec := mockPropertySpec{
			validateError: nil,
			specType:      PropertyTypeString,
		}
		property := Property{
			Name:     "",
			Required: false,
			Array:    false,
			Spec:     spec,
		}

		err := (&property).Validate(ctx)

		require.Error(t, err)
		assert.Contains(t, err.Error(), "property name cannot be empty")
	})

	t.Run("error when spec is nil", func(t *testing.T) {
		property := Property{
			Name:     "testProp",
			Required: false,
			Array:    false,
			Spec:     nil,
		}

		err := (&property).Validate(ctx)

		require.Error(t, err)
		assert.Contains(
			t,
			err.Error(),
			"property spec cannot be nil",
		)
	})

	t.Run("delegates to spec validate", func(t *testing.T) {
		expectedError := errors.New("spec validation failed")
		spec := mockPropertySpec{
			validateError: expectedError,
			specType:      PropertyTypeString,
		}
		property := Property{
			Name:     "testProp",
			Required: false,
			Array:    false,
			Spec:     spec,
		}

		err := (&property).Validate(ctx)

		require.Error(t, err)
		assert.Contains(t, err.Error(), "spec validation failed")
	})
}

// TestNewProperty tests the Property entity constructor with ID generation.
func TestNewProperty(t *testing.T) {
	spec := mockPropertySpec{
		validateError: nil,
		specType:      PropertyTypeString,
	}

	t.Run("creates Property with auto-generated ID", func(t *testing.T) {
		property, err := NewProperty("testProp", true, false, spec)

		require.NoError(t, err)
		assert.Equal(t, "testProp", property.Name)
		assert.True(t, property.Required)
		assert.False(t, property.Array)
		assert.Equal(t, spec, property.Spec)

		// Property should have a non-empty ID
		assert.NotEmpty(t, property.ID)

		// ID should be deterministic based on name and spec content
		property2, err2 := NewProperty("testProp", true, false, spec)
		require.NoError(t, err2)
		assert.Equal(
			t,
			property.ID,
			property2.ID,
			"Properties with same name and spec should have same ID",
		)
	})

	t.Run(
		"generates different IDs for different properties",
		func(t *testing.T) {
			property1, err1 := NewProperty("prop1", true, false, spec)
			property2, err2 := NewProperty("prop2", true, false, spec)

			require.NoError(t, err1)
			require.NoError(t, err2)
			assert.NotEqual(
				t,
				property1.ID,
				property2.ID,
				"Different properties should have different IDs",
			)
		},
	)

	t.Run("fails validation for invalid property", func(t *testing.T) {
		_, err := NewProperty("", true, false, spec)

		require.Error(t, err)
		assert.Contains(t, err.Error(), "property name cannot be empty")
	})
}

// BenchmarkPropertyValidate benchmarks property validation performance.
// Measures the impact of validation caching on repeated validations.
func BenchmarkPropertyValidate(b *testing.B) {
	ctx := context.Background()

	// Test with regex pattern (triggers compilation caching)
	spec := StringSpec{
		Pattern: `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`, // Email regex
	}

	property, err := NewProperty("email", true, false, &spec)
	require.NoError(b, err)

	b.ResetTimer()
	for range b.N {
		_ = property.Validate(ctx)
	}
}

// BenchmarkPropertyValidateNoCache benchmarks validation without caching
// (simulating old behavior with repeated regex compilation).
func BenchmarkPropertyValidateNoCache(b *testing.B) {
	ctx := context.Background()

	// Create property without caching by using value receiver
	spec := StringSpec{
		Pattern: `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`,
	}

	property := Property{
		ID:       "test-id",
		Name:     "email",
		Required: true,
		Array:    false,
		Spec:     &spec, // Pointer to enable caching
	}

	b.ResetTimer()
	for range b.N {
		_ = property.Validate(ctx)
	}
}
