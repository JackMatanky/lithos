package domain

import (
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
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

// TestNewTemplate tests the NewTemplate constructor and execution.
func TestNewTemplate(t *testing.T) {
	id := NewTemplateID("contact-header")
	content := "Hello {{.name}}"

	template := NewTemplate(id, content)

	if template.ID() != id {
		t.Errorf("expected ID %v, got %v", id, template.ID())
	}

	// Test execution
	result, err := template.Execute(map[string]string{"name": "World"})
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}
	if result != "Hello World" {
		t.Errorf("expected 'Hello World', got %q", result)
	}
}

// TestTemplateWithGoTemplateSyntax tests that Template can execute complex Go
// template syntax.
func TestTemplateWithGoTemplateSyntax(t *testing.T) {
	id := NewTemplateID("complex-template")
	content := `---
fileClass: contact
name: {{ .name }}
email: {{ .email }}
created: {{ .created }}
---

# {{ .name }}

**Email:** {{ .email }}
**Created:** {{ .created }}`

	template := NewTemplate(id, content)

	data := map[string]string{
		"name":    "John Doe",
		"email":   "john@example.com",
		"created": "2025-01-01",
	}

	result, err := template.Execute(data)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	expected := `---
fileClass: contact
name: John Doe
email: john@example.com
created: 2025-01-01
---

# John Doe

**Email:** john@example.com
**Created:** 2025-01-01`

	if result != expected {
		t.Errorf(
			"Template execution failed.\nExpected:\n%s\nGot:\n%s",
			expected,
			result,
		)
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
	if v.Kind() == reflect.Ptr {
		v = v.Elem()
	}
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
	if v.Kind() == reflect.Ptr {
		v = v.Elem()
	}
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

// TDD RED Phase Tests - These will fail until interface is implemented

// TestTemplateInterfaceContract tests that the Template struct implements the
// expected interface.
func TestTemplateInterfaceContract(t *testing.T) {
	// Test that Template has ID() and Execute() methods
	template := NewTemplate(NewTemplateID("test"), "{{invalid syntax")

	// Test ID method
	id := template.ID()
	if id != NewTemplateID("test") {
		t.Errorf("expected ID 'test', got %v", id)
	}

	// Test Execute method exists and can be called
	_, err := template.Execute(nil)
	// We expect an error for invalid template syntax
	if err == nil {
		t.Error("expected error for invalid template syntax, but got none")
	}
}

// TestTemplateIDTypeSafety tests that TemplateID provides type safety.
func TestTemplateIDTypeSafety(t *testing.T) {
	// This test will fail until TemplateID is defined
	// TemplateID should be a string alias for type safety

	var id TemplateID = "test-template"
	assert.Equal(t, "test-template", string(id))
	assert.IsType(t, TemplateID(""), id)
}

// TestGoTemplateConstruction tests GoTemplate struct creation.
func TestGoTemplateConstruction(t *testing.T) {
	// This test will fail until GoTemplate is implemented
	// GoTemplate should wrap *template.Template

	// This is a placeholder - actual test will be written after implementation
	t.Skip("Test will be implemented after GoTemplate struct is created")
}

// TestGoTemplateExecuteWrapping tests that GoTemplate.Execute() wraps
// *template.Template.
func TestGoTemplateExecuteWrapping(t *testing.T) {
	// This test will fail until GoTemplate.Execute is implemented
	// Execute should delegate to *template.Template and handle buffering

	t.Skip("Test will be implemented after GoTemplate.Execute is created")
}

// TestMockTemplateImplementation tests that mock Template implementations work.
func TestMockTemplateImplementation(t *testing.T) {
	// This test will fail until Template interface exists
	// We should be able to create mock implementations for testing

	t.Skip("Test will be implemented after Template interface exists")
}

// TestTemplateWithFunctions tests template execution with built-in functions.
func TestTemplateWithFunctions(t *testing.T) {
	id := NewTemplateID("function-template")
	content := `Name: {{.name | toLower}}
Time: {{now "2006-01-02"}}
Folder: {{folder "path/to/file"}}
Basename: {{basename "path/to/file.md"}}
Extension: {{extension "file.md"}}
Join: {{join "/" "a" "b"}}`

	template := NewTemplate(id, content)

	data := map[string]string{"name": "John"}

	result, err := template.Execute(data)
	if err != nil {
		t.Fatalf("Execute failed: %v", err)
	}

	// Check that functions are called
	if !strings.Contains(result, "Name: john") {
		t.Errorf("Expected lowercase name, got: %s", result)
	}

	expectedDate := time.Now().Format("2006-01-02")
	if !strings.Contains(result, "Time: "+expectedDate) {
		t.Errorf("Expected time %s, got: %s", expectedDate, result)
	}
	if !strings.Contains(result, "Folder: path/to") {
		t.Errorf("Expected folder, got: %s", result)
	}
	if !strings.Contains(result, "Basename: file") {
		t.Errorf("Expected basename, got: %s", result)
	}
	if !strings.Contains(result, "Extension: .md") {
		t.Errorf("Expected extension, got: %s", result)
	}
	if !strings.Contains(result, "Join: /a/b") {
		t.Errorf("Expected join, got: %s", result)
	}
}
