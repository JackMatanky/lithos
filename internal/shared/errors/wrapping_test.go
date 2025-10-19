package errors

import (
	"errors"
	"strings"
	"testing"
)

func TestWrap(t *testing.T) {
	originalErr := errors.New("original error")
	wrappedErr := Wrap(originalErr, "additional context")

	errStr := wrappedErr.Error()
	if !strings.Contains(errStr, "additional context") {
		t.Errorf("Wrap() error message should contain context: %s", errStr)
	}
	if !strings.Contains(errStr, "original error") {
		t.Errorf(
			"Wrap() error message should contain original error: %s",
			errStr,
		)
	}

	// Test that wrapped error still contains original
	if !errors.Is(wrappedErr, originalErr) {
		t.Error("Wrap() should preserve original error for errors.Is()")
	}
}

func TestWrapWithContext(t *testing.T) {
	originalErr := errors.New("original error")
	context := map[string]interface{}{
		"operation": "read",
		"path":      "/test/file",
		"attempt":   3,
	}
	wrappedErr := WrapWithContext(originalErr, context)

	errStr := wrappedErr.Error()
	expectedParts := []string{
		"operation=read",
		"path=/test/file",
		"attempt=3",
		"original error",
	}
	for _, part := range expectedParts {
		if !strings.Contains(errStr, part) {
			t.Errorf(
				"WrapWithContext() error message should contain %s: %s",
				part,
				errStr,
			)
		}
	}

	// Test that wrapped error still contains original
	if !errors.Is(wrappedErr, originalErr) {
		t.Error(
			"WrapWithContext() should preserve original error for errors.Is()",
		)
	}
}

func TestJoinErrors(t *testing.T) {
	err1 := errors.New("error 1")
	err2 := errors.New("error 2")
	joinedErr := JoinErrors(err1, err2)

	// Test that joined error contains both
	if !errors.Is(joinedErr, err1) {
		t.Error("JoinErrors() should contain first error")
	}
	if !errors.Is(joinedErr, err2) {
		t.Error("JoinErrors() should contain second error")
	}

	errStr := joinedErr.Error()
	if !strings.Contains(errStr, "error 1") ||
		!strings.Contains(errStr, "error 2") {
		t.Errorf("JoinErrors() should contain both error messages: %s", errStr)
	}
}
