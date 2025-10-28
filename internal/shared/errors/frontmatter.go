package errors

// FrontmatterError represents frontmatter parsing or processing failures.
// It embeds BaseError to provide standard error functionality.
type FrontmatterError struct {
	BaseError

	field string
}

// NewFrontmatterError creates a new FrontmatterError with field context.
// The cause provides additional context about the frontmatter processing
// failure.
func NewFrontmatterError(message, field string, cause error) *FrontmatterError {
	return &FrontmatterError{
		BaseError: NewBaseError(message, cause),
		field:     field,
	}
}

// Field returns the name of the frontmatter field that caused the error.
func (e *FrontmatterError) Field() string {
	return e.field
}
