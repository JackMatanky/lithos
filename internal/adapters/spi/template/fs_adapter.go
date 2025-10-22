// Package template provides SPI adapter implementations for template
// operations.
package template

import (
	"context"
	"path/filepath"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// FSAdapter implements TemplateRepositoryPort using filesystem
// operations.
// It provides template loading capabilities from the local filesystem.
type FSAdapter struct {
	fileSystemPort spi.FileSystemPort
	parser         spi.TemplateParser
}

// NewFSAdapter creates a new filesystem-based template repository
// adapter with injected parser dependency.
func NewFSAdapter(
	fileSystemPort spi.FileSystemPort,
	parser spi.TemplateParser,
) *FSAdapter {
	return &FSAdapter{
		fileSystemPort: fileSystemPort,
		parser:         parser,
	}
}

// List returns metadata for all available templates.
// Currently returns an empty list as template enumeration is not yet
// implemented.
func (a *FSAdapter) List(
	ctx context.Context,
) ([]spi.TemplateMetadata, error) {
	// TODO: Implement template enumeration from configured template directories
	// This will be needed for Epic 4 (interactive template selection)
	return []spi.TemplateMetadata{}, nil
}

// Get retrieves a specific template by ID.
// Currently not implemented as ID-based lookup requires template enumeration.
func (a *FSAdapter) Get(
	ctx context.Context,
	id string,
) (*domain.Template, error) {
	// TODO: Implement ID-based template lookup
	// This requires implementing List first
	return nil, errors.NewTemplateError(id, 0, "not found", nil)
}

// GetByPath loads a template from a specific file path.
// This method supports the current CLI workflow where users specify template
// paths.
func (a *FSAdapter) GetByPath(
	ctx context.Context,
	path string,
) (*domain.Template, error) {
	// Read the template file
	content, err := a.fileSystemPort.ReadFile(path)
	if err != nil {
		return nil, errors.WrapWithContext(
			errors.Wrap(err, "failed to read template file"),
			map[string]interface{}{"path": path},
		)
	}

	// Parse the template content
	parseResult := a.parser.Parse(ctx, string(content))
	if parseResult.IsErr() {
		return nil, errors.WrapWithContext(
			errors.Wrap(parseResult.Error(), "failed to parse template"),
			map[string]interface{}{"path": path},
		)
	}

	// Extract template name from path
	templateName := filepath.Base(path)
	if ext := filepath.Ext(templateName); ext != "" {
		templateName = templateName[:len(templateName)-len(ext)]
	}

	// Create and return domain template object
	return &domain.Template{
		FilePath: path,
		Name:     templateName,
		Content:  string(content),
		Parsed:   parseResult.Value(),
	}, nil
}
