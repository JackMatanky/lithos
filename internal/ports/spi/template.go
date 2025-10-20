// Package spi defines service provider interface ports for template operations.
package spi

import (
	"context"
	"text/template"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// TemplateMetadata provides information about an available template.
type TemplateMetadata struct {
	ID       string
	Name     string
	FilePath string
	Content  string
}

// TemplateRepositoryPort provides access to template storage and enumeration.
// This port allows the domain to access templates without knowing the specific
// storage mechanism (filesystem, remote, etc.).
type TemplateRepositoryPort interface {
	// ListTemplates returns metadata for all available templates.
	ListTemplates(ctx context.Context) ([]TemplateMetadata, error)

	// GetTemplate retrieves a specific template by ID.
	// Returns an error if the template is not found.
	GetTemplate(ctx context.Context, id string) (*domain.Template, error)

	// GetTemplateByPath loads a template from a specific file path.
	// This method supports the current CLI workflow where users specify
	// template paths.
	GetTemplateByPath(
		ctx context.Context,
		path string,
	) (*domain.Template, error)
}

// TemplateParser defines the interface for parsing template content.
// This port allows the domain to parse templates without depending on
// specific template engines or parsing implementations.
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

// TemplateExecutor defines the interface for executing parsed templates.
// This port allows the domain to execute templates without depending on
// specific template execution implementations.
type TemplateExecutor interface {
	// Execute executes a parsed template with the given data and returns the
	// rendered content.
	// Returns an error if template execution fails.
	Execute(
		ctx context.Context,
		tmpl *domain.Template,
		data interface{},
	) (string, error)
}
