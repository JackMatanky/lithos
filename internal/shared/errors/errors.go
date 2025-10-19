// Package errors provides domain-specific error types and functional error
// handling patterns for consistent error handling across the application. This
// package implements a custom Result[T] pattern inspired by Rust's Result type
// for better error handling ergonomics.
package errors

import (
	"errors"
	"fmt"
)

// Result represents a value that can be either a success (T) or an error.
// This implements a functional error handling pattern similar to Rust's
// Result<T>.
type Result[T any] struct {
	value T
	err   error
}

// IsOk returns true if the result contains a value (no error).
func (r Result[T]) IsOk() bool {
	return r.err == nil
}

// IsErr returns true if the result contains an error.
func (r Result[T]) IsErr() bool {
	return r.err != nil
}

// Unwrap returns the value and error. Panics if called on an error result.
// Use IsOk() or IsErr() to check state before calling.
func (r Result[T]) Unwrap() (T, error) {
	return r.value, r.err
}

// Value returns the contained value. Panics if the result is an error.
// Use IsOk() to check state before calling.
func (r Result[T]) Value() T {
	if r.err != nil {
		panic("called Value() on error result")
	}
	return r.value
}

// Error returns the contained error. Returns nil if the result is ok.
// Use IsErr() to check state before calling.
func (r Result[T]) Error() error {
	return r.err
}

// Ok creates a successful Result[T] containing the given value.
func Ok[T any](value T) Result[T] {
	return Result[T]{value: value, err: nil}
}

// Err creates an error Result[T] containing the given error.
func Err[T any](err error) Result[T] {
	var zero T
	return Result[T]{value: zero, err: err}
}

// Wrap wraps an error with additional context using a message.
func Wrap(err error, message string) error {
	return fmt.Errorf("%s: %w", message, err)
}

// WrapWithContext wraps an error with structured context information.
func WrapWithContext(err error, context map[string]interface{}) error {
	if len(context) == 0 {
		return err
	}

	contextStr := ""
	for key, value := range context {
		contextStr += fmt.Sprintf("%s=%v ", key, value)
	}

	return fmt.Errorf("%s: %w", contextStr[:len(contextStr)-1], err)
}

// JoinErrors joins multiple errors into a single error using errors.Join.
// This provides compatibility with Go 1.20+ error joining functionality.
func JoinErrors(errs ...error) error {
	return errors.Join(errs...)
}

// ValidationError represents validation failures with field-specific
// information.
type ValidationError struct {
	Field   string
	Message string
	Value   interface{}
}

// NewValidationError creates a new ValidationError.
func NewValidationError(
	field, message string,
	value interface{},
) ValidationError {
	return ValidationError{
		Field:   field,
		Message: message,
		Value:   value,
	}
}

// Error implements the error interface for ValidationError.
func (e ValidationError) Error() string {
	return fmt.Sprintf("[ValidationError] field '%s': %s", e.Field, e.Message)
}

// NotFoundError represents resource not found errors.
type NotFoundError struct {
	Resource   string
	Identifier string
}

// NewNotFoundError creates a new NotFoundError.
func NewNotFoundError(resource, identifier string) NotFoundError {
	return NotFoundError{
		Resource:   resource,
		Identifier: identifier,
	}
}

// Error implements the error interface for NotFoundError.
func (e NotFoundError) Error() string {
	return fmt.Sprintf(
		"[NotFoundError] %s '%s' not found",
		e.Resource,
		e.Identifier,
	)
}

// ConfigurationError represents configuration-related errors.
type ConfigurationError struct {
	Key     string
	Message string
}

// NewConfigurationError creates a new ConfigurationError.
func NewConfigurationError(key, message string) ConfigurationError {
	return ConfigurationError{
		Key:     key,
		Message: message,
	}
}

// Error implements the error interface for ConfigurationError.
func (e ConfigurationError) Error() string {
	return fmt.Sprintf("[ConfigurationError] key '%s': %s", e.Key, e.Message)
}

// TemplateError represents template processing errors.
type TemplateError struct {
	Template string
	Line     int
	Message  string
}

// NewTemplateError creates a new TemplateError.
func NewTemplateError(template string, line int, message string) TemplateError {
	return TemplateError{
		Template: template,
		Line:     line,
		Message:  message,
	}
}

// Error implements the error interface for TemplateError.
func (e TemplateError) Error() string {
	if e.Line > 0 {
		return fmt.Sprintf(
			"[TemplateError] template '%s' line %d: %s",
			e.Template,
			e.Line,
			e.Message,
		)
	}
	return fmt.Sprintf(
		"[TemplateError] template '%s': %s",
		e.Template,
		e.Message,
	)
}

// SchemaError represents schema-related errors.
type SchemaError struct {
	Schema  string
	Message string
}

// NewSchemaError creates a new SchemaError.
func NewSchemaError(schema, message string) SchemaError {
	return SchemaError{
		Schema:  schema,
		Message: message,
	}
}

// Error implements the error interface for SchemaError.
func (e SchemaError) Error() string {
	return fmt.Sprintf("[SchemaError] schema '%s': %s", e.Schema, e.Message)
}

// StorageError represents storage operation failures.
type StorageError struct {
	Operation string
	Path      string
	Cause     error
}

// NewStorageError creates a new StorageError.
func NewStorageError(operation, path string, cause error) StorageError {
	return StorageError{
		Operation: operation,
		Path:      path,
		Cause:     cause,
	}
}

// Error implements the error interface for StorageError.
func (e StorageError) Error() string {
	if e.Cause != nil {
		return fmt.Sprintf(
			"[StorageError] %s '%s': %v",
			e.Operation,
			e.Path,
			e.Cause,
		)
	}
	return fmt.Sprintf("[StorageError] %s '%s' failed", e.Operation, e.Path)
}

// FileSystemError represents filesystem operation failures.
type FileSystemError struct {
	Operation string
	Path      string
	Cause     error
}

// NewFileSystemError creates a new FileSystemError.
func NewFileSystemError(operation, path string, cause error) FileSystemError {
	return FileSystemError{
		Operation: operation,
		Path:      path,
		Cause:     cause,
	}
}

// Error implements the error interface for FileSystemError.
func (e FileSystemError) Error() string {
	if e.Cause != nil {
		return fmt.Sprintf(
			"[FileSystemError] %s '%s': %v",
			e.Operation,
			e.Path,
			e.Cause,
		)
	}
	return fmt.Sprintf("[FileSystemError] %s '%s' failed", e.Operation, e.Path)
}
