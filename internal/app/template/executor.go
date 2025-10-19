// Package template provides template processing services for the Lithos
// application.
// This package handles template parsing, function registration, and execution
// to generate rendered content from template files.
package template

import (
	"bytes"
	"context"

	"github.com/jack/lithos/internal/domain"
	"github.com/jack/lithos/internal/shared/errors"
)

// TemplateExecutor defines the interface for template execution services.
// It provides methods to execute parsed templates with data to generate final
// content.
type TemplateExecutor interface {
	// Execute executes a parsed template with the given data and returns the
	// rendered content.
	// The context parameter allows for future cancellation support.
	Execute(
		ctx context.Context,
		tmpl *domain.Template,
		data interface{},
	) (string, error)
}

// GoTemplateExecutor implements TemplateExecutor using Go's text/template
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
