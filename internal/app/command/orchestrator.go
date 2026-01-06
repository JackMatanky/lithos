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
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/metrics"
	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/api"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

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
//   - VaultIndexer: Domain service for vault indexing operations
//   - VaultWriter: Vault persistence adapter
//   - FrontmatterService: Domain service for frontmatter validation
//   - MarkdownParserPort: SPI port for markdown parsing
//   - Config: Application configuration (vault path, etc.)
//   - Logger: Structured logging for workflow operations and debugging
//
// Reference: docs/architecture/components.md#domain-services -
// CLIComander (v0.6.4).
type CLIComander struct {
	cliPort            api.CLIPort
	templateEngine     *template.TemplateEngine
	vaultIndexer       vault.VaultIndexerInterface
	vaultWriter        spi.VaultWriterPort
	frontmatterService *frontmatter.FrontmatterService
	markdownParserPort spi.MarkdownParserPort
	config             domain.Config
	log                zerolog.Logger
	eventBus           events.EventBus
}

// NewCLIComander creates a new CLIComander with injected
// dependencies. This constructor follows dependency injection principles,
// ensuring the orchestrator
// has all required collaborators without creating them internally.
//
// Parameters:
//   - cliPort: CLI framework adapter implementing CLIPort interface
//   - templateEngine: Template rendering service for note creation
//   - vaultIndexer: Vault indexing service for cache rebuild operations
//   - vaultWriter: Vault writer service for note persistence
//   - frontmatterService: Frontmatter validation service
//   - markdownParserPort: Markdown parsing port for frontmatter extraction
//   - config: Application configuration containing vault paths and settings
//   - log: Structured logger for workflow operations and debugging
//   - eventBus: Event bus for domain events
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
	vaultIndexer vault.VaultIndexerInterface,
	vaultWriter spi.VaultWriterPort,
	frontmatterService *frontmatter.FrontmatterService,
	markdownParserPort spi.MarkdownParserPort,
	config *domain.Config,
	log *zerolog.Logger,
	eventBus events.EventBus,
) *CLIComander {
	return &CLIComander{
		cliPort:            cliPort,
		templateEngine:     templateEngine,
		vaultIndexer:       vaultIndexer,
		vaultWriter:        vaultWriter,
		frontmatterService: frontmatterService,
		markdownParserPort: markdownParserPort,
		config:             *config,
		log:                *log,
		eventBus:           eventBus,
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
// This method implements the CommandPort interface and coordinates template
// rendering, frontmatter validation, and vault persistence.
//
// Workflow:
// 1. Render template content
// 2. Parse and validate frontmatter against schemas
// 3. Create note with validated frontmatter
// 4. Persist note to vault
// 5. Publish note indexed event
//
// Parameters:
//   - ctx: Context for cancellation and timeout control
//   - templateID: Identifier of the template to use for note creation
//
// Returns:
//   - domain.Note: The created and validated note
//   - error: Template rendering, validation, or persistence errors
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

	// Step 2: Parse and validate frontmatter
	fm, err := o.parseAndValidateFrontmatter(ctx, templateID, content)
	if err != nil {
		return domain.Note{}, err
	}

	// Step 3: Create note from template with validated frontmatter
	note, relativePath := o.createNoteFromTemplateWithFrontmatter(
		templateID,
		fm,
	)

	// Step 6: Write file to vault
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
	o.publishNoteCreatedEvent(ctx, note, templateID)

	// Step 7: Return Note
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
// - metrics.IndexStats: Statistics from the indexing operation (scanned,
// indexed,
//
//	  failures, duration)
//	- error: Wrapped error if indexing fails (schema load, vault scan, or
//	  critical failures)
//
// Reference: docs/architecture/components.md#commandorchestrator - IndexVault
// Reference: docs/architecture/components.md#commandorchestrator (legacy
// anchor).
func (o *CLIComander) IndexVault(
	ctx context.Context,
) (metrics.IndexStats, error) {
	o.log.Info().Msg("starting vault indexing")

	if o.vaultIndexer == nil {
		return metrics.IndexStats{}, fmt.Errorf(
			"vault indexer not configured for IndexVault",
		)
	}

	stats, err := o.vaultIndexer.Build(ctx)
	if err != nil {
		o.log.Error().Err(err).Msg("vault indexing failed")
		return metrics.IndexStats{}, fmt.Errorf("indexing failed: %w", err)
	}

	o.log.Info().
		Int("scanned", stats.ScannedCount).
		Int("indexed", stats.IndexedCount).
		Int("validation_failures", stats.ValidationFailures).
		Int("cache_failures", stats.CacheFailures).
		Dur("duration", stats.Duration).
		Msg("vault indexing completed")

	return stats, nil
}

func (o *CLIComander) parseAndValidateFrontmatter(
	ctx context.Context,
	templateID domain.TemplateID,
	content string,
) (domain.Frontmatter, error) {
	// Parse frontmatter from rendered content (if parser available)
	var fm domain.Frontmatter
	if o.markdownParserPort != nil {
		fmData, err := o.markdownParserPort.ParseFrontmatter(
			ctx,
			[]byte(content),
		)
		if err != nil {
			o.log.Error().
				Err(err).
				Str("templateID", string(templateID)).
				Msg("Frontmatter parsing failed")
			return domain.Frontmatter{}, lithosErr.WrapWithContext(
				err,
				"failed to parse frontmatter from template %s", templateID,
			)
		}
		o.log.Debug().
			Str("templateID", string(templateID)).
			Msg("Frontmatter parsed successfully")
		fm = domain.NewFrontmatter(fmData)
	} else {
		// No parser available, use empty frontmatter
		fm = domain.NewFrontmatter(map[string]interface{}{})
		o.log.Debug().
			Str("templateID", string(templateID)).
			Msg("No frontmatter parser available, using empty frontmatter")
	}

	// Validate frontmatter against schema
	if o.frontmatterService != nil {
		if validationErr := o.frontmatterService.Validate(
			ctx, string(templateID), fm,
		); validationErr != nil {
			o.log.Error().
				Err(validationErr).
				Str("templateID", string(templateID)).
				Msg("Frontmatter validation failed")
			return domain.Frontmatter{}, lithosErr.WrapWithContext(
				validationErr,
				"frontmatter validation failed for template %s", templateID,
			)
		}
		o.log.Debug().
			Str("templateID", string(templateID)).
			Msg("Frontmatter validation passed")
	}

	return fm, nil
}

func (o *CLIComander) createNoteFromTemplateWithFrontmatter(
	templateID domain.TemplateID,
	fm domain.Frontmatter,
) (note domain.Note, relativePath string) {
	basename := filepath.Base(string(templateID))
	relativePath = basename + ".md"
	o.log.Debug().Str("relativePath", relativePath).Msg("Note path generated")

	o.log.Debug().Msg("Using provided frontmatter")

	note, _ = domain.NewNote(
		relativePath,
		fm,
		[]domain.Link{},
		[]domain.Heading{},
		[]string{},
		[]domain.TaskItem{},
	)
	o.log.Debug().Str("notePath", relativePath).Msg("Note constructed")
	return note, relativePath
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
	if publishErr := events.PublishSync(ctx, o.eventBus, event); publishErr != nil {
		o.log.Warn().
			Err(publishErr).
			Str("path", note.Path).
			Msg("failed to publish note indexed event")
	}
}

func (o *CLIComander) publishNoteCreatedEvent(
	ctx context.Context,
	note domain.Note,
	templateID domain.TemplateID,
) {
	if o.eventBus == nil {
		return
	}

	fileClass := note.FileClass()
	if fileClass == "" {
		fileClass = "unknown"
	}

	event, err := domain.NewNoteCreatedEvent(
		note.Path,
		fileClass,
		string(templateID),
		time.Now(),
	)
	if err != nil {
		o.log.Warn().
			Err(err).
			Str("path", note.Path).
			Msg("failed to create note created event")
		return
	}

	if publishErr := events.PublishSync(ctx, o.eventBus, event); publishErr != nil {
		o.log.Warn().
			Err(publishErr).
			Str("path", note.Path).
			Msg("failed to publish note created event")
	}
}
