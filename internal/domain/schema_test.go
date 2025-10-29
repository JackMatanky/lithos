package domain

import (
	"context"
	"encoding/json"
	"errors"
	"reflect"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"
)

// Test helpers and constants

const (
	testSchemaName    = "contact"
	testSchemaExtends = "base"
	testModifiedValue = "modified"
	testTag1          = "tag1"
	testTag2          = "tag2"
	testProp1         = "prop1"
	testProp2         = "prop2"
	testInvalidProp   = "invalid"
	testEmptyName     = ""
	testEmptyExtends  = ""
	testExcludesTag   = "tags"
	testNameProp      = "name"
	testEmailProp     = "email"
	testAttendeesProp = "attendees"
	testInternalID    = "internal_id"
	testLegacyField   = "legacy_field"
	testProp1Ref      = "#/properties/prop1"
	testProp2Ref      = "#/properties/prop2"
	testProp3Ref      = "#/properties/prop3"
	testNameRef       = "#/properties/name"
	testEmailRef      = "#/properties/email"
	testAttendeesRef  = "#/properties/attendees"
	testMeetingNote   = "meeting-note"
	testBaseNote      = "base-note"
	testStandalone    = "standalone"
	testProject       = "project"
)

// createValidTestProperty returns a valid Property for testing.
func createValidTestProperty(name string) Property {
	return Property{
		Name:     name,
		Required: false,
		Array:    false,
		Spec: mockPropertySpec{
			validateError: nil,
			specType:      PropertyTypeString,
		},
	}
}

// createInvalidTestProperty returns an invalid Property for delegation testing.
func createInvalidTestProperty(name string, validationErr error) Property {
	return Property{
		Name:     name,
		Required: false,
		Array:    false,
		Spec: mockPropertySpec{
			validateError: validationErr,
			specType:      PropertyTypeString,
		},
	}
}

// createRefProperty returns a Property with $ref for serialization testing.
func createRefProperty(name, ref string) Property {
	return Property{
		Name:     name,
		Required: false,
		Array:    false,
		Ref:      ref,
	}
}

// 2.1-UNIT-013: NewSchema creates valid schema with all fields.
func TestNewSchemaCreatesValidSchema(t *testing.T) {
	name := testSchemaName
	extends := testSchemaExtends
	excludes := []string{testExcludesTag}
	properties := []Property{createValidTestProperty(testNameProp)}

	schema, err := NewSchema(name, extends, excludes, properties)

	require.NoError(t, err)
	assert.Equal(t, testSchemaName, schema.Name)
	assert.Equal(t, testSchemaExtends, schema.Extends)
	assert.Equal(t, []string{testExcludesTag}, schema.Excludes)
	assert.Len(t, schema.Properties, 1)
	assert.Equal(t, testNameProp, schema.Properties[0].Name)
}

// 2.1-UNIT-014: NewSchema defensive copy of excludes slice.
func TestNewSchemaDefensiveCopyExcludes(t *testing.T) {
	name := testSchemaName
	excludes := []string{testTag1, testTag2}
	properties := []Property{createValidTestProperty(testNameProp)}

	schema, err := NewSchema(name, testSchemaExtends, excludes, properties)
	require.NoError(t, err)

	// Modify original slice
	excludes[0] = testModifiedValue
	excludes = append(
		excludes,
		"new",
	)

	// Verify schema unchanged
	assert.Equal(t, []string{testTag1, testTag2}, schema.Excludes)
	assert.Len(t, schema.Excludes, 2)
}

// 2.1-UNIT-015: NewSchema defensive copy of properties slice.
func TestNewSchemaDefensiveCopyProperties(t *testing.T) {
	name := testSchemaName
	properties := []Property{
		createValidTestProperty(testProp1),
		createValidTestProperty(testProp2),
	}

	schema, err := NewSchema(name, testEmptyExtends, nil, properties)
	require.NoError(t, err)

	// Modify original slice
	properties[0] = createValidTestProperty(testModifiedValue)
	properties = append(
		properties,
		createValidTestProperty("new"),
	)

	// Verify schema unchanged
	assert.Len(t, schema.Properties, 2)
	assert.Equal(t, testProp1, schema.Properties[0].Name)
	assert.Equal(t, testProp2, schema.Properties[1].Name)
}

// 2.1-UNIT-016: Modifying constructor args doesn't affect Schema.
func TestNewSchemaCompleteImmutability(t *testing.T) {
	name := testSchemaName
	extends := testSchemaExtends
	excludes := []string{testTag1, testTag2}
	properties := []Property{
		createValidTestProperty(testProp1),
		createValidTestProperty(testProp2),
	}

	schema, err := NewSchema(name, extends, excludes, properties)
	require.NoError(t, err)

	// Modify all original args
	excludes[0] = testModifiedValue
	excludes = append(
		excludes,
		"new",
	)
	properties[0] = createValidTestProperty(testModifiedValue)
	properties = append(
		properties,
		createValidTestProperty("new"),
	)

	// Verify schema completely unchanged
	assert.Equal(t, testSchemaName, schema.Name)
	assert.Equal(t, testSchemaExtends, schema.Extends)
	assert.Equal(t, []string{testTag1, testTag2}, schema.Excludes)
	assert.Len(t, schema.Properties, 2)
	assert.Equal(t, testProp1, schema.Properties[0].Name)
	assert.Equal(t, testProp2, schema.Properties[1].Name)
}

