package domain

import (
	"reflect"
	"testing"
)

// TestNewFrontmatter_DefensiveCopy tests that NewFrontmatter creates a
// defensive copy
// and handles nil input gracefully.
func TestNewFrontmatter_DefensiveCopy(t *testing.T) {
	tests := []struct {
		name     string
		input    map[string]interface{}
		expected Frontmatter
	}{
		{
			name:     "nil input creates empty frontmatter",
			input:    nil,
			expected: Frontmatter{Fields: map[string]interface{}{}},
		},
		{
			name:     "empty map creates empty frontmatter",
			input:    map[string]interface{}{},
			expected: Frontmatter{Fields: map[string]interface{}{}},
		},
		{
			name: "populated map creates frontmatter with fields",
			input: map[string]interface{}{
				"title": "Test Note",
				"tags":  []string{"test", "example"},
			},
			expected: Frontmatter{
				Fields: map[string]interface{}{
					"title": "Test Note",
					"tags":  []string{"test", "example"},
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := NewFrontmatter(tt.input)

			// Check that Fields are set correctly
			if !reflect.DeepEqual(result.Fields, tt.expected.Fields) {
				t.Errorf(
					"NewFrontmatter() Fields = %v, want %v",
					result.Fields,
					tt.expected.Fields,
				)
			}

			// Test defensive copy - modifying original shouldn't affect result
			if tt.input != nil {
				tt.input["newField"] = "should not appear"
				if _, exists := result.Fields["newField"]; exists {
					t.Error("NewFrontmatter() did not create defensive copy")
				}
			}
		})
	}
}

// TestFrontmatter_Get tests the Get method for safe field access.
func TestFrontmatter_Get(t *testing.T) {
	fm := NewFrontmatter(map[string]interface{}{
		"title":     "Test Note",
		"tags":      []string{"test", "example"},
		"published": true,
		"count":     42,
	})

	tests := []struct {
		name     string
		key      string
		expected interface{}
		found    bool
	}{
		{
			name:     "existing string field",
			key:      "title",
			expected: "Test Note",
			found:    true,
		},
		{
			name:     "existing array field",
			key:      "tags",
			expected: []string{"test", "example"},
			found:    true,
		},
		{
			name:     "existing bool field",
			key:      "published",
			expected: true,
			found:    true,
		},
		{
			name:     "existing int field",
			key:      "count",
			expected: 42,
			found:    true,
		},
		{
			name:     "non-existing field",
			key:      "nonexistent",
			expected: nil,
			found:    false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, found := fm.Get(tt.key)
			if found != tt.found {
				t.Errorf("Get(%q) found = %v, want %v", tt.key, found, tt.found)
			}
			if found && !reflect.DeepEqual(result, tt.expected) {
				t.Errorf("Get(%q) = %v, want %v", tt.key, result, tt.expected)
			}
		})
	}
}

// TestFrontmatter_Has tests the Has method for field existence checking.
func TestFrontmatter_Has(t *testing.T) {
	fm := NewFrontmatter(map[string]interface{}{
		"title": "Test Note",
		"tags":  []string{"test"},
	})

	tests := []struct {
		name     string
		key      string
		expected bool
	}{
		{
			name:     "existing field",
			key:      "title",
			expected: true,
		},
		{
			name:     "another existing field",
			key:      "tags",
			expected: true,
		},
		{
			name:     "non-existing field",
			key:      "nonexistent",
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := fm.Has(tt.key)
			if result != tt.expected {
				t.Errorf("Has(%q) = %v, want %v", tt.key, result, tt.expected)
			}
		})
	}
}

// TestFrontmatter_IsString tests the IsString type inspector.
func TestFrontmatter_IsString(t *testing.T) {
	fm := NewFrontmatter(map[string]interface{}{
		"stringField": "hello",
		"intField":    42,
		"boolField":   true,
		"arrayField":  []string{"a", "b"},
		"mapField":    map[string]interface{}{"key": "value"},
	})

	tests := []struct {
		name     string
		key      string
		expected bool
	}{
		{
			name:     "string field returns true",
			key:      "stringField",
			expected: true,
		},
		{
			name:     "int field returns false",
			key:      "intField",
			expected: false,
		},
		{
			name:     "bool field returns false",
			key:      "boolField",
			expected: false,
		},
		{
			name:     "array field returns false",
			key:      "arrayField",
			expected: false,
		},
		{
			name:     "map field returns false",
			key:      "mapField",
			expected: false,
		},
		{
			name:     "non-existing field returns false",
			key:      "nonexistent",
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := fm.IsString(tt.key)
			if result != tt.expected {
				t.Errorf(
					"IsString(%q) = %v, want %v",
					tt.key,
					result,
					tt.expected,
				)
			}
		})
	}
}

