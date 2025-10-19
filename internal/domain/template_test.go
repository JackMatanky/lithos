package domain

import (
	"testing"
	"text/template"
)

func TestTemplateStruct(t *testing.T) {
	// Test that Template struct can be created and accessed
	tmpl := Template{
		FilePath: "/vault/templates/note.md",
		Name:     "Basic Note",
		Content:  "# {{.title}}\n\nContent: {{.content}}",
		Parsed:   nil,
	}

	if tmpl.FilePath != "/vault/templates/note.md" {
		t.Errorf(
			"FilePath = %q, want %q",
			tmpl.FilePath,
			"/vault/templates/note.md",
		)
	}
	if tmpl.Name != "Basic Note" {
		t.Errorf("Name = %q, want %q", tmpl.Name, "Basic Note")
	}
	if tmpl.Content != "# {{.title}}\n\nContent: {{.content}}" {
		t.Errorf(
			"Content = %q, want %q",
			tmpl.Content,
			"# {{.title}}\n\nContent: {{.content}}",
		)
	}
	if tmpl.Parsed != nil {
		t.Errorf("Parsed = %v, want nil", tmpl.Parsed)
	}
}

func TestTemplateWithParsedTemplate(t *testing.T) {
	// Test Template with a parsed Go template
	content := "Hello {{.name}}!"
	parsed, err := template.New("test").Parse(content)
	if err != nil {
		t.Fatalf("Failed to parse template: %v", err)
	}

	tmpl := Template{
		FilePath: "/vault/templates/greeting.md",
		Name:     "Greeting Template",
		Content:  content,
		Parsed:   parsed,
	}

	if tmpl.FilePath != "/vault/templates/greeting.md" {
		t.Errorf(
			"FilePath = %q, want %q",
			tmpl.FilePath,
			"/vault/templates/greeting.md",
		)
	}
	if tmpl.Name != "Greeting Template" {
		t.Errorf("Name = %q, want %q", tmpl.Name, "Greeting Template")
	}
	if tmpl.Content != content {
		t.Errorf("Content = %q, want %q", tmpl.Content, content)
	}
	if tmpl.Parsed == nil {
		t.Error("Parsed = nil, want non-nil")
	}
	if tmpl.Parsed != nil && tmpl.Parsed.Name() != "test" {
		t.Errorf("Parsed.Name() = %q, want %q", tmpl.Parsed.Name(), "test")
	}
}

func TestTemplateFieldTypes(t *testing.T) {
	// Test that all fields have the correct types
	const testPath = "/test/path"

	tmpl := Template{
		FilePath: "",
		Name:     "",
		Content:  "",
		Parsed:   nil,
	}

	// Test zero values
	if tmpl.FilePath != "" {
		t.Errorf("Zero value FilePath = %q, want empty string", tmpl.FilePath)
	}
	if tmpl.Name != "" {
		t.Errorf("Zero value Name = %q, want empty string", tmpl.Name)
	}
	if tmpl.Content != "" {
		t.Errorf("Zero value Content = %q, want empty string", tmpl.Content)
	}
	if tmpl.Parsed != nil {
		t.Errorf("Zero value Parsed = %v, want nil", tmpl.Parsed)
	}

	// Test assignment
	tmpl.FilePath = testPath
	tmpl.Name = "Test Name"
	tmpl.Content = "Test Content"
	tmpl.Parsed = &template.Template{Tree: nil}

	if tmpl.FilePath != testPath {
		t.Errorf("FilePath assignment failed")
	}
	if tmpl.Name != "Test Name" {
		t.Errorf("Name assignment failed")
	}
	if tmpl.Content != "Test Content" {
		t.Errorf("Content assignment failed")
	}
	if tmpl.Parsed == nil {
		t.Errorf("Parsed assignment failed")
	}
}

func TestTemplateComplexContent(t *testing.T) {
	// Test with more complex template content
	complexContent := `# {{.title}}

Created: {{.created | formatDate}}
Author: {{.author}}

{{range .tags}}
- #{{.}}
{{end}}

## Content

{{.body}}

{{if .footer}}
---
{{.footer}}
{{end}}`

	tmpl := Template{
		Content: complexContent,
	}

	if tmpl.Content != complexContent {
		t.Errorf("Complex content not preserved correctly")
	}

	// Verify the content contains expected template syntax
	if tmpl.Content == "" {
		t.Error("Template content is empty")
	}
}
