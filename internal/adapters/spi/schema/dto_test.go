package schema

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestPropertyBankDTOToDomain verifies successful conversion of a property
// bank DTO into a domain.PropertyBank.
func TestPropertyBankDTOToDomain(t *testing.T) {
	dto := propertyBankDTO{
		Properties: map[string]json.RawMessage{
			"title": json.RawMessage(
				`{"type":"string","required":true,"array":false}`,
			),
		},
	}

	bank, err := dto.toDomain(context.Background(), "bank-path")
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
			"b": json.RawMessage(
				`{"type":"string","required":false,"array":false}`,
			),
			"a": json.RawMessage(
				`{"type":"string","required":false,"array":false}`,
			),
		},
	}

	// Create empty bank for this test
	bankDTO := propertyBankDTO{Properties: map[string]json.RawMessage{}}
	bank, err := bankDTO.toDomain(context.Background(), "bank-path")
	require.NoError(t, err)

	schema, err := dto.toDomain(context.Background(), "schema-path", bank)
	require.NoError(t, err)
	require.Len(t, schema.Properties, 2)
	assert.Equal(t, "a", schema.Properties[0].Name)
	assert.Equal(t, "b", schema.Properties[1].Name)
}

// TestSchemaDTOToDomainMixedProperties tests that schemas with both inline
// property definitions and $ref references are resolved correctly.
func TestSchemaDTOToDomainMixedProperties(t *testing.T) {
	// Create property bank with a referenceable property
	bankDTO := propertyBankDTO{
		Properties: map[string]json.RawMessage{
			"shared_title": json.RawMessage(
				`{"type":"string","required":true,"array":false}`,
			),
		},
	}
	bank, err := bankDTO.toDomain(context.Background(), "bank-path")
	require.NoError(t, err)

	// Create schema with mixed properties: inline + $ref
	dto := schemaDTO{
		Name: "test_schema",
		Properties: map[string]json.RawMessage{
			"inline_prop": json.RawMessage(
				`{"type":"number","required":false,"array":false}`,
			),
			"ref_prop": json.RawMessage(
				`{"$ref":"#/properties/shared_title"}`,
			),
		},
	}

	schema, err := dto.toDomain(context.Background(), "schema-path", bank)
	require.NoError(t, err)
	require.Len(t, schema.Properties, 2)

	// Check inline property
	inlineProp := schema.Properties[0]
	assert.Equal(t, "inline_prop", inlineProp.Name)
	assert.False(t, inlineProp.Required)
	assert.False(t, inlineProp.Array)

	// Check $ref resolved property
	refProp := schema.Properties[1]
	assert.Equal(t, "ref_prop", refProp.Name)
	assert.True(t, refProp.Required) // From bank definition
	assert.False(t, refProp.Array)   // From bank definition
}

// TestSchemaDTOToDomainInvalidRef tests that invalid $ref references produce
// appropriate errors.
func TestSchemaDTOToDomainInvalidRef(t *testing.T) {
	// Create empty property bank
	bankDTO := propertyBankDTO{Properties: map[string]json.RawMessage{}}
	bank, err := bankDTO.toDomain(context.Background(), "bank-path")
	require.NoError(t, err)

	// Create schema with invalid $ref
	dto := schemaDTO{
		Name: "test_schema",
		Properties: map[string]json.RawMessage{
			"invalid_ref": json.RawMessage(
				`{"$ref":"#/properties/nonexistent"}`,
			),
		},
	}

	_, err = dto.toDomain(context.Background(), "schema-path", bank)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "not found in property bank")
}
