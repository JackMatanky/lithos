// Package frontmatter provides domain services for frontmatter validation.
// This package implements the application layer business logic for validating
// frontmatter fields against schema definitions.
package frontmatter

import (
	"context"
	"errors"
	"fmt"
	"regexp"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// FrontmatterValidator implements domain service for validating frontmatter
// fields against schema definitions. FrontmatterValidator validates frontmatter
// data structures against pre-validated schemas accessed through SchemaEngine.
//
// FrontmatterValidator follows domain service patterns with dependency
// injection
// and uses Result[T] pattern for functional error handling.
type FrontmatterValidator struct {
	schemaEngine interface {
		GetSchema(ctx context.Context, name string) lithoserrors.Result[domain.Schema]
	}
}

// NewFrontmatterValidator creates a new FrontmatterValidator with dependency
// injection.
// FrontmatterValidator requires SchemaEngine for schema access - it does NOT
// access SchemaRegistryPort directly, maintaining the architectural boundary
// where SchemaEngine is the ONLY gateway for schema operations.
func NewFrontmatterValidator(
	schemaEngine interface {
		GetSchema(ctx context.Context, name string) lithoserrors.Result[domain.Schema]
	},
) *FrontmatterValidator {
	return &FrontmatterValidator{
		schemaEngine: schemaEngine,
	}
}

// Validate validates frontmatter fields against a schema.
// Returns Result[ValidationResult] with detailed field-level validation
// lithoserrors.
// The schema must be pre-loaded and validated by SchemaEngine before
// validation.
//
// This method validates:
// - Required field constraints (field must exist in frontmatter.Fields)
// - Array constraints (Array=true requires slice values, Array=false requires
// scalar)
// - Type-specific PropertySpec validation using polymorphism
// - Inheritance support through schema.ResolvedProperties
//
// Context cancellation is supported for long-running validations.
func (v *FrontmatterValidator) Validate(
	ctx context.Context,
	schemaName string,
	frontmatter domain.Frontmatter,
) lithoserrors.Result[lithoserrors.ValidationResult] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[lithoserrors.ValidationResult](ctx.Err())
	default:
	}

	// CRITICAL: Schema must be loaded/validated by SchemaEngine BEFORE
	// frontmatter validation
	schemaResult := v.schemaEngine.GetSchema(ctx, schemaName)
	if schemaResult.IsErr() {
		return lithoserrors.Err[lithoserrors.ValidationResult](
			schemaResult.Error(),
		)
	}

	schema, _ := schemaResult.Unwrap()

	// Use ResolvedProperties for inheritance support
	properties := schema.GetResolvedProperties()

	// Validate each frontmatter field against schema properties
	return v.validateFields(ctx, frontmatter, properties)
}

// validateFields validates individual frontmatter fields against schema
// properties.
// This is a private helper that implements the core validation logic.
func (v *FrontmatterValidator) validateFields(
	ctx context.Context,
	frontmatter domain.Frontmatter,
	properties []domain.Property,
) lithoserrors.Result[lithoserrors.ValidationResult] {
	result := lithoserrors.NewValidationResult()

	// Validate each property in the schema
	for _, property := range properties {
		// Check for context cancellation during field validation loop
		select {
		case <-ctx.Done():
			return lithoserrors.Err[lithoserrors.ValidationResult](ctx.Err())
		default:
		}

		fieldResult := v.validateField(frontmatter, property)
		if fieldResult.IsErr() {
			fieldError := func() lithoserrors.FieldValidationError {
				var target lithoserrors.FieldValidationError
				_ = errors.As(fieldResult.Error(), &target)
				return target
			}()
			result.AddError(fieldError)
		}
	}

	if result.IsValid() {
		return lithoserrors.Ok[lithoserrors.ValidationResult](result)
	}
	return lithoserrors.Err[lithoserrors.ValidationResult](
		lithoserrors.NewFieldValidationError(
			"validation",
			"multiple field validation errors",
			nil,
			nil,
		),
	)
}

