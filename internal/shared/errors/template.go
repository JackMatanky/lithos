package errors

// TemplateError represents template processing failures.
// It embeds BaseError to provide standard error functionality.
type TemplateError struct {
	BaseError

	templateID string
}

// NewTemplateError creates a new TemplateError with template context.
// The cause provides additional context about the template processing failure.
func NewTemplateError(
	message string,
	templateID string,
	cause error,
) *TemplateError {
	return &TemplateError{
		BaseError:  NewBaseError(message, cause),
		templateID: templateID,
	}
}

// TemplateID returns the identifier of the template that failed processing.
func (e *TemplateError) TemplateID() string {
	return e.templateID
}
