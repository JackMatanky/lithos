package converters

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestToSlice(t *testing.T) {
	t.Run("AlreadyAnySlice", func(t *testing.T) {
		input := []any{1, "two", true}
		result, ok := ToSlice(input)
		assert.True(t, ok)
		assert.Equal(t, input, result)
	})

	t.Run("StringSlice", func(t *testing.T) {
		input := []string{"one", "two"}
		result, ok := ToSlice(input)
		assert.True(t, ok)
		assert.Equal(t, []any{"one", "two"}, result)
	})

	t.Run("UnsupportedType", func(t *testing.T) {
		result, ok := ToSlice(123)
		assert.False(t, ok)
		assert.Nil(t, result)
	})

	t.Run("Nil", func(t *testing.T) {
		result, ok := ToSlice(nil)
		assert.False(t, ok)
		assert.Nil(t, result)
	})
}

func TestIsSlice(t *testing.T) {
	assert.True(t, IsSlice([]any{}))
	assert.True(t, IsSlice([]string{}))
	assert.False(t, IsSlice(123))
	assert.False(t, IsSlice("not a slice"))
	assert.False(t, IsSlice(nil))
}
