package errors

// SchemaError represents schema validation or processing failures.
// It embeds BaseError to provide standard error functionality.
type SchemaError struct {
	BaseError

	SchemaName  string
	Remediation string
}

// NewSchemaError creates a new SchemaError with schema context.
// The cause provides additional context about the schema processing failure.
func NewSchemaError(message, schemaName string, cause error) *SchemaError {
	return &SchemaError{
		BaseError:   NewBaseError(message, cause),
		SchemaName:  schemaName,
		Remediation: "",
	}
}

// NewSchemaErrorWithRemediation creates a new SchemaError with schema context
// and remediation hint.
// The cause provides additional context about the schema processing failure.
func NewSchemaErrorWithRemediation(
	message, schemaName, remediation string,
	cause error,
) *SchemaError {
	return &SchemaError{
		BaseError:   NewBaseError(message, cause),
		SchemaName:  schemaName,
		Remediation: remediation,
	}
}
