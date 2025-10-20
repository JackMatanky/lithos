package domain

import (
	"encoding/json"
	"fmt"
	"testing"
)

const testPropertyName = "test_prop"

func TestNewProperty(t *testing.T) {
	spec := StringPropertySpec{}
	prop := NewProperty(testPropertyName, true, false, spec)

	if prop.Name != testPropertyName {
		t.Errorf("Name = %q, want %q", prop.Name, testPropertyName)
	}
	if !prop.Required {
		t.Errorf("Required = %v, want %v", prop.Required, true)
	}
	if prop.Array {
		t.Errorf("Array = %v, want %v", prop.Array, false)
	}
	if prop.Spec == nil {
		t.Error("Spec should not be nil")
	}
}

func TestPropertyValidate(t *testing.T) {
	tests := []struct {
		name    string
		prop    Property
		wantErr bool
	}{
		{
			name: "valid property",
			prop: Property{
				Name:     "valid_name",
				Required: true,
				Array:    false,
				Spec:     StringPropertySpec{},
			},
			wantErr: false,
		},
		{
			name: "empty name",
			prop: Property{
				Name:     "",
				Required: true,
				Array:    false,
				Spec:     StringPropertySpec{},
			},
			wantErr: true,
		},
		{
			name: "invalid name characters",
			prop: Property{
				Name:     "invalid name",
				Required: true,
				Array:    false,
				Spec:     StringPropertySpec{},
			},
			wantErr: true,
		},
		{
			name: "nil spec",
			prop: Property{
				Name:     "test",
				Required: true,
				Array:    false,
				Spec:     nil,
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.prop.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"Property.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestStringPropertySpecValidate(t *testing.T) {
	tests := []struct {
		name    string
		spec    StringPropertySpec
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid string",
			spec:    StringPropertySpec{},
			value:   "test",
			wantErr: false,
		},
		{
			name:    "wrong type",
			spec:    StringPropertySpec{},
			value:   123,
			wantErr: true,
		},
		{
			name: "enum match",
			spec: StringPropertySpec{
				Enum: []string{"red", "green", "blue"},
			},
			value:   "red",
			wantErr: false,
		},
		{
			name: "enum no match",
			spec: StringPropertySpec{
				Enum: []string{"red", "green", "blue"},
			},
			value:   "yellow",
			wantErr: true,
		},
		{
			name: "pattern match",
			spec: StringPropertySpec{
				Pattern: "^[A-Z]+$",
			},
			value:   "ABC",
			wantErr: false,
		},
		{
			name: "pattern no match",
			spec: StringPropertySpec{
				Pattern: "^[A-Z]+$",
			},
			value:   "abc",
			wantErr: true,
		},
		{
			name: "invalid regex",
			spec: StringPropertySpec{
				Pattern: "[invalid",
			},
			value:   "test",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"StringPropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestNumberPropertySpecValidate(t *testing.T) {
	minValue := 0.0
	maxValue := 100.0
	stepValue := 1.0

	tests := []struct {
		name    string
		spec    NumberPropertySpec
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid number",
			spec:    NumberPropertySpec{},
			value:   42.5,
			wantErr: false,
		},
		{
			name:    "wrong type",
			spec:    NumberPropertySpec{},
			value:   "not a number",
			wantErr: true,
		},
		{
			name: "within bounds",
			spec: NumberPropertySpec{
				Min: &minValue,
				Max: &maxValue,
			},
			value:   50.0,
			wantErr: false,
		},
		{
			name: "below min",
			spec: NumberPropertySpec{
				Min: &minValue,
			},
			value:   -5.0,
			wantErr: true,
		},
		{
			name: "above max",
			spec: NumberPropertySpec{
				Max: &maxValue,
			},
			value:   150.0,
			wantErr: true,
		},
		{
			name: "integer constraint satisfied",
			spec: NumberPropertySpec{
				Step: &stepValue,
			},
			value:   42.0,
			wantErr: false,
		},
		{
			name: "integer constraint violated",
			spec: NumberPropertySpec{
				Step: &stepValue,
			},
			value:   42.5,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"NumberPropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestDatePropertySpecValidate(t *testing.T) {
	tests := []struct {
		name    string
		spec    DatePropertySpec
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid RFC3339",
			spec:    DatePropertySpec{},
			value:   "2023-10-20T10:30:00Z",
			wantErr: false,
		},
		{
			name: "valid custom format",
			spec: DatePropertySpec{
				Format: "2006-01-02",
			},
			value:   "2023-10-20",
			wantErr: false,
		},
		{
			name: "invalid format",
			spec: DatePropertySpec{
				Format: "2006-01-02",
			},
			value:   "10/20/2023",
			wantErr: true,
		},
		{
			name:    "wrong type",
			spec:    DatePropertySpec{},
			value:   123,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"DatePropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestFilePropertySpecValidate(t *testing.T) {
	tests := []struct {
		name    string
		spec    FilePropertySpec
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid string",
			spec:    FilePropertySpec{},
			value:   "some/path.md",
			wantErr: false,
		},
		{
			name:    "empty string",
			spec:    FilePropertySpec{},
			value:   "",
			wantErr: true,
		},
		{
			name:    "wrong type",
			spec:    FilePropertySpec{},
			value:   123,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"FilePropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestBoolPropertySpecValidate(t *testing.T) {
	tests := []struct {
		name    string
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid true",
			value:   true,
			wantErr: false,
		},
		{
			name:    "valid false",
			value:   false,
			wantErr: false,
		},
		{
			name:    "wrong type",
			value:   "true",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			spec := BoolPropertySpec{}
			err := spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"BoolPropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestPropertyJSONMarshaling(t *testing.T) {
	// Test marshaling
	prop := Property{
		Name:     testPropertyName,
		Required: true,
		Array:    false,
		Type:     propertyTypeString,
		Spec: StringPropertySpec{
			Enum: []string{"a", "b", "c"},
		},
	}

	data, err := json.Marshal(prop)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	// Verify the JSON contains expected fields
	var result map[string]interface{}
	if errUnmarshal := json.Unmarshal(data, &result); errUnmarshal != nil {
		t.Fatalf("Unmarshal result failed: %v", errUnmarshal)
	}

	if result["name"] != testPropertyName {
		t.Errorf("name = %v, want %v", result["name"], testPropertyName)
	}
	if result["required"] != true {
		t.Errorf("required = %v, want %v", result["required"], true)
	}
	if result["type"] != propertyTypeString {
		t.Errorf("type = %v, want %v", result["type"], propertyTypeString)
	}
	if enum, ok := result["enum"].([]interface{}); !ok || len(enum) != 3 {
		t.Errorf("enum = %v, want 3 elements", result["enum"])
	}
}

func TestPropertyJSONUnmarshaling(t *testing.T) {
	jsonData := fmt.Sprintf(`{
		"name": %q,
		"required": true,
		"array": false,
		"type": "string",
		"enum": ["a", "b", "c"]
	}`, testPropertyName)

	var prop Property
	err := json.Unmarshal([]byte(jsonData), &prop)
	if err != nil {
		t.Fatalf("Unmarshal failed: %v", err)
	}

	if prop.Name != testPropertyName {
		t.Errorf("Name = %q, want %q", prop.Name, testPropertyName)
	}
	if !prop.Required {
		t.Errorf("Required = %v, want %v", prop.Required, true)
	}
	if prop.Array {
		t.Errorf("Array = %v, want %v", prop.Array, false)
	}
	if prop.Type != propertyTypeString {
		t.Errorf("Type = %q, want %q", prop.Type, propertyTypeString)
	}

	spec, ok := prop.Spec.(StringPropertySpec)
	if !ok {
		t.Fatal("Spec is not StringPropertySpec")
	}
	if len(spec.Enum) != 3 || spec.Enum[0] != "a" {
		t.Errorf("Enum = %v, want [a b c]", spec.Enum)
	}
}

func TestPropertyJSONRoundTrip(t *testing.T) {
	original := Property{
		Name:     "round_trip",
		Required: false,
		Array:    true,
		Type:     propertyTypeNumber,
		Spec: NumberPropertySpec{
			Min: func() *float64 { v := 0.0; return &v }(),
			Max: func() *float64 { v := 100.0; return &v }(),
		},
	}

	// Marshal
	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	// Unmarshal
	var unmarshaled Property
	err = json.Unmarshal(data, &unmarshaled)
	if err != nil {
		t.Fatalf("Unmarshal failed: %v", err)
	}

	// Compare
	if unmarshaled.Name != original.Name {
		t.Errorf("Name = %q, want %q", unmarshaled.Name, original.Name)
	}
	if unmarshaled.Required != original.Required {
		t.Errorf(
			"Required = %v, want %v",
			unmarshaled.Required,
			original.Required,
		)
	}
	if unmarshaled.Array != original.Array {
		t.Errorf("Array = %v, want %v", unmarshaled.Array, original.Array)
	}
	if unmarshaled.Type != original.Type {
		t.Errorf("Type = %q, want %q", unmarshaled.Type, original.Type)
	}
}
