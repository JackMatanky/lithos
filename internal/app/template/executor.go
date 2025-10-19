// Package template provides domain services for template parsing and execution.
package template

import (
	"bytes"
	"context"

	"github.com/jack/lithos/internal/domain"
	"github.com/jack/lithos/internal/ports/spi"
	"github.com/jack/lithos/internal/shared/errors"
)

// GoTemplateExecutor implements spi.TemplateExecutor using Go's text/template
// package.
// It executes templates that have been parsed with custom function maps.
type GoTemplateExecutor struct{}

// NewGoTemplateExecutor creates a new GoTemplateExecutor instance.
func NewGoTemplateExecutor() *GoTemplateExecutor {
	return &GoTemplateExecutor{}
}

// Execute executes the parsed template with the provided data and returns the
// rendered content. For MVP, the data parameter is always nil as templates
// receive no external data. The context parameter is accepted for future
// cancellation support but not currently used.
func (e *GoTemplateExecutor) Execute(
	ctx context.Context,
	tmpl *domain.Template,
	data interface{},
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

	// For MVP, ensure no external data is passed (data should be nil)
	if data != nil {
		return "", errors.Wrap(
			errors.NewTemplateError(
				tmpl.Name,
				0,
				"external data not supported in MVP",
			),
			"template execution must use nil data parameter",
		)
	}

	// Execute template into a buffer
	var buf bytes.Buffer
	if err := tmpl.Parsed.Execute(&buf, nil); err != nil {
		return "", errors.Wrap(
			errors.NewTemplateError(tmpl.Name, 0, err.Error()),
			"template execution failed",
		)
	}

	return buf.String(), nil
}

// Ensure GoTemplateExecutor implements spi.TemplateExecutor.
var _ spi.TemplateExecutor = (*GoTemplateExecutor)(nil)
