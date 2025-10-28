package errors

// SchemaError represents schema validation or processing failures.
// It embeds BaseError to provide standard error functionality.
type SchemaError struct {
	BaseError

	schemaName string
}

// NewSchemaError creates a new SchemaError with schema context.
// The cause provides additional context about the schema processing failure.
func NewSchemaError(message, schemaName string, cause error) *SchemaError {
	return &SchemaError{
		BaseError:  NewBaseError(message, cause),
		schemaName: schemaName,
	}
}

// SchemaName returns the name of the schema that failed processing.
func (e *SchemaError) SchemaName() string {
	return e.schemaName
}
