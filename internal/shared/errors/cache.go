package errors

import (
	"fmt"
)

// CacheWriteError represents cache write operation failures.
// It embeds BaseError to provide standard error functionality.
type CacheWriteError struct {
	BaseError

	NoteID    string
	Path      string
	Operation string
}

// CacheReadError represents cache read operation failures.
// It embeds BaseError to provide standard error functionality.
type CacheReadError struct {
	BaseError

	NoteID    string
	Path      string
	Operation string
}

// CacheDeleteError represents cache delete operation failures.
// It embeds BaseError to provide standard error functionality.
type CacheDeleteError struct {
	BaseError

	NoteID    string
	Path      string
	Operation string
}

// NewCacheWriteError creates a new CacheWriteError with operation context.
// The message is fixed as "cache write failed" and the cause provides
// additional context.
func NewCacheWriteError(
	noteID string,
	path string,
	operation string,
	cause error,
) *CacheWriteError {
	return &CacheWriteError{
		BaseError: NewBaseError("cache write failed", cause),
		NoteID:    noteID,
		Path:      path,
		Operation: operation,
	}
}

// NewCacheReadError creates a new CacheReadError with operation context.
// The message is fixed as "cache read failed" and the cause provides
// additional context.
func NewCacheReadError(
	noteID string,
	path string,
	operation string,
	cause error,
) *CacheReadError {
	return &CacheReadError{
		BaseError: NewBaseError("cache read failed", cause),
		NoteID:    noteID,
		Path:      path,
		Operation: operation,
	}
}

// NewCacheDeleteError creates a new CacheDeleteError with operation context.
// The message is fixed as "cache delete failed" and the cause provides
// additional context.
func NewCacheDeleteError(
	noteID string,
	path string,
	operation string,
	cause error,
) *CacheDeleteError {
	return &CacheDeleteError{
		BaseError: NewBaseError("cache delete failed", cause),
		NoteID:    noteID,
		Path:      path,
		Operation: operation,
	}
}

// Error implements the error interface for CacheWriteError.
// Returns a formatted error message including operation context.
func (e *CacheWriteError) Error() string {
	return fmt.Sprintf("cache write failed for note %s at %s during %s: %v",
		e.NoteID, e.Path, e.Operation, e.Cause())
}

// Cause returns the underlying cause error for CacheWriteError.
func (e *CacheWriteError) Cause() error {
	return e.BaseError.Cause()
}

// Error implements the error interface for CacheReadError.
// Returns a formatted error message including operation context.
func (e *CacheReadError) Error() string {
	return fmt.Sprintf("cache read failed for note %s at %s during %s: %v",
		e.NoteID, e.Path, e.Operation, e.Cause())
}

// Cause returns the underlying cause error for CacheReadError.
func (e *CacheReadError) Cause() error {
	return e.BaseError.Cause()
}

// Error implements the error interface for CacheDeleteError.
// Returns a formatted error message including operation context.
func (e *CacheDeleteError) Error() string {
	return fmt.Sprintf("cache delete failed for note %s at %s during %s: %v",
		e.NoteID, e.Path, e.Operation, e.Cause())
}

// Cause returns the underlying cause error for CacheDeleteError.
func (e *CacheDeleteError) Cause() error {
	return e.BaseError.Cause()
}
