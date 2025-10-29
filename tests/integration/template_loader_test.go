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
// filesystem
//
// validation.
//
//nolint:gocognit // Integration test with multiple sub-tests for comprehensive
func TestTemplateLoaderAdapter_Integration(t *testing.T) {
	// Get the testdata/templates directory path
	templatesDir := filepath.Join("..", "..", "testdata", "templates")

	logger := zerolog.New(zerolog.NewTestWriter(t))
	config := domain.Config{
		TemplatesDir: templatesDir,
	}

	adapter := template.NewTemplateLoaderAdapter(&config, &logger)
	ctx := context.Background()

	// Test List() finds expected templates
	t.Run("List", func(t *testing.T) {
		templates, err := adapter.List(ctx)
		if err != nil {
			t.Fatalf("List() error = %v", err)
		}

		// Should find basic-note.md and static-template.md (ignoring .txt
		// files)
		expectedTemplates := []domain.TemplateID{
			"basic-note",
			"static-template",
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
	})

	// Test Load() for static-template
	t.Run("Load_static-template", func(t *testing.T) {
		// First populate metadata cache
		_, err := adapter.List(ctx)
		if err != nil {
			t.Fatalf("List() error = %v", err)
		}

		tmpl, err := adapter.Load(ctx, "static-template")
		if err != nil {
			t.Fatalf("Load() error = %v", err)
		}

		if tmpl.ID != "static-template" {
			t.Errorf("Template ID = %s, want static-template", tmpl.ID)
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
	})

	// Test Load() for basic-note
	t.Run("Load_basic-note", func(t *testing.T) {
		// Metadata cache should already be populated from previous test
		tmpl, err := adapter.Load(ctx, "basic-note")
		if err != nil {
			t.Fatalf("Load() error = %v", err)
		}

		if tmpl.ID != "basic-note" {
			t.Errorf("Template ID = %s, want basic-note", tmpl.ID)
		}

		// Check that content contains expected text
		expectedContent := "# {{.title}}"
		if tmpl.Content == "" {
			t.Error("Template content is empty")
		}
		if !contains(tmpl.Content, expectedContent) {
			t.Errorf(
				"Template content does not contain expected text %q",
				expectedContent,
			)
		}
	})

	// Test Load() with nonexistent template
	t.Run("Load_nonexistent", func(t *testing.T) {
		_, err := adapter.Load(ctx, "nonexistent-template")
		if err == nil {
			t.Error("Load() with nonexistent template should return error")
		}
	})
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
