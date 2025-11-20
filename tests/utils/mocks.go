// Package utils provides mock implementations of ports for testing
//
// readability
//
//nolint:decorder // types and their methods are grouped together for
package utils

import (
	"context"
	"fmt"
	"time"

	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/api"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// Ensure MockMetadataQueryPort implements MetadataQueryPort.
var _ spi.MetadataQueryPort = (*MockMetadataQueryPort)(nil)

// Ensure MockCacheWriterPort implements CacheWriterPort.
var _ spi.CacheWriterPort = (*MockCacheWriterPort)(nil)

// Ensure MockMarkdownParserPort implements MarkdownParserPort.
var _ spi.MarkdownParserPort = (*MockMarkdownParserPort)(nil)

// Ensure MockVaultWriterPort implements VaultWriterPort.
var _ spi.VaultWriterPort = (*MockVaultWriterPort)(nil)

// Ensure MockTemplatePort implements TemplatePort.
var _ spi.TemplatePort = (*MockTemplatePort)(nil)

// Ensure MockCommandPort implements CommandPort.
var _ api.CommandPort = (*MockCommandPort)(nil)

// Ensure MockCLIPort implements CLIPort.
var _ api.CLIPort = (*MockCLIPort)(nil)

// Ensure MockCacheWriterPort implements CacheWriterPort.
var _ spi.CacheWriterPort = (*MockCacheWriterPort)(nil)

// Ensure MockMarkdownParserPort implements MarkdownParserPort.
var _ spi.MarkdownParserPort = (*MockMarkdownParserPort)(nil)

// Ensure MockVaultWriterPort implements VaultWriterPort.
var _ spi.VaultWriterPort = (*MockVaultWriterPort)(nil)

// MockCacheWriterPort provides a mock implementation of CacheWriterPort for
// testing.
// It allows configuring mock responses for cache persistence operations.
type MockCacheWriterPort struct {
	persistResult error
	deleteResult  error
	PersistFunc   func(ctx context.Context, note domain.Note, indexTime time.Time) error
	DeleteFunc    func(ctx context.Context, id domain.NoteID) error
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
	indexTime time.Time,
) error {
	if m.PersistFunc != nil {
		return m.PersistFunc(ctx, note, indexTime)
	}
	return m.persistResult
}

// Delete returns the configured mock result for cache deletion.
func (m *MockCacheWriterPort) Delete(
	ctx context.Context,
	id domain.NoteID,
) error {
	if m.DeleteFunc != nil {
		return m.DeleteFunc(ctx, id)
	}
	return m.deleteResult
}

// MockMarkdownParserPort provides a mock implementation of MarkdownParserPort
// for
// testing.
type MockMarkdownParserPort struct {
	parseResult map[string]any
	parseError  error
}

// NewMockMarkdownParserPort creates a new MockMarkdownParserPort with default
// values.
func NewMockMarkdownParserPort() *MockMarkdownParserPort {
	return &MockMarkdownParserPort{}
}

// SetParseResult configures the mock to return the specified frontmatter and
// error on ParseFrontmatter calls.
func (m *MockMarkdownParserPort) SetParseResult(
	fm map[string]any,
	err error,
) {
	m.parseResult = fm
	m.parseError = err
}

// ParseFrontmatter returns the configured mock result for frontmatter parsing.
func (m *MockMarkdownParserPort) ParseFrontmatter(
	ctx context.Context,
	content []byte,
) (map[string]any, error) {
	return m.parseResult, m.parseError
}

// MockVaultWriterPort provides a mock implementation of VaultWriterPort for
// testing workflows that need to observe vault write side effects.
type MockVaultWriterPort struct {
	persistResult      error
	deleteResult       error
	writeContentResult error
	lastWritePath      string
	lastWriteContent   []byte
}

// NewMockVaultWriterPort creates a new MockVaultWriterPort with default values.
func NewMockVaultWriterPort() *MockVaultWriterPort {
	return &MockVaultWriterPort{}
}

// SetPersistResult configures the result returned from Persist.
func (m *MockVaultWriterPort) SetPersistResult(err error) {
	m.persistResult = err
}

// SetDeleteResult configures the result returned from Delete.
func (m *MockVaultWriterPort) SetDeleteResult(err error) {
	m.deleteResult = err
}

// SetWriteContentResult configures the result returned from WriteContent.
func (m *MockVaultWriterPort) SetWriteContentResult(err error) {
	m.writeContentResult = err
}

// LastWrite returns the path and content captured during the most recent
// WriteContent call.
func (m *MockVaultWriterPort) LastWrite() (path string, content []byte) {
	return m.lastWritePath, m.lastWriteContent
}

// Persist implements VaultWriterPort.Persist.
func (m *MockVaultWriterPort) Persist(
	ctx context.Context,
	note domain.Note,
	path string,
) error {
	return m.persistResult
}

// Delete implements VaultWriterPort.Delete.
func (m *MockVaultWriterPort) Delete(
	ctx context.Context,
	path string,
) error {
	return m.deleteResult
}

// WriteContent implements VaultWriterPort.WriteContent.
func (m *MockVaultWriterPort) WriteContent(
	ctx context.Context,
	path string,
	content []byte,
) error {
	m.lastWritePath = path
	m.lastWriteContent = append([]byte(nil), content...)
	return m.writeContentResult
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

// MockMetadataQueryPort provides a mock implementation of MetadataQueryPort for
// testing. It allows configuring mock responses for each query method and
// tracks call counts for assertion purposes.
type MockMetadataQueryPort struct {
	// Function fields for method delegation
	BasenameQueryFunc    func(ctx context.Context, basename string) ([]domain.Note, error)
	AliasQueryFunc       func(ctx context.Context, alias string) ([]domain.Note, error)
	FileClassQueryFunc   func(ctx context.Context, fileClass string) ([]domain.Note, error)
	PathQueryFunc        func(ctx context.Context, opts spi.PathQueryOptions) ([]domain.Note, error)
	TagQueryFunc         func(ctx context.Context, tag string) ([]domain.Note, error)
	FrontmatterQueryFunc func(ctx context.Context, field, value string) ([]domain.Note, error)

	// Call tracking for assertions
	BasenameQueryCallCount    int
	AliasQueryCallCount       int
	FileClassQueryCallCount   int
	PathQueryCallCount        int
	TagQueryCallCount         int
	FrontmatterQueryCallCount int

	// Last call arguments for detailed assertions
	LastBasenameQueryArg         string
	LastAliasQueryArg            string
	LastFileClassQueryArg        string
	LastPathQueryOpts            spi.PathQueryOptions
	LastTagQueryArg              string
	LastFrontmatterQueryArgField string
	LastFrontmatterQueryArgValue string
}

// NewMockMetadataQueryPort creates a new MockMetadataQueryPort with default
// behavior. By default, all methods return empty slices and nil errors.
// Configure specific behavior using the Set*Result methods.
func NewMockMetadataQueryPort() *MockMetadataQueryPort {
	return &MockMetadataQueryPort{
		BasenameQueryFunc: func(ctx context.Context, basename string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		AliasQueryFunc: func(ctx context.Context, alias string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		FileClassQueryFunc: func(ctx context.Context, fileClass string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		PathQueryFunc: func(ctx context.Context, opts spi.PathQueryOptions) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		TagQueryFunc: func(ctx context.Context, tag string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		FrontmatterQueryFunc: func(ctx context.Context, field, value string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		BasenameQueryCallCount:    0,
		AliasQueryCallCount:       0,
		FileClassQueryCallCount:   0,
		PathQueryCallCount:        0,
		TagQueryCallCount:         0,
		FrontmatterQueryCallCount: 0,
		LastBasenameQueryArg:      "",
		LastAliasQueryArg:         "",
		LastFileClassQueryArg:     "",
		LastPathQueryOpts: spi.PathQueryOptions{
			Value: "",
			Scope: "",
		},
		LastTagQueryArg:              "",
		LastFrontmatterQueryArgField: "",
		LastFrontmatterQueryArgValue: "",
	}
}

// SetBasenameQueryResult configures the mock to return the specified result for
// BasenameQuery calls.
func (m *MockMetadataQueryPort) SetBasenameQueryResult(
	notes []domain.Note,
	err error,
) {
	m.BasenameQueryFunc = func(ctx context.Context, basename string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetAliasQueryResult configures the mock to return the specified result for
// AliasQuery calls.
func (m *MockMetadataQueryPort) SetAliasQueryResult(
	notes []domain.Note,
	err error,
) {
	m.AliasQueryFunc = func(ctx context.Context, alias string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetFileClassQueryResult configures the mock to return the specified result
// for
// FileClassQuery calls.
func (m *MockMetadataQueryPort) SetFileClassQueryResult(
	notes []domain.Note,
	err error,
) {
	m.FileClassQueryFunc = func(ctx context.Context, fileClass string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetPathQueryResult configures the mock to return the specified result for
// PathQuery calls.
func (m *MockMetadataQueryPort) SetPathQueryResult(
	notes []domain.Note,
	err error,
) {
	m.PathQueryFunc = func(ctx context.Context, opts spi.PathQueryOptions) ([]domain.Note, error) {
		return notes, err
	}
}

// SetTagQueryResult configures the mock to return the specified result for
// TagQuery calls.
func (m *MockMetadataQueryPort) SetTagQueryResult(
	notes []domain.Note,
	err error,
) {
	m.TagQueryFunc = func(ctx context.Context, tag string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetFrontmatterQueryResult configures the mock to return the specified result
// for FrontmatterQuery calls.
func (m *MockMetadataQueryPort) SetFrontmatterQueryResult(
	notes []domain.Note,
	err error,
) {
	m.FrontmatterQueryFunc = func(ctx context.Context, field, value string) ([]domain.Note, error) {
		return notes, err
	}
}

// BasenameQuery implements MetadataQueryPort.BasenameQuery with mock behavior.
func (m *MockMetadataQueryPort) BasenameQuery(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	m.BasenameQueryCallCount++
	m.LastBasenameQueryArg = basename
	return m.BasenameQueryFunc(ctx, basename)
}

// AliasQuery implements MetadataQueryPort.AliasQuery with mock behavior.
func (m *MockMetadataQueryPort) AliasQuery(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	m.AliasQueryCallCount++
	m.LastAliasQueryArg = alias
	return m.AliasQueryFunc(ctx, alias)
}

// FileClassQuery implements MetadataQueryPort.FileClassQuery with mock
// behavior.
func (m *MockMetadataQueryPort) FileClassQuery(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	m.FileClassQueryCallCount++
	m.LastFileClassQueryArg = fileClass
	return m.FileClassQueryFunc(ctx, fileClass)
}

// PathQuery implements MetadataQueryPort.PathQuery with mock behavior.
func (m *MockMetadataQueryPort) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	m.PathQueryCallCount++
	m.LastPathQueryOpts = opts
	return m.PathQueryFunc(ctx, opts)
}

// TagQuery implements MetadataQueryPort.TagQuery with mock behavior.
func (m *MockMetadataQueryPort) TagQuery(
	ctx context.Context,
	tag string,
) ([]domain.Note, error) {
	m.TagQueryCallCount++
	m.LastTagQueryArg = tag
	return m.TagQueryFunc(ctx, tag)
}

// FrontmatterQuery implements MetadataQueryPort.FrontmatterQuery with mock
// behavior.
func (m *MockMetadataQueryPort) FrontmatterQuery(
	ctx context.Context,
	field, value string,
) ([]domain.Note, error) {
	m.FrontmatterQueryCallCount++
	m.LastFrontmatterQueryArgField = field
	m.LastFrontmatterQueryArgValue = value
	return m.FrontmatterQueryFunc(ctx, field, value)
}

// Reset resets all call tracking counters and last arguments.
// Useful for testing multiple scenarios in the same test.
func (m *MockMetadataQueryPort) Reset() {
	m.BasenameQueryCallCount = 0
	m.AliasQueryCallCount = 0
	m.FileClassQueryCallCount = 0
	m.PathQueryCallCount = 0
	m.TagQueryCallCount = 0
	m.FrontmatterQueryCallCount = 0
	m.LastBasenameQueryArg = ""
	m.LastAliasQueryArg = ""
	m.LastFileClassQueryArg = ""
	m.LastPathQueryOpts = spi.PathQueryOptions{Value: "", Scope: ""}
	m.LastTagQueryArg = ""
	m.LastFrontmatterQueryArgField = ""
	m.LastFrontmatterQueryArgValue = ""
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
