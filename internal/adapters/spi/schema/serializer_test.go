package schema

import (
	"encoding/json"
	"strings"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
)

func TestMarshalProperty_StringSpec(t *testing.T) {
	property := domain.NewProperty(
		"status",
		true,
		false,
		domain.StringPropertySpec{
			Enum: []string{"active", "inactive"},
		},
	)

	data, err := MarshalProperty(property)
	if err != nil {
		t.Fatalf("MarshalProperty() error = %v", err)
	}

	var result map[string]interface{}
	if decodeErr := json.Unmarshal(data, &result); decodeErr != nil {
		t.Fatalf("json.Unmarshal() error = %v", decodeErr)
	}

	if result["type"] != propertyTypeString {
		t.Fatalf("type = %v, want %s", result["type"], propertyTypeString)
	}
	enum, ok := result["enum"].([]interface{})
	if !ok || len(enum) != 2 {
		t.Fatalf("enum = %v, want length 2", result["enum"])
	}
}

func TestMarshalProperty_NumberSpec(t *testing.T) {
	step := 1.0
	property := domain.NewProperty(
		"age",
		false,
		false,
		domain.NumberPropertySpec{
			Min:  pointerToFloat(0),
			Max:  pointerToFloat(120),
			Step: &step,
		},
	)

	data, err := MarshalProperty(property)
	if err != nil {
		t.Fatalf("MarshalProperty() error = %v", err)
	}

	var result map[string]interface{}
	if decodeErr := json.Unmarshal(data, &result); decodeErr != nil {
		t.Fatalf("json.Unmarshal() error = %v", decodeErr)
	}

	if result["min"] != float64(0) || result["max"] != float64(120) {
		t.Fatalf(
			"min/max = (%v,%v), want (0,120)",
			result["min"],
			result["max"],
		)
	}
	if result["step"] != float64(1) {
		t.Fatalf("step = %v, want 1", result["step"])
	}
}

func TestMarshalProperty_BoolSpec(t *testing.T) {
	property := domain.NewProperty(
		"enabled",
		true,
		false,
		domain.BoolPropertySpec{},
	)

	data, err := MarshalProperty(property)
	if err != nil {
		t.Fatalf("MarshalProperty() error = %v", err)
	}

	var result map[string]interface{}
	if decodeErr := json.Unmarshal(data, &result); decodeErr != nil {
		t.Fatalf("json.Unmarshal() error = %v", decodeErr)
	}

	if result["type"] != propertyTypeBool {
		t.Fatalf("type = %v, want %s", result["type"], propertyTypeBool)
	}
}

func TestUnmarshalProperty_StringSpec(t *testing.T) {
	jsonBody := `
	{
		"name": "status",
		"required": true,
		"array": false,
		"type": "string",
		"enum": ["active", "inactive"]
	}`

	property, err := UnmarshalProperty([]byte(strings.TrimSpace(jsonBody)))
	if err != nil {
		t.Fatalf("UnmarshalProperty() error = %v", err)
	}

	typeName, err := property.TypeName()
	if err != nil {
		t.Fatalf("TypeName() error = %v", err)
	}

	if typeName != propertyTypeString {
		t.Fatalf("TypeName() = %s, want %s", typeName, propertyTypeString)
	}
}

func TestUnmarshalProperty_FileSpec(t *testing.T) {
	jsonBody := `
	{
		"name": "doc",
		"required": false,
		"array": false,
		"type": "file",
		"fileClass": "project",
		"directory": "projects/"
	}`

	property, err := UnmarshalProperty([]byte(strings.TrimSpace(jsonBody)))
	if err != nil {
		t.Fatalf("UnmarshalProperty() error = %v", err)
	}

	typeName, err := property.TypeName()
	if err != nil {
		t.Fatalf("TypeName() error = %v", err)
	}

	if typeName != propertyTypeFile {
		t.Fatalf("TypeName() = %s, want %s", typeName, propertyTypeFile)
	}
}

func pointerToFloat(value float64) *float64 {
	return &value
}
