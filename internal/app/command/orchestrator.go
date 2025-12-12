// Package command provides the CLIComander domain service for CLI
// command orchestration. It implements the hexagonal callback pattern where the
// domain starts the application and delegates command parsing to CLI adapters.
package command

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/api"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

const indexingCompletionTimeout = 30 * time.Second

// CLIComander orchestrates use case workflows by coordinating domain
// services. It acts as the application service layer that CLI, TUI, and LSP
// adapters invoke
// via CLIPort. The orchestrator owns application startup and control flow.
//
// Responsibilities:
//   - Orchestrate the complete note creation workflow (NewNote use case)
//   - Orchestrate the vault indexing workflow (IndexVault use case)
//
// - Coordinate domain services (TemplateEngine, SchemaEngine, VaultIndexer,
// Config, Logger)
//   - Implement hexagonal callback pattern (pass self to CLIPort.Start)
//   - Handle business logic orchestration without infrastructure concerns
//
// Dependencies (injected via constructor):
//   - CLIPort: CLI framework adapter for command parsing and user interaction
//   - TemplateEngine: Domain service for template loading and rendering
//   - SchemaEngine: Domain service for schema loading and validation
//
// - VaultIndexer: Domain service for vault scanning, frontmatter processing,
// and cache persistence
//   - Config: Application configuration (vault path, etc.)
//   - Logger: Structured logging for workflow operations and debugging
//
// Reference: docs/architecture/components.md#domain-services -
// CLIComander (v0.6.4).
type CLIComander struct {
	cliPort        api.CLIPort
	templateEngine *template.TemplateEngine
	vaultWriter    spi.VaultWriterPort
	config         domain.Config
	log            zerolog.Logger
	eventBus       events.EventBus
}

// NewCLIComander creates a new CLIComander with injected
// dependencies. This constructor follows dependency injection principles,
// ensuring the orchestrator
// has all required collaborators without creating them internally.
//
// Parameters:
//   - cliPort: CLI framework adapter implementing CLIPort interface
//   - templateEngine: Template rendering service for note creation
//   - schemaEngine: Schema loading and validation service
//   - vaultIndexer: Vault indexing service for cache rebuild operations
//   - config: Application configuration containing vault paths and settings
//   - log: Structured logger for workflow operations and debugging
//
// Returns:
// - *CLIComander: Fully initialized workflow coordinator (formerly
// CommandOrchestrator)
//
// Reference: docs/architecture/components.md#domain-services -
// CLIComander constructor.
func NewCLIComander(
	cliPort api.CLIPort,
	templateEngine *template.TemplateEngine,
	vaultWriter spi.VaultWriterPort,
	config *domain.Config,
	log *zerolog.Logger,
	eventBus events.EventBus,
) *CLIComander {
	return &CLIComander{
		cliPort:        cliPort,
		templateEngine: templateEngine,
		vaultWriter:    vaultWriter,
		config:         *config,
		log:            *log,
		eventBus:       eventBus,
	}
}

// Run begins the CLI event loop and command processing.
// This method implements the hexagonal callback pattern where the domain
// starts the application and delegates command parsing to the CLI adapter.
//
// The CLI adapter receives the orchestrator itself as the CommandPort handler,
// allowing it to delegate business logic execution back to the domain through
// the CommandPort interface.
//
// Parameters:
//   - ctx: Context for cancellation and timeout control during CLI execution
//
// Returns:
//   - error: Any startup or execution errors from the CLI framework
//
// Reference: docs/architecture/components.md#api-port-interfaces -
// CLIPort.Start.
func (o *CLIComander) Run(ctx context.Context) error {
	// Hexagonal callback pattern: pass self as CommandPort handler to CLI
	// adapter
	return o.cliPort.Start(ctx, o)
}

// NewNote orchestrates the complete note creation workflow.
// This method implements the CommandPort interface and will be implemented
// in Task 3. For now, it returns a placeholder error.
//
// Parameters:
//   - ctx: Context for cancellation and timeout control
//   - templateID: Identifier of the template to use for note creation
//
// Returns:
//   - domain.Note: The created note (placeholder for now)
//   - error: Implementation pending error
//
// Reference: docs/architecture/components.md#api-port-interfaces -
// CommandPort.NewNote.
func (o *CLIComander) NewNote(
	ctx context.Context,
	templateID domain.TemplateID,
) (domain.Note, error) {
	o.log.Info().
		Str("templateID", string(templateID)).
		Msg("Starting NewNote workflow")

	// Step 1: Render template content
	content, err := o.templateEngine.Render(ctx, templateID)
	if err != nil {
		o.log.Error().
			Err(err).
			Str("templateID", string(templateID)).
			Msg("Template rendering failed")
		return domain.Note{}, err // ResourceError or TemplateError from TemplateEngine
	}
	o.log.Debug().
		Str("templateID", string(templateID)).
		Msg("Template rendered successfully")

	// Step 2-4: Create note from template
	note, relativePath := o.createNoteFromTemplate(templateID)

	// Step 5: Write file to vault
	absolutePath := filepath.Join(o.config.VaultPath, relativePath)
	if o.vaultWriter != nil {
		if writeErr := o.vaultWriter.WriteContent(ctx, relativePath, []byte(content)); writeErr != nil {
			o.log.Error().
				Err(writeErr).
				Str("filePath", absolutePath).
				Msg("Failed to write note file")
			return domain.Note{}, lithosErr.WrapWithContext(
				writeErr,
				"failed to write note to %s", absolutePath,
			)
		}
	} else if err = os.WriteFile( //nolint:gosec // 0o644 is required for note files that may be shared
		absolutePath,
		[]byte(content),
		0o644,
	); err != nil {
		o.log.Error().
			Err(err).
			Str("filePath", absolutePath).
			Msg("Failed to write note file")
		return domain.Note{}, lithosErr.WrapWithContext(
			err,
			"failed to write note to %s", absolutePath,
		)
	}
	o.log.Info().
		Str("filePath", absolutePath).
		Msg("Note file written successfully")
	o.publishNoteIndexedEvent(ctx, note)

	// Step 6: Return Note
	return note, nil
}

