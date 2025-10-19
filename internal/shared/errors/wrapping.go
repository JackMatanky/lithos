package errors

import (
	"errors"
	"fmt"
)

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
