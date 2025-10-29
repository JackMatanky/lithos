package domain

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestStringSpecType tests that StringSpec returns the correct type.
func TestStringSpecType(t *testing.T) {
	spec := StringSpec{}
	assert.Equal(t, PropertyTypeString, spec.Type())
}

// TestStringSpecValidate tests StringSpec validation logic.
func TestStringSpecValidate(t *testing.T) {
	ctx := context.Background()

	t.Run("valid with no pattern", func(t *testing.T) {
		spec := StringSpec{}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("valid with valid pattern", func(t *testing.T) {
		spec := StringSpec{Pattern: "^[a-z]+$"}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("invalid with invalid regex pattern", func(t *testing.T) {
		spec := StringSpec{Pattern: "[invalid"}
		err := spec.Validate(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "invalid pattern regex")
	})
}

// TestNumberSpecType tests that NumberSpec returns the correct type.
func TestNumberSpecType(t *testing.T) {
	spec := NumberSpec{}
	assert.Equal(t, PropertyTypeNumber, spec.Type())
}

// TestNumberSpecValidate tests NumberSpec validation logic.
func TestNumberSpecValidate(t *testing.T) {
	ctx := context.Background()

	t.Run("valid with no constraints", func(t *testing.T) {
		spec := NumberSpec{}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("valid with Min and Max", func(t *testing.T) {
		minVal := 1.0
		maxVal := 10.0
		spec := NumberSpec{Min: &minVal, Max: &maxVal}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("valid with Step", func(t *testing.T) {
		step := 0.5
		spec := NumberSpec{Step: &step}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("invalid Min > Max", func(t *testing.T) {
		minVal := 10.0
		maxVal := 1.0
		spec := NumberSpec{Min: &minVal, Max: &maxVal}
		err := spec.Validate(ctx)
		require.Error(t, err)
		assert.Contains(
			t,
			err.Error(),
			"min (10.000000) cannot be greater than max (1.000000)",
		)
	})

	t.Run("invalid Step <= 0", func(t *testing.T) {
		step := 0.0
		spec := NumberSpec{Step: &step}
		err := spec.Validate(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "step must be positive, got 0.000000")
	})

	t.Run("invalid negative Step", func(t *testing.T) {
		step := -1.0
		spec := NumberSpec{Step: &step}
		err := spec.Validate(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "step must be positive, got -1.000000")
	})
}

// TestBoolSpecType tests that BoolSpec returns the correct type.
func TestBoolSpecType(t *testing.T) {
	spec := BoolSpec{}
	assert.Equal(t, PropertyTypeBool, spec.Type())
}

// TestBoolSpecValidate tests BoolSpec validation logic.
func TestBoolSpecValidate(t *testing.T) {
	ctx := context.Background()

	spec := BoolSpec{}
	err := spec.Validate(ctx)
	assert.NoError(t, err)
}

// TestDateSpecType tests that DateSpec returns the correct type.
func TestDateSpecType(t *testing.T) {
	spec := DateSpec{}
	assert.Equal(t, PropertyTypeDate, spec.Type())
}

// TestDateSpecValidate tests DateSpec validation logic.
func TestDateSpecValidate(t *testing.T) {
	ctx := context.Background()

	t.Run("valid with empty format defaults to RFC3339", func(t *testing.T) {
		spec := DateSpec{}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("valid with RFC3339 format", func(t *testing.T) {
		spec := DateSpec{Format: "2006-01-02T15:04:05Z07:00"}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("valid with custom format", func(t *testing.T) {
		spec := DateSpec{Format: "2006-01-02"}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("accepts any format string", func(t *testing.T) {
		spec := DateSpec{Format: "invalid"}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})
}

// TestFileSpecType tests that FileSpec returns the correct type.
func TestFileSpecType(t *testing.T) {
	spec := FileSpec{}
	assert.Equal(t, PropertyTypeFile, spec.Type())
}

// TestFileSpecValidate tests FileSpec validation logic.
func TestFileSpecValidate(t *testing.T) {
	ctx := context.Background()

	t.Run("valid with no patterns", func(t *testing.T) {
		spec := FileSpec{}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("valid with valid FileClass pattern", func(t *testing.T) {
		spec := FileSpec{FileClass: "notes"}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("valid with valid Directory pattern", func(t *testing.T) {
		spec := FileSpec{Directory: "vault"}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("valid with negation prefix", func(t *testing.T) {
		spec := FileSpec{FileClass: "^temp"}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("invalid FileClass regex", func(t *testing.T) {
		spec := FileSpec{FileClass: "[invalid"}
		err := spec.Validate(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "invalid fileClass pattern")
	})

	t.Run("invalid Directory regex", func(t *testing.T) {
		spec := FileSpec{Directory: "[invalid"}
		err := spec.Validate(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "invalid directory pattern")
	})

	t.Run("both patterns valid", func(t *testing.T) {
		spec := FileSpec{FileClass: "notes", Directory: "vault"}
		err := spec.Validate(ctx)
		assert.NoError(t, err)
	})

	t.Run("FileClass invalid, Directory valid", func(t *testing.T) {
		spec := FileSpec{FileClass: "[invalid", Directory: "vault"}
		err := spec.Validate(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "invalid fileClass pattern")
	})
}
