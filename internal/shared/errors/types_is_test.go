package errors

import (
	"errors"
	"testing"
)

func TestBaseError_Is(t *testing.T) {
	notFoundErr := ErrNotFound
	resourceErr := NewResourceError("cache", "read", "test.md", notFoundErr)

	t.Logf("notFoundErr: %v (type: %T)", notFoundErr, notFoundErr)
	t.Logf("resourceErr: %v (type: %T)", resourceErr, resourceErr)
	t.Logf(
		"errors.Is(resourceErr, notFoundErr): %v",
		errors.Is(resourceErr, notFoundErr),
	)

	if !errors.Is(resourceErr, notFoundErr) {
		t.Error("Expected errors.Is to return true for wrapped ErrNotFound")
	}
}
