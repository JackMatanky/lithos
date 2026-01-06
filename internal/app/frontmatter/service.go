package frontmatter

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/converters"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// FrontmatterService validates frontmatter against schema rules.
type FrontmatterService struct {
	schemaEngine  *schema.SchemaEngine
	validators    *ValidatorRegistry
	metadataQuery spi.MetadataQueryPort
	eventBus      events.EventBus
	logger        zerolog.Logger
}

// NewFrontmatterService creates a new FrontmatterService with standard
// validators.
func NewFrontmatterService(
	schemaEngine *schema.SchemaEngine,
	logger zerolog.Logger,
	eventBus events.EventBus,
	metadataQuery spi.MetadataQueryPort,
) *FrontmatterService {
	return &FrontmatterService{
		schemaEngine:  schemaEngine,
		validators:    DefaultValidatorRegistry(),
		metadataQuery: metadataQuery,
		eventBus:      eventBus,
		logger:        logger,
	}
}

// Validate validates frontmatter against schema rules and FileSpec properties.
func (s *FrontmatterService) Validate(
	ctx context.Context,
	noteID string,
	fm domain.Frontmatter,
) error {
	start := time.Now()
	var errs []error

	if err := ctx.Err(); err != nil {
		return err
	}

	if err := s.IsSchemaCompliant(ctx, noteID, fm); err != nil {
		errs = append(errs, err)
	}

	if fm.FileClass() != "" {
		if err := s.validateFileSpecProperties(ctx, fm); err != nil {
			errs = append(errs, err)
		}
	}

	aggErr := s.aggregateValidationErrors(errs)
	duration := time.Since(start)
	publishValidation(ctx, s.eventBus, s.logger, noteID, fm, aggErr, duration)
	return aggErr
}

// IsSchemaCompliant validates frontmatter against schema rules.
func (s *FrontmatterService) IsSchemaCompliant(
	ctx context.Context,
	noteID string,
	fm domain.Frontmatter,
) error {
	start := time.Now()
	var errs []error

	if err := ctx.Err(); err != nil {
		return err
	}

	if fm.FileClass() != "" {
		if err := s.validateAgainstSchema(ctx, fm); err != nil {
			errs = append(errs, err)
		}
	}

	aggErr := s.aggregateValidationErrors(errs)
	duration := time.Since(start)
	publishValidation(ctx, s.eventBus, s.logger, noteID, fm, aggErr, duration)
	return aggErr
}

func (s *FrontmatterService) getSchemaForValidation(
	ctx context.Context,
	fileClass string,
) (domain.Schema, error) {
	if s.schemaEngine == nil {
		return domain.Schema{}, errors.New("schema engine not available")
	}
	return schema.Get[domain.Schema](s.schemaEngine, ctx, fileClass)
}

func (s *FrontmatterService) validateRequiredFields(
	fm domain.Frontmatter,
	sch domain.Schema,
) error {
	var errs []error
	for i := range sch.Properties {
		prop := sch.Properties[i]
		if prop.Required && !fm.Has(prop.Name) {
			missingErr := lithosErr.NewFrontmatterError(
				"required field missing",
				prop.Name,
				nil,
			)
			errs = append(errs, missingErr)
		}
	}
	return s.aggregateValidationErrors(errs)
}

func (s *FrontmatterService) validateFieldTypes(
	fm domain.Frontmatter,
	sch domain.Schema,
) error {
	var errs []error
	for i := range sch.Properties {
		prop := sch.Properties[i]
		val, exists := fm.Get(prop.Name)
		if !exists {
			continue
		}

		v, ok := s.validators.Get(prop.Spec.Type())
		if !ok {
			continue
		}

		if err := s.validatePropertyValue(prop, val, v); err != nil {
			errs = append(errs, err)
		}
	}
	return s.aggregateValidationErrors(errs)
}

func (s *FrontmatterService) validatePropertyValue(
	prop domain.Property,
	val any,
	v FieldValidator,
) error {
	if prop.Array {
		return s.validateArrayProperty(prop.Name, val, prop.Spec, v)
	}
	return s.validateScalarProperty(prop.Name, val, prop.Spec, v)
}

func (s *FrontmatterService) validateArrayProperty(
	name string,
	val any,
	spec domain.PropertySpec,
	v FieldValidator,
) error {
	vals, ok := converters.ToSlice(val)
	if !ok {
		return lithosErr.NewFrontmatterError(
			"field value is not an array",
			name,
			nil,
		)
	}

	var errs []error
	for i := range vals {
		if err := v.Validate(name, vals[i], spec); err != nil {
			errs = append(errs, err)
		}
	}
	return s.aggregateValidationErrors(errs)
}