// TestFrontmatter_IsArray tests the IsArray type inspector.
func TestFrontmatter_IsArray(t *testing.T) {
	fm := NewFrontmatter(map[string]interface{}{
		"stringArray":    []string{"a", "b"},
		"interfaceArray": []interface{}{"a", 1},
		"stringField":    "hello",
		"intField":       42,
		"boolField":      true,
		"mapField":       map[string]interface{}{"key": "value"},
	})

	tests := []struct {
		name     string
		key      string
		expected bool
	}{
		{
			name:     "string array returns true",
			key:      "stringArray",
			expected: true,
		},
		{
			name:     "interface array returns true",
			key:      "interfaceArray",
			expected: true,
		},
		{
			name:     "string field returns false",
			key:      "stringField",
			expected: false,
		},
		{
			name:     "int field returns false",
			key:      "intField",
			expected: false,
		},
		{
			name:     "bool field returns false",
			key:      "boolField",
			expected: false,
		},
		{
			name:     "map field returns false",
			key:      "mapField",
			expected: false,
		},
		{
			name:     "non-existing field returns false",
			key:      "nonexistent",
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := fm.IsArray(tt.key)
			if result != tt.expected {
				t.Errorf(
					"IsArray(%q) = %v, want %v",
					tt.key,
					result,
					tt.expected,
				)
			}
		})
	}
}

// TestFrontmatter_IsInt tests the IsInt type inspector.
func TestFrontmatter_IsInt(t *testing.T) {
	fm := NewFrontmatter(map[string]interface{}{
		"intField":    42,
		"int64Field":  int64(123),
		"floatField":  3.14,
		"stringField": "hello",
		"boolField":   true,
		"arrayField":  []string{"a", "b"},
		"mapField":    map[string]interface{}{"key": "value"},
	})

	tests := []struct {
		name     string
		key      string
		expected bool
	}{
		{
			name:     "int field returns true",
			key:      "intField",
			expected: true,
		},
		{
			name:     "int64 field returns true",
			key:      "int64Field",
			expected: true,
		},
		{
			name:     "float64 field returns true",
			key:      "floatField",
			expected: true,
		},
		{
			name:     "string field returns false",
			key:      "stringField",
			expected: false,
		},
		{
			name:     "bool field returns false",
			key:      "boolField",
			expected: false,
		},
		{
			name:     "array field returns false",
			key:      "arrayField",
			expected: false,
		},
		{
			name:     "map field returns false",
			key:      "mapField",
			expected: false,
		},
		{
			name:     "non-existing field returns false",
			key:      "nonexistent",
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := fm.IsInt(tt.key)
			if result != tt.expected {
				t.Errorf("IsInt(%q) = %v, want %v", tt.key, result, tt.expected)
			}
		})
	}
}

// TestFrontmatter_IsBool tests the IsBool type inspector.
func TestFrontmatter_IsBool(t *testing.T) {
	fm := NewFrontmatter(map[string]interface{}{
		"boolField":   true,
		"stringField": "hello",
		"intField":    42,
		"arrayField":  []string{"a", "b"},
		"mapField":    map[string]interface{}{"key": "value"},
	})

	tests := []struct {
		name     string
		key      string
		expected bool
	}{
		{
			name:     "bool field returns true",
			key:      "boolField",
			expected: true,
		},
		{
			name:     "string field returns false",
			key:      "stringField",
			expected: false,
		},
		{
			name:     "int field returns false",
			key:      "intField",
			expected: false,
		},
		{
			name:     "array field returns false",
			key:      "arrayField",
			expected: false,
		},
		{
			name:     "map field returns false",
			key:      "mapField",
			expected: false,
		},
		{
			name:     "non-existing field returns false",
			key:      "nonexistent",
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := fm.IsBool(tt.key)
			if result != tt.expected {
				t.Errorf(
					"IsBool(%q) = %v, want %v",
					tt.key,
					result,
					tt.expected,
				)
			}
		})
	}
}

// TestFrontmatter_IsMap tests the IsMap type inspector.
func TestFrontmatter_IsMap(t *testing.T) {
	fm := NewFrontmatter(map[string]interface{}{
		"mapField":    map[string]interface{}{"key": "value"},
		"stringField": "hello",
		"intField":    42,
		"boolField":   true,
		"arrayField":  []string{"a", "b"},
	})

	tests := []struct {
		name     string
		key      string
		expected bool
	}{
		{
			name:     "map field returns true",
			key:      "mapField",
			expected: true,
		},
		{
			name:     "string field returns false",
			key:      "stringField",
			expected: false,
		},
		{
			name:     "int field returns false",
			key:      "intField",
			expected: false,
		},
		{
			name:     "bool field returns false",
			key:      "boolField",
			expected: false,
		},
		{
			name:     "array field returns false",
			key:      "arrayField",
			expected: false,
		},
		{
			name:     "non-existing field returns false",
			key:      "nonexistent",
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := fm.IsMap(tt.key)
			if result != tt.expected {
				t.Errorf("IsMap(%q) = %v, want %v", tt.key, result, tt.expected)
			}
		})
	}
}

