package frontmatter

import (
	"context"
	"fmt"
	"math"
	"slices"
	"strconv"
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
	// Coerce numeric and boolean scalars to strings so YAML numbers without
	// quotes can still satisfy string specs (common user shorthand).
	stringValue, ok := coerceToStringValue(value)
	if !ok {
		return errors.NewFrontmatterError(
			"field value is not a string",
			fieldName,
			nil,
		)
	}

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

	// Validate pattern compliance if pattern is specified
	if stringSpec.Pattern != "" {
		// Ensure regex is compiled (should be done by spec.Validate() but
		// extra safety here)
		if err := stringSpec.Validate(context.Background()); err != nil {
			return errors.NewFrontmatterError(
				"invalid pattern regex in schema",
				fieldName,
				err,
			)
		}

		if stringSpec.Match(stringValue) {
			return nil
		}

		return errors.NewFrontmatterError(
			fmt.Sprintf("value does not match pattern: %s", stringSpec.Pattern),
			fieldName,
			nil,
		)
	}

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
		if valueSpec, isValue := spec.(domain.NumberSpec); isValue {
			numberSpec = &valueSpec
		} else {
			return errors.NewFrontmatterError(
				"property spec is not NumberSpec",
				fieldName,
				nil,
			)
		}
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
	// Type assertion to handle string or time.Time values
	// YAML parsers often decode dates into time.Time when unquoted
	dateSpec, ok := spec.(*domain.DateSpec)
	if !ok {
		if valueSpec, isValue := spec.(domain.DateSpec); isValue {
			dateSpec = &valueSpec
		} else {
			return errors.NewFrontmatterError(
				"property spec is not DateSpec",
				fieldName,
				nil,
			)
		}
	}

	format := dateSpec.Format
	if format == "" {
		format = "2006-01-02T15:04:05Z07:00" // RFC3339
	}

	var dateValue string
	switch v := value.(type) {
	case string:
		dateValue = v
	case time.Time:
		dateValue = v.Format(format)
	default:
		return errors.NewFrontmatterError(
			"field value is not a string",
			fieldName,
			nil,
		)
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
	if _, isPtr := spec.(*domain.BoolSpec); isPtr {
		return nil
	}
	if _, isValue := spec.(domain.BoolSpec); isValue {
		return nil
	}

	return errors.NewFrontmatterError(
		"property spec is not BoolSpec",
		fieldName,
		nil,
	)
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

	// Validate step compliance
	if numberSpec.Step != nil {
		step := *numberSpec.Step
		base := 0.0
		if numberSpec.Min != nil {
			base = *numberSpec.Min
		}

		// (numValue - base) / step must be an integer
		// Use epsilon for float precision issues
		const epsilon = 1e-9
		remainder := math.Mod(numValue-base, step)
		if math.Abs(remainder) > epsilon &&
			math.Abs(remainder-step) > epsilon {
			return errors.NewFrontmatterError(
				fmt.Sprintf(
					"value must be a multiple of %g starting from %g",
					step,
					base,
				),
				fieldName,
				nil,
			)
		}
	}

	return nil
}

func coerceToStringValue(value any) (string, bool) {
	switch v := value.(type) {
	case string:
		return v, true
	case fmt.Stringer:
		return v.String(), true
	case int:
		return strconv.Itoa(v), true
	case int64:
		return strconv.FormatInt(v, 10), true
	case float64:
		return strconv.FormatFloat(v, 'f', -1, 64), true
	case float32:
		return strconv.FormatFloat(float64(v), 'f', -1, 32), true
	case bool:
		return strconv.FormatBool(v), true
	default:
		return "", false
	}
}
