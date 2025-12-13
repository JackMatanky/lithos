package template

import (
	"os"
	"path/filepath"
	"testing"
	"text/template"
)

// TestGoTemplateAdapterCreation tests GoTemplateAdapter construction.
func TestGoTemplateAdapterCreation(t *testing.T) {
	// Create a temp dir with no template files
	dir := t.TempDir()
	adapter, err := NewGoTemplateAdapter(dir, template.FuncMap{})
	// Should not error even with no template files
	if err != nil {
		t.Fatalf("unexpected error creating adapter: %v", err)
	}
	if adapter == nil {
		t.Error("adapter should not be nil")
	}
}

// TestGoTemplateAdapterParseSingle tests parsing a single template.
func TestGoTemplateAdapterParseSingle(t *testing.T) {
	// Create a temporary directory with a template file
	dir := t.TempDir()
	templateFile := filepath.Join(dir, "test.tmpl")
	err := os.WriteFile(templateFile, []byte("Hello {{.name}}"), 0o644)
	if err != nil {
		t.Fatalf("failed to create template file: %v", err)
	}

	// Create adapter after file exists
	adapter, err := NewGoTemplateAdapter(dir, template.FuncMap{})
	if err != nil {
		t.Fatalf("failed to create adapter: %v", err)
	}

	tmpl, err := adapter.GetTemplate("test")
	if err != nil {
		t.Fatalf("failed to get template: %v", err)
	}

	if tmpl.ID() != "test" {
		t.Errorf("expected ID 'test', got %v", tmpl.ID())
	}

	result, err := tmpl.Execute(map[string]string{"name": "World"})
	if err != nil {
		t.Fatalf("failed to execute template: %v", err)
	}

	if result != "Hello World" {
		t.Errorf("expected 'Hello World', got %q", result)
	}
}

// TestGoTemplateAdapterParseMultiple tests parsing multiple templates with
// shared FuncMap.
func TestGoTemplateAdapterParseMultiple(t *testing.T) {
	dir := t.TempDir()

	// Create multiple template files
	files := map[string]string{
		"header.tmpl": "<h1>{{.title}}</h1>",
		"footer.tmpl": "<footer>{{.year}}</footer>",
		"page.tmpl":   "{{template \"header.tmpl\" .}}{{template \"footer.tmpl\" .}}",
	}

	for name, content := range files {
		err := os.WriteFile(filepath.Join(dir, name), []byte(content), 0o644)
		if err != nil {
			t.Fatalf("failed to create %s: %v", name, err)
		}
	}

	// Create adapter after files exist
	adapter, err := NewGoTemplateAdapter(dir, template.FuncMap{})
	if err != nil {
		t.Fatalf("failed to create adapter: %v", err)
	}

	pageTmpl, err := adapter.GetTemplate("page")
	if err != nil {
		t.Fatalf("failed to get page template: %v", err)
	}

	result, err := pageTmpl.Execute(
		map[string]string{"title": "Test", "year": "2025"},
	)
	if err != nil {
		t.Fatalf("failed to execute template: %v", err)
	}

	expected := "<h1>Test</h1><footer>2025</footer>"
	if result != expected {
		t.Errorf("expected %q, got %q", expected, result)
	}
}

// TestGoTemplateAdapterErrorHandling tests error handling for malformed
// templates.
func TestGoTemplateAdapterErrorHandling(t *testing.T) {
	dir := t.TempDir()

	// Create a malformed template
	err := os.WriteFile(
		filepath.Join(dir, "bad.tmpl"),
		[]byte("{{invalid syntax"),
		0o644,
	)
	if err != nil {
		t.Fatalf("failed to create bad template: %v", err)
	}

	_, err = NewGoTemplateAdapter(dir, template.FuncMap{})
	if err == nil {
		t.Error("expected error for malformed template, got nil")
	}
}

// TestGoTemplateExecuteSuccess tests successful template execution.
func TestGoTemplateExecuteSuccess(t *testing.T) {
	// Create a template set with the expected name
	tmpl := template.New("base")
	_, err := tmpl.Parse("Result: {{.value}}")
	if err != nil {
		t.Fatalf("failed to parse template: %v", err)
	}

	// Create a new template with the correct name for ExecuteTemplate
	correctTmpl := template.New("test.tmpl")
	correctTmpl, err = correctTmpl.AddParseTree("test.tmpl", tmpl.Tree)
	if err != nil {
		t.Fatalf("failed to add parse tree: %v", err)
	}

	goTmpl := &GoTemplate{
		id:   "test",
		tmpl: correctTmpl,
	}

	result, err := goTmpl.Execute(map[string]string{"value": "success"})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result != "Result: success" {
		t.Errorf("expected 'Result: success', got %q", result)
	}
}

// TestGoTemplateExecuteError tests error handling in template execution.
func TestGoTemplateExecuteError(t *testing.T) {
	// Create a template that will panic during execution
	tmpl, err := template.New("test").Parse("{{.Value | len}}")
	if err != nil {
		t.Fatalf("failed to parse template: %v", err)
	}

	goTmpl := &GoTemplate{
		id:   "test",
		tmpl: tmpl,
	}

	// Pass data that will cause len to fail (nil)
	_, err = goTmpl.Execute(map[string]any{"Value": nil})
	if err == nil {
		t.Error("expected error for len of nil, got nil")
	}
}

// TestGoTemplateIDMethod tests the ID() method.
func TestGoTemplateIDMethod(t *testing.T) {
	tmpl, _ := template.New("test").Parse("content")
	goTmpl := &GoTemplate{
		id:   "test-id",
		tmpl: tmpl,
	}

	if goTmpl.ID() != "test-id" {
		t.Errorf("expected 'test-id', got %q", goTmpl.ID())
	}
}
