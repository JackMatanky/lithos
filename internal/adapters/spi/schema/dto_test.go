package schema

import (
	"encoding/json"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestPropertyBankDTOToDomain verifies successful conversion of a property
// bank DTO into a domain.PropertyBank.
func TestPropertyBankDTOToDomain(t *testing.T) {
	rawSpec := json.RawMessage(`{"type":"string"}`)
	dto := propertyBankDTO{
		Properties: map[string]json.RawMessage{
			"title": json.RawMessage(
				`{"name":"title","required":true,"spec":` + string(
					rawSpec,
				) + `}`,
			),
		},
	}

	bank, err := dto.toDomain("bank-path")
	require.NoError(t, err)
	property, ok := bank.Properties["title"]
	require.True(t, ok)
	assert.True(t, property.Required)
	assert.Equal(t, "title", property.Name)
}

// TestSchemaDTOToDomainSortsProperties ensures schema DTO conversion sorts
// properties deterministically by name.
func TestSchemaDTOToDomainSortsProperties(t *testing.T) {
	dto := schemaDTO{
		Name: "note",
		Properties: map[string]json.RawMessage{
			"b": json.RawMessage(`{"name":"b","spec":{"type":"string"}}`),
			"a": json.RawMessage(`{"name":"a","spec":{"type":"string"}}`),
		},
	}

	schema, err := dto.toDomain("schema-path")
	require.NoError(t, err)
	require.Len(t, schema.Properties, 2)
	assert.Equal(t, "a", schema.Properties[0].Name)
	assert.Equal(t, "b", schema.Properties[1].Name)
}

// TestParsePropertyDefinitionInvalidSpec asserts that invalid specs raise an
// error with the expected message.
func TestParsePropertyDefinitionInvalidSpec(t *testing.T) {
	_, err := parsePropertyDefinition(
		"invalid",
		json.RawMessage(`{"name":"invalid","spec":{"type":"unknown"}}`),
		"schema-path",
		"note",
	)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "unsupported property type")
}

// TestParsePropertySpecUnsupportedType verifies unsupported property spec
// types are rejected.
func TestParsePropertySpecUnsupportedType(t *testing.T) {
	_, err := parsePropertySpec(json.RawMessage(`{"type":"unsupported"}`))
	require.Error(t, err)
	assert.Contains(t, err.Error(), "unsupported property type")
}

// TestPropertyDefinitionErrorWrapsCause confirms schema errors retain their
// original cause.
func TestPropertyDefinitionErrorWrapsCause(t *testing.T) {
	base := errors.New("cause")
	err := propertyDefinitionError("message", "schema", "path", base)
	require.Error(t, err)
	require.ErrorIs(t, err, base)
	assert.Contains(t, err.Error(), "message")
}
