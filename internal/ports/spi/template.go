package spi

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

// TemplatePort defines the Service Provider Interface for template operations.
// This port provides access to template content stored in the filesystem.
// It abstracts the storage mechanism, allowing different adapters (filesystem,
// database, remote storage) to implement template loading.
//
// The port focuses on two core operations:
// 1. Discovering available templates (List)
// 2. Retrieving specific template content (Load)
//
// TemplatePort is implemented by adapters like TemplateLoaderAdapter.
type TemplatePort interface {
	// List returns all available template IDs in the system.
	// Template IDs are derived from template filenames (basename without
	// extension).
	// The returned slice is sorted alphabetically for consistent ordering.
	// Returns an error if the template directory cannot be accessed.
	List(ctx context.Context) ([]domain.TemplateID, error)

	// Load retrieves the content of a specific template by its ID.
	// The ID corresponds to the template filename without path or extension.
	// Returns the complete Template with ID and content.
	// Returns ResourceError if template not found or cannot be read.
	// Returns ValidationError if template content is invalid (e.g., not valid
	// UTF-8).
	Load(ctx context.Context, id domain.TemplateID) (domain.Template, error)
}
