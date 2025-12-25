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

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// IndexStats represents vault indexing statistics.
// Defined locally to avoid import cycles.
type IndexStats struct {
	ScannedCount        int
	IndexedCount        int
	ParseFailures       int
	CacheFailures       int
	ValidationSuccesses int
	ValidationFailures  int
	Duration            time.Duration
}

// Ensure MockMetadataQueryPort implements MetadataQueryPort.
var _ spi.MetadataQueryPort = (*MockMetadataQueryPort)(nil)
var _ spi.CacheReaderPort = (*MockMetadataQueryPort)(nil)

// Ensure MockCacheWriterPort implements CacheWriterPort.
var _ spi.CacheWriterPort = (*MockCacheWriterPort)(nil)

// Ensure MockMarkdownParserPort implements MarkdownParserPort.
var _ spi.MarkdownParserPort = (*MockMarkdownParserPort)(nil)

// Ensure MockVaultWriterPort implements VaultWriterPort.
var _ spi.VaultWriterPort = (*MockVaultWriterPort)(nil)

// Ensure MockTemplatePort implements TemplatePort.
var _ spi.TemplatePort = (*MockTemplatePort)(nil)

// Ensure MockVaultScannerPort implements VaultScannerPort.
var _ spi.VaultScannerPort = (*MockVaultScannerPort)(nil)

// Ensure MockCacheReaderPort implements CacheReaderPort.
var _ spi.CacheReaderPort = (*MockCacheReaderPort)(nil)

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
	PersistFunc   func(ctx context.Context, note domain.Note, metadata spi.CacheWriteMetadata) error
	DeleteFunc    func(ctx context.Context, path string) error
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
	metadata spi.CacheWriteMetadata,
) error {
	if m.PersistFunc != nil {
		return m.PersistFunc(ctx, note, metadata)
	}
	return m.persistResult
}

