package errors

import "fmt"

// ValidationResult represents the result of a validation operation.
// It contains all field-level validation errors found during validation.
type ValidationResult struct {
	Errors []FieldValidationError
}

// NewValidationResult creates a new empty validation result.
func NewValidationResult() ValidationResult {
	return ValidationResult{
		Errors: make([]FieldValidationError, 0),
	}
}

// AddError adds a field validation error to the result.
func (vr *ValidationResult) AddError(fieldErr FieldValidationError) {
	vr.Errors = append(vr.Errors, fieldErr)
}

// IsValid returns true if there are no validation errors.
func (vr ValidationResult) IsValid() bool {
	return len(vr.Errors) == 0
}

// FieldValidationError represents a validation error for a specific field.
// It contains detailed information about what validation constraint was
// violated.
type FieldValidationError interface {
	error
	Field() string
	Value() interface{}
	ConstraintType() string
}

// RequiredFieldError represents a missing required field error.
type RequiredFieldError struct {
	FieldName string
}

// NewRequiredFieldError creates a new RequiredFieldError.
func NewRequiredFieldError(fieldName string) RequiredFieldError {
	return RequiredFieldError{
		FieldName: fieldName,
	}
}

// Error implements the error interface for RequiredFieldError.
func (e RequiredFieldError) Error() string {
	return fmt.Sprintf("field '%s' is required but missing", e.FieldName)
}

// Field returns the field name that was missing.
func (e RequiredFieldError) Field() string {
	return e.FieldName
}

// Value returns nil for missing fields.
func (e RequiredFieldError) Value() interface{} {
	return nil
}

// ConstraintType returns the constraint type that was violated.
func (e RequiredFieldError) ConstraintType() string {
	return "required"
}

// ArrayConstraintError represents an array constraint violation.
type ArrayConstraintError struct {
	FieldName    string
	ActualValue  interface{}
	ExpectedType string // "array" or "scalar"
}

// NewArrayConstraintError creates a new ArrayConstraintError.
func NewArrayConstraintError(
	fieldName string,
	actualValue interface{},
	expectedType string,
) ArrayConstraintError {
	return ArrayConstraintError{
		FieldName:    fieldName,
		ActualValue:  actualValue,
		ExpectedType: expectedType,
	}
}

// Error implements the error interface for ArrayConstraintError.
func (e ArrayConstraintError) Error() string {
	return fmt.Sprintf(
		"field '%s' must be %s, got %T",
		e.FieldName,
		e.ExpectedType,
		e.ActualValue,
	)
}

// Field returns the field name that violated the constraint.
func (e ArrayConstraintError) Field() string {
	return e.FieldName
}

// Value returns the actual value that violated the constraint.
func (e ArrayConstraintError) Value() interface{} {
	return e.ActualValue
}

// ConstraintType returns the constraint type that was violated.
func (e ArrayConstraintError) ConstraintType() string {
	return "array"
}

// PropertySpecError represents a PropertySpec-specific validation error.
type PropertySpecError struct {
	FieldName   string
	ActualValue interface{}
	Cause       error
}

// NewPropertySpecError creates a new PropertySpecError.
func NewPropertySpecError(
	fieldName string,
	actualValue interface{},
	cause error,
) PropertySpecError {
	return PropertySpecError{
		FieldName:   fieldName,
		ActualValue: actualValue,
		Cause:       cause,
	}
}

// Error implements the error interface for PropertySpecError.
func (e PropertySpecError) Error() string {
	return fmt.Sprintf("field '%s' validation failed: %v", e.FieldName, e.Cause)
}

// Field returns the field name that failed validation.
func (e PropertySpecError) Field() string {
	return e.FieldName
}

// Value returns the actual value that failed validation.
func (e PropertySpecError) Value() interface{} {
	return e.ActualValue
}

// ConstraintType returns the constraint type that was violated.
func (e PropertySpecError) ConstraintType() string {
	return "property_spec"
}

// Unwrap returns the underlying cause error for error wrapping chains.
func (e PropertySpecError) Unwrap() error {
	return e.Cause
}

// SchemaNotFoundError represents an error when a schema is not found in the
// registry.
type SchemaNotFoundError struct {
	SchemaName string
}

// NewSchemaNotFoundError creates a new SchemaNotFoundError.
func NewSchemaNotFoundError(schemaName string) SchemaNotFoundError {
	return SchemaNotFoundError{
		SchemaName: schemaName,
	}
}

// Error implements the error interface for SchemaNotFoundError.
func (e SchemaNotFoundError) Error() string {
	return fmt.Sprintf("schema '%s' not found in registry", e.SchemaName)
}
