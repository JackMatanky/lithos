package frontmatter

import (
	"context"
	"errors"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// FrontmatterService validates frontmatter against schema rules with semantic
// business logic enforcement. Pure domain service focused on schema compliance
// validation. Delegates markdown parsing to MarkdownParserPort (adapter layer)
// to maintain clean hexagonal architecture separation.
//
// Architecture: Domain service orchestrates semantic validation using
// MarkdownParserPort for syntactic parsing and schema validation for business
// rules.
//
// Dependencies:
//   - SchemaEngine: For loading and resolving schemas before validation
//   - MarkdownParserPort: For syntactic frontmatter parsing (YAML structure)
//   - Logger: For structured logging of validation operations
type FrontmatterService struct {
	schemaEngine       *schema.SchemaEngine
	markdownParserPort spi.MarkdownParserPort
	logger             zerolog.Logger
	eventBus           events.EventBus
}

// NewFrontmatterService creates a new FrontmatterService with required
// dependencies. It initializes the service for frontmatter validation
// operations.
//
// Parameters:
//   - schemaEngine: Required for schema loading and resolution
//   - markdownParserPort: Required for syntactic frontmatter parsing
//   - logger: Required for observability and error tracking
//
// Returns a configured FrontmatterService ready for use.
func NewFrontmatterService(
	schemaEngine *schema.SchemaEngine,
	markdownParserPort spi.MarkdownParserPort,
	logger zerolog.Logger,
	eventBus events.EventBus,
) *FrontmatterService {
	return &FrontmatterService{
		schemaEngine:       schemaEngine,
		markdownParserPort: markdownParserPort,
		logger:             logger,
		eventBus:           eventBus,
	}
}

// IsSchemaCompliant validates frontmatter against schema rules with semantic
// business logic enforcement. Pure domain service focused on schema compliance
// validation. Frontmatter must be pre-parsed using MarkdownParserPort.
//
// Validation Layer: Domain Layer (Semantic)
// - Performs business rule validation (schema compliance, required fields)
// - Uses pre-parsed frontmatter (syntactic validation done in adapter layer)
// - Does NOT perform parsing or structural validation
//
// Validation Process:
//  1. Validates all required fields are present
//  2. Validates field types using appropriate field validators
//  3. Preserves unknown fields (FR6 compliance)
//  4. Enforces array vs scalar expectations without auto-coercion
//  5. Aggregates all validation errors
//
// Parameters:
//   - ctx: Context for cancellation support
//   - fm: Pre-parsed frontmatter to validate (from MarkdownParserPort)
//
// Returns:
//   - error: Aggregated validation errors or nil if validation passes
//
// Error Handling:
//   - Returns structured FrontmatterError with field context
//   - Aggregates multiple validation errors using errors.Join
//   - Supports cancellation via context
func (s *FrontmatterService) IsSchemaCompliant(
	ctx context.Context,
	noteID string,
	fm domain.Frontmatter,
) error {
	var validationErrors []error

	// Check for cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	// Validate against schema if fileClass is present
	if fm.FileClass() != "" {
		if schemaErr := s.validateAgainstSchema(ctx, fm); schemaErr != nil {
			validationErrors = append(validationErrors, schemaErr)
		}
	}

	validationErr := s.aggregateValidationErrors(validationErrors)
	s.publishValidationEvent(ctx, noteID, fm, validationErr)
	return validationErr
}

// getSchemaForValidation retrieves a schema for frontmatter validation.
// Helper method that wraps schema engine access with error handling.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - fileClass: The schema name to retrieve
//
// Returns:
//   - domain.Schema: The resolved schema for validation
//   - error: Schema retrieval error if schema not found or engine unavailable
func (s *FrontmatterService) getSchemaForValidation(
	ctx context.Context,
	fileClass string,
) (domain.Schema, error) {
	if s.schemaEngine == nil {
		return domain.Schema{}, errors.New("schema engine not available")
	}
	// Use generic Get function from schema package
	return schema.Get[domain.Schema](s.schemaEngine, ctx, fileClass)
}

// validateRequiredFields validates that all required fields are present in
// frontmatter. Helper method for FrontmatterService.Validate to check required
// field constraints.
func (s *FrontmatterService) validateRequiredFields(
	fm domain.Frontmatter,
	sch domain.Schema,
) error {
	var validationErrors []error

	for _, property := range sch.Properties {
		if property.Required {
			if !fm.Has(property.Name) {
				validationErrors = append(
					validationErrors,
					lithosErr.NewFrontmatterError(
						"required field missing",
						property.Name,
						nil,
					),
				)
			}
		}
	}

	if len(validationErrors) > 0 {
		return errors.Join(validationErrors...)
	}
	return nil
}

// validateFieldTypes validates frontmatter field types using polymorphic
// validators. Helper method for FrontmatterService.Validate to perform
// type-specific validation.
func (s *FrontmatterService) validateFieldTypes(
	fm domain.Frontmatter,
	sch domain.Schema,
) error {
	var validationErrors []error

	// Create validator instances
	stringValidator := &StringValidator{}
	numberValidator := &NumberValidator{}
	dateValidator := &DateValidator{}
	boolValidator := &BoolValidator{}

	for _, property := range sch.Properties {
		value, exists := fm.Get(property.Name)
		if !exists {
			continue
		}

		validator := s.selectValidator(
			property.Spec.Type(),
			stringValidator,
			numberValidator,
			dateValidator,
			boolValidator,
		)
		if validator == nil {
			continue
		}

		if err := s.validatePropertyValue(property, value, validator); err != nil {
			validationErrors = append(validationErrors, err)
		}
	}

	if len(validationErrors) > 0 {
		return errors.Join(validationErrors...)
	}
	return nil
}

// selectValidator returns the appropriate validator for a property type.
func (s *FrontmatterService) selectValidator(
	propType domain.PropertySpecType,
	stringVal, numberVal, dateVal, boolVal FieldValidator,
) FieldValidator {
	switch propType {
	case domain.PropertyTypeString:
		return stringVal
	case domain.PropertyTypeNumber:
		return numberVal
	case domain.PropertyTypeDate:
		return dateVal
	case domain.PropertyTypeBool:
		return boolVal
	default:
		return nil
	}
}

// validatePropertyValue validates a single property value (scalar or array).
func (s *FrontmatterService) validatePropertyValue(
	property domain.Property,
	value any,
	validator FieldValidator,
) error {
	if property.Array {
		return s.validateArrayProperty(
			property.Name,
			value,
			property.Spec,
			validator,
		)
	}
	return s.validateScalarProperty(
		property.Name,
		value,
		property.Spec,
		validator,
	)
}

// validateArrayProperty validates array property values.
func (s *FrontmatterService) validateArrayProperty(
	fieldName string,
	value any,
	spec domain.PropertySpec,
	validator FieldValidator,
) error {
	sliceValues, ok := coerceToInterfaceSlice(value)
	if !ok {
		return lithosErr.NewFrontmatterError(
			"field value is not an array",
			fieldName,
			nil,
		)
	}

	var validationErrors []error
	for _, element := range sliceValues {
		if err := validator.Validate(fieldName, element, spec); err != nil {
			validationErrors = append(validationErrors, err)
		}
	}

	if len(validationErrors) > 0 {
		return errors.Join(validationErrors...)
	}
	return nil
}

// validateScalarProperty validates scalar property values.
func (s *FrontmatterService) validateScalarProperty(
	fieldName string,
	value any,
	spec domain.PropertySpec,
	validator FieldValidator,
) error {
	if isSliceValue(value) {
		return lithosErr.NewFrontmatterError(
			"field value must not be an array",
			fieldName,
			nil,
		)
	}
	return validator.Validate(fieldName, value, spec)
}

// validateAgainstSchema performs comprehensive schema validation for
// frontmatter.
// Validates required fields and field types against the schema specification.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - fm: Frontmatter to validate against schema
//
// Returns:
//   - error: Validation errors or nil if validation passes
func (s *FrontmatterService) validateAgainstSchema(
	ctx context.Context,
	fm domain.Frontmatter,
) error {
	var validationErrors []error

	// Get schema for validation
	sch, schemaErr := s.getSchemaForValidation(ctx, fm.FileClass())
	if schemaErr != nil {
		return schemaErr
	}

	// Validate required fields
	if reqErr := s.validateRequiredFields(fm, sch); reqErr != nil {
		validationErrors = append(validationErrors, reqErr)
	}

	// Validate field types
	if typeErr := s.validateFieldTypes(fm, sch); typeErr != nil {
		validationErrors = append(validationErrors, typeErr)
	}

	// Aggregate validation errors
	return s.aggregateValidationErrors(validationErrors)
}

// aggregateValidationErrors aggregates multiple validation errors into a single
// error.
// Helper method for FrontmatterService.Validate to combine validation results.
func (s *FrontmatterService) aggregateValidationErrors(
	validationErrors []error,
) error {
	if len(validationErrors) == 0 {
		return nil
	}
	return errors.Join(validationErrors...)
}

func (s *FrontmatterService) publishValidationEvent(
	ctx context.Context,
	noteID string,
	fm domain.Frontmatter,
	validationErr error,
) {
	if s.eventBus == nil {
		return
	}
	schemaName := fm.FileClass()
	if strings.TrimSpace(noteID) == "" {
		noteID = "frontmatter/" + schemaName
	}
	messages := flattenErrors(validationErr)

	// Create a minimal Note object for the event
	minimalNote := domain.Note{
		Path:        noteID,
		Frontmatter: fm,
		Links:       []domain.Link{},
		Headings:    []domain.Heading{},
		Tags:        []string{},
		Tasks:       []domain.TaskItem{},
		Backlinks:   []domain.Link{},
	}

	event, err := domain.NewFrontmatterValidatedEvent(
		minimalNote,
		schemaName,
		validationErr == nil,
		messages,
		time.Now(),
	)
	if err != nil {
		s.logger.Error().
			Err(err).
			Msg("failed to create frontmatter validated event")
		return
	}
	if publishErr := s.eventBus.Publish(ctx, event); publishErr != nil {
		s.logger.Warn().
			Err(publishErr).
			Msg("failed to publish frontmatter validated event")
	}
}

func coerceToInterfaceSlice(value any) ([]any, bool) {
	switch v := value.(type) {
	case []any:
		return v, true
	case []string:
		result := make([]any, len(v))
		for i, item := range v {
			result[i] = item
		}
		return result, true
	default:
		return nil, false
	}
}

func isSliceValue(value any) bool {
	switch value.(type) {
	case []any, []string:
		return true
	default:
		return false
	}
}

func flattenErrors(err error) []string {
	if err == nil {
		return nil
	}
	type unwrapper interface{ Unwrap() []error }
	if u, ok := err.(unwrapper); ok {
		var result []string
		for _, inner := range u.Unwrap() {
			result = append(result, flattenErrors(inner)...)
		}
		return result
	}
	return []string{err.Error()}
}
