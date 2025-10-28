package errors

import (
	"errors"
	"testing"
)

// TestTemplateError tests TemplateError construction and functionality.
func TestTemplateError(t *testing.T) {
	t.Run("construction with cause", func(t *testing.T) {
		cause := errors.New("template syntax error")
		err := NewTemplateError(
			"failed to render template",
			"user-profile",
			cause,
		)
		if err.TemplateID() != "user-profile" {
			t.Errorf(
				"expected templateID 'user-profile', got '%s'",
				err.TemplateID(),
			)
		}
		expectedMsg := "failed to render template: template syntax error"
		if err.Error() != expectedMsg {
			t.Errorf("expected '%s', got '%s'", expectedMsg, err.Error())
		}
	})

	t.Run("construction without cause", func(t *testing.T) {
		err := NewTemplateError("template not found", "missing-template", nil)
		if err.TemplateID() != "missing-template" {
			t.Errorf(
				"expected templateID 'missing-template', got '%s'",
				err.TemplateID(),
			)
		}
		if err.Error() != "template not found" {
			t.Errorf("expected 'template not found', got '%s'", err.Error())
		}
	})

	t.Run("errors.Is compatibility", func(t *testing.T) {
		baseErr := NewBaseError("base", nil)
		templateErr := NewTemplateError("template failed", "test-id", baseErr)

		if !errors.Is(templateErr, baseErr) {
			t.Error("errors.Is should find base error in chain")
		}
	})

	t.Run("errors.As compatibility", func(t *testing.T) {
		var target *TemplateError
		err := NewTemplateError("test", "template-123", nil)

		if !errors.As(err, &target) {
			t.Error("errors.As should extract TemplateError")
		}
		if target.TemplateID() != "template-123" {
			t.Errorf("extracted templateID incorrect: %s", target.TemplateID())
		}
	})
}
