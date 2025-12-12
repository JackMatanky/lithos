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
	sharedlogger "github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type failingEventBus struct {
	publishErr error
}

func (f *failingEventBus) Publish(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	return f.publishErr
}

func (f *failingEventBus) Subscribe(
	eventType string,
	handler events.EventHandler,
) error {
	return nil
}

func (f *failingEventBus) Unsubscribe(
	eventType string,
	handler events.EventHandler,
) error {
	return nil
}

func (f *failingEventBus) Shutdown(ctx context.Context) error {
	return nil
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
	mockCLIPort := utils.NewMockCLIPort()
	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	// Create orchestrator with mock dependencies
	// Note: We pass nil for optional dependencies since they're irrelevant here
	orchestrator := NewCLIComander(
		mockCLIPort,
		nil,
		nil,
		&config,
		&logger,
		nil,
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
	orchestrator := NewCLIComander(
		mockCLIPort,
		nil,
		nil,
		&config,
		&logger,
		nil,
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

	orchestrator := NewCLIComander(
		nil,
		templateEngine,
		vaultAdapter.NewVaultWriterAdapter(config, logger),
		&config,
		&logger,
		nil,
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

	orchestrator := NewCLIComander(
		nil,
		templateEngine,
		nil,
		&config,
		&logger,
		nil,
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

	mockVaultWriter := utils.NewMockVaultWriterPort()
	mockVaultWriter.SetWriteContentResult(assert.AnError)
	orchestrator := NewCLIComander(
		nil,
		templateEngine,
		mockVaultWriter,
		&config,
		&logger,
		nil,
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

	busLog := sharedlogger.NewZerologAdapter(
		zerolog.New(zerolog.NewTestWriter(t)),
	)
	bus := events.NewInMemoryEventBus(busLog)
	t.Cleanup(func() {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		defer cancel()
		_ = bus.Shutdown(ctx)
	})

	expectedStats := vault.IndexStats{
		ScannedCount:        10,
		IndexedCount:        8,
		CacheFailures:       1,
		ValidationSuccesses: 7,
		ValidationFailures:  2,
		Duration:            150 * time.Millisecond,
	}

	commandHandler := func(ctx context.Context, event domain.DomainEvent) error {
		cmd, ok := event.(*domain.CommandIssuedEvent)
		if !ok || cmd.Command() != "IndexVault" {
			return nil
		}
		completion := domain.MustNewVaultIndexingCompleteEvent(
			statsToSummary(expectedStats),
			expectedStats.Duration,
			time.Now(),
		)
		publishErr := make(chan error, 1)
		go func() {
			publishErr <- bus.Publish(ctx, completion)
		}()
		return <-publishErr
	}
	require.NoError(t, bus.Subscribe("CommandIssued", commandHandler))
	t.Cleanup(func() {
		_ = bus.Unsubscribe("CommandIssued", commandHandler)
	})

	orchestrator := NewCLIComander(nil, nil, nil, &config, &logger, bus)

	stats, err := orchestrator.IndexVault(context.Background())
	require.NoError(t, err)
	assert.Equal(t, expectedStats, stats)
}

// TestIndexVaultPublishError verifies errors propagate when publish fails.
func TestIndexVaultPublishError(t *testing.T) {
	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	publishErr := assert.AnError
	bus := &failingEventBus{publishErr: publishErr}

	orchestrator := NewCLIComander(nil, nil, nil, &config, &logger, bus)

	_, err := orchestrator.IndexVault(context.Background())
	require.Error(t, err)
	assert.Equal(t, publishErr, err)
}

func statsToSummary(stats vault.IndexStats) domain.VaultIndexingSummary {
	return domain.VaultIndexingSummary{
		ScannedCount:        stats.ScannedCount,
		IndexedCount:        stats.IndexedCount,
		ParseFailures:       stats.ParseFailures,
		CacheFailures:       stats.CacheFailures,
		ValidationSuccesses: stats.ValidationSuccesses,
		ValidationFailures:  stats.ValidationFailures,
	}
}
