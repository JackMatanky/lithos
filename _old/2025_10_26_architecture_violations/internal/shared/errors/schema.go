package errors

import "fmt"

const (
	domainSchema = "schema"
)

// SchemaError represents high level schema failures (e.g. malformed file,
// inconsistent configuration).
type SchemaError struct {
	BaseError
	schema string
}

// NewSchemaError constructs a SchemaError for the given schema name.
func NewSchemaError(schema, reason string, cause error) SchemaError {
	message := fmt.Sprintf("schema '%s': %s", schema, reason)
	if cause != nil {
		message = fmt.Sprintf("%s: %v", message, cause)
	}

	return SchemaError{
		BaseError: NewBaseError(message, cause),
		schema:    schema,
	}
}

// Schema returns the schema identifier associated with the error.
func (e SchemaError) Schema() string {
	return e.schema
}

// Domain identifies the schema validation domain.
func (e SchemaError) Domain() string {
	return domainSchema
}

type SchemaValidationError struct {
	ValidationError
	schema string
}

// NewSchemaValidationError builds a SchemaValidationError. Callers may pass a
// non-nil cause to link deeper validation context.
func NewSchemaValidationError(
	schema,
	property,
	reason string,
	value interface{},
	cause error,
) SchemaValidationError {
	message := fmt.Sprintf(
		"schema '%s' property '%s': %s",
		schema,
		property,
		reason,
	)
	if value != nil {
		message = fmt.Sprintf("%s (value: %v)", message, value)
	}

	return SchemaValidationError{
		ValidationError: ValidationError{
			BaseError: NewBaseError(message, cause),
			property:  property,
			reason:    reason,
			value:     value,
		},
		schema: schema,
	}
}

// Schema returns the schema identifier where the validation failure occurred.
func (e *SchemaValidationError) Schema() string {
	return e.schema
}

func (e *SchemaValidationError) Domain() string {
	return "schema"
}

func (e *SchemaValidationError) Property() string {
	return e.ValidationError.Property()
}

func (e *SchemaValidationError) Reason() string {
	return e.ValidationError.Reason()
}

func (e *SchemaValidationError) Value() interface{} {
	return e.ValidationError.Value()
}

// SchemaNotFoundError indicates a schema lookup failure.
type SchemaNotFoundError struct {
	BaseError
	schema string
}

// NewSchemaNotFoundError constructs a not found error for schemaName.
func NewSchemaNotFoundError(schemaName string) SchemaNotFoundError {
	message := fmt.Sprintf("schema '%s' not found", schemaName)

	return SchemaNotFoundError{
		BaseError: NewBaseError(message, nil),
		schema:    schemaName,
	}
}

// Schema returns the missing schema identifier.
func (e SchemaNotFoundError) Schema() string {
	return e.schema
}

// Domain identifies the schema validation domain.
func (e SchemaNotFoundError) Domain() string {
	return domainSchema
}