// IndexVault orchestrates the vault indexing workflow.
// This method implements the CommandPort interface and delegates to
// VaultIndexer.Build() for the complete indexing operation.
//
// Workflow:
// 1. Log indexing start
// 2. Delegate to VaultIndexer.Build() for scanning, frontmatter processing, and
// caching
// 3. Log summary statistics on completion
// 4. Wrap errors with context for CLI error handling
//
// Parameters:
//   - ctx: Context for cancellation and timeout control during indexing
//
// Returns:
//   - vault.IndexStats: Statistics from the indexing operation (scanned, indexed,
//     failures, duration)
//   - error: Wrapped error if indexing fails (schema load, vault scan, or
//     critical failures)
//
// Reference: docs/architecture/components.md#commandorchestrator - IndexVault
// Reference: docs/architecture/components.md#commandorchestrator (legacy
// anchor)
// IndexVault implementation. Component renamed to CLIComander; anchor retained
// for backward compatibility until docs are fully migrated.
func (o *CLIComander) IndexVault(
	ctx context.Context,
) (vault.IndexStats, error) {
	o.log.Info().Msg("starting vault indexing")

	if o.eventBus == nil {
		return vault.IndexStats{}, fmt.Errorf(
			"event bus not configured for IndexVault",
		)
	}

	statsCh, cleanup, err := o.subscribeToIndexingComplete()
	if err != nil {
		return vault.IndexStats{}, err
	}
	defer cleanup()

	if publishErr := o.publishIndexCommand(ctx); publishErr != nil {
		return vault.IndexStats{}, publishErr
	}

	return o.awaitIndexingComplete(ctx, statsCh)
}

func (o *CLIComander) createNoteFromTemplate(
	templateID domain.TemplateID,
) (note domain.Note, relativePath string) {
	basename := filepath.Base(string(templateID))
	relativePath = basename + ".md"
	o.log.Debug().Str("relativePath", relativePath).Msg("Note path generated")

	frontmatter := domain.NewFrontmatter(map[string]interface{}{})
	o.log.Debug().Msg("Empty frontmatter created")

	note, _ = domain.NewNote(
		relativePath,
		frontmatter,
		[]domain.Link{},
		[]domain.Heading{},
		[]string{},
		[]domain.TaskItem{},
	)
	o.log.Debug().Str("notePath", relativePath).Msg("Note constructed")
	return note, relativePath
}

func (o *CLIComander) subscribeToIndexingComplete() (
	statsCh chan vault.IndexStats,
	cleanup func(),
	err error,
) {
	statsCh = make(chan vault.IndexStats, 1)
	handler := func(handlerCtx context.Context, event domain.DomainEvent) error {
		completeEvent, ok := event.(*domain.VaultIndexingCompleteEvent)
		if !ok {
			return nil
		}
		select {
		case statsCh <- vault.IndexStats{
			ScannedCount:        completeEvent.ScannedCount(),
			IndexedCount:        completeEvent.NotesIndexed(),
			ParseFailures:       completeEvent.ParseFailures(),
			CacheFailures:       completeEvent.CacheFailures(),
			ValidationSuccesses: completeEvent.ValidationSuccesses(),
			ValidationFailures:  completeEvent.ValidationFailures(),
			Duration:            completeEvent.Duration(),
		}:
			return nil
		case <-handlerCtx.Done():
			return handlerCtx.Err()
		}
	}

	err = o.eventBus.Subscribe("VaultIndexingComplete", handler)
	if err != nil {
		return nil, nil, err
	}

	cleanup = func() {
		if unsubErr := o.eventBus.Unsubscribe("VaultIndexingComplete", handler); unsubErr != nil {
			o.log.Warn().Err(unsubErr).Msg("failed to unsubscribe")
		}
	}

	return statsCh, cleanup, nil
}

func (o *CLIComander) publishIndexCommand(ctx context.Context) error {
	commandEvent := domain.MustNewCommandIssuedEvent(
		"IndexVault",
		map[string]string{"source": "cli"},
		time.Now(),
	)
	return o.eventBus.Publish(ctx, commandEvent)
}

func (o *CLIComander) awaitIndexingComplete(
	ctx context.Context,
	statsCh chan vault.IndexStats,
) (vault.IndexStats, error) {
	select {
	case stats := <-statsCh:
		return stats, nil
	case <-ctx.Done():
		return vault.IndexStats{}, ctx.Err()
	case <-time.After(indexingCompletionTimeout):
		return vault.IndexStats{}, fmt.Errorf(
			"timeout waiting for indexing completion",
		)
	}
}

func (o *CLIComander) publishNoteIndexedEvent(
	ctx context.Context,
	note domain.Note,
) {
	if o.eventBus == nil {
		return
	}
	event, err := domain.NewNoteIndexedEvent(note, time.Now())
	if err != nil {
		o.log.Warn().
			Err(err).
			Str("path", note.Path).
			Msg("failed to create note indexed event")
		return
	}
	if publishErr := o.eventBus.Publish(ctx, event); publishErr != nil {
		o.log.Warn().
			Err(publishErr).
			Str("path", note.Path).
			Msg("failed to publish note indexed event")
	}
}