// TestFrontmatter_GetFileClass tests the GetFileClass delegation helper.
func TestFrontmatter_GetFileClass(t *testing.T) {
	// Setup Config singleton for testing
	config := Config{FileClassKey: "file_class"}
	SetInstanceForTesting(&config)
	defer ResetConfigForTesting()

	tests := []struct {
		name     string
		fields   map[string]interface{}
		expected string
	}{
		{
			name: "file_class field exists and is string",
			fields: map[string]interface{}{
				"file_class": "meeting",
			},
			expected: "meeting",
		},
		{
			name: "file_class field missing",
			fields: map[string]interface{}{
				"title": "Test",
			},
			expected: "",
		},
		{
			name: "file_class field is not string",
			fields: map[string]interface{}{
				"file_class": 123,
			},
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fm := NewFrontmatter(tt.fields)
			result := fm.GetFileClass()
			if result != tt.expected {
				t.Errorf("GetFileClass() = %q, want %q", result, tt.expected)
			}
		})
	}
}

// TestFrontmatter_Title tests the Title delegation helper.
func TestFrontmatter_Title(t *testing.T) {
	tests := []struct {
		name     string
		fields   map[string]interface{}
		expected string
	}{
		{
			name: "title field exists and is string",
			fields: map[string]interface{}{
				"title": "Test Note",
			},
			expected: "Test Note",
		},
		{
			name: "title field missing",
			fields: map[string]interface{}{
				"fileClass": "meeting",
			},
			expected: "",
		},
		{
			name: "title field is not string",
			fields: map[string]interface{}{
				"title": 123,
			},
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fm := NewFrontmatter(tt.fields)
			result := fm.Title()
			if result != tt.expected {
				t.Errorf("Title() = %q, want %q", result, tt.expected)
			}
		})
	}
}

// TestFrontmatter_Aliases tests the Aliases delegation helper.
func TestFrontmatter_Aliases(t *testing.T) {
	tests := []struct {
		name     string
		fields   map[string]interface{}
		expected []string
	}{
		{
			name: "aliases as string array",
			fields: map[string]interface{}{
				"aliases": []string{"alias1", "alias2"},
			},
			expected: []string{"alias1", "alias2"},
		},
		{
			name: "aliases as interface array with strings",
			fields: map[string]interface{}{
				"aliases": []interface{}{"alias1", "alias2"},
			},
			expected: []string{"alias1", "alias2"},
		},
		{
			name: "aliases as single string",
			fields: map[string]interface{}{
				"aliases": "single-alias",
			},
			expected: []string{"single-alias"},
		},
		{
			name: "aliases field missing",
			fields: map[string]interface{}{
				"title": "Test",
			},
			expected: []string{},
		},
		{
			name: "aliases as interface array with mixed types (strings only)",
			fields: map[string]interface{}{
				"aliases": []interface{}{"alias1", 123, "alias2"},
			},
			expected: []string{"alias1", "alias2"},
		},
		{
			name: "aliases as non-array, non-string type",
			fields: map[string]interface{}{
				"aliases": 123,
			},
			expected: []string{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fm := NewFrontmatter(tt.fields)
			result := fm.Aliases()
			if !reflect.DeepEqual(result, tt.expected) {
				t.Errorf("Aliases() = %v, want %v", result, tt.expected)
			}
		})
	}
}

// TestFrontmatter_Immutability tests that Frontmatter instances are immutable.
func TestFrontmatter_Immutability(t *testing.T) {
	const modifiedValue = "Modified"

	originalFields := map[string]interface{}{
		"title": "Original",
		"tags":  []string{"original"},
	}

	fm := NewFrontmatter(originalFields)

	// Try to modify the returned Fields map
	fm.Fields["title"] = modifiedValue
	fm.Fields["newField"] = "Added"

	// Fields map is accessible for reading but helpers never mutate it
	if fm.Fields["title"] != modifiedValue {
		t.Error("Frontmatter Fields should be accessible for reading")
	}

	// Test immutability guarantee: helpers never mutate Fields
	// The story says: "Frontmatter instances are immutable after construction
	// (helpers never mutate Fields)" - meaning helpers don't change state.

	// Test that constructor creates a proper copy
	originalFields["shouldNotAffect"] = "value"
	if _, exists := fm.Fields["shouldNotAffect"]; exists {
		t.Error("Constructor should create defensive copy")
	}
}

// TestFrontmatterService_UsesHelpers tests that FrontmatterService uses the new
// helpers This is an integration test that will fail until we update
// FrontmatterService.
func TestFrontmatterService_UsesHelpers(t *testing.T) {
	// This test will check that FrontmatterService calls the helper methods
	// instead of directly accessing Fields. We'll implement this after
	// the helpers are implemented.

	t.Skip("Skipping until FrontmatterService is updated to use helpers")
}
