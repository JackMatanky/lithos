package errors

import "fmt"

// TemplateError captures problems encountered while parsing or executing
// templates. Only the template identifier, optional line number, and cause are
// retained to keep the type lean.
type TemplateError struct {
	BaseError
	template string
	line     int
}

// NewTemplateError creates a TemplateError for the provided template name.
func NewTemplateError(
	template string,
	line int,
	reason string,
	cause error,
) TemplateError {
	context := fmt.Sprintf("template '%s'", template)
	if line > 0 {
		context = fmt.Sprintf("%s line %d", context, line)
	}

	message := fmt.Sprintf("%s: %s", context, reason)
	if cause != nil {
		message = fmt.Sprintf("%s: %v", message, cause)
	}

	return TemplateError{
		BaseError: NewBaseError(message, cause),
		template:  template,
		line:      line,
	}
}

// Template returns the template identifier.
func (e TemplateError) Template() string {
	return e.template
}

// Line returns the 1-based line number when available, or 0 when unspecified.
func (e TemplateError) Line() int {
	return e.line
}
