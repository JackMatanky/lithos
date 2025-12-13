package spi

import (
	"context"
	"errors"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
)

const testTemplateID = "test"

// mockTemplatePort implements TemplatePort for testing.
type mockTemplatePort struct {
	templates map[domain.TemplateID]domain.Template
}

type mockTemplateError struct {
	id domain.TemplateID
}

func (e *mockTemplateError) Error() string {
	return "template not found: " + string(e.id)
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
		return nil, &mockTemplateError{id: id}
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
	template, err := port.Load(ctx, testTemplateID)
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}
	if template.ID() != testTemplateID {
		t.Errorf("Load() returned ID %v, want test", template.ID())
	}
	if template.Content() != "content" {
		t.Errorf(
			"Load() returned content %q, want %q",
			template.Content(),
			"content",
		)
	}

	// Test Load method with non-existent template
	_, err = port.Load(ctx, "nonexistent")
	if err == nil {
		t.Fatal("Load() with nonexistent template should error in mock")
	}
	var mockErr *mockTemplateError
	if !errors.As(err, &mockErr) {
		t.Fatalf("Expected mockTemplateError, got %T", err)
	}
	if mockErr.id != "nonexistent" {
		t.Errorf(
			"Expected error for 'nonexistent', got error for '%s'",
			mockErr.id,
		)
	}
}
