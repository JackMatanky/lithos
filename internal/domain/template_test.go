package domain

import (
	"reflect"
	"testing"
)

const (
	testTemplateValue    = "test-template"
	testContactHeader    = "contact-header"
	testTemplate1        = "template1"
	testTemplate2        = "template2"
	templateTestContent1 = "content1"
	templateTestContent2 = "content2"
)

// TestNewTemplateID tests the NewTemplateID constructor creates a valid
// TemplateID instance.
func TestNewTemplateID(t *testing.T) {
	id := NewTemplateID(testTemplateValue)
	if id.String() != testTemplateValue {
		t.Errorf("expected '%s', got %s", testTemplateValue, id.String())
	}
}

// TestTemplateIDString tests the String method returns the underlying value.
func TestTemplateIDString(t *testing.T) {
	id := NewTemplateID(testContactHeader)
	result := id.String()
	if result != testContactHeader {
		t.Errorf("expected '%s', got %s", testContactHeader, result)
	}
}

// TestTemplateIDAsMapKey tests that TemplateID can be used as a map key.
func TestTemplateIDAsMapKey(t *testing.T) {
	id1 := NewTemplateID(testTemplate1)
	id2 := NewTemplateID(testTemplate2)

	templateMap := make(map[TemplateID]string)
	templateMap[id1] = templateTestContent1
	templateMap[id2] = templateTestContent2

	if templateMap[id1] != templateTestContent1 {
		t.Errorf(
			"expected '%s', got %s",
			templateTestContent1,
			templateMap[id1],
		)
	}
	if templateMap[id2] != templateTestContent2 {
		t.Errorf(
			"expected '%s', got %s",
			templateTestContent2,
			templateMap[id2],
		)
	}
}

// TestNewTemplate tests the NewTemplate constructor.
func TestNewTemplate(t *testing.T) {
	id := NewTemplateID("contact-header")
	content := "Hello {{.name}}"

	template := NewTemplate(id, content)

	if template.ID != id {
		t.Errorf("expected ID %v, got %v", id, template.ID)
	}
	if template.Content != content {
		t.Errorf("expected Content %q, got %q", content, template.Content)
	}
}

// TestTemplateWithGoTemplateSyntax tests that Template can store complex Go
// template syntax.
func TestTemplateWithGoTemplateSyntax(t *testing.T) {
	id := NewTemplateID("complex-template")
	content := `---
fileClass: contact
name: {{ prompt "name" "Contact Name" "" }}
email: {{ prompt "email" "Email Address" "" }}
created: {{ now "2006-01-02" }}
---

# {{ .name }}

**Email:** {{ .email }}
**Created:** {{ .created }}

{{ template "contact-footer" }}`

	template := NewTemplate(id, content)

	if template.Content != content {
		t.Errorf("Template Content does not match expected Go template syntax")
	}
}

// TestTemplate_NoFilePathField tests that Template struct has no FilePath
// field.
func TestTemplate_NoFilePathField(t *testing.T) {
	// This test verifies that the Template struct does not have a FilePath
	// field
	// We use reflection to inspect the struct fields
	template := NewTemplate(NewTemplateID("test"), "content")

	v := reflect.ValueOf(template)
	typ := v.Type()

	for i := 0; i < typ.NumField(); i++ { //nolint:intrange // reflection requires index-based access
		field := typ.Field(i)
		if field.Name == "FilePath" {
			t.Errorf(
				"Template struct should not have a FilePath field, but found: %s",
				field.Name,
			)
		}
	}
}

// TestTemplate_NoParsedField tests that Template struct has no Parsed field.
func TestTemplate_NoParsedField(t *testing.T) {
	// This test verifies that the Template struct does not have a Parsed field
	// We use reflection to inspect the struct fields
	template := NewTemplate(NewTemplateID("test"), "content")

	v := reflect.ValueOf(template)
	typ := v.Type()

	for i := 0; i < typ.NumField(); i++ { //nolint:intrange // reflection requires index-based access
		field := typ.Field(i)
		if field.Name == "Parsed" {
			t.Errorf(
				"Template struct should not have a Parsed field, but found: %s",
				field.Name,
			)
		}
	}
}
