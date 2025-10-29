package spi

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
)

const testTemplateID = "test"

// mockTemplatePort implements TemplatePort for testing.
type mockTemplatePort struct {
	templates map[domain.TemplateID]domain.Template
}

// List returns all template IDs from the mock.
func (m *mockTemplatePort) List(
	ctx context.Context,
) ([]domain.TemplateID, error) {
	var ids []domain.TemplateID
	for id := range m.templates {
		ids = append(ids, id)
	}
	return ids, nil
}

// Load returns a template by ID from the mock.
func (m *mockTemplatePort) Load(
	ctx context.Context,
	id domain.TemplateID,
) (domain.Template, error) {
	template, exists := m.templates[id]
	if !exists {
		return domain.Template{}, nil // Simplified for testing
	}
	return template, nil
}

// TestTemplatePortInterface verifies TemplatePort interface contract.
func TestTemplatePortInterface(t *testing.T) {
	// This test verifies that TemplatePort is a valid interface
	// and can be implemented by different adapters

	var port TemplatePort = &mockTemplatePort{
		templates: map[domain.TemplateID]domain.Template{
			"test": domain.NewTemplate("test", "content"),
		},
	}

	ctx := context.Background()

	// Test List method
	ids, err := port.List(ctx)
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}
	if len(ids) != 1 {
		t.Errorf("List() returned %d templates, want 1", len(ids))
	}
	if ids[0] != testTemplateID {
		t.Errorf("List() returned %v, want [%s]", ids, testTemplateID)
	}

	// Test Load method
	template, err := port.Load(ctx, "test")
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}
	if template.ID != "test" {
		t.Errorf("Load() returned ID %v, want test", template.ID)
	}
	if template.Content != "content" {
		t.Errorf(
			"Load() returned content %q, want %q",
			template.Content,
			"content",
		)
	}

	// Test Load method with non-existent template
	_, err = port.Load(ctx, "nonexistent")
	if err != nil {
		t.Fatalf(
			"Load() with nonexistent template should not error in mock, got %v",
			err,
		)
	}
}
