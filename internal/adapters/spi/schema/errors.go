package schema

import (
	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// propertyDefinitionError constructs a SchemaError with a consistent
// remediation hint for malformed property data.
func propertyDefinitionError(
	message, schemaName, path string,
	cause error,
) error {
	return domainerrors.NewSchemaErrorWithRemediation(
		message,
		schemaName,
		syntaxRemediation(path),
		cause,
	)
}
