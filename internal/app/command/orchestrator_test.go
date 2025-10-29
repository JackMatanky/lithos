package command

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestCommandOrchestratorStructExists verifies that CommandOrchestrator struct
// can be compiled. This is a compilation test to ensure the struct definition
// is syntactically correct.
func TestCommandOrchestratorStructExists(t *testing.T) {
	// This test verifies the CommandOrchestrator struct exists and can be
	// instantiated
	// (though it will be nil since we don't have dependencies yet)
	var orchestrator *CommandOrchestrator
	assert.Nil(
		t,
		orchestrator,
		"CommandOrchestrator struct should exist and be nil initially",
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
	// Note: We pass nil for templateEngine since we're not testing that in this
	// test
	var templateEngine *template.TemplateEngine
	orchestrator := NewCommandOrchestrator(
		mockCLIPort,
		templateEngine,
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
	orchestrator := NewCommandOrchestrator(
		mockCLIPort,
		templateEngine,
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

	orchestrator := NewCommandOrchestrator(
		nil,
		templateEngine,
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

	orchestrator := NewCommandOrchestrator(
		nil,
		templateEngine,
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

	// Use invalid path to cause write error
	config.VaultPath = "/invalid/path/that/does/not/exist"

	templateEngine := template.NewTemplateEngine(
		mockTemplatePort,
		&config,
		&logger,
	)

	orchestrator := NewCommandOrchestrator(
		nil,
		templateEngine,
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
