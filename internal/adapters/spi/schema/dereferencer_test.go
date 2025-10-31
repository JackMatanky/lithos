package schema

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestPropertyDereferencer_DereferenceProperties tests successful $ref
// substitution.
func TestPropertyDereferencer_DereferenceProperties(t *testing.T) {
	dereferencer := NewPropertyDereferencer()

	// PropertyBank with standard property
	bank := domain.PropertyBank{
		Properties: map[string]domain.Property{
			"standard_title": {
				Name:     "standard_title",
				Required: true,
				Spec:     domain.StringSpec{},
			},
		},
	}

	// Mixed properties - some with $ref, some direct
	properties := []MixedProperty{
		{
			Property: nil,
			PropertyRef: &PropertyRef{
				Name: "title",
				Ref:  "standard_title",
			},
		},
		{
			Property: &domain.Property{
				Name:     "content",
				Required: false,
				Spec:     domain.StringSpec{},
			},
			PropertyRef: nil,
		},
	}

	result, err := dereferencer.DereferenceProperties(
		context.Background(),
		"test_schema",
		properties,
		bank,
	)

	require.NoError(t, err)
	require.Len(t, result, 2)

	// Verify $ref was substituted
	assert.Equal(t, "title", result[0].Name)
	assert.True(t, result[0].Required) // Should get Required from bank
	assert.Equal(t, domain.StringSpec{}, result[0].Spec)

	// Verify regular property unchanged
	assert.Equal(t, "content", result[1].Name)
	assert.False(t, result[1].Required)
}

// TestPropertyDereferencer_MissingRefError tests error handling for missing
// $ref.
func TestPropertyDereferencer_MissingRefError(t *testing.T) {
	dereferencer := NewPropertyDereferencer()

	// Empty PropertyBank
	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	// Property with invalid $ref
	properties := []MixedProperty{
		{
			Property: nil,
			PropertyRef: &PropertyRef{
				Name: "title",
				Ref:  "missing_property",
			},
		},
	}

	_, err := dereferencer.DereferenceProperties(
		context.Background(),
		"test_schema",
		properties,
		bank,
	)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "missing_property")
	assert.Contains(t, err.Error(), "test_schema")
	assert.Contains(t, err.Error(), "not found in property bank")
}

// TestPropertyDereferencer_EmptyProperties tests handling of empty property
// list.
func TestPropertyDereferencer_EmptyProperties(t *testing.T) {
	dereferencer := NewPropertyDereferencer()

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	result, err := dereferencer.DereferenceProperties(
		context.Background(),
		"test_schema",
		[]MixedProperty{},
		bank,
	)

	require.NoError(t, err)
	assert.Empty(t, result)
}

// TestPropertyDereferencer_NoRefsProperties tests handling properties without
// $refs.
func TestPropertyDereferencer_NoRefsProperties(t *testing.T) {
	dereferencer := NewPropertyDereferencer()

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	// Only regular properties, no refs
	properties := []MixedProperty{
		{
			Property: &domain.Property{
				Name:     "title",
				Required: true,
				Spec:     domain.StringSpec{},
			},
			PropertyRef: nil,
		},
		{
			Property: &domain.Property{
				Name:     "content",
				Required: false,
				Spec:     domain.StringSpec{},
			},
			PropertyRef: nil,
		},
	}

	result, err := dereferencer.DereferenceProperties(
		context.Background(),
		"test_schema",
		properties,
		bank,
	)

	require.NoError(t, err)
	require.Len(t, result, 2)

	// Properties should be unchanged
	assert.Equal(t, "title", result[0].Name)
	assert.True(t, result[0].Required)
	assert.Equal(t, "content", result[1].Name)
	assert.False(t, result[1].Required)
}

// TestPropertyDereferencer_ContextCancellation tests context cancellation
// handling.
func TestPropertyDereferencer_ContextCancellation(t *testing.T) {
	dereferencer := NewPropertyDereferencer()

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	// Create canceled context
	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	properties := []MixedProperty{
		{
			Property: &domain.Property{
				Name:     "title",
				Required: true,
				Spec:     domain.StringSpec{},
			},
			PropertyRef: nil,
		},
	}

	_, err := dereferencer.DereferenceProperties(
		ctx,
		"test_schema",
		properties,
		bank,
	)

	require.Error(t, err)
	assert.Equal(t, context.Canceled, err)
}
