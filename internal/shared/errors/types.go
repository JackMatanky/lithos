package errors

// Package errors provides structured error types for the Lithos application.
// This file contains base error types and constructors.

import "fmt"

// BaseError is the minimal building block for all Lithos error types. It keeps
// only the information that matters everywhere: a human readable message and
// an optional underlying cause that preserves the error chain.
type BaseError struct {
	message string
	cause   error
}

// NewBaseError creates a BaseError with the provided message and optional
// cause. Callers are expected to supply fully formatted messages.
func NewBaseError(message string, cause error) BaseError {
	return BaseError{
		message: message,
		cause:   cause,
	}
}

// Error implements the error interface.
func (e BaseError) Error() string {
	return e.message
}

// Unwrap exposes the underlying cause to support errors.Is / errors.As.
func (e BaseError) Unwrap() error {
	return e.cause
}

// Message returns the human readable message attached to the error.
func (e BaseError) Message() string {
	return e.message
}

// Cause returns the underlying cause (may be nil).
func (e BaseError) Cause() error {
	return e.cause
}

// ValidationError represents a failure to satisfy a domain rule for a specific
// property. It intentionally keeps only the property name, a short reason, and
// the optional offending value for debuggability.
type ValidationError struct {
	BaseError
	property string
	reason   string
	value    interface{}
}

// NewValidationError constructs a ValidationError for the supplied property.
// The message automatically reflects Lithos terminology (property instead of
// field) and includes the offending value when provided.
func NewValidationError(
	property, reason string,
	value interface{},
) ValidationError {
	message := fmt.Sprintf("property '%s': %s", property, reason)
	if value != nil {
		message = fmt.Sprintf("%s (value: %v)", message, value)
	}

	return ValidationError{
		BaseError: NewBaseError(message, nil),
		property:  property,
		reason:    reason,
		value:     value,
	}
}

// Property returns the property name associated with the validation failure.
func (e *ValidationError) Property() string {
	return e.property
}

func (e *ValidationError) Reason() string {
	return e.reason
}

// Value returns the value that triggered validation failure.
func (e *ValidationError) Value() interface{} {
	return e.value
}

// ResourceError captures failures while performing an operation against a
// specific resource (files, schemas, templates, etc.).
type ResourceError struct {
	BaseError
	resource  string
	operation string
	target    string
}

// NewResourceError creates a ResourceError with consistent messaging.
func NewResourceError(
	resource, operation, target string,
	cause error,
) ResourceError {
	detail := "operation failed"
	if cause != nil {
		detail = cause.Error()
	}

	message := fmt.Sprintf(
		"%s %s '%s': %s",
		resource,
		operation,
		target,
		detail,
	)

	return ResourceError{
		BaseError: NewBaseError(message, cause),
		resource:  resource,
		operation: operation,
		target:    target,
	}
}

// Resource returns the resource category (e.g. "schema", "file").
func (e *ResourceError) Resource() string {
	return e.resource
}

// Operation returns the attempted action (e.g. "read", "load").
func (e *ResourceError) Operation() string {
	return e.operation
}

// Target returns the resource identifier or path.
func (e *ResourceError) Target() string {
	return e.target
}