// validateField validates a single frontmatter field against a property
// specification. Returns Result[FieldValidationError] - success means field is
// valid, error contains field details.
func (v *FrontmatterValidator) validateField(
	frontmatter domain.Frontmatter,
	property domain.Property,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	fieldName := property.Name
	fieldValue, exists := frontmatter.Fields[fieldName]

	if err := v.checkRequiredField(fieldName, exists, property.Required); err.IsErr() {
		return err
	}

	if !exists {
		return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
	}

	if property.Array {
		return v.validateArrayField(fieldName, fieldValue, property.Spec)
	}

	return v.validateScalarField(fieldName, fieldValue, property.Spec)
}

// checkRequiredField validates that a required field exists.
func (v *FrontmatterValidator) checkRequiredField(
	fieldName string,
	exists, required bool,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	if required && !exists {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewRequiredFieldError(fieldName),
		)
	}
	return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
}

// validateArrayField validates an array-type field.
func (v *FrontmatterValidator) validateArrayField(
	fieldName string,
	fieldValue interface{},
	spec domain.PropertySpec,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	if !isArrayValue(fieldValue) {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewArrayConstraintError(
				fieldName,
				fieldValue,
				"array",
			),
		)
	}

	arrayValues, _ := fieldValue.([]interface{})
	for i, elem := range arrayValues {
		elemFieldName := fmt.Sprintf("%s[%d]", fieldName, i)
		if result := v.validatePropertySpecValue(elemFieldName, elem, spec); result.IsErr() {
			return result
		}
	}

	return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
}

// validateScalarField validates a scalar (non-array) field.
func (v *FrontmatterValidator) validateScalarField(
	fieldName string,
	fieldValue interface{},
	spec domain.PropertySpec,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	if isArrayValue(fieldValue) {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewArrayConstraintError(
				fieldName,
				fieldValue,
				"scalar",
			),
		)
	}

	return v.validatePropertySpecValue(fieldName, fieldValue, spec)
}

// validatePropertySpecValue validates a field value against its PropertySpec
// using polymorphism. This delegates to type-specific validation methods based
// on the PropertySpec type.
func (v *FrontmatterValidator) validatePropertySpecValue(
	fieldName string,
	fieldValue interface{},
	spec domain.PropertySpec,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	// Use type assertion to determine the specific PropertySpec type
	// and delegate to appropriate validation logic

	switch s := spec.(type) {
	case *domain.StringPropertySpec:
		return v.validateStringPropertySpec(fieldName, fieldValue, s)
	case *domain.NumberPropertySpec:
		return v.validateNumberPropertySpec(fieldName, fieldValue, s)
	case *domain.DatePropertySpec:
		return v.validateDatePropertySpec(fieldName, fieldValue, s)
	case *domain.FilePropertySpec:
		return v.validateFilePropertySpec(fieldName, fieldValue, s)
	case *domain.BoolPropertySpec:
		return v.validateBoolPropertySpec(fieldName, fieldValue, s)
	default:
		// Unknown PropertySpec type - this should not happen in a well-formed
		// schema
		return lithoserrors.Err[lithoserrors.FieldValidationError](lithoserrors.NewFieldValidationError(
			fieldName,
			"unknown property specification type",
			fieldValue,
			fmt.Errorf("unsupported PropertySpec type: %T", spec),
		))
	}
}

// validateStringPropertySpec validates a field value against StringPropertySpec
// constraints.
// validateStringEnum checks if a string value is in the allowed enum values.
func validateStringEnum(
	fieldName, str string,
	enum []string,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	for _, enumValue := range enum {
		if str == enumValue {
			return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
		}
	}
	return lithoserrors.Err[lithoserrors.FieldValidationError](
		lithoserrors.NewFieldValidationError(
			fieldName,
			fmt.Sprintf("must be one of: %v", enum),
			str,
			fmt.Errorf("value not in allowed enum values"),
		),
	)
}

