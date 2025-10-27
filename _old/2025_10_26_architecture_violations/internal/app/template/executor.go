// Package template provides domain services for template parsing and execution.
package template

import (
	"bytes"
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
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
) errors.Result[string] {
	if err := e.validateTemplate(tmpl); err != nil {
		return errors.Err[string](err)
	}

	if err := e.validateData(data, tmpl.Name); err != nil {
		return errors.Err[string](err)
	}

	return e.executeTemplate(tmpl)
}

// validateTemplate performs validation checks on the template before execution.
func (e *GoTemplateExecutor) validateTemplate(tmpl *domain.Template) error {
	return validateTemplateForExecution(tmpl)
}

// validateData ensures no external data is passed for MVP.
func (e *GoTemplateExecutor) validateData(
	data interface{},
	tmplName string,
) error {
	if data != nil {
		return errors.Wrap(
			errors.NewTemplateError(
				tmplName,
				0,
				"external data not supported in MVP",
				nil,
			),
			"template execution must use nil data parameter",
		)
	}

	return nil
}

// executeTemplate performs the actual template execution into a buffer.
func (e *GoTemplateExecutor) executeTemplate(
	tmpl *domain.Template,
) errors.Result[string] {
	var buf bytes.Buffer
	if err := tmpl.Parsed.Execute(&buf, nil); err != nil {
		return errors.Err[string](errors.Wrap(
			errors.NewTemplateError(tmpl.Name, 0, err.Error(), err),
			"template execution failed",
		))
	}

	return errors.Ok(buf.String())
}

// Ensure GoTemplateExecutor implements spi.TemplateExecutor.
var _ spi.TemplateExecutor = (*GoTemplateExecutor)(nil)
