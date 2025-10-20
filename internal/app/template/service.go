// Package template provides template processing services for the Lithos
// application.
// This package handles template parsing, function registration, and execution
// to generate rendered content from template files.
package template

import (
	"context"
	"fmt"
	"text/template"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
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
	if err := e.validateContent(content, templateName); err != nil {
		return "", err
	}

	parsed, err := e.parseTemplate(ctx, content)
	if err != nil {
		return "", err
	}

	tmpl := e.createDomainTemplate(templateName, content, parsed)

	return e.executeTemplate(ctx, tmpl)
}

// ExecuteParsedTemplate executes a pre-parsed template.
// This method is used when the template has already been parsed by the
// repository.
func (e *TemplateEngine) ExecuteParsedTemplate(
	ctx context.Context,
	tmpl *domain.Template,
) (string, error) {
	if err := e.validateTemplate(tmpl); err != nil {
		return "", err
	}

	return e.executeTemplate(ctx, tmpl)
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

// validateContent checks if the template content is not empty.
func (e *TemplateEngine) validateContent(
	content string,
	templateName string,
) error {
	if content == "" {
		return errors.Wrap(
			errors.NewTemplateError(templateName, 0, "content is empty"),
			"cannot process empty template content",
		)
	}
	return nil
}

// parseTemplate parses the template content using the injected parser.
func (e *TemplateEngine) parseTemplate(
	ctx context.Context,
	content string,
) (*template.Template, error) {
	parseResult := e.parser.Parse(ctx, content)
	if parseResult.IsErr() {
		return nil, errors.Wrap(
			parseResult.Error(),
			"failed to parse template content",
		)
	}
	return parseResult.Value(), nil
}

// createDomainTemplate creates a domain template object from the parsed data.
func (e *TemplateEngine) createDomainTemplate(
	templateName string,
	content string,
	parsed *template.Template,
) *domain.Template {
	return &domain.Template{
		FilePath: "",
		Name:     templateName,
		Content:  content,
		Parsed:   parsed,
	}
}

// validateTemplateForExecution performs validation checks on a template before
// execution.
// This function is shared across the package to avoid code duplication.
func validateTemplateForExecution(tmpl *domain.Template) error {
	if tmpl == nil {
		return errors.Wrap(
			errors.NewTemplateError("unknown", 0, "template is nil"),
			"cannot execute nil template",
		)
	}

	if tmpl.Parsed == nil {
		return errors.Wrap(
			errors.NewTemplateError(tmpl.Name, 0, "template not parsed"),
			"template must be parsed before execution",
		)
	}

	return nil
}

// validateTemplate performs validation on a pre-parsed template.
func (e *TemplateEngine) validateTemplate(tmpl *domain.Template) error {
	return validateTemplateForExecution(tmpl)
}

// executeTemplate executes the template using the injected executor.
func (e *TemplateEngine) executeTemplate(
	ctx context.Context,
	tmpl *domain.Template,
) (string, error) {
	renderedContent, err := e.executor.Execute(ctx, tmpl, nil)
	if err != nil {
		return "", errors.Wrap(
			err,
			"failed to execute parsed template",
		)
	}
	return renderedContent, nil
}
