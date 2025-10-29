package template

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

// createTempTemplate creates a temporary template file for testing.
func createTempTemplate(t *testing.T, dir, name, content string) {
	t.Helper()
	path := filepath.Join(dir, name+".md")
	err := os.WriteFile(path, []byte(content), 0o600)
	if err != nil {
		t.Fatalf("Failed to create temp template %s: %v", name, err)
	}
}

// TestNewTemplateLoaderAdapter tests TemplateLoaderAdapter constructor.
func TestNewTemplateLoaderAdapter(t *testing.T) {
	logger := zerolog.New(zerolog.NewTestWriter(t))
	config := &domain.Config{
		TemplatesDir: "/tmp/templates",
	}

	adapter := NewTemplateLoaderAdapter(config, &logger)

	if adapter == nil {
		t.Fatal("NewTemplateLoaderAdapter returned nil")
	}
	if adapter.config.TemplatesDir != "/tmp/templates" {
		t.Errorf(
			"Config not set correctly, got %s",
			adapter.config.TemplatesDir,
		)
	}
	if adapter.metadata == nil {
		t.Error("Metadata map not initialized")
	}
}

// TestTemplateLoaderAdapter_List_EmptyDirectory tests List with no templates.
func TestTemplateLoaderAdapter_List_EmptyDirectory(t *testing.T) {
	tempDir := t.TempDir()
	logger := zerolog.New(zerolog.NewTestWriter(t))
	config := domain.Config{
		TemplatesDir: tempDir,
	}

	adapter := NewTemplateLoaderAdapter(&config, &logger)
	ctx := context.Background()

	templates, err := adapter.List(ctx)
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}
	if len(templates) != 0 {
		t.Errorf("List() returned %d templates, want 0", len(templates))
	}
}

// TestTemplateLoaderAdapter_List_WithTemplates tests List with template files.
func TestTemplateLoaderAdapter_List_WithTemplates(t *testing.T) {
	tempDir := t.TempDir()
	logger := zerolog.New(zerolog.NewTestWriter(t))
	config := domain.Config{
		TemplatesDir: tempDir,
	}

	// Create test templates
	createTempTemplate(t, tempDir, "basic-note", "# Basic Note\n\nContent")
	createTempTemplate(t, tempDir, "contact", "# Contact\n\nInfo")
	createTempTemplate(t, tempDir, "meeting", "# Meeting\n\nNotes")

	adapter := NewTemplateLoaderAdapter(&config, &logger)
	ctx := context.Background()

	templates, err := adapter.List(ctx)
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}

	expected := []domain.TemplateID{"basic-note", "contact", "meeting"}
	if len(templates) != len(expected) {
		t.Errorf(
			"List() returned %d templates, want %d",
			len(templates),
			len(expected),
		)
	}

	// Check that all expected templates are present (order may vary)
	found := make(map[domain.TemplateID]bool)
	for _, id := range templates {
		found[id] = true
	}

	for _, expectedID := range expected {
		if !found[expectedID] {
			t.Errorf("List() missing template %s", expectedID)
		}
	}
}

// TestTemplateLoaderAdapter_Load_Success tests successful template loading.
func TestTemplateLoaderAdapter_Load_Success(t *testing.T) {
	tempDir := t.TempDir()
	logger := zerolog.New(zerolog.NewTestWriter(t))
	config := domain.Config{
		TemplatesDir: tempDir,
	}

	content := "# Test Template\n\nThis is test content."
	createTempTemplate(t, tempDir, "test-template", content)

	adapter := NewTemplateLoaderAdapter(&config, &logger)
	ctx := context.Background()

	// First list to populate metadata
	_, err := adapter.List(ctx)
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}

	template, err := adapter.Load(ctx, "test-template")
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	if template.ID != "test-template" {
		t.Errorf("Template ID = %s, want test-template", template.ID)
	}
	if template.Content != content {
		t.Errorf("Template content = %q, want %q", template.Content, content)
	}
}

// TestTemplateLoaderAdapter_Load_NotFound tests loading non-existent template.
func TestTemplateLoaderAdapter_Load_NotFound(t *testing.T) {
	tempDir := t.TempDir()
	logger := zerolog.New(zerolog.NewTestWriter(t))
	config := domain.Config{
		TemplatesDir: tempDir,
	}

	adapter := NewTemplateLoaderAdapter(&config, &logger)
	ctx := context.Background()

	_, err := adapter.Load(ctx, "nonexistent")
	if err == nil {
		t.Error("Load() with nonexistent template should return error")
	}
}

// TestTemplateLoaderAdapter_Load_InvalidUTF8 tests loading invalid UTF-8
// content.
func TestTemplateLoaderAdapter_Load_InvalidUTF8(t *testing.T) {
	tempDir := t.TempDir()
	logger := zerolog.New(zerolog.NewTestWriter(t))
	config := domain.Config{
		TemplatesDir: tempDir,
	}

	// Create file with invalid UTF-8
	path := filepath.Join(tempDir, "invalid.md")
	err := os.WriteFile(path, []byte{0xff, 0xfe, 0xfd}, 0o600)
	if err != nil {
		t.Fatalf("Failed to create invalid UTF-8 file: %v", err)
	}

	adapter := NewTemplateLoaderAdapter(&config, &logger)
	ctx := context.Background()

	// First list to populate metadata
	_, err = adapter.List(ctx)
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}

	_, err = adapter.Load(ctx, "invalid")
	if err == nil {
		t.Error("Load() with invalid UTF-8 should return error")
	}
}

// TestTemplateLoaderAdapter_List_Sorted tests template ID sorting.
func TestTemplateLoaderAdapter_List_Sorted(t *testing.T) {
	tempDir := t.TempDir()
	logger := zerolog.New(zerolog.NewTestWriter(t))
	config := domain.Config{
		TemplatesDir: tempDir,
	}

	// Create templates in reverse alphabetical order
	createTempTemplate(t, tempDir, "zebra", "content")
	createTempTemplate(t, tempDir, "alpha", "content")
	createTempTemplate(t, tempDir, "beta", "content")

	adapter := NewTemplateLoaderAdapter(&config, &logger)
	ctx := context.Background()

	templates, err := adapter.List(ctx)
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}

	expected := []domain.TemplateID{"alpha", "beta", "zebra"}
	if len(templates) != len(expected) {
		t.Fatalf(
			"List() returned %d templates, want %d",
			len(templates),
			len(expected),
		)
	}

	for i, expectedID := range expected {
		if templates[i] != expectedID {
			t.Errorf("List()[%d] = %s, want %s", i, templates[i], expectedID)
		}
	}
}
