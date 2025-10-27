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
