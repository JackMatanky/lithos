package command

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/api"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

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
	mockCLIPort := utils.NewMockCLIPort()
	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	// Create orchestrator with mock dependencies
	// Note: We pass nil for templateEngine, schemaEngine, and vaultIndexer
	// since we're not
	// testing that in this test
	var templateEngine *template.TemplateEngine
	var schemaEngine *schema.SchemaEngine
	var vaultIndexer *vault.VaultIndexer
	orchestrator := NewCLIComander(
		mockCLIPort,
		templateEngine,
		schemaEngine,
		vaultIndexer,
		nil,
		&config,
		&logger,
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
	mockCLIPort := utils.NewMockCLIPort()
	expectedError := assert.AnError
	mockCLIPort.SetStartError(expectedError)

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	// Create orchestrator with mock dependencies
	var templateEngine *template.TemplateEngine
	var schemaEngine *schema.SchemaEngine
	var vaultIndexer *vault.VaultIndexer
	orchestrator := NewCLIComander(
		mockCLIPort,
		templateEngine,
		schemaEngine,
		vaultIndexer,
		nil,
		&config,
		&logger,
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
	expectedNoteID := domain.NoteID("test-template") // basename strategy

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	// Setup mock template
	mockTemplatePort.SetTemplates(map[domain.TemplateID]domain.Template{
		expectedTemplateID: {
			ID:      expectedTemplateID,
			Content: expectedContent,
		},
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

	var schemaEngine *schema.SchemaEngine
	var vaultIndexer *vault.VaultIndexer
	orchestrator := NewCLIComander(
		nil,
		templateEngine,
		schemaEngine,
		vaultIndexer,
		vaultAdapter.NewVaultWriterAdapter(config, logger),
		&config,
		&logger,
	)

	// Execute
	ctx := context.Background()
	note, err := orchestrator.NewNote(ctx, expectedTemplateID)

	// Verify
	require.NoError(t, err, "NewNote should succeed")
	assert.Equal(
		t,
		expectedNoteID,
		note.ID,
		"NoteID should be generated from templateID basename",
	)
	assert.Empty(
		t,
		note.Frontmatter.Fields,
		"Frontmatter should be empty for Epic 1",
	)

	// Verify file was written
	expectedFilePath := filepath.Join(tempDir, string(expectedNoteID)+".md")
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
	// Setup
	mockTemplatePort := utils.NewMockTemplatePort()
	mockTemplatePort.SetLoadError(assert.AnError) // Simulate not found

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	templateEngine := template.NewTemplateEngine(
		mockTemplatePort,
		&config,
		&logger,
	)

	var schemaEngine *schema.SchemaEngine
	var vaultIndexer *vault.VaultIndexer
	orchestrator := NewCLIComander(
		nil,
		templateEngine,
		schemaEngine,
		vaultIndexer,
		nil,
		&config,
		&logger,
	)

	// Execute
	ctx := context.Background()
	_, err := orchestrator.NewNote(ctx, domain.TemplateID("nonexistent"))

	// Verify
	require.Error(t, err, "NewNote should fail when template not found")
	// Error should be ResourceError from TemplateEngine
}

// TestNewNoteFileWriteError verifies error handling when file write fails.
func TestNewNoteFileWriteError(t *testing.T) {
	// Setup
	mockTemplatePort := utils.NewMockTemplatePort()
	expectedTemplateID := domain.TemplateID("test-template")

	mockTemplatePort.SetTemplates(map[domain.TemplateID]domain.Template{
		expectedTemplateID: {
			ID:      expectedTemplateID,
			Content: "content",
		},
	})

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	templateEngine := template.NewTemplateEngine(
		mockTemplatePort,
		&config,
		&logger,
	)

	var schemaEngine *schema.SchemaEngine
	var vaultIndexer *vault.VaultIndexer
	mockVaultWriter := utils.NewMockVaultWriterPort()
	mockVaultWriter.SetWriteContentResult(assert.AnError)
	orchestrator := NewCLIComander(
		nil,
		templateEngine,
		schemaEngine,
		vaultIndexer,
		mockVaultWriter,
		&config,
		&logger,
	)

	// Execute
	ctx := context.Background()
	_, err := orchestrator.NewNote(ctx, expectedTemplateID)

	// Verify
	require.Error(t, err, "NewNote should fail when file write fails")
	assert.Contains(
		t,
		err.Error(),
		"failed to write note",
		"Error should be wrapped with context",
	)
}

// TestIndexVaultSuccess verifies the complete IndexVault workflow succeeds.
func TestIndexVaultSuccess(t *testing.T) {
	// Setup mock VaultIndexer
	mockVaultIndexer := utils.NewMockVaultIndexer()
	expectedStats := vault.IndexStats{
		ScannedCount:        10,
		IndexedCount:        8,
		CacheFailures:       1,
		ValidationSuccesses: 7,
		ValidationFailures:  2,
		Duration:            150000000, // 150ms
	}
	mockVaultIndexer.SetBuildResult(expectedStats, nil)

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	// Create orchestrator with mock VaultIndexer
	var templateEngine *template.TemplateEngine
	var schemaEngine *schema.SchemaEngine
	var cliPort api.CLIPort
	orchestrator := NewCLIComander(
		cliPort,
		templateEngine,
		schemaEngine,
		mockVaultIndexer,
		nil,
		&config,
		&logger,
	)

	// Execute
	ctx := context.Background()
	stats, err := orchestrator.IndexVault(ctx)

	// Verify
	require.NoError(t, err, "IndexVault should succeed")
	assert.Equal(t, expectedStats, stats, "Stats should match expected")
}

// TestIndexVaultIndexerError verifies error handling when VaultIndexer.Build
// fails.
func TestIndexVaultIndexerError(t *testing.T) {
	// Setup mock VaultIndexer
	mockVaultIndexer := utils.NewMockVaultIndexer()
	expectedError := assert.AnError
	mockVaultIndexer.SetBuildResult(vault.IndexStats{}, expectedError)

	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	// Create orchestrator with mock VaultIndexer
	var templateEngine *template.TemplateEngine
	var schemaEngine *schema.SchemaEngine
	var cliPort api.CLIPort
	// Using deprecated alias to ensure backward compatibility still works.
	orchestrator := NewCommandOrchestrator(
		cliPort,
		templateEngine,
		schemaEngine,
		mockVaultIndexer,
		nil,
		&config,
		&logger,
	)

	// Execute
	ctx := context.Background()
	_, err := orchestrator.IndexVault(ctx)

	// Verify
	require.Error(
		t,
		err,
		"IndexVault should fail when VaultIndexer.Build fails",
	)
	assert.Contains(
		t,
		err.Error(),
		"vault indexing operation failed",
		"Error should be wrapped with context",
	)
}