// validateStringPattern checks if a string matches the required regex pattern.
func validateStringPattern(
	fieldName, str, pattern string,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	matched, err := regexp.MatchString(pattern, str)
	if err != nil {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewFieldValidationError(
				fieldName,
				"invalid regex pattern in schema",
				str,
				fmt.Errorf("pattern validation error: %w", err),
			),
		)
	}
	if !matched {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewFieldValidationError(
				fieldName,
				fmt.Sprintf("must match pattern: %s", pattern),
				str,
				fmt.Errorf("value does not match required pattern"),
			),
		)
	}
	return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
}

func (v *FrontmatterValidator) validateStringPropertySpec(
	fieldName string,
	fieldValue interface{},
	spec *domain.StringPropertySpec,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	str, ok := fieldValue.(string)
	if !ok {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewFieldValidationError(
				fieldName,
				"must be a string value",
				fieldValue,
				fmt.Errorf("expected string, got %T", fieldValue),
			),
		)
	}

	// Check enum constraint if specified
	if len(spec.Enum) > 0 {
		if result := validateStringEnum(fieldName, str, spec.Enum); result.IsErr() {
			return result
		}
	}

	// Check pattern constraint if specified
	if spec.Pattern != "" {
		if result := validateStringPattern(fieldName, str, spec.Pattern); result.IsErr() {
			return result
		}
	}

	// StringPropertySpec has no length constraints in domain model
	// Length validation would need to be added to domain model if required

	return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
}

// validateNumberPropertySpec validates a field value against NumberPropertySpec
// constraints.
func (v *FrontmatterValidator) validateNumberPropertySpec(
	fieldName string,
	fieldValue interface{},
	spec *domain.NumberPropertySpec,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	numResult := v.convertToFloat64(fieldName, fieldValue)
	if numResult.IsErr() {
		// Convert Result[float64] error to Result[FieldValidationError]
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			numResult.Error(),
		)
	}

	numValue, _ := numResult.Unwrap()

	if err := v.validateNumberMin(fieldName, fieldValue, numValue, spec.Min); err.IsErr() {
		return err
	}

	if err := v.validateNumberMax(fieldName, fieldValue, numValue, spec.Max); err.IsErr() {
		return err
	}

	if err := v.validateNumberStep(fieldName, fieldValue, numValue, spec); err.IsErr() {
		return err
	}

	return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
}

// convertToFloat64 converts an interface{} value to float64, accepting both int
// and float64.
func (v *FrontmatterValidator) convertToFloat64(
	fieldName string,
	fieldValue interface{},
) lithoserrors.Result[float64] {
	switch val := fieldValue.(type) {
	case int:
		return lithoserrors.Ok[float64](float64(val))
	case float64:
		return lithoserrors.Ok[float64](val)
	default:
		return lithoserrors.Err[float64](
			lithoserrors.NewFieldValidationError(
				fieldName,
				"must be a numeric value",
				fieldValue,
				fmt.Errorf("expected number, got %T", fieldValue),
			),
		)
	}
}

// validateNumberMin validates the minimum constraint for a number.
func (v *FrontmatterValidator) validateNumberMin(
	fieldName string,
	fieldValue interface{},
	num float64,
	minConstraint *float64,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	if minConstraint == nil {
		return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
	}

	if num >= *minConstraint {
		return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
	}

	return lithoserrors.Err[lithoserrors.FieldValidationError](
		lithoserrors.NewFieldValidationError(
			fieldName,
			fmt.Sprintf("minimum value is %v", *minConstraint),
			fieldValue,
			fmt.Errorf("value too small: %v < %v", num, *minConstraint),
		),
	)
}

// validateNumberMax validates the maximum constraint for a number.
func (v *FrontmatterValidator) validateNumberMax(
	fieldName string,
	fieldValue interface{},
	num float64,
	maxConstraint *float64,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	if maxConstraint == nil {
		return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
	}

	if num <= *maxConstraint {
		return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
	}

	return lithoserrors.Err[lithoserrors.FieldValidationError](
		lithoserrors.NewFieldValidationError(
			fieldName,
			fmt.Sprintf("maximum value is %v", *maxConstraint),
			fieldValue,
			fmt.Errorf("value too large: %v > %v", num, *maxConstraint),
		),
	)
}

