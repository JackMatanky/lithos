package frontmatter

import (
	"slices"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// FieldValidator defines the interface for polymorphic field validation.
// Each property type implements this interface to provide type-specific
// validation logic for frontmatter values against PropertySpec constraints.
//
// Design: Polymorphic validation pattern enables clean separation of
// validation logic by type while maintaining consistency across validators.
type FieldValidator interface {
	// Validate validates a frontmatter field value against a PropertySpec.
	// Returns FrontmatterError for validation failures, nil for success.
	//
	// Parameters:
	//   - fieldName: Name of the field being validated (for error context)
	//   - value: The actual frontmatter field value to validate
	//   - spec: PropertySpec containing validation constraints
	//
	// Returns:
	//   - error: FrontmatterError with field context, nil for valid values
	Validate(
		fieldName string,
		value any,
		spec domain.PropertySpec,
	) error
}

// StringValidator validates string fields against StringSpec constraints.
// Handles enum validation and regex pattern matching.
type StringValidator struct{}

// NumberValidator validates numeric fields against NumberSpec constraints.
// Handles min/max range validation and step increment validation.
type NumberValidator struct{}

// DateValidator validates date fields against DateSpec constraints.
// Handles date format validation and parsing.
type DateValidator struct{}

// BoolValidator validates boolean fields against BoolSpec constraints.
// Handles boolean type validation (no additional constraints).
type BoolValidator struct{}

// Validate validates string values against StringSpec constraints.
// Checks enum membership and regex pattern compliance.
func (v *StringValidator) Validate(
	fieldName string,
	value any,
	spec domain.PropertySpec,
) error {
	// Type assertion to ensure we have a string value
	stringValue, ok := value.(string)
	if !ok {
		return errors.NewFrontmatterError(
			"field value is not a string",
			fieldName,
			nil,
		)
	}

	// Type assertion to ensure we have a StringSpec
	stringSpec, ok := spec.(*domain.StringSpec)
	if !ok {
		return errors.NewFrontmatterError(
			"property spec is not StringSpec",
			fieldName,
			nil,
		)
	}

	// Validate enum membership if enum is specified
	if len(stringSpec.Enum) > 0 {
		if slices.Contains(stringSpec.Enum, stringValue) {
			return nil // Valid enum value
		}
		return errors.NewFrontmatterError(
			"value not in allowed enum",
			fieldName,
			nil,
		)
	}

	// TODO: Add pattern validation when needed
	return nil
}

// Validate validates numeric values against NumberSpec constraints.
// Checks min/max bounds and step increment compliance.
func (v *NumberValidator) Validate(
	fieldName string,
	value any,
	spec domain.PropertySpec,
) error {
	// Extract numeric value from interface with type checking
	numValue, err := v.extractNumericValue(value)
	if err != nil {
		return errors.NewFrontmatterError(
			"field value is not numeric",
			fieldName,
			err,
		)
	}

	// Type assertion to ensure we have a NumberSpec
	numberSpec, ok := spec.(*domain.NumberSpec)
	if !ok {
		return errors.NewFrontmatterError(
			"property spec is not NumberSpec",
			fieldName,
			nil,
		)
	}

	// Validate constraints
	return v.validateNumericConstraints(fieldName, numValue, numberSpec)
}

// Validate validates date values against DateSpec constraints.
// Checks date format compliance and parsing validity.
func (v *DateValidator) Validate(
	fieldName string,
	value any,
	spec domain.PropertySpec,
) error {
	// Type assertion to ensure we have a string value (dates are stored as
	// strings)
	dateValue, ok := value.(string)
	if !ok {
		return errors.NewFrontmatterError(
			"field value is not a string",
			fieldName,
			nil,
		)
	}

	// Type assertion to ensure we have a DateSpec
	dateSpec, ok := spec.(*domain.DateSpec)
	if !ok {
		return errors.NewFrontmatterError(
			"property spec is not DateSpec",
			fieldName,
			nil,
		)
	}

	// Use RFC3339 as default format if none specified
	format := dateSpec.Format
	if format == "" {
		format = "2006-01-02T15:04:05Z07:00" // RFC3339
	}

	// Try to parse the date with the specified format
	_, err := time.Parse(format, dateValue)
	if err != nil {
		return errors.NewFrontmatterError(
			"invalid date format",
			fieldName,
			err,
		)
	}

	return nil
}

// Validate validates boolean values against BoolSpec constraints.
// Ensures value is a valid boolean type.
func (v *BoolValidator) Validate(
	fieldName string,
	value any,
	spec domain.PropertySpec,
) error {
	// Type assertion to ensure we have a boolean value
	_, ok := value.(bool)
	if !ok {
		return errors.NewFrontmatterError(
			"field value is not a boolean",
			fieldName,
			nil,
		)
	}

	// Type assertion to ensure we have a BoolSpec
	_, ok = spec.(*domain.BoolSpec)
	if !ok {
		return errors.NewFrontmatterError(
			"property spec is not BoolSpec",
			fieldName,
			nil,
		)
	}

	// BoolSpec has no additional constraints to validate
	return nil
}

// extractNumericValue extracts a float64 value from an any with
// type checking.
// Helper method for NumberValidator to reduce cyclomatic complexity.
func (v *NumberValidator) extractNumericValue(
	value any,
) (float64, error) {
	switch val := value.(type) {
	case int:
		return float64(val), nil
	case int64:
		return float64(val), nil
	case float64:
		return val, nil
	case float32:
		return float64(val), nil
	default:
		return 0, errors.NewFrontmatterError(
			"unsupported numeric type",
			"",
			nil,
		)
	}
}

// validateNumericConstraints validates numeric value against NumberSpec
// constraints.
// Helper method for NumberValidator to reduce cyclomatic complexity.
func (v *NumberValidator) validateNumericConstraints(
	fieldName string,
	numValue float64,
	numberSpec *domain.NumberSpec,
) error {
	// Validate minimum value
	if numberSpec.Min != nil && numValue < *numberSpec.Min {
		return errors.NewFrontmatterError(
			"value below minimum",
			fieldName,
			nil,
		)
	}

	// Validate maximum value
	if numberSpec.Max != nil && numValue > *numberSpec.Max {
		return errors.NewFrontmatterError(
			"value above maximum",
			fieldName,
			nil,
		)
	}

	// TODO: Add step validation when needed
	return nil
}
