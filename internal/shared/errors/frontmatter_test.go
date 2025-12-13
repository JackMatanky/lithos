package errors

import (
	"errors"
	"testing"
)

// TestFrontmatterError tests FrontmatterError construction and functionality.
func TestFrontmatterError(t *testing.T) {
	t.Run("construction with cause", func(t *testing.T) {
		cause := errors.New("invalid YAML syntax")
		err := NewFrontmatterError("frontmatter parsing failed", "tags", cause)
		if err.Field() != "tags" {
			t.Errorf("expected field 'tags', got '%s'", err.Field())
		}
		expectedMsg := "frontmatter parsing failed: invalid YAML syntax"
		if err.Error() != expectedMsg {
			t.Errorf("expected '%s', got '%s'", expectedMsg, err.Error())
		}
	})

	t.Run("construction without cause", func(t *testing.T) {
		err := NewFrontmatterError(
			"missing frontmatter delimiter",
			"title",
			nil,
		)
		if err.Field() != "title" {
			t.Errorf("expected field 'title', got '%s'", err.Field())
		}
		if err.Error() != "missing frontmatter delimiter" {
			t.Errorf(
				"expected 'missing frontmatter delimiter', got '%s'",
				err.Error(),
			)
		}
	})

	t.Run("errors.Is compatibility", func(t *testing.T) {
		baseErr := NewBaseError("base", nil)
		frontErr := NewFrontmatterError("frontmatter failed", "field", baseErr)

		if !errors.Is(frontErr, baseErr) {
			t.Error("errors.Is should find base error in chain")
		}
	})

	t.Run("errors.As compatibility", func(t *testing.T) {
		var target *FrontmatterError
		err := NewFrontmatterError("test", "test-field", nil)

		if !errors.As(err, &target) {
			t.Error("errors.As should extract FrontmatterError")
		}
		if target.Field() != "test-field" {
			t.Errorf("extracted field incorrect: %s", target.Field())
		}
	})
}