// validateNumberStep validates the step constraint for a number.
func (v *FrontmatterValidator) validateNumberStep(
	fieldName string,
	fieldValue interface{},
	num float64,
	spec *domain.NumberPropertySpec,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	if spec.Step == nil || *spec.Step <= 0 {
		return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
	}

	minValue := v.getMinValueForStep(spec.Min)
	if v.isValidStep(num, minValue, *spec.Step) {
		return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
	}

	return lithoserrors.Err[lithoserrors.FieldValidationError](
		lithoserrors.NewFieldValidationError(
			fieldName,
			fmt.Sprintf(
				"must be a multiple of %v from minimum %v",
				*spec.Step,
				minValue,
			),
			fieldValue,
			fmt.Errorf("value does not satisfy step constraint"),
		),
	)
}

// getMinValueForStep returns the minimum value to use for step validation.
func (v *FrontmatterValidator) getMinValueForStep(
	minConstraint *float64,
) float64 {
	if minConstraint != nil {
		return *minConstraint
	}
	return 0.0
}

// isValidStep checks if a number satisfies the step constraint.
func (v *FrontmatterValidator) isValidStep(num, minValue, step float64) bool {
	quotient := (num - minValue) / step
	return quotient == float64(int(quotient))
}

// validateDatePropertySpec validates a field value against DatePropertySpec
// constraints.
func (v *FrontmatterValidator) validateDatePropertySpec(
	fieldName string,
	fieldValue interface{},
	spec *domain.DatePropertySpec,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	str, ok := fieldValue.(string)
	if !ok {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewFieldValidationError(
				fieldName,
				"must be a string date value",
				fieldValue,
				fmt.Errorf("expected string for date, got %T", fieldValue),
			),
		)
	}

	// Try to parse the date using the specified format
	// Default to RFC3339 if no format specified
	layout := time.RFC3339
	if spec.Format != "" {
		layout = spec.Format
	}

	if _, err := time.Parse(layout, str); err != nil {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewFieldValidationError(
				fieldName,
				fmt.Sprintf("must be a valid date in format: %s", layout),
				fieldValue,
				fmt.Errorf("date parsing error: %w", err),
			),
		)
	}

	return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
}

// validateFilePropertySpec validates a field value against FilePropertySpec
// constraints.
func (v *FrontmatterValidator) validateFilePropertySpec(
	fieldName string,
	fieldValue interface{},
	spec *domain.FilePropertySpec,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	_, ok := fieldValue.(string)
	if !ok {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewFieldValidationError(
				fieldName,
				"must be a string file path",
				fieldValue,
				fmt.Errorf("expected string for file path, got %T", fieldValue),
			),
		)
	}

	// Check fileClass constraint if specified
	// File class validation would require integration with file system
	// For now, we accept the constraint exists but don't validate it
	// This would be validated at runtime by the file system adapter
	_ = spec.FileClass

	return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
}

// validateBoolPropertySpec validates a field value against BoolPropertySpec
// constraints.
func (v *FrontmatterValidator) validateBoolPropertySpec(
	fieldName string,
	fieldValue interface{},
	spec *domain.BoolPropertySpec,
) lithoserrors.Result[lithoserrors.FieldValidationError] {
	// Verify spec is non-nil for interface consistency
	if spec == nil {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewFieldValidationError(
				fieldName,
				"bool property spec cannot be nil",
				fieldValue,
				fmt.Errorf("nil spec provided"),
			),
		)
	}

	_, ok := fieldValue.(bool)
	if !ok {
		return lithoserrors.Err[lithoserrors.FieldValidationError](
			lithoserrors.NewFieldValidationError(
				fieldName,
				"must be a boolean value",
				fieldValue,
				fmt.Errorf("expected bool, got %T", fieldValue),
			),
		)
	}

	return lithoserrors.Ok[lithoserrors.FieldValidationError](nil)
}

// isArrayValue checks if a value is an array/slice type.
func isArrayValue(value interface{}) bool {
	switch value.(type) {
	case []interface{}, []string, []int, []float64, []bool:
		return true
	default:
		return false
	}
}
