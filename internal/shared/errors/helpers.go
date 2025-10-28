package errors

import "fmt"

// Wrap adds context to an error using fmt.Errorf with %w verb.
// Returns nil if err is nil.
func Wrap(err error, message string) error {
	if err == nil {
		return nil
	}
	return fmt.Errorf("%s: %w", message, err)
}

// WrapWithContext adds formatted context to an error.
// Uses fmt.Sprintf for message formatting, then fmt.Errorf with %w for
// wrapping.
// Returns nil if err is nil.
func WrapWithContext(err error, format string, args ...interface{}) error {
	if err == nil {
		return nil
	}
	message := fmt.Sprintf(format, args...)
	return fmt.Errorf("%s: %w", message, err)
}
