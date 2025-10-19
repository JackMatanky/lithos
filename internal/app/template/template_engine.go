// Package template provides template processing services for the Lithos
// application.
// This package handles template parsing, function registration, and execution
// to generate rendered content from template files.
package template

import (
	"context"
	"fmt"

	"github.com/jack/lithos/internal/domain"
	"github.com/jack/lithos/internal/ports/spi"
	"github.com/jack/lithos/internal/shared/errors"
)

// TemplateEngine provides high-level template processing operations.
// It combines template parsing and execution into a cohesive service
// that handles the complete template rendering workflow.
type TemplateEngine struct {
	parser   spi.TemplateParser
	executor spi.TemplateExecutor
}

// NewTemplateEngine creates a new TemplateEngine instance with
// injected parser and executor implementations.
// This follows dependency injection principles - domain depends on ports,
// not concrete adapters.
func NewTemplateEngine(
	parser spi.TemplateParser,
	executor spi.TemplateExecutor,
) *TemplateEngine {
	return &TemplateEngine{
		parser:   parser,
		executor: executor,
	}
}

// ProcessTemplate processes a template from content string to rendered output.
// This is the main entry point for template processing in the domain layer.
// It handles parsing the template content and executing it to produce final
// output.
func (e *TemplateEngine) ProcessTemplate(
	ctx context.Context,
	content string,
	templateName string,
) (string, error) {
	if content == "" {
		return "", errors.Wrap(
			errors.NewTemplateError(templateName, 0, "content is empty"),
			"cannot process empty template content",
		)
	}

	// Parse the template
	parseResult := e.parser.Parse(ctx, content)
	if parseResult.IsErr() {
		return "", errors.Wrap(
			parseResult.Error(),
			"failed to parse template content",
		)
	}

	// Create domain template object
	tmpl := &domain.Template{
		FilePath: "",
		Name:     templateName,
		Content:  content,
		Parsed:   parseResult.Value(),
	}

	// Execute the template
	renderedContent, err := e.executor.Execute(ctx, tmpl, nil)
	if err != nil {
		return "", errors.Wrap(
			err,
			"failed to execute parsed template",
		)
	}

	return renderedContent, nil
}

// ExecuteParsedTemplate executes a pre-parsed template.
// This method is used when the template has already been parsed by the
// repository.
func (e *TemplateEngine) ExecuteParsedTemplate(
	ctx context.Context,
	tmpl *domain.Template,
) (string, error) {
	if tmpl == nil {
		return "", errors.Wrap(
			errors.NewTemplateError("unknown", 0, "template is nil"),
			"cannot execute nil template",
		)
	}

	if tmpl.Parsed == nil {
		return "", errors.Wrap(
			errors.NewTemplateError(tmpl.Name, 0, "template not parsed"),
			"template must be parsed before execution",
		)
	}

	renderedContent, err := e.executor.Execute(ctx, tmpl, nil)
	if err != nil {
		return "", errors.Wrap(
			err,
			"failed to execute parsed template",
		)
	}

	return renderedContent, nil
}

// ProcessTemplateFromPath processes a template from a file path.
// This method combines template reading, parsing, and execution.
// Note: This method is transitional - in full hexagonal architecture,
// template reading should be handled by the repository port.
func (e *TemplateEngine) ProcessTemplateFromPath(
	ctx context.Context,
	templatePath string,
) (string, error) {
	// This method is kept for backward compatibility during refactoring
	// In the future, this logic should be moved to use TemplateRepositoryPort
	return "", errors.Wrap(
		fmt.Errorf("method not implemented"),
		"ProcessTemplateFromPath not implemented - use repository port",
	)
}
