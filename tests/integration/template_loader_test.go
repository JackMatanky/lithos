package integration

import (
	"context"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/adapters/spi/template"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

// TestTemplateLoaderAdapter_Integration tests template loading with real
// filesystem validation.
func TestTemplateLoaderAdapter_Integration(t *testing.T) {
	// Get the testdata/templates directory path
	templatesDir := filepath.Join("..", "..", "testdata", "templates")

	logger := zerolog.New(zerolog.NewTestWriter(t))
	config := domain.Config{
		TemplatesDir: templatesDir,
	}

	adapter := template.NewTemplateLoaderAdapter(&config, &logger)
	ctx := context.Background()

	t.Run("List", func(t *testing.T) { testList(t, adapter, ctx) })
	t.Run(
		"Load_static_template",
		func(t *testing.T) { testLoadStaticTemplate(t, adapter, ctx) },
	)
	t.Run(
		"Load_basic_note",
		func(t *testing.T) { testLoadBasicNote(t, adapter, ctx) },
	)
	t.Run(
		"Load_nonexistent",
		func(t *testing.T) { testLoadNonexistentTemplate(t, adapter, ctx) },
	)
}

func testList(
	t *testing.T,
	adapter *template.TemplateLoaderAdapter,
	ctx context.Context,
) {
	templates, err := adapter.List(ctx)
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}

	// Should find basic_note.md and static_template.md (ignoring .txt
	// files)
	expectedTemplates := []domain.TemplateID{
		"basic_note",
		"static_template",
	}
	if len(templates) != len(expectedTemplates) {
		t.Errorf(
			"List() returned %d templates, want %d",
			len(templates),
			len(expectedTemplates),
		)
		t.Logf("Found templates: %v", templates)
		return
	}

	// Check that expected templates are present
	found := make(map[domain.TemplateID]bool)
	for _, id := range templates {
		found[id] = true
	}

	for _, expectedID := range expectedTemplates {
		if !found[expectedID] {
			t.Errorf("List() missing expected template %s", expectedID)
		}
	}
}

func testLoadStaticTemplate(
	t *testing.T,
	adapter *template.TemplateLoaderAdapter,
	ctx context.Context,
) {
	// First populate metadata cache
	_, err := adapter.List(ctx)
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}

	tmpl, err := adapter.Load(ctx, "static_template")
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	if tmpl.ID != "static_template" {
		t.Errorf("Template ID = %s, want static_template", tmpl.ID)
	}

	// Check that content contains expected text
	expectedContent := "# Static Template Example"
	if tmpl.Content == "" {
		t.Error("Template content is empty")
	}
	if !contains(tmpl.Content, expectedContent) {
		t.Errorf(
			"Template content does not contain expected text %q",
			expectedContent,
		)
	}
}

func testLoadBasicNote(
	t *testing.T,
	adapter *template.TemplateLoaderAdapter,
	ctx context.Context,
) {
	// Metadata cache should already be populated from previous test
	tmpl, err := adapter.Load(ctx, "basic_note")
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	if tmpl.ID != "basic_note" {
		t.Errorf("Template ID = %s, want basic_note", tmpl.ID)
	}

	// Check that content contains expected text
	expectedContent := "# {{if .title}}{{.title}}{{else}}Untitled Note{{end}}"
	if tmpl.Content == "" {
		t.Error("Template content is empty")
	}
	if !contains(tmpl.Content, expectedContent) {
		t.Errorf(
			"Template content does not contain expected text %q",
			expectedContent,
		)
	}
}

func testLoadNonexistentTemplate(
	t *testing.T,
	adapter *template.TemplateLoaderAdapter,
	ctx context.Context,
) {
	_, err := adapter.Load(ctx, "nonexistent-template")
	if err == nil {
		t.Error("Load() with nonexistent template should return error")
	}
}

// contains checks if a string contains a substring.
func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > len(substr) &&
		(s[:len(substr)] == substr || s[len(s)-len(substr):] == substr ||
			containsMiddle(s, substr)))
}

func containsMiddle(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