func (s *FrontmatterService) validateScalarProperty(
	name string,
	val any,
	spec domain.PropertySpec,
	v FieldValidator,
) error {
	if converters.IsSlice(val) {
		return lithosErr.NewFrontmatterError(
			"field value must not be an array",
			name,
			nil,
		)
	}
	return v.Validate(name, val, spec)
}

func (s *FrontmatterService) validateAgainstSchema(
	ctx context.Context,
	fm domain.Frontmatter,
) error {
	var errs []error
	sch, schemaErr := s.getSchemaForValidation(ctx, fm.FileClass())
	if schemaErr != nil {
		return schemaErr
	}

	if err := s.validateRequiredFields(fm, sch); err != nil {
		errs = append(errs, err)
	}
	if err := s.validateFieldTypes(fm, sch); err != nil {
		errs = append(errs, err)
	}
	return s.aggregateValidationErrors(errs)
}

func (s *FrontmatterService) validateFileSpecField(
	ctx context.Context,
	name string,
	val any,
	isArray bool,
) []error {
	if isArray {
		return s.validateFileSpecArray(ctx, name, val)
	}
	var errs []error
	if str, ok := val.(string); ok {
		if err := s.validateFileReference(ctx, name, str); err != nil {
			errs = append(errs, err)
		}
	}
	return errs
}

func (s *FrontmatterService) validateFileSpecArray(
	ctx context.Context,
	name string,
	val any,
) []error {
	var errs []error
	vals, ok := converters.ToSlice(val)
	if !ok {
		return errs
	}
	for i := range vals {
		if str, ok2 := vals[i].(string); ok2 {
			if err := s.validateFileReference(ctx, name, str); err != nil {
				errs = append(errs, err)
			}
		}
	}
	return errs
}

func (s *FrontmatterService) validateFileSpecProperties(
	ctx context.Context,
	fm domain.Frontmatter,
) error {
	var errs []error
	sch, err := s.getSchemaForValidation(ctx, fm.FileClass())
	if err != nil {
		return err
	}

	for i := range sch.Properties {
		prop := sch.Properties[i]
		if prop.Spec.Type() != domain.PropertyTypeFile {
			continue
		}

		val, exists := fm.Get(prop.Name)
		if !exists {
			continue
		}

		fErrs := s.validateFileSpecField(ctx, prop.Name, val, prop.Array)
		errs = append(errs, fErrs...)
	}
	return s.aggregateValidationErrors(errs)
}

func (s *FrontmatterService) validateFileReference(
	ctx context.Context,
	name, val string,
) error {
	if s.metadataQuery == nil {
		return lithosErr.NewValidationErrorWithRemediation(
			name, "query service unavailable", val,
			"Ensure QueryService is properly injected", nil)
	}

	path, isWikilink := s.resolveWikilink(val)
	path = s.normalizePath(path)

	opts := spi.PathQueryOptions{Value: path, Scope: spi.PathQueryScopeFull}
	if isWikilink {
		opts.Scope = spi.PathQueryScopeBasename
	}

	notes, err := s.metadataQuery.PathQuery(ctx, opts)
	if err != nil {
		return lithosErr.NewValidationErrorWithRemediation(
			name, "query failed", val,
			"Check QueryService configuration", err)
	}

	if len(notes) == 0 {
		hint := s.generateQueryHint(path, isWikilink)
		return lithosErr.NewValidationErrorWithRemediation(
			name, "file not found", val, hint, nil)
	}

	if len(notes) > 1 && isWikilink {
		return s.createAmbiguousError(name, val, path, notes)
	}
	return nil
}

func (s *FrontmatterService) resolveWikilink(val string) (string, bool) {
	if strings.HasPrefix(val, "[[") && strings.HasSuffix(val, "]]") &&
		len(val) > 4 {
		return val[2 : len(val)-2], true
	}
	return val, false
}

func (s *FrontmatterService) normalizePath(path string) string {
	return strings.TrimSuffix(filepath.Clean(path), "/")
}

func (s *FrontmatterService) createAmbiguousError(
	name, val, path string,
	notes []domain.Note,
) error {
	paths := make([]string, len(notes))
	for i := range notes {
		paths[i] = notes[i].Path
	}
	hint := fmt.Sprintf("ambiguous wikilink [[%s]]: matches %v", path, paths)
	return lithosErr.NewValidationErrorWithRemediation(
		name,
		"ambiguous reference",
		val,
		hint,
		nil,
	)
}

func (s *FrontmatterService) generateQueryHint(
	path string,
	isWikilink bool,
) string {
	if isWikilink {
		return fmt.Sprintf("Check if [[%s]] exists in vault.", path)
	}
	return fmt.Sprintf("Check if '%s' exists in vault.", path)
}

func (s *FrontmatterService) aggregateValidationErrors(errs []error) error {
	if len(errs) == 0 {
		return nil
	}
	return errors.Join(errs...)
}
