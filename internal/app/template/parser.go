// Package template provides domain services for template parsing and execution.
package template

import (
	"context"
	"strings"
	"text/template"

	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// StaticTemplateParser implements spi.TemplateParser using Go's text/template
// engine with custom functions for enhanced template capabilities.
type StaticTemplateParser struct{}

// NewStaticTemplateParser creates a new StaticTemplateParser instance.
func NewStaticTemplateParser() *StaticTemplateParser {
	return &StaticTemplateParser{}
}

// checkContextCancellation checks if the context has been canceled.
// Returns true if canceled, false otherwise.
func (p *StaticTemplateParser) checkContextCancellation(
	ctx context.Context,
) bool {
	select {
	case <-ctx.Done():
		return true
	default:
		return false
	}
}

// createTemplate creates a new template with custom functions registered.
// Returns a template ready for parsing.
func (p *StaticTemplateParser) createTemplate() *template.Template {
	return template.New("template").Funcs(NewFuncMap())
}

// parseTemplate parses the given content using the provided template.
// Returns the parsed template or an error if parsing fails.
func (p *StaticTemplateParser) parseTemplate(
	tmpl *template.Template,
	content string,
) (*template.Template, error) {
	return tmpl.Parse(content)
}

// executeTemplate executes the given template with the provided data.
// Returns the rendered content or an error if execution fails.
func (p *StaticTemplateParser) executeTemplate(
	tmpl *template.Template,
	data interface{},
) (string, error) {
	var buf strings.Builder
	err := tmpl.Execute(&buf, data)
	if err != nil {
		return "", err
	}
	return buf.String(), nil
}

// Parse parses the template content using Go's text/template engine.
// It accepts a context for cancellation support and returns a Result
// containing the parsed template or an error if parsing fails.
func (p *StaticTemplateParser) Parse(
	ctx context.Context,
	content string,
) errors.Result[*template.Template] {
	// Check for context cancellation before starting
	if p.checkContextCancellation(ctx) {
		return errors.Err[*template.Template](ctx.Err())
	}

	// Create a new template with custom functions registered
	tmpl := p.createTemplate()

	// Parse the template content
	parsedTemplate, err := p.parseTemplate(tmpl, content)
	if err != nil {
		return errors.Err[*template.Template](err)
	}

	return errors.Ok(parsedTemplate)
}

// Ensure StaticTemplateParser implements spi.TemplateParser.
var _ spi.TemplateParser = (*StaticTemplateParser)(nil)
