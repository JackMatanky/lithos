// Package template provides filesystem-based template loading adapters.
// This package implements the TemplatePort interface using filesystem
// operations to discover and load template files from configured directories.
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
// operations for efficient Load() operations without repeated filesystem scans.
type TemplateLoaderAdapter struct {
	// config holds the application configuration, specifically TemplatesDir
	config *domain.Config
	// log provides structured logging for operations
	log *zerolog.Logger
	// metadata caches TemplateID → FileMetadata mappings for fast lookups
	metadata map[domain.TemplateID]spi.FileMetadata
	// readFile reads file contents from filesystem (injected for testing)
	readFile func(string) ([]byte, error)
	// walkDir walks directory tree (injected for testing)
	walkDir func(string, filepath.WalkFunc) error
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
		readFile: os.ReadFile,
		walkDir:  filepath.Walk,
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

	err := a.walkDir(
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

	metadata, err := a.findTemplateMetadata(ctx, id)
	if err != nil {
		return domain.Template{}, err
	}

	content, err := a.readTemplateContent(&metadata)
	if err != nil {
		return domain.Template{}, err
	}

	err = a.validateTemplateContent(content, metadata.Path)
	if err != nil {
		return domain.Template{}, err
	}

	template := domain.NewTemplate(id, string(content))
	a.log.Debug().Str("template_id", string(id)).Int("size", len(content)).
		Msg("template loaded successfully")

	return template, nil
}

// findTemplateMetadata locates template metadata, populating cache if
// necessary.
func (a *TemplateLoaderAdapter) findTemplateMetadata(
	ctx context.Context,
	id domain.TemplateID,
) (spi.FileMetadata, error) {
	metadata, exists := a.metadata[id]
	if exists {
		return metadata, nil
	}

	// Template not in cache, try to populate cache if empty
	if len(a.metadata) == 0 {
		if err := a.populateCache(ctx); err != nil {
			return spi.FileMetadata{}, errors.NewResourceError(
				"template",
				"load",
				string(id),
				fmt.Errorf("template not found and scan failed: %w", err),
			)
		}
		// Check again after scanning
		metadata, exists = a.metadata[id]
	}

	if !exists {
		a.log.Warn().
			Str("template_id", string(id)).
			Msg("template not found in cache")
		return spi.FileMetadata{}, errors.NewResourceError(
			"template",
			"load",
			string(id),
			fmt.Errorf("template not found"),
		)
	}

	return metadata, nil
}

// populateCache scans the templates directory and populates the metadata cache.
func (a *TemplateLoaderAdapter) populateCache(ctx context.Context) error {
	a.log.Debug().Msg("cache is empty, scanning templates directory")
	_, err := a.List(ctx)
	if err != nil {
		a.log.Error().Err(err).Msg("failed to scan templates directory")
		return err
	}
	return nil
}

// readTemplateContent reads the raw file content from disk.
func (a *TemplateLoaderAdapter) readTemplateContent(
	metadata *spi.FileMetadata,
) ([]byte, error) {
	content, err := a.readFile(metadata.Path)
	if err != nil {
		a.log.Error().
			Err(err).
			Str("path", metadata.Path).
			Msg("failed to read template file")
		return nil, errors.NewResourceError(
			"template",
			"read",
			metadata.Path,
			err,
		)
	}
	return content, nil
}

// validateTemplateContent ensures the template content is valid UTF-8.
func (a *TemplateLoaderAdapter) validateTemplateContent(
	content []byte,
	path string,
) error {
	if !utf8.Valid(content) {
		a.log.Warn().
			Str("path", path).
			Msg("template contains invalid UTF-8")
		return errors.NewValidationError(
			"content",
			"invalid UTF-8 encoding",
			path,
			nil,
		)
	}
	return nil
}
