package domain

import (
	"testing"
)

const (
	testSchemaName        = "test_schema"
	testInheritedProperty = "inherited"
)

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

func TestNewSchemaWithExtends(t *testing.T) {
	props := []Property{
		NewProperty("title", true, false, StringPropertySpec{}),
		NewProperty("count", false, false, NumberPropertySpec{}),
	}
	excludes := []string{"old_prop"}

	schema := NewSchemaWithExtends(
		"child_schema",
		"base_schema",
		excludes,
		props,
	)

	if schema.Name != "child_schema" {
		t.Errorf("Name = %q, want %q", schema.Name, "child_schema")
	}
	if schema.Extends != "base_schema" {
		t.Errorf("Extends = %q, want %q", schema.Extends, "base_schema")
	}
	if len(schema.Excludes) != 1 || schema.Excludes[0] != "old_prop" {
		t.Errorf("Excludes = %v, want [old_prop]", schema.Excludes)
	}
	if len(schema.Properties) != 2 {
		t.Errorf("Properties length = %d, want 2", len(schema.Properties))
	}
	if schema.ResolvedProperties != nil {
		t.Errorf(
			"ResolvedProperties should be nil initially, got %v",
			schema.ResolvedProperties,
		)
	}
}

func TestSchemaSetResolvedProperties(t *testing.T) {
	schema := NewSchema("test_schema", []Property{
		NewProperty("original", true, false, StringPropertySpec{}),
	})

	resolvedProps := []Property{
		NewProperty("inherited", false, false, StringPropertySpec{}),
		NewProperty("original", true, false, StringPropertySpec{}),
	}

	schema.SetResolvedProperties(resolvedProps)

	if len(schema.ResolvedProperties) != 2 {
		t.Errorf(
			"ResolvedProperties length = %d, want 2",
			len(schema.ResolvedProperties),
		)
	}
	if schema.ResolvedProperties[0].Name != testInheritedProperty {
		t.Errorf(
			"First resolved property name = %q, want %q",
			schema.ResolvedProperties[0].Name,
			testInheritedProperty,
		)
	}
}

func TestSchemaGetResolvedProperties(t *testing.T) {
	originalProps := []Property{
		NewProperty("original", true, false, StringPropertySpec{}),
	}
	schema := NewSchema("test_schema", originalProps)

	// When ResolvedProperties is nil, should return Properties
	resolved := schema.GetResolvedProperties()
	if len(resolved) != 1 {
		t.Errorf("GetResolvedProperties length = %d, want 1", len(resolved))
	}
	if resolved[0].Name != "original" {
		t.Errorf(
			"GetResolvedProperties[0].Name = %q, want %q",
			resolved[0].Name,
			"original",
		)
	}

	// When ResolvedProperties is set, should return ResolvedProperties
	resolvedProps := []Property{
		NewProperty("inherited", false, false, StringPropertySpec{}),
		NewProperty("original", true, false, StringPropertySpec{}),
	}
	schema.SetResolvedProperties(resolvedProps)

	resolved = schema.GetResolvedProperties()
	if len(resolved) != 2 {
		t.Errorf("GetResolvedProperties length = %d, want 2", len(resolved))
	}
	if resolved[0].Name != "inherited" {
		t.Errorf(
			"GetResolvedProperties[0].Name = %q, want %q",
			resolved[0].Name,
			"inherited",
		)
	}
}

func TestSchemaConstructorBehavior(t *testing.T) {
	tests := []struct {
		name          string
		constructor   func() Schema
		expectedName  string
		expectedProps int
	}{
		{
			name: "NewSchema creates simple schema",
			constructor: func() Schema {
				return NewSchema("simple", []Property{
					NewProperty("title", true, false, StringPropertySpec{}),
				})
			},
			expectedName:  "simple",
			expectedProps: 1,
		},
		{
			name: "NewSchemaWithExtends creates extended schema",
			constructor: func() Schema {
				return NewSchemaWithExtends(
					"extended",
					"base",
					[]string{"old"},
					[]Property{
						NewProperty(
							"new_prop",
							false,
							false,
							StringPropertySpec{},
						),
					},
				)
			},
			expectedName:  "extended",
			expectedProps: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			schema := tt.constructor()
			if schema.Name != tt.expectedName {
				t.Errorf("Name = %q, want %q", schema.Name, tt.expectedName)
			}
			if len(schema.Properties) != tt.expectedProps {
				t.Errorf(
					"Properties length = %d, want %d",
					len(schema.Properties),
					tt.expectedProps,
				)
			}
		})
	}
}
