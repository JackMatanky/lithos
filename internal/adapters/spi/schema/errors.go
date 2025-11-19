package schema

import (
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
)

// propertyDefinitionError constructs a SchemaError with a consistent
// remediation hint for malformed property data.
func propertyDefinitionError(
	message, schemaName, path string,
	cause error,
) error {
	return lithosErr.NewSchemaErrorWithRemediation(
		message,
		schemaName,
		syntaxRemediation(path),
		cause,
	)
}
