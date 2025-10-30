// Package utils provides mock implementations of ports for testing
package utils

import (
	"context"
	"io"
	"os"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/api"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// Ensure MockTemplatePort implements TemplatePort.
var _ spi.TemplatePort = (*MockTemplatePort)(nil)

// Ensure MockCommandPort implements CommandPort.
var _ api.CommandPort = (*MockCommandPort)(nil)

// Ensure MockCLIPort implements CLIPort.
var _ api.CLIPort = (*MockCLIPort)(nil)

// MockCommandPort provides a mock implementation of CommandPort for testing.
// It allows configuring mock responses for command operations.
type MockCommandPort struct {
	newNoteResult domain.Note
	newNoteError  error
}

// MockCLIPort provides a mock implementation of CLIPort for testing.
// It allows configuring mock responses for CLI operations and verifying
// interactions with the CommandOrchestrator.
type MockCLIPort struct {
	startCalled  bool
	startHandler api.CommandPort
	startError   error
}

// MockTemplatePort provides a mock implementation of TemplatePort for testing.
// It allows configuring mock responses for template operations.
type MockTemplatePort struct {
	templates map[domain.TemplateID]domain.Template
	loadError error
}

// NewMockCommandPort creates a new MockCommandPort with default values.
func NewMockCommandPort() *MockCommandPort {
	return &MockCommandPort{}
}

// SetNewNoteResult configures the mock to return the specified note and error
// on NewNote calls.
// This allows testing success and error scenarios in command operations.
func (m *MockCommandPort) SetNewNoteResult(note domain.Note, err error) {
	m.newNoteResult = note
	m.newNoteError = err
}

// NewNote returns the configured mock result for note creation.
// If newNoteError is set, it returns the error. Otherwise, it returns the
// configured note.
func (m *MockCommandPort) NewNote(
	ctx context.Context,
	templateID domain.TemplateID,
) (domain.Note, error) {
	return m.newNoteResult, m.newNoteError
}

// NewMockCLIPort creates a new MockCLIPort with default values.
// MockCLIPort provides a mock implementation of CLIPort for testing.
func NewMockCLIPort() *MockCLIPort {
	return &MockCLIPort{}
}

// SetStartError configures the mock to return the specified error on Start
// calls.
// This allows testing error handling in CLI startup scenarios.
func (m *MockCLIPort) SetStartError(err error) {
	m.startError = err
}

// Start simulates CLI startup by recording the handler and returning configured
// error.
// This method implements the CLIPort interface for testing purposes.
// It stores the provided handler and sets the startCalled flag to true.
func (m *MockCLIPort) Start(
	ctx context.Context,
	handler api.CommandPort,
) error {
	m.startCalled = true
	m.startHandler = handler
	return m.startError
}

// WasStartCalled returns true if the Start method was called.
func (m *MockCLIPort) WasStartCalled() bool {
	return m.startCalled
}

// GetHandler returns the CommandPort handler that was passed to Start.
func (m *MockCLIPort) GetHandler() api.CommandPort {
	return m.startHandler
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

// CopyFile copies a file from src to dst. Used in e2e tests.
func CopyFile(t *testing.T, src, dst string) {
	t.Helper()

	srcFile, err := os.Open( //nolint:gosec // Controlled by test code, not user input
		src,
	)
	if err != nil {
		t.Fatalf("Failed to open source file %s: %v", src, err)
	}
	defer func() { _ = srcFile.Close() }()

	dstFile, err := os.Create( //nolint:gosec // Controlled by test code, not user input
		dst,
	)
	if err != nil {
		t.Fatalf("Failed to create destination file %s: %v", dst, err)
	}
	defer func() { _ = dstFile.Close() }()

	_, err = io.Copy(dstFile, srcFile)
	if err != nil {
		t.Fatalf("Failed to copy from %s to %s: %v", src, dst, err)
	}
}
