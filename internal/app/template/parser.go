// Package template provides domain services for template parsing and execution.
package template

import (
	"context"
	"strings"
	"text/template"

	"github.com/jack/lithos/internal/shared/errors"
)

// TemplateParser defines the interface for parsing template content.
type TemplateParser interface {
	// Parse parses the given template content and returns a parsed template.
	// Returns an error if the template has syntax errors.
	Parse(
		ctx context.Context,
		templateContent string,
	) errors.Result[*template.Template]

	// Execute executes a parsed template with the given data and returns the
	// rendered content.
	// Returns an error if template execution fails.
	Execute(
		ctx context.Context,
		tmpl *template.Template,
		data interface{},
	) errors.Result[string]
}

// StaticTemplateParser implements TemplateParser using Go's text/template
// engine
// for static template parsing without custom functions.
type StaticTemplateParser struct{}

// NewStaticTemplateParser creates a new StaticTemplateParser instance.
func NewStaticTemplateParser() *StaticTemplateParser {
	return &StaticTemplateParser{}
}

// Parse parses the template content using Go's text/template engine.
// It accepts a context for cancellation support and returns a Result containing
// the parsed template or an error if parsing fails.
func (p *StaticTemplateParser) Parse(
	ctx context.Context,
	content string,
) errors.Result[*template.Template] {
	// Check for context cancellation before starting
	select {
	case <-ctx.Done():
		return errors.Err[*template.Template](ctx.Err())
	default:
	}

	// Create a new template with a generic name for parsing
	tmpl := template.New("template")

	// Parse the template content
	parsedTemplate, err := tmpl.Parse(content)
	if err != nil {
		return errors.Err[*template.Template](err)
	}

	return errors.Ok(parsedTemplate)
}

// Execute executes a parsed template with the given data and returns the
// rendered content.
// It accepts a context for cancellation support and returns a Result containing
// the rendered string or an error if execution fails.
func (p *StaticTemplateParser) Execute(ctx context.Context,
	tmpl *template.Template, data interface{}) errors.Result[string] {
	// Check for context cancellation before starting
	select {
	case <-ctx.Done():
		return errors.Err[string](ctx.Err())
	default:
	}

	// Create a buffer to capture the template output
	var buf strings.Builder

	// Execute the template with the provided data
	err := tmpl.Execute(&buf, data)
	if err != nil {
		return errors.Err[string](err)
	}

	return errors.Ok(buf.String())
}
