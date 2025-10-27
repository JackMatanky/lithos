package errors

import (
	"errors"
	"fmt"
)

const (
	domainFrontmatter    = "frontmatter"
	constraintRequired   = "required"
	constraintArray      = "array"
	constraintValidation = "validation"
)

// FieldValidationError represents a validation issue for a specific frontmatter
// field. The interface is intentionally narrow to keep call sites decoupled
// from concrete implementations.
type FieldValidationError interface {
	error
	Field() string
	Value() interface{}
	ConstraintType() string
	Reason() string
}

type frontmatterError struct {
	ValidationError
	field      string
	constraint string
}

func newFrontmatterError(
	field,
	reason,
	constraint string,
	value interface{},
	cause error,
) frontmatterError {
	message := fmt.Sprintf("field '%s': %s", field, reason)
	if value != nil {
		message = fmt.Sprintf("%s (value: %v)", message, value)
	}

	return frontmatterError{
		ValidationError: ValidationError{
			BaseError: NewBaseError(message, cause),
			property:  field,
			reason:    reason,
			value:     value,
		},
		field:      field,
		constraint: constraint,
	}
}

func (e *frontmatterError) Field() string {
	return e.field
}

func (e *frontmatterError) Value() interface{} {
	return e.value
}

func (e *frontmatterError) ConstraintType() string {
	return e.constraint
}

func (e *frontmatterError) Reason() string {
	return e.reason
}

func (e *frontmatterError) Domain() string {
	return domainFrontmatter
}

// RequiredFieldError represents a missing required field.
type RequiredFieldError struct {
	frontmatterError
}

// NewRequiredFieldError creates a required field validation error.
func NewRequiredFieldError(field string) *RequiredFieldError {
	return &RequiredFieldError{
		frontmatterError: newFrontmatterError(
			field,
			"is required but missing",
			constraintRequired,
			nil,
			nil,
		),
	}
}

// ArrayConstraintError represents mismatched array/scalar semantics.
type ArrayConstraintError struct {
	frontmatterError
	expected string
}

// NewArrayConstraintError creates an array constraint error.
func NewArrayConstraintError(
	field string,
	actual interface{},
	expectedType string,
) *ArrayConstraintError {
	reason := "must be an array"
	if expectedType == "scalar" {
		reason = "must be a single value"
	}

	return &ArrayConstraintError{
		frontmatterError: newFrontmatterError(
			field,
			reason,
			constraintArray,
			actual,
			nil,
		),
		expected: expectedType,
	}
}

// Expected reports the expected cardinality ("array" or "scalar").
func (e *ArrayConstraintError) Expected() string {
	return e.expected
}

// fieldValidationError wraps arbitrary validation failures surfaced from deeper
// validation routines.
type fieldValidationError struct {
	frontmatterError
}

// NewFieldValidationError creates a field validation error with optional cause.
func NewFieldValidationError(
	field,
	reason string,
	value interface{},
	cause error,
) *fieldValidationError {
	return &fieldValidationError{
		frontmatterError: newFrontmatterError(
			field,
			reason,
			constraintValidation,
			value,
			cause,
		),
	}
}

// NewPropertySpecError adapts property-level validation failures into
// frontmatter field errors, preserving the underlying validation context when
// available.
func NewPropertySpecError(
	field string,
	actual interface{},
	cause error,
) FieldValidationError {
	var validationErr ValidationError
	if errors.As(cause, &validationErr) {
		return &fieldValidationError{
			frontmatterError: newFrontmatterError(
				field,
				validationErr.Reason(),
				constraintValidation,
				validationErr.Value(),
				cause,
			),
		}
	}

	// Fallback to generic validation error.
	return &fieldValidationError{
		frontmatterError: newFrontmatterError(
			field,
			cause.Error(),
			constraintValidation,
			actual,
			cause,
		),
	}
}

// ValidationResult aggregates all field validation errors encountered during
// frontmatter processing.
type ValidationResult struct {
	Errors []FieldValidationError
}

// NewValidationResult constructs an empty validation result.
func NewValidationResult() ValidationResult {
	return ValidationResult{
		Errors: make([]FieldValidationError, 0),
	}
}

// AddError appends a field validation error to the result.
func (vr *ValidationResult) AddError(fieldErr FieldValidationError) {
	vr.Errors = append(vr.Errors, fieldErr)
}

// IsValid reports whether any validation issues were recorded.
func (vr ValidationResult) IsValid() bool {
	return len(vr.Errors) == 0
}
