// Package schema provides filesystem-based implementations of schema loading
// ports for hexagonal architecture.
//
// This file contains JSON parsing logic for schema and property bank files.
package schema

import (
	"encoding/json"
	"fmt"
	"strings"

	"github.com/JackMatanky/lithos/internal/shared/errors"
)

/* ---------------------------------------------------------- */
/*                     Schema JSON Parsing                    */
/* ---------------------------------------------------------- */

// parseSchemaJSON parses JSON data into a schemaDTO.
func (s *SchemaLoaderAdapter) parseSchemaJSON(
	path string,
	data []byte,
) (schemaDTO, error) {
	var dto schemaDTO
	if err := s.unmarshalSchemaJSON(data, &dto); err != nil {
		return schemaDTO{}, s.createJSONParseError(path, err)
	}
	return dto, nil
}

// unmarshalSchemaJSON unmarshals JSON data into schema DTO.
func (s *SchemaLoaderAdapter) unmarshalSchemaJSON(
	data []byte,
	dto *schemaDTO,
) error {
	return json.Unmarshal(data, dto)
}

// createJSONParseError creates a schema error for JSON parsing failures.
func (s *SchemaLoaderAdapter) createJSONParseError(
	path string,
	err error,
) error {
	return errors.NewSchemaError(
		path,
		fmt.Sprintf("malformed JSON: %v", err),
		err,
	)
}

/* ---------------------------------------------------------- */
/*                 Property Bank JSON Parsing                 */
/* ---------------------------------------------------------- */

// parsePropertyBankJSON parses JSON data into a propertyBankDTO.
func (s *SchemaLoaderAdapter) parsePropertyBankJSON(
	path string,
	data []byte,
) (propertyBankDTO, error) {
	var dto propertyBankDTO
	if err := s.unmarshalPropertyBankJSON(data, &dto); err != nil {
		return propertyBankDTO{}, s.createPropertyBankJSONParseError(path, err)
	}
	return dto, nil
}

// unmarshalPropertyBankJSON unmarshals JSON data into property bank DTO.
func (s *SchemaLoaderAdapter) unmarshalPropertyBankJSON(
	data []byte,
	dto *propertyBankDTO,
) error {
	return json.Unmarshal(data, dto)
}

// createPropertyBankJSONParseError creates a schema error for property bank
// JSON parsing failures.
func (s *SchemaLoaderAdapter) createPropertyBankJSONParseError(
	path string,
	err error,
) error {
	return errors.NewSchemaError(
		path,
		fmt.Sprintf("malformed JSON in property bank: %v", err),
		err,
	)
}

/* ---------------------------------------------------------- */
/*                    Reference Resolution                    */
/* ---------------------------------------------------------- */

// extractRefPropertyName extracts property name from $ref JSON pointer.
// Supports formats like "#/properties/property-name" and
// "file.json#/properties/property-name".
func (s *SchemaLoaderAdapter) extractRefPropertyName(ref string) string {
	// Handle cross-file references (simplified - would need full
	// implementation)
	if strings.Contains(ref, "#/properties/") {
		parts := strings.Split(ref, "#/properties/")
		if len(parts) == 2 {
			return parts[1]
		}
	}
	return ""
}
