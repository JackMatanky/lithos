package errors

// Result represents a value that can be either a success (Ok) or a failure
// (Err). It mirrors Rust's Result type to provide expressive, type-safe error
// handling while staying idiomatic to Go.
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
	if err == nil {
		panic("errors.Err called with nil error")
	}
	var zero T
	return Result[T]{value: zero, err: err}
}

// ErrOrNil returns the contained error without panicking when Result is ok.
func (r Result[T]) Err() error {
	return r.err
}

// ValueOr returns the wrapped value when the Result is ok, otherwise returns
// the provided fallback value.
func (r Result[T]) ValueOr(fallback T) T {
	if r.err != nil {
		return fallback
	}
	return r.value
}

// OrElse allows recovery from errors by invoking fn when the Result is an
// error. When the Result is ok the original value is preserved.
func (r Result[T]) OrElse(fn func(error) Result[T]) Result[T] {
	if r.err != nil {
		return fn(r.err)
	}
	return r
}

// Inspect executes fn with the contained value when the Result is ok.
// The original Result is returned to support fluent chaining.
func (r Result[T]) Inspect(fn func(T)) Result[T] {
	if r.err == nil && fn != nil {
		fn(r.value)
	}
	return r
}

// InspectErr executes fn with the contained error when the Result is an error.
// The original Result is returned to support fluent chaining.
func (r Result[T]) InspectErr(fn func(error)) Result[T] {
	if r.err != nil && fn != nil {
		fn(r.err)
	}
	return r
}

// Map transforms the contained value with fn when the Result is ok. When the
// Result is an error, the original error is propagated unchanged.
func Map[T, U any](r Result[T], fn func(T) U) Result[U] {
	if r.err != nil {
		return Err[U](r.err)
	}
	return Ok(fn(r.value))
}

// AndThen chains computations that return Result. When the current Result is
// ok, fn is invoked with the contained value. If fn returns an error Result,
// that error propagates to the caller. If the current Result is already an
// error, fn is skipped and the existing error propagates.
func AndThen[T, U any](r Result[T], fn func(T) Result[U]) Result[U] {
	if r.err != nil {
		return Err[U](r.err)
	}
	return fn(r.value)
}