// 2.1-UNIT-017: Validate() succeeds with valid schema.
func TestSchemaValidateSuccess(t *testing.T) {
	schema := Schema{
		Name: testSchemaName,
		Properties: []Property{
			createValidTestProperty(testNameProp),
			createValidTestProperty(testEmailProp),
		},
	}

	err := schema.Validate(context.Background())

	assert.NoError(t, err)
}

// 2.1-UNIT-018: Validate() fails when Name is empty.
func TestSchemaValidateEmptyName(t *testing.T) {
	schema := Schema{
		Name:       testEmptyName,
		Properties: []Property{createValidTestProperty(testNameProp)},
	}

	err := schema.Validate(context.Background())

	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema name cannot be empty")
}

// 2.1-UNIT-019: Validate() fails when Excludes without Extends.
func TestSchemaValidateExcludesWithoutExtends(t *testing.T) {
	schema := Schema{
		Name:       testSchemaName,
		Extends:    testEmptyExtends,
		Excludes:   []string{testExcludesTag},
		Properties: []Property{createValidTestProperty(testNameProp)},
	}

	err := schema.Validate(context.Background())

	require.Error(t, err)
	assert.Contains(
		t,
		err.Error(),
		"excludes can only be set when extends is not empty",
	)
}

// 2.1-UNIT-020: Validate() delegates to Property.Validate().
func TestSchemaValidateDelegatesToProperty(t *testing.T) {
	expectedErr := errors.New("property validation failed")
	schema := Schema{
		Name: testSchemaName,
		Properties: []Property{
			createInvalidTestProperty(testInvalidProp, expectedErr),
		},
	}

	err := schema.Validate(context.Background())

	require.Error(t, err)
	assert.Contains(t, err.Error(), "property validation failed")
}

// 2.1-UNIT-021: Validate() aggregates multiple property errors.
func TestSchemaValidateAggregatesErrors(t *testing.T) {
	err1 := errors.New("error1")
	err2 := errors.New("error2")
	err3 := errors.New("error3")

	schema := Schema{
		Name: testSchemaName,
		Properties: []Property{
			createInvalidTestProperty("prop1", err1),
			createInvalidTestProperty("prop2", err2),
			createInvalidTestProperty("prop3", err3),
		},
	}

	err := schema.Validate(context.Background())

	require.Error(t, err)
	// errors.Join creates a multi-error that contains all errors
	assert.Contains(t, err.Error(), "error1")
	assert.Contains(t, err.Error(), "error2")
	assert.Contains(t, err.Error(), "error3")
}

// 2.1-UNIT-022: Validation errors include schema name (FR5).
func TestSchemaValidationErrorsIncludeSchemaName(t *testing.T) {
	expectedErr := errors.New("property validation failed")
	schema := Schema{
		Name: testSchemaName,
		Properties: []Property{
			createInvalidTestProperty(testInvalidProp, expectedErr),
		},
	}

	err := schema.Validate(context.Background())

	require.Error(t, err)
	// FR5: SchemaError must include schema name for traceability
	// The schema name is stored in the SchemaError.SchemaName field
	assert.Contains(t, err.Error(), testInvalidProp)
	assert.Contains(t, err.Error(), "validation failed")
}

// 2.1-UNIT-023: Schema has no setter methods (immutability).
func TestSchemaHasNoSetters(t *testing.T) {
	schemaType := reflect.TypeOf(Schema{})

	for i := range schemaType.NumMethod() {
		method := schemaType.Method(i)
		// Check that no method starts with "Set"
		assert.False(
			t,
			strings.HasPrefix(method.Name, "Set"),
			"found setter method: %s (violates immutability)",
			method.Name,
		)
	}

	// Also check pointer methods
	schemaPtrType := reflect.TypeOf(&Schema{})
	for i := range schemaPtrType.NumMethod() {
		method := schemaPtrType.Method(i)
		assert.False(
			t,
			strings.HasPrefix(method.Name, "Set"),
			"found setter method on pointer: %s (violates immutability)",
			method.Name,
		)
	}
}

