// Package errors provides domain-specific error types and helpers for idiomatic
// Go error handling.
//
// This package implements the standard error interface with unwrapping support,
// enabling use of errors.Is() and errors.As() for error comparison and type
// extraction.
// All errors follow the (T, error) signature pattern without Result[T] types.
package errors

import "fmt"

// ErrNotFound is returned when a requested resource or item cannot be found.
// It follows the standard Go convention for "not found" errors.
var ErrNotFound = NewBaseError("not found", nil)

// BaseError provides a lightweight foundation for domain-specific errors.
// It implements the standard error interface and supports error unwrapping.
type BaseError struct {
	message string
	cause   error
}

// ValidationError represents property-level validation failures.
// It embeds BaseError to provide standard error functionality.
type ValidationError struct {
	BaseError

	property string
	reason   string
	value    interface{}
}

// ResourceError represents resource operation failures.
// It embeds BaseError to provide standard error functionality.
type ResourceError struct {
	BaseError

	resource  string
	operation string
	target    string
}

// FileSystemError represents filesystem operation failures.
// It embeds ResourceError to provide standard error functionality with
// filesystem-specific context.
type FileSystemError struct {
	ResourceError
}

// NewBaseError creates a new BaseError with an optional cause.
// If cause is nil, the error contains only the message.
// If cause is provided, the error message will include the cause.
func NewBaseError(message string, cause error) BaseError {
	return BaseError{
		message: message,
		cause:   cause,
	}
}

// Error implements the error interface.
// Returns the message, and includes the cause if present.
func (e BaseError) Error() string {
	if e.cause != nil {
		return fmt.Sprintf("%s: %v", e.message, e.cause)
	}
	return e.message
}

// Unwrap returns the underlying cause error, or nil if none exists.
// This enables compatibility with errors.Is() and errors.As().
func (e BaseError) Unwrap() error {
	return e.cause
}

// Cause returns the underlying cause error, or nil if none exists.
// This provides direct access to the cause for error formatting.
func (e BaseError) Cause() error {
	return e.cause
}

// NewValidationError creates a new ValidationError with property validation
// context. The message is fixed as "validation failed" and the cause provides
// additional context.
func NewValidationError(
	property, reason string,
	value interface{},
	cause error,
) *ValidationError {
	return &ValidationError{
		BaseError: NewBaseError("validation failed", cause),
		property:  property,
		reason:    reason,
		value:     value,
	}
}

// Property returns the name of the property that failed validation.
func (e *ValidationError) Property() string {
	return e.property
}

// Reason returns the reason for the validation failure.
func (e *ValidationError) Reason() string {
	return e.reason
}

// Value returns the invalid value that caused the validation failure.
func (e *ValidationError) Value() interface{} {
	return e.value
}

// NewResourceError creates a new ResourceError with resource operation context.
// The message is fixed as "resource operation failed" and the cause provides
// additional context.
func NewResourceError(
	resource, operation, target string,
	cause error,
) *ResourceError {
	return &ResourceError{
		BaseError: NewBaseError("resource operation failed", cause),
		resource:  resource,
		operation: operation,
		target:    target,
	}
}

// Resource returns the type of resource that failed (e.g., "file", "database",
// "api").
func (e *ResourceError) Resource() string {
	return e.resource
}

// Operation returns the operation that failed (e.g., "read", "write",
// "connect").
func (e *ResourceError) Operation() string {
	return e.operation
}

// Target returns the specific target of the operation (e.g., file path, URL,
// connection string).
func (e *ResourceError) Target() string {
	return e.target
}

// NewFileSystemError creates a new FileSystemError with filesystem operation
// context. The resource is fixed as "file" and the operation/target provide
// specific context.
func NewFileSystemError(
	operation, target string,
	cause error,
) *FileSystemError {
	return &FileSystemError{
		ResourceError: *NewResourceError("file", operation, target, cause),
	}
}
