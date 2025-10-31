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
	specType      PropertyType
}

// Type returns the mock spec type.
func (m mockPropertySpec) Type() PropertyType {
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

		err := property.Validate(ctx)

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

		err := property.Validate(ctx)

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

		err := property.Validate(ctx)

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

		err := property.Validate(ctx)

		require.Error(t, err)
		assert.Contains(t, err.Error(), "spec validation failed")
	})
}