// 2.1-UNIT-024: JSON round-trip preserves all fields.
func TestSchemaJSONRoundTrip(t *testing.T) {
	original := Schema{
		Name:     testSchemaName,
		Extends:  testSchemaExtends,
		Excludes: []string{testTag1, testTag2},
		Properties: []Property{
			createRefProperty(testNameProp, testNameRef),
			createRefProperty(testEmailProp, testEmailRef),
		},
	}

	// Marshal to JSON
	jsonBytes, err := json.Marshal(original)
	require.NoError(t, err)

	// Unmarshal back
	var unmarshaled Schema
	err = json.Unmarshal(jsonBytes, &unmarshaled)
	require.NoError(t, err)

	// Verify all fields preserved
	assert.Equal(t, original.Name, unmarshaled.Name)
	assert.Equal(t, original.Extends, unmarshaled.Extends)
	assert.Equal(t, original.Excludes, unmarshaled.Excludes)
	assert.Len(t, unmarshaled.Properties, len(original.Properties))
	assert.Equal(t, original.Properties[0].Name, unmarshaled.Properties[0].Name)
	assert.Equal(t, original.Properties[1].Name, unmarshaled.Properties[1].Name)
}

// 2.1-UNIT-025: YAML round-trip preserves all fields.
func TestSchemaYAMLRoundTrip(t *testing.T) {
	original := Schema{
		Name:     testSchemaName,
		Extends:  testSchemaExtends,
		Excludes: []string{testTag1, testTag2},
		Properties: []Property{
			createRefProperty(testNameProp, testNameRef),
			createRefProperty(testEmailProp, testEmailRef),
		},
	}

	// Marshal to YAML
	yamlBytes, err := yaml.Marshal(original)
	require.NoError(t, err)

	// Unmarshal back
	var unmarshaled Schema
	err = yaml.Unmarshal(yamlBytes, &unmarshaled)
	require.NoError(t, err)

	// Verify all fields preserved
	assert.Equal(t, original.Name, unmarshaled.Name)
	assert.Equal(t, original.Extends, unmarshaled.Extends)
	assert.Equal(t, original.Excludes, unmarshaled.Excludes)
	assert.Len(t, unmarshaled.Properties, len(original.Properties))
	assert.Equal(t, original.Properties[0].Name, unmarshaled.Properties[0].Name)
	assert.Equal(t, original.Properties[1].Name, unmarshaled.Properties[1].Name)
}

// 2.1-UNIT-026: Schema with Extends and Excludes marshals correctly.
func TestSchemaWithInheritanceFieldsMarshal(t *testing.T) {
	schema := Schema{
		Name:     testMeetingNote,
		Extends:  testBaseNote,
		Excludes: []string{testInternalID, testLegacyField},
		Properties: []Property{
			createRefProperty(testAttendeesProp, testAttendeesRef),
		},
	}

	// Marshal to JSON
	jsonBytes, err := json.Marshal(schema)
	require.NoError(t, err)

	// Unmarshal back
	var unmarshaled Schema
	err = json.Unmarshal(jsonBytes, &unmarshaled)
	require.NoError(t, err)

	// Verify all inheritance fields preserved
	assert.Equal(t, testMeetingNote, unmarshaled.Name)
	assert.Equal(t, testBaseNote, unmarshaled.Extends)
	assert.Equal(
		t,
		[]string{testInternalID, testLegacyField},
		unmarshaled.Excludes,
	)
	assert.Len(t, unmarshaled.Properties, 1)
}

// 2.1-UNIT-027: Schema with multiple Properties marshals correctly.
func TestSchemaWithMultiplePropertiesMarshal(t *testing.T) {
	schema := Schema{
		Name: testProject,
		Properties: []Property{
			createRefProperty("prop1", testProp1Ref),
			createRefProperty("prop2", testProp2Ref),
			createRefProperty("prop3", testProp3Ref),
		},
	}

	// Marshal to JSON
	jsonBytes, err := json.Marshal(schema)
	require.NoError(t, err)

	// Unmarshal back
	var unmarshaled Schema
	err = json.Unmarshal(jsonBytes, &unmarshaled)
	require.NoError(t, err)

	// Verify all properties preserved in order
	assert.Len(t, unmarshaled.Properties, 3)
	assert.Equal(t, "prop1", unmarshaled.Properties[0].Name)
	assert.Equal(t, "prop2", unmarshaled.Properties[1].Name)
	assert.Equal(t, "prop3", unmarshaled.Properties[2].Name)
}

// 2.1-UNIT-028: Schema with empty Extends omits field in JSON.
func TestSchemaWithEmptyExtendsOmitsField(t *testing.T) {
	schema := Schema{
		Name:       testStandalone,
		Extends:    testEmptyExtends, // Empty should be omitted
		Properties: []Property{createRefProperty(testNameProp, testNameRef)},
	}

	// Marshal to JSON
	jsonBytes, err := json.Marshal(schema)
	require.NoError(t, err)

	// Parse JSON to verify field omission
	var jsonMap map[string]interface{}
	err = json.Unmarshal(jsonBytes, &jsonMap)
	require.NoError(t, err)

	// Verify "extends" field is not present
	_, hasExtends := jsonMap["extends"]
	assert.False(t, hasExtends, "extends field should be omitted when empty")

	// Verify "excludes" field is not present (also empty)
	_, hasExcludes := jsonMap["excludes"]
	assert.False(t, hasExcludes, "excludes field should be omitted when empty")
}
