// Package mocks provides mock implementations of ports for testing
package mocks

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// Ensure MockTemplatePort implements TemplatePort.
var _ spi.TemplatePort = (*MockTemplatePort)(nil)

// MockTemplatePort provides a mock implementation of TemplatePort for testing.
// It allows configuring mock responses for template operations.
type MockTemplatePort struct {
	templates map[domain.TemplateID]domain.Template
	loadError error
}

// NewMockTemplatePort creates a new MockTemplatePort with empty storage.
func NewMockTemplatePort() *MockTemplatePort {
	return &MockTemplatePort{
		templates: make(map[domain.TemplateID]domain.Template),
		loadError: nil,
	}
}

// SetTemplates configures the mock to return the specified templates.
// This method allows setting up test data for template operations.
func (m *MockTemplatePort) SetTemplates(
	templates map[domain.TemplateID]domain.Template,
) {
	m.templates = templates
}

// SetLoadError configures the mock to return the specified error on Load calls.
// This allows testing error handling in template operations.
func (m *MockTemplatePort) SetLoadError(err error) {
	m.loadError = err
}

// List returns all template IDs from the internal storage map.
// The returned slice contains the keys of the configured templates.
func (m *MockTemplatePort) List(
	ctx context.Context,
) ([]domain.TemplateID, error) {
	var ids []domain.TemplateID
	for id := range m.templates {
		ids = append(ids, id)
	}
	return ids, nil
}

// Load returns the template from internal storage or the configured error.
// If loadError is set, it returns the error. Otherwise, it returns the template
// with the specified ID from the internal storage map.
func (m *MockTemplatePort) Load(
	ctx context.Context,
	id domain.TemplateID,
) (domain.Template, error) {
	if m.loadError != nil {
		return domain.Template{}, m.loadError
	}
	tmpl, exists := m.templates[id]
	if !exists {
		return domain.Template{}, errors.NewResourceError(
			"template",
			"load",
			string(id),
			nil,
		)
	}
	return tmpl, nil
}
