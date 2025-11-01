// Package utils provides mock implementations of ports for testing
//
// readability
//
//nolint:decorder // types and their methods are grouped together for
package utils

import (
	"context"
	"fmt"

	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/api"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// Ensure MockTemplatePort implements TemplatePort.
var _ spi.TemplatePort = (*MockTemplatePort)(nil)

// Ensure MockCommandPort implements CommandPort.
var _ api.CommandPort = (*MockCommandPort)(nil)

// Ensure MockCLIPort implements CLIPort.
var _ api.CLIPort = (*MockCLIPort)(nil)

// Ensure MockCacheWriterPort implements CacheWriterPort.
var _ spi.CacheWriterPort = (*MockCacheWriterPort)(nil)

// MockCacheWriterPort provides a mock implementation of CacheWriterPort for
// testing.
// It allows configuring mock responses for cache persistence operations.
type MockCacheWriterPort struct {
	persistResult error
	deleteResult  error
}

// NewMockCacheWriterPort creates a new MockCacheWriterPort with default values.
func NewMockCacheWriterPort() *MockCacheWriterPort {
	return &MockCacheWriterPort{}
}

// SetPersistResult configures the mock to return the specified error on Persist
// calls.
func (m *MockCacheWriterPort) SetPersistResult(err error) {
	m.persistResult = err
}

// SetDeleteResult configures the mock to return the specified error on Delete
// calls.
func (m *MockCacheWriterPort) SetDeleteResult(err error) {
	m.deleteResult = err
}

// Persist returns the configured mock result for cache persistence.
func (m *MockCacheWriterPort) Persist(
	ctx context.Context,
	note domain.Note,
) error {
	return m.persistResult
}

// Delete returns the configured mock result for cache deletion.
func (m *MockCacheWriterPort) Delete(
	ctx context.Context,
	id domain.NoteID,
) error {
	return m.deleteResult
}

// MockFrontmatterService provides a mock implementation for frontmatter
// extraction.
// It allows configuring mock responses for frontmatter operations.
type MockFrontmatterService struct {
	extractResult domain.Frontmatter
	extractError  error
}

// NewMockFrontmatterService creates a new MockFrontmatterService with default
// values.
func NewMockFrontmatterService() *MockFrontmatterService {
	return &MockFrontmatterService{}
}

// SetExtractResult configures the mock to return the specified frontmatter and
// error
// on Extract calls.
func (m *MockFrontmatterService) SetExtractResult(
	fm domain.Frontmatter,
	err error,
) {
	m.extractResult = fm
	m.extractError = err
}

// Extract returns the configured mock result for frontmatter extraction.
func (m *MockFrontmatterService) Extract(
	content []byte,
) (domain.Frontmatter, error) {
	return m.extractResult, m.extractError
}

// MockVaultIndexer provides a mock implementation for vault indexing.
// It allows configuring mock responses for Build operations.
type MockVaultIndexer struct {
	buildResult vault.IndexStats
	buildError  error
}

// NewMockVaultIndexer creates a new MockVaultIndexer with default values.
func NewMockVaultIndexer() *MockVaultIndexer {
	return &MockVaultIndexer{}
}

// SetBuildResult configures the mock to return the specified stats and error
// on Build calls.
func (m *MockVaultIndexer) SetBuildResult(stats vault.IndexStats, err error) {
	m.buildResult = stats
	m.buildError = err
}

// Build returns the configured mock result for vault indexing.
func (m *MockVaultIndexer) Build(
	ctx context.Context,
) (vault.IndexStats, error) {
	return m.buildResult, m.buildError
}

// MockSchemaEngine provides a mock implementation for schema operations.
// It allows configuring mock responses for schema retrieval.
type MockSchemaEngine struct {
	getSchemaResult domain.Schema
	getSchemaError  error
	hasSchemaResult bool
}

// NewMockSchemaEngine creates a new MockSchemaEngine with default values.
func NewMockSchemaEngine() *MockSchemaEngine {
	return &MockSchemaEngine{}
}

// SetGetSchemaResult configures the mock to return the specified schema and
// error
// on Get calls.
func (m *MockSchemaEngine) SetGetSchemaResult(schema domain.Schema, err error) {
	m.getSchemaResult = schema
	m.getSchemaError = err
}

// SetHasSchemaResult configures the mock to return the specified boolean
// on HasSchema calls.
func (m *MockSchemaEngine) SetHasSchemaResult(result bool) {
	m.hasSchemaResult = result
}

// Get returns the configured mock result for schema retrieval.
func Get[T domain.Schema | domain.Property](
	m *MockSchemaEngine,
	ctx context.Context,
	name string,
) (T, error) {
	var zero T
	if m.getSchemaError != nil {
		return zero, m.getSchemaError
	}
	return any(m.getSchemaResult).(T), nil
}

// HasSchema returns the configured mock result for schema existence check.
func (m *MockSchemaEngine) HasSchema(ctx context.Context, name string) bool {
	return m.hasSchemaResult
}

// MockCommandPort provides a mock implementation of CommandPort for testing.
type MockCommandPort struct {
	newNoteResult    domain.Note
	newNoteError     error
	indexVaultResult vault.IndexStats
	indexVaultError  error
}

// NewMockCommandPort creates a new MockCommandPort with default values.
func NewMockCommandPort() *MockCommandPort {
	return &MockCommandPort{}
}

// SetNewNoteResult configures the mock to return the specified note and error
// on NewNote calls.
func (m *MockCommandPort) SetNewNoteResult(note domain.Note, err error) {
	m.newNoteResult = note
	m.newNoteError = err
}

// SetIndexVaultResult configures the mock to return the specified stats and
// error
// on IndexVault calls.
func (m *MockCommandPort) SetIndexVaultResult(
	stats vault.IndexStats,
	err error,
) {
	m.indexVaultResult = stats
	m.indexVaultError = err
}

// NewNote returns the configured mock result for note creation.
func (m *MockCommandPort) NewNote(
	ctx context.Context,
	templateID domain.TemplateID,
) (domain.Note, error) {
	return m.newNoteResult, m.newNoteError
}

// IndexVault returns the configured mock result for vault indexing.
func (m *MockCommandPort) IndexVault(
	ctx context.Context,
) (vault.IndexStats, error) {
	return m.indexVaultResult, m.indexVaultError
}

// MockCLIPort provides a mock implementation of CLIPort for testing.
type MockCLIPort struct {
	startResult error
	startCalled bool
	handler     api.CommandPort
}

// NewMockCLIPort creates a new MockCLIPort with default values.
func NewMockCLIPort() *MockCLIPort {
	return &MockCLIPort{}
}

// SetStartError configures the mock to return the specified error on Start
// calls.
func (m *MockCLIPort) SetStartError(err error) {
	m.startResult = err
}

// Start returns the configured mock result for CLI startup.
func (m *MockCLIPort) Start(
	ctx context.Context,
	handler api.CommandPort,
) error {
	m.startCalled = true
	m.handler = handler
	return m.startResult
}

// WasStartCalled returns true if Start was called.
func (m *MockCLIPort) WasStartCalled() bool {
	return m.startCalled
}

// GetHandler returns the handler passed to Start.
func (m *MockCLIPort) GetHandler() api.CommandPort {
	return m.handler
}

// MockTemplatePort provides a mock implementation of TemplatePort for testing.
type MockTemplatePort struct {
	templates map[domain.TemplateID]domain.Template
	loadError error
}

// NewMockTemplatePort creates a new MockTemplatePort with default values.
func NewMockTemplatePort() *MockTemplatePort {
	return &MockTemplatePort{
		templates: make(map[domain.TemplateID]domain.Template),
		loadError: nil,
	}
}

// SetTemplates configures the mock to return the specified templates.
func (m *MockTemplatePort) SetTemplates(
	templates map[domain.TemplateID]domain.Template,
) {
	m.templates = templates
}

// SetLoadError configures the mock to return the specified error on Load calls.
func (m *MockTemplatePort) SetLoadError(err error) {
	m.loadError = err
}

// List returns the configured mock result for template listing.
func (m *MockTemplatePort) List(
	ctx context.Context,
) ([]domain.TemplateID, error) {
	var ids []domain.TemplateID
	for id := range m.templates {
		ids = append(ids, id)
	}
	return ids, nil
}

// Load returns the configured mock result for template loading.
func (m *MockTemplatePort) Load(
	ctx context.Context,
	id domain.TemplateID,
) (domain.Template, error) {
	if m.loadError != nil {
		return domain.Template{}, m.loadError
	}
	tmpl, exists := m.templates[id]
	if !exists {
		return domain.Template{}, fmt.Errorf("template not found: %s", id)
	}
	return tmpl, nil
}
