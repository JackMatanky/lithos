package frontmatter

import (
	"context"
	"errors"

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
) *FrontmatterService {
	return &FrontmatterService{
		schemaEngine:       schemaEngine,
		markdownParserPort: markdownParserPort,
		logger:             logger,
	}
}

// IsSchemaCompliant validates frontmatter against schema rules with semantic
// business logic enforcement. Pure domain service focused on schema compliance
// validation. Frontmatter must be pre-parsed using MarkdownParserPort.
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

	// Unknown fields are preserved per FR6 - no validation needed

	// Aggregate validation errors
	return s.aggregateValidationErrors(validationErrors)
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
			// Field not present - not an error for type validation
			continue
		}

		// Select appropriate validator based on property spec type
		var validator FieldValidator
		switch property.Spec.Type() {
		case domain.PropertyTypeString:
			validator = stringValidator
		case domain.PropertyTypeNumber:
			validator = numberValidator
		case domain.PropertyTypeDate:
			validator = dateValidator
		case domain.PropertyTypeBool:
			validator = boolValidator
		default:
			// Unknown property type - skip validation
			continue
		}

		// Validate field value using appropriate validator
		if err := validator.Validate(property.Name, value, property.Spec); err != nil {
			validationErrors = append(validationErrors, err)
		}
	}

	if len(validationErrors) > 0 {
		return errors.Join(validationErrors...)
	}
	return nil
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
