// Package domain contains the core business logic models for Lithos.
// These models represent domain concepts and contain no infrastructure
// dependencies.
package domain

import (
	"fmt"
	"math"
	"regexp"
	"time"

	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// StringPropertySpec validates string values with optional enum and pattern
// constraints.
type StringPropertySpec struct {
	// Enum contains allowed values as fixed list. If non-empty, value must be
	// in list.
	// Empty list means no enum constraint (any string valid).
	Enum []string

	// Pattern is a regex pattern for custom string validation.
	// If non-empty, value must match pattern. Uses Go regexp package.
	Pattern string
}

// Validate implements PropertySpec for StringPropertySpec.
func (s StringPropertySpec) Validate(value interface{}) error {
	str, err := validateStringType(value)
	if err != nil {
		return err
	}

	if enumErr := validateStringEnum(str, s.Enum, value); enumErr != nil {
		return enumErr
	}

	return validateStringPattern(str, s.Pattern, value)
}

// NumberPropertySpec validates numeric values with optional min/max/step
// constraints.
type NumberPropertySpec struct {
	// Min is the minimum allowed value (inclusive). Nil means no minimum
	// constraint.
	Min *float64

	// Max is the maximum allowed value (inclusive). Nil means no maximum
	// constraint.
	Max *float64

	// Step is the increment/decrement amount. If 1.0, implies integer values.
	// If nil, any precision allowed.
	Step *float64
}

// Validate implements PropertySpec for NumberPropertySpec.
func (n NumberPropertySpec) Validate(value interface{}) error {
	num, ok := value.(float64)
	if !ok {
		return errors.NewValidationError("value", "must be number", value)
	}

	if err := validateNumberBounds(n.Min, n.Max, num, value); err != nil {
		return err
	}

	return validateStepConstraint(n.Step, num, value)
}

// DatePropertySpec validates date/time values with format constraints.
type DatePropertySpec struct {
	// Format is the Go time layout string for parsing.
	// If empty, defaults to RFC3339.
	Format string
}

// Validate implements PropertySpec for DatePropertySpec.
func (d DatePropertySpec) Validate(value interface{}) error {
	str, err := validateDateType(value)
	if err != nil {
		return err
	}

	return validateDateFormat(str, d.Format, value)
}

// FilePropertySpec validates file reference values with optional
// class/directory constraints.
type FilePropertySpec struct {
	// FileClass restricts valid file references to notes with specific
	// fileClass value. Supports negation via ^ prefix. Empty string means no
	// fileClass restriction.
	FileClass string

	// Directory restricts valid file references to notes within specific vault
	// directory path.
	// Path is relative to vault root. Supports negation via ^ prefix.
	Directory string
}

// Validate implements PropertySpec for FilePropertySpec.
// Note: This is a simplified validation for MVP. Full validation requires vault
// index lookup.
func (f FilePropertySpec) Validate(value interface{}) error {
	str, err := validateFileType(value)
	if err != nil {
		return err
	}

	return validateFilePath(str, value)
}

// BoolPropertySpec validates boolean values. No additional configuration
// needed.
type BoolPropertySpec struct{}

// Validate implements PropertySpec for BoolPropertySpec.
func (b BoolPropertySpec) Validate(value interface{}) error {
	_, ok := value.(bool)
	if !ok {
		return errors.NewValidationError("value", "must be boolean", value)
	}
	return nil
}

// Validation helper functions

func isBelowMin(minValue *float64, value float64) bool {
	return minValue != nil && value < *minValue
}

func isAboveMax(maxValue *float64, value float64) bool {
	return maxValue != nil && value > *maxValue
}

func validateNumberBounds(
	minValue *float64,
	maxValue *float64,
	value float64,
	raw interface{},
) error {
	if isBelowMin(minValue, value) {
		return errors.NewValidationError(
			"value",
			fmt.Sprintf("must be >= %v", *minValue),
			raw,
		)
	}

	if isAboveMax(maxValue, value) {
		return errors.NewValidationError(
			"value",
			fmt.Sprintf("must be <= %v", *maxValue),
			raw,
		)
	}

	return nil
}

func validateStepConstraint(
	step *float64,
	value float64,
	raw interface{},
) error {
	if step == nil {
		return nil
	}

	if *step == 1.0 && value != math.Floor(value) {
		return errors.NewValidationError(
			"value",
			"must be integer",
			raw,
		)
	}

	return nil
}

func validateStringType(value interface{}) (string, error) {
	str, ok := value.(string)
	if !ok {
		return "", errors.NewValidationError("value", "must be string", value)
	}
	return str, nil
}

func validateStringEnum(str string, enum []string, raw interface{}) error {
	if len(enum) == 0 {
		return nil // no enum constraint
	}

	for _, allowed := range enum {
		if str == allowed {
			return nil
		}
	}

	return errors.NewValidationError(
		"value",
		fmt.Sprintf("must be one of: %v", enum),
		raw,
	)
}

func validateStringPattern(str, pattern string, raw interface{}) error {
	if pattern == "" {
		return nil // no pattern constraint
	}

	matched, err := regexp.MatchString(pattern, str)
	if err != nil {
		return errors.NewValidationError(
			"value",
			fmt.Sprintf("invalid pattern: %s", err.Error()),
			raw,
		)
	}

	if !matched {
		return errors.NewValidationError(
			"value",
			fmt.Sprintf("must match pattern: %s", pattern),
			raw,
		)
	}

	return nil
}

func validateFileType(value interface{}) (string, error) {
	str, ok := value.(string)
	if !ok {
		return "", errors.NewValidationError("value", "must be string", value)
	}
	return str, nil
}

func validateFilePath(str string, raw interface{}) error {
	if str == "" {
		return errors.NewValidationError("value", "cannot be empty", raw)
	}
	return nil
}

func validateDateType(value interface{}) (string, error) {
	str, ok := value.(string)
	if !ok {
		return "", errors.NewValidationError("value", "must be string", value)
	}
	return str, nil
}

func validateDateFormat(str, format string, raw interface{}) error {
	if format == "" {
		format = time.RFC3339
	}

	_, err := time.Parse(format, str)
	if err != nil {
		return errors.NewValidationError(
			"value",
			fmt.Sprintf("must be valid date in format: %s", format),
			raw,
		)
	}

	return nil
}
