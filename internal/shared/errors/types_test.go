package errors

import (
	"errors"
	"testing"
)

const (
	testPropertyTitle        = "title"
	testSchemaArticle        = "article"
	testConstraintValidation = "validation"
)

func TestBaseError(t *testing.T) {
	cause := errors.New("root cause")
	err := NewBaseError("failure", cause)

	if err.Error() != "failure" {
		t.Fatalf("unexpected base error message: %s", err.Error())
	}
	if !errors.Is(err, cause) {
		t.Fatalf("base error should unwrap to the original cause")
	}
}

func TestValidationError(t *testing.T) {
	err := NewValidationError(testPropertyTitle, "cannot be empty", "")

	if err.Property() != testPropertyTitle {
		t.Fatalf("expected property to equal 'title'")
	}
	if err.Reason() != "cannot be empty" {
		t.Fatalf("unexpected reason: %s", err.Reason())
	}
	if err.Value() != "" {
		t.Fatalf("expected empty string value")
	}
	expected := "property '" + testPropertyTitle + "': cannot be empty (value: )"
	if err.Error() != expected {
		t.Fatalf("unexpected error string: %s", err.Error())
	}
}

func TestResourceError(t *testing.T) {
	cause := errors.New("disk full")
	err := NewResourceError("file", "write", "/vault/note.md", cause)

	if err.Resource() != "file" || err.Operation() != "write" ||
		err.Target() != "/vault/note.md" {
		t.Fatalf("resource metadata not preserved")
	}
	expected := "file write '/vault/note.md': disk full"
	if err.Error() != expected {
		t.Fatalf("unexpected resource error string: %s", err.Error())
	}
	if !errors.Is(err, cause) {
		t.Fatalf("resource error must unwrap to cause")
	}
}

func TestSchemaErrors(t *testing.T) {
	cause := errors.New("invalid enum values")
	err := NewSchemaError(testSchemaArticle, "invalid property bank", cause)

	if err.Schema() != testSchemaArticle {
		t.Fatalf("schema metadata missing")
	}
	if err.Domain() != domainSchema {
		t.Fatalf("expected schema domain, got %s", err.Domain())
	}
	if !errors.Is(err, cause) {
		t.Fatalf("schema error must wrap cause")
	}

	propErr := NewSchemaValidationError(
		testSchemaArticle,
		"summary",
		"must be <= 140 chars",
		"long value",
		nil,
	)
	if propErr.Schema() != testSchemaArticle {
		t.Fatalf("schema validation error missing schema metadata")
	}
	if propErr.Property() != "summary" {
		t.Fatalf("schema validation error missing property metadata")
	}
	if propErr.Domain() != domainSchema {
		t.Fatalf("expected schema domain for validation error")
	}

	notFound := NewSchemaNotFoundError("article")
	if notFound.Schema() != "article" {
		t.Fatalf("schema not found error missing schema metadata")
	}
	if notFound.Error() != "schema 'article' not found" {
		t.Fatalf("unexpected schema not found message: %s", notFound.Error())
	}
	if notFound.Domain() != domainSchema {
		t.Fatalf("expected schema domain for not found error")
	}
}

func TestFrontmatterErrors(t *testing.T) {
	missing := NewRequiredFieldError(testPropertyTitle)
	if missing.Field() != testPropertyTitle ||
		missing.ConstraintType() != "required" {
		t.Fatalf("missing field metadata incorrect")
	}
	if missing.Domain() != domainFrontmatter {
		t.Fatalf("expected frontmatter domain")
	}

	arrayMismatch := NewArrayConstraintError("tags", "foo", "array")
	if arrayMismatch.Field() != "tags" ||
		arrayMismatch.ConstraintType() != "array" {
		t.Fatalf("array constraint metadata incorrect")
	}
	expected := "field 'tags': must be an array (value: foo)"
	if arrayMismatch.Error() != expected {
		t.Fatalf("unexpected error string: %s", arrayMismatch.Error())
	}

	withCause := NewFieldValidationError(
		testPropertyTitle,
		"invalid casing",
		"VALUE",
		errors.New("boom"),
	)
	if withCause.Field() != testPropertyTitle ||
		withCause.ConstraintType() != testConstraintValidation {
		t.Fatalf("field validation metadata incorrect")
	}
	if withCause.Domain() != "frontmatter" {
		t.Fatalf("expected frontmatter domain for validation error")
	}
	if withCause.Reason() != "invalid casing" || withCause.Value() != "VALUE" {
		t.Fatalf("field validation reason/value not preserved")
	}

	propErr := NewPropertySpecError(
		testPropertyTitle,
		"VALUE",
		NewValidationError("title", "invalid casing", "VALUE"),
	)
	if propErr.Field() != "title" || propErr.ConstraintType() != "validation" {
		t.Fatalf("property spec conversion metadata incorrect")
	}
}

func TestTemplateError(t *testing.T) {
	err := NewTemplateError("header.tmpl", 5, "undefined placeholder", nil)
	if err.Template() != "header.tmpl" || err.Line() != 5 {
		t.Fatalf("template metadata incorrect")
	}
	expected := "template 'header.tmpl' line 5: undefined placeholder"
	if err.Error() != expected {
		t.Fatalf("unexpected template error string: %s", err.Error())
	}
}
