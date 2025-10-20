package domain

import (
	"encoding/json"
	"fmt"
	"testing"
)

const testSchemaName = "test_schema"

func TestNewSchema(t *testing.T) {
	props := []Property{
		NewProperty("title", true, false, StringPropertySpec{}),
		NewProperty("count", false, false, NumberPropertySpec{}),
	}

	schema := NewSchema(testSchemaName, props)

	if schema.Name != testSchemaName {
		t.Errorf("Name = %q, want %q", schema.Name, testSchemaName)
	}
	if len(schema.Properties) != 2 {
		t.Errorf("Properties length = %d, want %d", len(schema.Properties), 2)
	}
	if schema.Extends != "" {
		t.Errorf("Extends = %q, want empty", schema.Extends)
	}
	if len(schema.Excludes) != 0 {
		t.Errorf("Excludes length = %d, want 0", len(schema.Excludes))
	}
}

func TestSchemaValidate(t *testing.T) {
	tests := []struct {
		name    string
		schema  Schema
		wantErr bool
	}{
		{
			name: "valid schema",
			schema: Schema{
				Name: "valid_schema",
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
				},
			},
			wantErr: false,
		},
		{
			name: "valid extends and excludes",
			schema: Schema{
				Name:     "child_schema",
				Extends:  "base_schema",
				Excludes: []string{"legacyProp"},
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
					NewProperty(
						"legacyProp",
						false,
						false,
						StringPropertySpec{},
					),
				},
			},
			wantErr: false,
		},
		{
			name: "invalid extends format",
			schema: Schema{
				Name:    "invalid_extends_schema",
				Extends: "invalid schema",
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
				},
			},
			wantErr: true,
		},
		{
			name: "self referencing extends",
			schema: Schema{
				Name:    "self_ref",
				Extends: "self_ref",
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
				},
			},
			wantErr: true,
		},
		{
			name: "empty excludes entry",
			schema: Schema{
				Name:     "empty_exclude",
				Excludes: []string{""},
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
				},
			},
			wantErr: true,
		},
		{
			name: "invalid exclude identifier",
			schema: Schema{
				Name:     "invalid_exclude",
				Excludes: []string{"invalid name"},
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
				},
			},
			wantErr: true,
		},
		{
			name: "duplicate excludes entry",
			schema: Schema{
				Name:     "duplicate_exclude",
				Excludes: []string{"dup", "dup"},
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
				},
			},
			wantErr: true,
		},
		{
			name: "empty name",
			schema: Schema{
				Name: "",
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
				},
			},
			wantErr: true,
		},
		{
			name: "invalid name",
			schema: Schema{
				Name: "invalid name",
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
				},
			},
			wantErr: true,
		},
		{
			name: "duplicate property names",
			schema: Schema{
				Name: "test",
				Properties: []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
					NewProperty("title", false, false, NumberPropertySpec{}),
				},
			},
			wantErr: true,
		},
		{
			name: "invalid property",
			schema: Schema{
				Name: "test",
				Properties: []Property{
					{Name: "", Required: true, Spec: StringPropertySpec{}},
				},
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.schema.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"Schema.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestSchemaGetProperty(t *testing.T) {
	schema := Schema{
		Name: "test",
		Properties: []Property{
			NewProperty("title", true, false, StringPropertySpec{}),
			NewProperty("count", false, false, NumberPropertySpec{}),
		},
	}

	// Test existing property
	prop := schema.GetProperty("title")
	if prop == nil {
		t.Fatal("GetProperty returned nil for existing property")
	}
	if prop.Name != "title" {
		t.Errorf("Property name = %q, want %q", prop.Name, "title")
	}

	// Test non-existing property
	prop = schema.GetProperty("nonexistent")
	if prop != nil {
		t.Errorf("GetProperty returned %v for non-existing property", prop)
	}
}

func TestSchemaHasProperty(t *testing.T) {
	schema := Schema{
		Name: "test",
		Properties: []Property{
			NewProperty("title", true, false, StringPropertySpec{}),
		},
	}

	if !schema.HasProperty("title") {
		t.Error("HasProperty returned false for existing property")
	}
	if schema.HasProperty("nonexistent") {
		t.Error("HasProperty returned true for non-existing property")
	}
}

func TestSchemaJSONMarshaling(t *testing.T) {
	schema := Schema{
		Name:     testSchemaName,
		Extends:  "base_schema",
		Excludes: []string{"old_prop"},
		Properties: []Property{
			{
				Name:     "title",
				Required: true,
				Array:    false,
				Type:     "string",
				Spec:     StringPropertySpec{},
			},
		},
	}

	data, err := json.Marshal(schema)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	// Verify the JSON contains expected fields
	var result map[string]interface{}
	if errUnmarshal := json.Unmarshal(data, &result); errUnmarshal != nil {
		t.Fatalf("Unmarshal result failed: %v", errUnmarshal)
	}

	if result["name"] != testSchemaName {
		t.Errorf("name = %v, want %v", result["name"], testSchemaName)
	}
	if result["extends"] != "base_schema" {
		t.Errorf("extends = %v, want %v", result["extends"], "base_schema")
	}
	if excludes, ok := result["excludes"].([]interface{}); !ok ||
		len(excludes) != 1 {
		t.Errorf("excludes = %v, want 1 element", result["excludes"])
	}
	if props, ok := result["properties"].([]interface{}); !ok ||
		len(props) != 1 {
		t.Errorf("properties = %v, want 1 element", result["properties"])
	}
}

func TestSchemaJSONUnmarshaling(t *testing.T) {
	jsonData := fmt.Sprintf(`{
	"name": %q,
	"extends": "base_schema",
	"excludes": ["old_prop"],
	"properties": [
		{
			"name": "title",
			"required": true,
			"array": false,
			"type": "string"
		}
	]
}`, testSchemaName)

	var schema Schema
	err := json.Unmarshal([]byte(jsonData), &schema)
	if err != nil {
		t.Fatalf("Unmarshal failed: %v", err)
	}

	if schema.Name != testSchemaName {
		t.Errorf("Name = %q, want %q", schema.Name, testSchemaName)
	}
	if schema.Extends != "base_schema" {
		t.Errorf("Extends = %q, want %q", schema.Extends, "base_schema")
	}
	if len(schema.Excludes) != 1 || schema.Excludes[0] != "old_prop" {
		t.Errorf("Excludes = %v, want [old_prop]", schema.Excludes)
	}
	if len(schema.Properties) != 1 {
		t.Errorf("Properties length = %d, want 1", len(schema.Properties))
	}
	if schema.Properties[0].Name != "title" {
		t.Errorf(
			"Property name = %q, want %q",
			schema.Properties[0].Name,
			"title",
		)
	}
}

func TestSchemaJSONRoundTrip(t *testing.T) {
	original := Schema{
		Name:     "round_trip",
		Extends:  "parent",
		Excludes: []string{"exclude_me"},
		Properties: []Property{
			NewProperty("title", true, false, StringPropertySpec{
				Enum: []string{"a", "b"},
			}),
		},
	}

	// Marshal
	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	// Unmarshal
	var unmarshaled Schema
	err = json.Unmarshal(data, &unmarshaled)
	if err != nil {
		t.Fatalf("Unmarshal failed: %v", err)
	}

	// Compare
	if unmarshaled.Name != original.Name {
		t.Errorf("Name = %q, want %q", unmarshaled.Name, original.Name)
	}
	if unmarshaled.Extends != original.Extends {
		t.Errorf("Extends = %q, want %q", unmarshaled.Extends, original.Extends)
	}
	if len(unmarshaled.Excludes) != len(original.Excludes) ||
		unmarshaled.Excludes[0] != original.Excludes[0] {
		t.Errorf(
			"Excludes = %v, want %v",
			unmarshaled.Excludes,
			original.Excludes,
		)
	}
	if len(unmarshaled.Properties) != len(original.Properties) {
		t.Errorf(
			"Properties length = %d, want %d",
			len(unmarshaled.Properties),
			len(original.Properties),
		)
	}
	if unmarshaled.Properties[0].Name != original.Properties[0].Name {
		t.Errorf(
			"Property name = %q, want %q",
			unmarshaled.Properties[0].Name,
			original.Properties[0].Name,
		)
	}
}

func TestSchemaJSONOptionalFields(t *testing.T) {
	// Test schema without optional fields
	jsonData := `{
		"name": "minimal_schema",
		"properties": []
	}`

	var schema Schema
	err := json.Unmarshal([]byte(jsonData), &schema)
	if err != nil {
		t.Fatalf("Unmarshal failed: %v", err)
	}

	if schema.Extends != "" {
		t.Errorf("Extends = %q, want empty", schema.Extends)
	}
	if len(schema.Excludes) != 0 {
		t.Errorf("Excludes length = %d, want 0", len(schema.Excludes))
	}

	// Test marshaling back - optional fields should be omitted
	data, err := json.Marshal(schema)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}

	var result map[string]interface{}
	if errUnmarshal := json.Unmarshal(data, &result); errUnmarshal != nil {
		t.Fatalf("Unmarshal result failed: %v", errUnmarshal)
	}

	// extends and excludes should not be present
	if _, hasExtends := result["extends"]; hasExtends {
		t.Error("extends field should not be present when empty")
	}
	if _, hasExcludes := result["excludes"]; hasExcludes {
		t.Error("excludes field should not be present when empty")
	}
}
