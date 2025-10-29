// Package template provides filesystem-based template loading adapters.
// This package implements the TemplatePort interface using filesystem
// operations
// to discover and load template files from configured directories.
package template

import (
	"context"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"unicode/utf8"

	"github.com/JackMatanky/lithos/internal/adapters/spi"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// TemplateLoaderAdapter implements TemplatePort by loading templates from
// filesystem. It scans the configured TemplatesDir for .md files, builds
// metadata for each template, and provides access to template content by ID.
//
// The adapter maintains an internal metadata cache populated during List()
// operations
// for efficient Load() operations without repeated filesystem scans.
type TemplateLoaderAdapter struct {
	// config holds the application configuration, specifically TemplatesDir
	config *domain.Config
	// log provides structured logging for operations
	log *zerolog.Logger
	// metadata caches TemplateID → FileMetadata mappings for fast lookups
	metadata map[domain.TemplateID]spi.FileMetadata
}

// NewTemplateLoaderAdapter creates a new TemplateLoaderAdapter with config and
// logger.
func NewTemplateLoaderAdapter(
	config *domain.Config,
	log *zerolog.Logger,
) *TemplateLoaderAdapter {
	return &TemplateLoaderAdapter{
		config:   config,
		log:      log,
		metadata: make(map[domain.TemplateID]spi.FileMetadata),
	}
}

// List discovers all available templates in the configured TemplatesDir.
// It walks the directory tree, identifies .md files, creates FileMetadata for
// each, and returns sorted TemplateID list.
//
// The metadata cache is populated during this operation for use by Load().
func (a *TemplateLoaderAdapter) List(
	ctx context.Context,
) ([]domain.TemplateID, error) {
	a.log.Debug().
		Str("templates_dir", a.config.TemplatesDir).
		Msg("scanning templates directory")

	var templates []domain.TemplateID

	err := filepath.Walk(
		a.config.TemplatesDir,
		func(path string, info fs.FileInfo, err error) error {
			if err != nil {
				a.log.Error().
					Err(err).
					Str("path", path).
					Msg("error accessing path")
				return err
			}

			// Skip directories
			if info.IsDir() {
				return nil
			}

			// Only process .md files
			if filepath.Ext(path) == ".md" {
				metadata := spi.NewFileMetadata(path, info)
				templateID := domain.TemplateID(metadata.Basename)

				a.metadata[templateID] = metadata
				templates = append(templates, templateID)

				a.log.Debug().
					Str("template_id", string(templateID)).
					Str("path", path).
					Msg("found template")
			}

			return nil
		},
	)

	if err != nil {
		a.log.Error().Err(err).Str("templates_dir", a.config.TemplatesDir).
			Msg("failed to scan templates directory")
		return nil, fmt.Errorf("failed to scan templates directory: %w", err)
	}

	// Sort templates for consistent ordering
	sort.Slice(templates, func(i, j int) bool {
		return string(templates[i]) < string(templates[j])
	})

	a.log.Info().Int("count", len(templates)).Msg("template scan completed")
	return templates, nil
}

// Load retrieves template content by ID from the metadata cache.
// Returns ResourceError if not found, ValidationError if invalid UTF-8.
func (a *TemplateLoaderAdapter) Load(
	ctx context.Context,
	id domain.TemplateID,
) (domain.Template, error) {
	a.log.Debug().Str("template_id", string(id)).Msg("loading template")

	metadata, exists := a.metadata[id]
	if !exists {
		a.log.Warn().
			Str("template_id", string(id)).
			Msg("template not found in cache")
		return domain.Template{}, errors.NewResourceError(
			"template",
			"load",
			string(id),
			fmt.Errorf("template not found"),
		)
	}

	content, err := os.ReadFile(metadata.Path)
	if err != nil {
		a.log.Error().
			Err(err).
			Str("path", metadata.Path).
			Msg("failed to read template file")
		return domain.Template{}, errors.NewResourceError(
			"template",
			"read",
			metadata.Path,
			err,
		)
	}

	// Validate UTF-8 encoding
	if !utf8.Valid(content) {
		a.log.Warn().
			Str("path", metadata.Path).
			Msg("template contains invalid UTF-8")
		return domain.Template{}, errors.NewValidationError(
			"content",
			"invalid UTF-8 encoding",
			metadata.Path,
			nil,
		)
	}

	template := domain.NewTemplate(id, string(content))
	a.log.Debug().Str("template_id", string(id)).Int("size", len(content)).
		Msg("template loaded successfully")

	return template, nil
}
