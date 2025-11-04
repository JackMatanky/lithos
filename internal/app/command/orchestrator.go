// Package command provides the CommandOrchestrator domain service for CLI
// command orchestration. It implements the hexagonal callback pattern where the
// domain starts the application and delegates command parsing to CLI adapters.
package command

import (
	"context"
	"os"
	"path/filepath"
	"time"

	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/api"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// CommandOrchestrator orchestrates use case workflows by coordinating domain
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
// CommandOrchestrator (v0.6.4).
type CommandOrchestrator struct {
	cliPort        api.CLIPort
	templateEngine *template.TemplateEngine
	schemaEngine   *schema.SchemaEngine
	vaultIndexer   vault.VaultIndexerInterface
	vaultWriter    spi.VaultWriterPort
	config         domain.Config
	log            zerolog.Logger
}

// NewCommandOrchestrator creates a new CommandOrchestrator with injected
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
//   - *CommandOrchestrator: Fully initialized orchestrator ready for use
//
// Reference: docs/architecture/components.md#domain-services -
// CommandOrchestrator constructor.
func NewCommandOrchestrator(
	cliPort api.CLIPort,
	templateEngine *template.TemplateEngine,
	schemaEngine *schema.SchemaEngine,
	vaultIndexer vault.VaultIndexerInterface,
	vaultWriter spi.VaultWriterPort,
	config *domain.Config,
	log *zerolog.Logger,
) *CommandOrchestrator {
	return &CommandOrchestrator{
		cliPort:        cliPort,
		templateEngine: templateEngine,
		schemaEngine:   schemaEngine,
		vaultIndexer:   vaultIndexer,
		vaultWriter:    vaultWriter,
		config:         *config,
		log:            *log,
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
func (o *CommandOrchestrator) Run(ctx context.Context) error {
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
func (o *CommandOrchestrator) NewNote(
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

	// Step 2: Generate NoteID from templateID (basename strategy)
	noteID := domain.NewNoteID(filepath.Base(string(templateID)))
	o.log.Debug().Str("noteID", string(noteID)).Msg("NoteID generated")

	// Step 3: Create empty Frontmatter (Epic 1 requirement)
	frontmatter := domain.NewFrontmatter(map[string]interface{}{})
	o.log.Debug().Msg("Empty frontmatter created")

	// Step 4: Construct Note
	note := domain.NewNote(noteID, time.Now(), frontmatter)
	o.log.Debug().Str("noteID", string(noteID)).Msg("Note constructed")

	// Step 5: Write file to vault
	relativePath := string(noteID) + ".md"
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
// implementation.
func (o *CommandOrchestrator) IndexVault(
	ctx context.Context,
) (vault.IndexStats, error) {
	o.log.Info().Msg("starting vault indexing")

	// Delegate to VaultIndexer.Build()
	stats, err := o.vaultIndexer.Build(ctx)
	if err != nil {
		o.log.Error().Err(err).Msg("vault indexing failed")
		return stats, lithosErr.WrapWithContext(
			err,
			"vault indexing operation failed",
		)
	}

	// Log summary statistics
	o.log.Info().
		Int("scanned", stats.ScannedCount).
		Int("indexed", stats.IndexedCount).
		Int("validation_failures", stats.ValidationFailures).
		Int("cache_failures", stats.CacheFailures).
		Dur("duration", stats.Duration).
		Msg("vault indexing complete")

	return stats, nil
}
