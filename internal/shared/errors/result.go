// Package errors provides domain-specific error types and functional error
// handling patterns for consistent error handling across the application. This
// package implements a custom Result[T] pattern inspired by Rust's Result type
// for better error handling ergonomics.
//
// The package is organized across multiple files:
// - result.go: Result[T] type and functional error handling
// - types.go: Domain-specific error types
// - wrapping.go: Error wrapping and utility functions
package errors

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
