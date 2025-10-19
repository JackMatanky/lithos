// Package spi defines service provider interface ports for template operations.
package spi

import (
	"context"

	"github.com/jack/lithos/internal/domain"
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
