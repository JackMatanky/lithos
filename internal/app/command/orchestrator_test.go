package command

import (
	"context"
	"os"
	"path/filepath"
	"testing"
	"time"

	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/api"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockVaultIndexer provides a mock implementation of VaultIndexerInterface for
// testing.
type mockVaultIndexer struct {
	buildResult vault.IndexStats
	buildError  error
}

// mockCLIPort provides a mock implementation of CLIPort for testing.
type mockCLIPort struct {
	startResult error
	startCalled bool
	handler     api.CommandPort
}

func (m *mockVaultIndexer) Build(
	ctx context.Context,
) (vault.IndexStats, error) {
	return m.buildResult, m.buildError
}

func (m *mockVaultIndexer) SetBuildResult(stats vault.IndexStats, err error) {
	m.buildResult = stats
	m.buildError = err
}

func (m *mockCLIPort) Start(
	ctx context.Context,
	handler api.CommandPort,
) error {
	m.startCalled = true
	m.handler = handler
	return m.startResult
}

func (m *mockCLIPort) SetStartError(err error) {
	m.startResult = err
}

func (m *mockCLIPort) WasStartCalled() bool {
	return m.startCalled
}

func (m *mockCLIPort) GetHandler() api.CommandPort {
	return m.handler
}

// TestCLIComanderStructExists verifies that CLIComander struct
// can be compiled. This is a compilation test to ensure the struct definition
// is syntactically correct.
func TestCLIComanderStructExists(t *testing.T) {
	// This test verifies the CLIComander struct (renamed from
	// CommandOrchestrator)
	// exists and can be instantiated. Backward-compatible constructor alias
	// NewCommandOrchestrator should still compile elsewhere.
	var orchestrator *CLIComander
	assert.Nil(
		t,
		orchestrator,
		"CLIComander struct should exist and be nil initially",
	)
}

// TestRunCallsCLIPortStart verifies that Run() calls CLIPort.Start() with
// correct parameters.
func TestRunCallsCLIPortStart(t *testing.T) {
	// Setup
	mockCLIPort := &mockCLIPort{}
	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	// Create orchestrator with mock dependencies
	// Note: We pass nil for optional dependencies since they're irrelevant here
	var mockVaultIndexer *vault.VaultIndexer
	var mockVaultWriter spi.VaultWriterPort
	var mockEventBus events.EventBus
	var mockTemplateEngine *template.TemplateEngine
	orchestrator := NewCLIComander(
		mockCLIPort,
		mockTemplateEngine,
		mockVaultIndexer,
		mockVaultWriter,
		&config,
		&logger,
		mockEventBus,
	)

	// Execute
	ctx := context.Background()
	err := orchestrator.Run(ctx)

	// Verify
	require.NoError(
		t,
		err,
		"Run should not return error when CLIPort.Start succeeds",
	)
	assert.True(
		t,
		mockCLIPort.WasStartCalled(),
		"CLIPort.Start should be called",
	)
	assert.Equal(t, orchestrator, mockCLIPort.GetHandler(),
		"Orchestrator should pass itself as handler")
}

// TestRunPropagatesCLIError verifies that Run() propagates errors from
// CLIPort.Start().
func TestRunPropagatesCLIError(t *testing.T) {
	// Setup
	mockCLIPort := &mockCLIPort{}
	expectedError := assert.AnError
	mockCLIPort.SetStartError(expectedError)

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	// Create orchestrator with mock dependencies
	var mockVaultIndexer *vault.VaultIndexer
	var mockVaultWriter spi.VaultWriterPort
	var mockEventBus events.EventBus
	var mockTemplateEngine *template.TemplateEngine
	orchestrator := NewCLIComander(
		mockCLIPort,
		mockTemplateEngine,
		mockVaultIndexer,
		mockVaultWriter,
		&config,
		&logger,
		mockEventBus,
	)

	// Execute
	ctx := context.Background()
	err := orchestrator.Run(ctx)

	// Verify
	require.Error(t, err, "Run should propagate error from CLIPort.Start")
	assert.Equal(
		t,
		expectedError,
		err,
		"Run should return the exact error from CLIPort.Start",
	)
	assert.True(
		t,
		mockCLIPort.WasStartCalled(),
		"CLIPort.Start should still be called",
	)
}

// TestNewNoteSuccess verifies the complete NewNote workflow succeeds.
func TestNewNoteSuccess(t *testing.T) {
	// Setup
	mockTemplatePort := utils.NewMockTemplatePort()
	expectedContent := "# Test Note\n\nThis is test content."
	expectedTemplateID := domain.TemplateID("test-template")
	expectedNotePath := "test-template.md" // basename + extension

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	// Setup mock template
	mockTemplatePort.SetTemplates(map[domain.TemplateID]domain.Template{
		expectedTemplateID: domain.NewTemplate(
			expectedTemplateID,
			expectedContent,
		),
	})

	// Create template engine with mock port
	templateEngine := template.NewTemplateEngine(
		mockTemplatePort,
		&config,
		&logger,
	)

	// Create temp dir for vault
	tempDir := t.TempDir()
	config.VaultPath = tempDir

	var mockVaultIndexer *vault.VaultIndexer
	var mockEventBus events.EventBus
	orchestrator := NewCLIComander(
		nil,
		templateEngine,
		mockVaultIndexer,
		vaultAdapter.NewVaultWriterAdapter(config, logger),
		&config,
		&logger,
		mockEventBus,
	)

	// Execute
	ctx := context.Background()
	note, err := orchestrator.NewNote(ctx, expectedTemplateID)

	// Verify
	require.NoError(t, err, "NewNote should succeed")
	assert.Equal(
		t,
		expectedNotePath,
		note.Path,
		"Note path should be generated from templateID basename",
	)
	assert.Empty(
		t,
		note.Frontmatter.Fields,
		"Frontmatter should be empty for Epic 1",
	)

	// Verify file was written
	expectedFilePath := filepath.Join(tempDir, expectedNotePath)
	assert.FileExists(
		t,
		expectedFilePath,
		"Note file should be written to vault",
	)

	// Verify file content
	content, err := os.ReadFile(expectedFilePath)
	require.NoError(t, err, "Should be able to read written file")
	assert.Equal(
		t,
		expectedContent,
		string(content),
		"File content should match rendered template",
	)
}

// TestNewNoteTemplateNotFound verifies error handling when template is not
// found.
func TestNewNoteTemplateNotFound(t *testing.T) {
	mockTemplatePort := utils.NewMockTemplatePort()
	mockTemplatePort.SetLoadError(assert.AnError)

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	templateEngine := template.NewTemplateEngine(
		mockTemplatePort,
		&config,
		&logger,
	)

	var mockVaultIndexer *vault.VaultIndexer
	var mockEventBus events.EventBus
	orchestrator := NewCLIComander(
		nil,
		templateEngine,
		mockVaultIndexer,
		nil,
		&config,
		&logger,
		mockEventBus,
	)

	ctx := context.Background()
	_, err := orchestrator.NewNote(ctx, domain.TemplateID("nonexistent"))

	require.Error(t, err, "NewNote should fail when template not found")
}

// TestNewNoteFileWriteError verifies error handling when file write fails.
func TestNewNoteFileWriteError(t *testing.T) {
	mockTemplatePort := utils.NewMockTemplatePort()
	expectedTemplateID := domain.TemplateID("test-template")

	mockTemplatePort.SetTemplates(map[domain.TemplateID]domain.Template{
		expectedTemplateID: domain.NewTemplate(expectedTemplateID, "content"),
	})

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	templateEngine := template.NewTemplateEngine(
		mockTemplatePort,
		&config,
		&logger,
	)

	mockVaultWriter := utils.NewMockVaultWriterPort()
	mockVaultWriter.SetWriteContentResult(assert.AnError)
	var mockVaultIndexer *vault.VaultIndexer
	var mockEventBus events.EventBus
	orchestrator := NewCLIComander(
		nil,
		templateEngine,
		mockVaultIndexer,
		mockVaultWriter,
		&config,
		&logger,
		mockEventBus,
	)

	ctx := context.Background()
	_, err := orchestrator.NewNote(ctx, expectedTemplateID)

	require.Error(t, err, "NewNote should fail when file write fails")
	assert.Contains(t, err.Error(), "failed to write note")
}

// TestIndexVaultSuccess verifies the event-driven indexing flow.
func TestIndexVaultSuccess(t *testing.T) {
	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	expectedStats := vault.IndexStats{
		ScannedCount:        10,
		IndexedCount:        8,
		CacheFailures:       1,
		ValidationSuccesses: 7,
		ValidationFailures:  2,
		Duration:            150 * time.Millisecond,
	}

	mockVaultIndexer := &mockVaultIndexer{}
	mockVaultIndexer.SetBuildResult(expectedStats, nil)

	var mockVaultWriter spi.VaultWriterPort
	var mockTemplateEngine *template.TemplateEngine
	var mockEventBus events.EventBus
	orchestrator := NewCLIComander(
		nil,
		mockTemplateEngine,
		mockVaultIndexer,
		mockVaultWriter,
		&config,
		&logger,
		mockEventBus,
	)

	stats, err := orchestrator.IndexVault(context.Background())
	require.NoError(t, err)
	assert.Equal(t, expectedStats, stats)
}

// TestIndexVaultBuildError verifies errors propagate when vault indexer fails.
func TestIndexVaultBuildError(t *testing.T) {
	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	buildErr := assert.AnError
	mockVaultIndexer := &mockVaultIndexer{}
	mockVaultIndexer.SetBuildResult(vault.IndexStats{}, buildErr)

	var mockVaultWriter spi.VaultWriterPort
	var mockTemplateEngine *template.TemplateEngine
	var mockEventBus events.EventBus
	orchestrator := NewCLIComander(
		nil,
		mockTemplateEngine,
		mockVaultIndexer,
		mockVaultWriter,
		&config,
		&logger,
		mockEventBus,
	)

	_, err := orchestrator.IndexVault(context.Background())
	require.Error(t, err)
	assert.Contains(t, err.Error(), buildErr.Error())
}