// Delete returns the configured mock result for cache deletion.
func (m *MockCacheWriterPort) Delete(
	ctx context.Context,
	path string,
) error {
	if m.DeleteFunc != nil {
		return m.DeleteFunc(ctx, path)
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

// ParseNote returns a mock note for testing. Uses the configured frontmatter
// to create a basic note.
func (m *MockMarkdownParserPort) ParseNote(
	ctx context.Context,
	path string,
	content []byte,
) (domain.Note, error) {
	if m.parseError != nil {
		return domain.Note{}, m.parseError
	}

	fm := domain.NewFrontmatter(m.parseResult)
	note, err := domain.NewNote(path, fm, nil, nil, nil, nil)
	if err != nil {
		return domain.Note{}, err
	}
	return note, nil
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
// operations.
// It allows configuring mock responses for frontmatter operations.
type MockFrontmatterService struct {
	extractResult           domain.Frontmatter
	extractError            error
	isSchemaCompliantResult error
	validateResult          error
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

// SetIsSchemaCompliantResult configures the mock to return the specified error
// on IsSchemaCompliant calls.
func (m *MockFrontmatterService) SetIsSchemaCompliantResult(err error) {
	m.isSchemaCompliantResult = err
}

// SetValidateResult configures the mock to return the specified error
// on Validate calls.
func (m *MockFrontmatterService) SetValidateResult(err error) {
	m.validateResult = err
}

// Extract returns the configured mock result for frontmatter extraction.
func (m *MockFrontmatterService) Extract(
	content []byte,
) (domain.Frontmatter, error) {
	return m.extractResult, m.extractError
}

// IsSchemaCompliant returns the configured mock result for schema compliance
// checking.
func (m *MockFrontmatterService) IsSchemaCompliant(
	ctx context.Context,
	noteID string,
	fm domain.Frontmatter,
) error {
	return m.isSchemaCompliantResult
}

// Validate returns the configured mock result for frontmatter validation.
func (m *MockFrontmatterService) Validate(
	ctx context.Context,
	noteID string,
	fm domain.Frontmatter,
) error {
	return m.validateResult
}

// MockSchemaEngine provides a mock implementation for schema operations.
// It allows configuring mock responses for schema operations.
type MockSchemaEngine struct {
	loadResult      error
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

// SetLoadResult configures the mock to return the specified error on Load
// calls.
func (m *MockSchemaEngine) SetLoadResult(err error) {
	m.loadResult = err
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

// Load returns the configured mock result for schema loading.
func (m *MockSchemaEngine) Load(ctx context.Context) error {
	return m.loadResult
}

// HasSchema returns the configured mock result for schema existence check.
func (m *MockSchemaEngine) HasSchema(ctx context.Context, name string) bool {
	return m.hasSchemaResult
}

// MockCommandPort provides a mock implementation of CommandPort for testing.
// Uses interface{} to avoid import cycles.
type MockCommandPort struct {
	newNoteResult    domain.Note
	newNoteError     error
	indexVaultResult interface{}
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
// error on IndexVault calls.
func (m *MockCommandPort) SetIndexVaultResult(
	stats interface{},
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
) (interface{}, error) {
	return m.indexVaultResult, m.indexVaultError
}

// MockCLIPort provides a mock implementation of CLIPort for testing.
type MockCLIPort struct {
	startResult error
	startCalled bool
	handler     interface{}
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
	handler interface{},
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
func (m *MockCLIPort) GetHandler() interface{} {
	return m.handler
}

// MockVaultIndexer provides a mock implementation for vault indexing.
// It allows configuring mock responses for Build operations.
type MockVaultIndexer struct {
	buildResult IndexStats
	buildError  error
}

// NewMockVaultIndexer creates a new MockVaultIndexer with default values.
func NewMockVaultIndexer() *MockVaultIndexer {
	return &MockVaultIndexer{}
}

// SetBuildResult configures the mock to return the specified stats and error
// on Build calls.
func (m *MockVaultIndexer) SetBuildResult(stats IndexStats, err error) {
	m.buildResult = stats
	m.buildError = err
}

// Build returns the configured mock result for vault indexing.
func (m *MockVaultIndexer) Build(
	ctx context.Context,
) (IndexStats, error) {
	return m.buildResult, m.buildError
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

// Read implements the CacheReaderPort interface for the mock.
func (m *MockMetadataQueryPort) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	// For simplicity, return an error. Tests that need this should implement
	// a more specific mock.
	return domain.Note{}, fmt.Errorf("not implemented")
}

// List implements the CacheReaderPort interface for the mock.
func (m *MockMetadataQueryPort) List(
	ctx context.Context,
) ([]domain.Note, error) {
	// For simplicity, return an error. Tests that need this should implement
	// a more specific mock.
	return nil, fmt.Errorf("not implemented")
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
		return nil, m.loadError
	}
	tmpl, exists := m.templates[id]
	if !exists {
		return nil, fmt.Errorf("template not found: %s", id)
	}
	return tmpl, nil
}

// MockVaultScannerPort provides a mock implementation of VaultScannerPort for
// testing.
type MockVaultScannerPort struct {
	scanAllResult      []dto.VaultFile
	scanAllError       error
	scanModifiedResult []dto.VaultFile
	scanModifiedError  error
}

// NewMockVaultScannerPort creates a new MockVaultScannerPort with default
// values.
func NewMockVaultScannerPort() *MockVaultScannerPort {
	return &MockVaultScannerPort{}
}

// SetScanAllResult configures the mock to return the specified files and error
// on ScanAll calls.
func (m *MockVaultScannerPort) SetScanAllResult(
	files []dto.VaultFile,
	err error,
) {
	m.scanAllResult = files
	m.scanAllError = err
}

// SetScanModifiedResult configures the mock to return the specified files and
// error on ScanModified calls.
func (m *MockVaultScannerPort) SetScanModifiedResult(
	files []dto.VaultFile,
	err error,
) {
	m.scanModifiedResult = files
	m.scanModifiedError = err
}

// ScanAll returns the configured mock result for vault scanning.
func (m *MockVaultScannerPort) ScanAll(
	ctx context.Context,
) ([]dto.VaultFile, error) {
	return m.scanAllResult, m.scanAllError
}

// ScanModified returns the configured mock result for modified file scanning.
func (m *MockVaultScannerPort) ScanModified(
	ctx context.Context,
	since time.Time,
) ([]dto.VaultFile, error) {
	return m.scanModifiedResult, m.scanModifiedError
}

// MockCacheReaderPort provides a mock implementation of CacheReaderPort for
// testing.
type MockCacheReaderPort struct {
	readResult domain.Note
	readError  error
	listResult []domain.Note
	listError  error
}

// NewMockCacheReaderPort creates a new MockCacheReaderPort with default values.
func NewMockCacheReaderPort() *MockCacheReaderPort {
	return &MockCacheReaderPort{}
}

// SetReadResult configures the mock to return the specified note and error on
// Read calls.
func (m *MockCacheReaderPort) SetReadResult(note domain.Note, err error) {
	m.readResult = note
	m.readError = err
}

// SetListResult configures the mock to return the specified notes and error on
// List calls.
func (m *MockCacheReaderPort) SetListResult(notes []domain.Note, err error) {
	m.listResult = notes
	m.listError = err
}

// Read returns the configured mock result for single note reading.
func (m *MockCacheReaderPort) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	return m.readResult, m.readError
}

// List returns the configured mock result for cache reading.
func (m *MockCacheReaderPort) List(ctx context.Context) ([]domain.Note, error) {
	return m.listResult, m.listError
}

// MockEventBus provides a mock implementation of EventBus for testing.
type MockEventBus struct {
	publishResult     error
	subscribeResult   error
	unsubscribeResult error
	shutdownResult    error
	publishedEvents   []domain.DomainEvent
	subscribedTypes   []string
}

// NewMockEventBus creates a new MockEventBus with default values.
func NewMockEventBus() *MockEventBus {
	return &MockEventBus{
		publishResult:     nil,
		subscribeResult:   nil,
		unsubscribeResult: nil,
		shutdownResult:    nil,
		publishedEvents:   make([]domain.DomainEvent, 0),
		subscribedTypes:   make([]string, 0),
	}
}

// SetPublishResult configures the mock to return the specified error on Publish
// calls.
func (m *MockEventBus) SetPublishResult(err error) {
	m.publishResult = err
}

// SetSubscribeResult configures the mock to return the specified error on
// Subscribe calls.
func (m *MockEventBus) SetSubscribeResult(err error) {
	m.subscribeResult = err
}

// SetUnsubscribeResult configures the mock to return the specified error on
// Unsubscribe calls.
func (m *MockEventBus) SetUnsubscribeResult(err error) {
	m.unsubscribeResult = err
}

// SetShutdownResult configures the mock to return the specified error on
// Shutdown calls.
func (m *MockEventBus) SetShutdownResult(err error) {
	m.shutdownResult = err
}

// GetPublishedEvents returns all events that were published.
func (m *MockEventBus) GetPublishedEvents() []domain.DomainEvent {
	return m.publishedEvents
}

// GetSubscribedTypes returns all event types that were subscribed to.
func (m *MockEventBus) GetSubscribedTypes() []string {
	return m.subscribedTypes
}

// Publish records the event and returns the configured mock result.
func (m *MockEventBus) Publish(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	m.publishedEvents = append(m.publishedEvents, event)
	return m.publishResult
}

// Subscribe records the subscription and returns the configured mock result.
func (m *MockEventBus) Subscribe(
	eventType string,
	handler events.EventHandler,
) error {
	m.subscribedTypes = append(m.subscribedTypes, eventType)
	return m.subscribeResult
}

// Unsubscribe returns the configured mock result.
func (m *MockEventBus) Unsubscribe(
	eventType string,
	handler events.EventHandler,
) error {
	return m.unsubscribeResult
}

// Shutdown returns the configured mock result.
func (m *MockEventBus) Shutdown(ctx context.Context) error {
	return m.shutdownResult
}
