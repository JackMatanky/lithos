package errors

import (
	"fmt"
)

// BaseError provides a common structure for domain errors with consistent
// formatting.
type BaseError struct {
	Type    string      // Error type identifier (e.g., "ValidationError", "NotFoundError")
	Context string      // Primary context (field name, resource type, etc.)
	Detail  string      // Specific error detail or message
	Value   interface{} // Optional associated value
	Cause   error       // Optional underlying cause
}

// NewBaseError creates a new BaseError with consistent formatting.
func NewBaseError(
	errType, context, detail string,
	value interface{},
	cause error,
) BaseError {
	return BaseError{
		Type:    errType,
		Context: context,
		Detail:  detail,
		Value:   value,
		Cause:   cause,
	}
}

// Error implements the error interface for BaseError.
func (e BaseError) Error() string {
	if e.Cause != nil {
		// Avoid duplicating cause if detail already contains it
		if e.Detail == e.Cause.Error() {
			return fmt.Sprintf("[%s] %s: %s", e.Type, e.Context, e.Detail)
		}
		return fmt.Sprintf(
			"[%s] %s: %s: %v",
			e.Type,
			e.Context,
			e.Detail,
			e.Cause,
		)
	}
	if e.Value != nil {
		return fmt.Sprintf(
			"[%s] %s: %s (value: %v)",
			e.Type,
			e.Context,
			e.Detail,
			e.Value,
		)
	}
	return fmt.Sprintf("[%s] %s: %s", e.Type, e.Context, e.Detail)
}

// Unwrap returns the underlying cause error for error chaining.
func (e BaseError) Unwrap() error {
	return e.Cause
}

// ValidationError represents simple validation failures with field-specific
// information. For comprehensive validation with multiple errors, use the
// validation.go system.
type ValidationError struct {
	BaseError
	Field string
}

// NewValidationError creates a new ValidationError.
func NewValidationError(
	field, message string,
	value interface{},
) ValidationError {
	return ValidationError{
		BaseError: NewBaseError(
			"ValidationError",
			"field '"+field+"'",
			message,
			nil,
			nil,
		),
		Field: field,
	}
}

// NotFoundError represents resource not found errors.
type NotFoundError struct {
	BaseError
	Resource   string
	Identifier string
}

// NewNotFoundError creates a new NotFoundError.
func NewNotFoundError(resource, identifier string) NotFoundError {
	context := fmt.Sprintf("%s '%s'", resource, identifier)
	return NotFoundError{
		BaseError: NewBaseError(
			"NotFoundError",
			context,
			"not found",
			nil,
			nil,
		),
		Resource:   resource,
		Identifier: identifier,
	}
}

// Error implements the error interface for NotFoundError.
func (e NotFoundError) Error() string {
	return fmt.Sprintf(
		"[%s] %s '%s' not found",
		e.Type,
		e.Resource,
		e.Identifier,
	)
}

// ConfigurationError represents configuration-related errors.
type ConfigurationError struct {
	BaseError
	Key string
}

// NewConfigurationError creates a new ConfigurationError.
func NewConfigurationError(key, message string) ConfigurationError {
	return ConfigurationError{
		BaseError: NewBaseError(
			"ConfigurationError",
			"key '"+key+"'",
			message,
			nil,
			nil,
		),
		Key: key,
	}
}

// TemplateError represents template processing errors.
type TemplateError struct {
	BaseError
	Template string
	Line     int
}

// NewTemplateError creates a new TemplateError.
func NewTemplateError(template string, line int, message string) TemplateError {
	context := fmt.Sprintf("template '%s'", template)
	if line > 0 {
		context = fmt.Sprintf("template '%s' line %d", template, line)
	}
	return TemplateError{
		BaseError: NewBaseError("TemplateError", context, message, nil, nil),
		Template:  template,
		Line:      line,
	}
}

// SchemaError represents schema-related errors.
type SchemaError struct {
	BaseError
	Schema string
}

// NewSchemaError creates a new SchemaError.
func NewSchemaError(schema, message string) SchemaError {
	return SchemaError{
		BaseError: NewBaseError(
			"SchemaError",
			"schema '"+schema+"'",
			message,
			nil,
			nil,
		),
		Schema: schema,
	}
}

// OperationError represents operation failures with path and cause information.
// This consolidates StorageError and FileSystemError to prevent duplication.
type OperationError struct {
	BaseError
	Operation string
	Path      string
}

// NewOperationError creates a new OperationError with the specified error type.
func NewOperationError(
	errType, operation, path string,
	cause error,
) OperationError {
	context := fmt.Sprintf("%s '%s'", operation, path)
	detail := "failed"
	if cause != nil {
		detail = cause.Error()
	}
	return OperationError{
		BaseError: NewBaseError(errType, context, detail, nil, cause),
		Operation: operation,
		Path:      path,
	}
}

// NewStorageError creates a new storage operation error.
func NewStorageError(operation, path string, cause error) OperationError {
	return NewOperationError("StorageError", operation, path, cause)
}

// NewFileSystemError creates a new filesystem operation error.
func NewFileSystemError(operation, path string, cause error) OperationError {
	return NewOperationError("FileSystemError", operation, path, cause)
}

// Error implements the error interface for OperationError.
func (e OperationError) Error() string {
	if e.Cause != nil {
		return fmt.Sprintf(
			"[%s] %s '%s': %s",
			e.Type,
			e.Operation,
			e.Path,
			e.Cause,
		)
	}
	return fmt.Sprintf("[%s] %s '%s' %s", e.Type, e.Operation, e.Path, e.Detail)
}

// StorageError alias for backward compatibility.
type StorageError = OperationError

// FileSystemError alias for backward compatibility.
type FileSystemError = OperationError
