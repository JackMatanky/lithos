package schema

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestPropertyDefinitionErrorWrapsCause confirms schema errors retain their
// original cause.
func TestPropertyDefinitionErrorWrapsCause(t *testing.T) {
	base := errors.New("cause")
	err := propertyDefinitionError("message", "schema", "path", base)
	require.Error(t, err)
	require.ErrorIs(t, err, base)
	assert.Contains(t, err.Error(), "message")
}
