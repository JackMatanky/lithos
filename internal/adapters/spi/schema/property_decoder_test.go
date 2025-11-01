package schema

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestDecodePropertyDefinitionValid asserts that valid property definitions
// are correctly decoded into their components.
func TestDecodePropertyDefinitionValid(t *testing.T) {
	tests := []struct {
		name          string
		input         string
		expectedType  string
		expectedReq   bool
		expectedArray bool
		expectedSpec  string
	}{
		{
			name:          "simple string property",
			input:         `{"type":"string","required":true,"array":false}`,
			expectedType:  "string",
			expectedReq:   true,
			expectedArray: false,
			expectedSpec:  "{}",
		},
		{
			name:          "array property with spec",
			input:         `{"type":"number","required":false,"array":true,"min":0,"max":100}`,
			expectedType:  "number",
			expectedReq:   false,
			expectedArray: true,
			expectedSpec:  `{"min":0,"max":100}`,
		},
		{
			name:          "property without flags",
			input:         `{"type":"boolean","custom":"value"}`,
			expectedType:  "boolean",
			expectedReq:   false,
			expectedArray: false,
			expectedSpec:  `{"custom":"value"}`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			typeName, required, array, specRaw, err := decodePropertyDefinition(
				json.RawMessage(tt.input),
			)
			require.NoError(t, err)
			assert.Equal(t, tt.expectedType, typeName)
			assert.Equal(t, tt.expectedReq, required)
			assert.Equal(t, tt.expectedArray, array)

			var spec map[string]interface{}
			require.NoError(t, json.Unmarshal(specRaw, &spec))
			expectedSpec := make(map[string]interface{})
			require.NoError(
				t,
				json.Unmarshal([]byte(tt.expectedSpec), &expectedSpec),
			)
			assert.Equal(t, expectedSpec, spec)
		})
	}
}

// TestDecodePropertyDefinitionErrors asserts that invalid property definitions
// produce appropriate errors.
func TestDecodePropertyDefinitionErrors(t *testing.T) {
	tests := []struct {
		name        string
		input       string
		expectedErr string
	}{
		{
			name:        "missing type field",
			input:       `{"required":true,"array":false}`,
			expectedErr: "type field is required",
		},
		{
			name:        "empty type field",
			input:       `{"type":"","required":true,"array":false}`,
			expectedErr: "type field is required",
		},
		{
			name:        "invalid JSON",
			input:       `{"type":"string","required":`,
			expectedErr: "unexpected end of JSON input",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, _, _, _, err := decodePropertyDefinition(
				json.RawMessage(tt.input),
			)
			require.Error(t, err)
			assert.Contains(t, err.Error(), tt.expectedErr)
		})
	}
}

// TestParsePropertyDefinitionInvalidSpec asserts that invalid specs raise an
// error with the expected message.
func TestParsePropertyDefinitionInvalidSpec(t *testing.T) {
	_, err := parsePropertyDef(
		context.Background(),
		"invalid",
		json.RawMessage(`{"type":"unknown","required":false,"array":false}`),
		"schema-path",
		"note",
	)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "unsupported property type")
}

// TestParsePropertyDefinitionValid asserts that valid property definitions
// are correctly parsed into domain.Property objects.
func TestParsePropertyDefinitionValid(t *testing.T) {
	property, err := parsePropertyDef(
		context.Background(),
		"title",
		json.RawMessage(
			`{"type":"string","required":true,"array":false,"minLength":1,"maxLength":100}`,
		),
		"schema-path",
		"note",
	)
	require.NoError(t, err)
	assert.Equal(t, "title", property.Name)
	assert.True(t, property.Required)
	assert.False(t, property.Array)
	assert.NotNil(t, property.Spec)
}
