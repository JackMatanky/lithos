// Package schema provides domain services for schema validation and processing.
// This package implements the application layer business logic for validating
// frontmatter data against schema definitions using PropertySpec polymorphism.
package schema

import (
	"context"
	"reflect"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// SchemaValidator implements domain service for validating frontmatter data
// against schema definitions. It orchestrates validation using PropertySpec
// polymorphism and provides structured error reporting.
type SchemaValidator struct {
	registry spi.SchemaRegistryPort
}

// NewSchemaValidator creates a new SchemaValidator with injected dependencies.
// Follows domain service patterns with proper dependency injection.
func NewSchemaValidator(registry spi.SchemaRegistryPort) *SchemaValidator {
	return &SchemaValidator{
		registry: registry,
	}
}

// Validate validates frontmatter data against a specific schema.
// Returns Result[ValidationResult] containing all field-level validation
// errors.
// Uses PropertySpec polymorphism for type-specific validation rules.
func (v *SchemaValidator) Validate(
	ctx context.Context,
	schemaName string,
	frontmatter domain.Frontmatter,
) errors.Result[errors.ValidationResult] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return errors.Err[errors.ValidationResult](ctx.Err())
	default:
	}

	// Look up schema from registry
	schema, found := v.registry.Get(schemaName)
	if !found {
		return errors.Err[errors.ValidationResult](
			errors.NewSchemaNotFoundError(schemaName),
		)
	}

	// Validate frontmatter.FileClass matches schema name
	if frontmatter.SchemaName() != schemaName {
		result := errors.NewValidationResult()
		result.AddError(errors.NewPropertySpecError(
			"fileClass",
			frontmatter.SchemaName(),
			errors.NewValidationError(
				"fileClass",
				"must match schema name",
				frontmatter.SchemaName(),
			),
		))
		return errors.Ok[errors.ValidationResult](result)
	}

	// Validate against resolved properties (post-inheritance)
	result := v.validateFrontmatter(
		ctx,
		frontmatter,
		schema.GetResolvedProperties(),
	)
	return errors.Ok[errors.ValidationResult](result)
}

// validateFrontmatter validates frontmatter fields against schema properties.
// Returns ValidationResult with all field-level errors found.
func (v *SchemaValidator) validateFrontmatter(
	ctx context.Context,
	frontmatter domain.Frontmatter,
	properties []domain.Property,
) errors.ValidationResult {
	result := errors.NewValidationResult()

	// Check required fields first
	v.validateRequiredFields(frontmatter, properties, &result)

	// Validate each property against frontmatter
	for _, prop := range properties {
		// Check for context cancellation periodically
		select {
		case <-ctx.Done():
			return result
		default:
		}

		v.validateProperty(ctx, frontmatter, prop, &result)
	}

	return result
}

// validateRequiredFields checks that all required properties are present in
// frontmatter.
func (v *SchemaValidator) validateRequiredFields(
	frontmatter domain.Frontmatter,
	properties []domain.Property,
	result *errors.ValidationResult,
) {
	for _, prop := range properties {
		if !prop.Required {
			continue
		}

		if _, exists := frontmatter.Fields[prop.Name]; !exists {
			result.AddError(errors.NewRequiredFieldError(prop.Name))
		}
	}
}

// validateProperty validates a single property against frontmatter data.
func (v *SchemaValidator) validateProperty(
	ctx context.Context, //nolint:unparam // for future PropertySpec validation
	frontmatter domain.Frontmatter,
	prop domain.Property,
	result *errors.ValidationResult,
) {
	fieldValue, exists := frontmatter.Fields[prop.Name]
	if !exists {
		// Optional field not present - skip validation
		return
	}

	// Check array constraints
	isArray := v.isArrayValue(fieldValue)
	if prop.Array && !isArray {
		result.AddError(
			errors.NewArrayConstraintError(prop.Name, fieldValue, "array"),
		)
		return
	}
	if !prop.Array && isArray {
		result.AddError(
			errors.NewArrayConstraintError(prop.Name, fieldValue, "scalar"),
		)
		return
	}

	// Validate using Property.ValidateValue which handles array iteration
	if err := prop.ValidateValue(fieldValue); err != nil {
		result.AddError(errors.NewPropertySpecError(prop.Name, fieldValue, err))
	}
}

// isArrayValue checks if a value is an array/slice type.
func (v *SchemaValidator) isArrayValue(value interface{}) bool {
	if value == nil {
		return false
	}
	kind := reflect.TypeOf(value).Kind()
	return kind == reflect.Slice || kind == reflect.Array
}
