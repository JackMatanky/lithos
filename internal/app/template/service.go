// Package template provides the TemplateEngine domain service for template
// rendering. It orchestrates template loading, parsing, and execution with
// custom functions
// for the lithos new and find commands.
package template

import (
	"context"
	"fmt"
	"path/filepath"
	"strings"
	"text/template"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// TemplateEngine provides template rendering capabilities with custom
// functions.
// It loads templates from the TemplatePort, parses them with Go text/template,
// and executes them with domain-specific functions for file path control and
// timestamp formatting.
//
// TemplateEngine follows hexagonal architecture principles:
// - Depends on TemplatePort (SPI) for template loading
// - Accepts Config for vault path access
// - Uses zerolog for structured logging
// - Returns domain errors (ResourceError, TemplateError) for clean error
// handling.
type TemplateEngine struct {
	templatePort spi.TemplatePort
	config       *domain.Config
	log          *zerolog.Logger
}

// NewTemplateEngine creates a new TemplateEngine with injected dependencies.
// The TemplateEngine is ready to load and render templates immediately after
// construction. Dependencies are injected following dependency inversion
// principles.
//
// Parameters:
//   - templatePort: SPI adapter for loading templates from storage
//   - config: Application configuration containing vault path and settings
//   - log: Structured logger for operation tracing and debugging
//
// Returns a pointer to the initialized TemplateEngine.
func NewTemplateEngine(
	templatePort spi.TemplatePort,
	config *domain.Config,
	log *zerolog.Logger,
) *TemplateEngine {
	return &TemplateEngine{
		templatePort: templatePort,
		config:       config,
		log:          log,
	}
}

// Render loads, parses, and executes a template with custom functions.
// This is the main public method for template rendering in Lithos.
//
// The method follows a 6-step workflow:
// 1. Load template via Load(ctx, templateID)
// 2. Create text/template instance with template.ID as name
// 3. Register function map via buildFuncMap()
// 4. Parse template.Content using template.Parse()
// 5. Execute template with empty data context (static rendering for Epic 1)
// 6. Return rendered string
//
// Parameters:
//   - ctx: Context for cancellation and timeout control
//   - templateID: The identifier of the template to render
//
// Returns:
//   - string: The rendered template content
//
// - error: ResourceError if template not found, TemplateError for parse/execute
// issues.
func (e *TemplateEngine) Render(
	ctx context.Context,
	templateID domain.TemplateID,
) (string, error) {
	// Step 1: Load template
	tmpl, err := e.Load(ctx, templateID)
	if err != nil {
		return "", err // ResourceError from Load()
	}

	// Step 2-3: Create text/template with function map
	t := template.New(string(tmpl.ID)).Funcs(e.buildFuncMap())

	// Step 4: Parse template content
	t, err = t.Parse(tmpl.Content)
	if err != nil {
		return "", errors.NewTemplateError(
			fmt.Sprintf("parse error in template '%s'", tmpl.ID),
			string(tmpl.ID),
			err,
		)
	}

	// Step 5-6: Execute with empty data context and return
	var buf strings.Builder
	if executeErr := t.Execute(&buf, nil); executeErr != nil {
		return "", errors.NewTemplateError(
			fmt.Sprintf("execute error in template '%s'", tmpl.ID),
			string(tmpl.ID),
			executeErr,
		)
	}

	e.log.Info().
		Str("templateID", string(templateID)).
		Msg("template rendered successfully")
	return buf.String(), nil
}

// Load retrieves a template by its ID from the TemplatePort.
// This method delegates to the injected TemplatePort adapter, providing
// a clean domain service interface while maintaining hexagonal architecture
// separation of concerns.
//
// The method logs the loading operation at debug level for observability
// and returns the template with its content ready for rendering.
//
// Parameters:
//   - ctx: Context for cancellation and timeout control
//   - templateID: The identifier of the template to load
//
// Returns:
//   - Template: The loaded template with ID and content
//   - error: ResourceError if template not found or loading fails
func (e *TemplateEngine) Load(
	ctx context.Context,
	templateID domain.TemplateID,
) (domain.Template, error) {
	e.log.Debug().Str("templateID", string(templateID)).Msg("loading template")
	return e.templatePort.Load(ctx, templateID)
}

// buildFuncMap creates and returns a template.FuncMap containing all custom
// template functions for Lithos. This includes basic string manipulation
// functions and file path control functions.
//
// The function map is registered with Go's text/template engine to enable
// domain-specific functionality in templates. Functions are organized into
// logical categories for maintainability.
//
// Returns a template.FuncMap ready for use with template.Funcs().
func (e *TemplateEngine) buildFuncMap() template.FuncMap {
	return template.FuncMap{
		// Basic functions
		"now":     func(format string) string { return time.Now().Format(format) },
		"toLower": strings.ToLower,
		"toUpper": strings.ToUpper,

		// File path control functions
		"path":   func() string { return "" }, // Empty for Epic 1
		"folder": filepath.Dir,
		"basename": func(p string) string {
			base := filepath.Base(p)
			return strings.TrimSuffix(base, filepath.Ext(base))
		},
		"extension": filepath.Ext,
		"join":      filepath.Join,
		"vaultPath": func() string { return e.config.VaultPath },
	}
}
