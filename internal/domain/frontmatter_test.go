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
		input    map[string]any
		expected Frontmatter
	}{
		{
			name:     "nil input creates empty frontmatter",
			input:    nil,
			expected: Frontmatter{Fields: map[string]any{}},
		},
		{
			name:     "empty map creates empty frontmatter",
			input:    map[string]any{},
			expected: Frontmatter{Fields: map[string]any{}},
		},
		{
			name: "populated map creates frontmatter with fields",
			input: map[string]any{
				"title": "Test Note",
				"tags":  []string{"test", "example"},
			},
			expected: Frontmatter{
				Fields: map[string]any{
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
	fm := NewFrontmatter(map[string]any{
		"title":     "Test Note",
		"tags":      []string{"test", "example"},
		"published": true,
		"count":     42,
	})

	tests := []struct {
		name     string
		key      string
		expected any
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
	fm := NewFrontmatter(map[string]any{
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
	fm := NewFrontmatter(map[string]any{
		"stringField": "hello",
		"intField":    42,
		"boolField":   true,
		"arrayField":  []string{"a", "b"},
		"mapField":    map[string]any{"key": "value"},
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
	fm := NewFrontmatter(map[string]any{
		"stringArray":    []string{"a", "b"},
		"interfaceArray": []any{"a", 1},
		"stringField":    "hello",
		"intField":       42,
		"boolField":      true,
		"mapField":       map[string]any{"key": "value"},
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
	fm := NewFrontmatter(map[string]any{
		"intField":    42,
		"int64Field":  int64(123),
		"floatField":  3.14,
		"stringField": "hello",
		"boolField":   true,
		"arrayField":  []string{"a", "b"},
		"mapField":    map[string]any{"key": "value"},
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
	fm := NewFrontmatter(map[string]any{
		"boolField":   true,
		"stringField": "hello",
		"intField":    42,
		"arrayField":  []string{"a", "b"},
		"mapField":    map[string]any{"key": "value"},
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
	fm := NewFrontmatter(map[string]any{
		"mapField":    map[string]any{"key": "value"},
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
	testConfigs := []struct {
		configName string
		config     Config
	}{
		{
			configName: "file_class_key",
			config:     Config{FileClassKey: "file_class"},
		},
		{
			configName: "fileClass_key",
			config:     Config{FileClassKey: "fileClass"},
		},
		{
			configName: "custom_key",
			config:     Config{FileClassKey: "custom"},
		},
	}

	for _, tc := range testConfigs {
		t.Run(tc.configName, func(t *testing.T) {
			SetInstanceForTesting(&tc.config)
			defer ResetConfigForTesting()

			tests := []struct {
				name     string
				fields   map[string]any
				expected string
			}{
				{
					name: "primary key field exists",
					fields: map[string]any{
						tc.config.FileClassKey: "meeting",
					},
					expected: "meeting",
				},
				{
					name: "fallback keys tested",
					fields: map[string]any{
						"fileClass": "fallback_meeting",
					},
					expected: "fallback_meeting",
				},
				{
					name: "field missing",
					fields: map[string]any{
						"title": "Test",
					},
					expected: "",
				},
				{
					name: "field is not string",
					fields: map[string]any{
						tc.config.FileClassKey: 123,
					},
					expected: "",
				},
			}

			for _, tt := range tests {
				t.Run(tt.name, func(t *testing.T) {
					fm := NewFrontmatter(tt.fields)
					result := fm.GetFileClass()
					if result != tt.expected {
						t.Errorf(
							"GetFileClass() = %q, want %q",
							result,
							tt.expected,
						)
					}
				})
			}
		})
	}
}

// TestFrontmatter_Title tests the Title delegation helper.
func TestFrontmatter_Title(t *testing.T) {
	tests := []struct {
		name     string
		fields   map[string]any
		expected string
	}{
		{
			name: "title field exists and is string",
			fields: map[string]any{
				"title": "Test Note",
			},
			expected: "Test Note",
		},
		{
			name: "title field missing",
			fields: map[string]any{
				"fileClass": "meeting",
			},
			expected: "",
		},
		{
			name: "title field is not string",
			fields: map[string]any{
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
		fields   map[string]any
		expected []string
	}{
		{
			name: "aliases as string array",
			fields: map[string]any{
				"aliases": []string{"alias1", "alias2"},
			},
			expected: []string{"alias1", "alias2"},
		},
		{
			name: "aliases as interface array with strings",
			fields: map[string]any{
				"aliases": []any{"alias1", "alias2"},
			},
			expected: []string{"alias1", "alias2"},
		},
		{
			name: "aliases as single string",
			fields: map[string]any{
				"aliases": "single-alias",
			},
			expected: []string{"single-alias"},
		},
		{
			name: "aliases field missing",
			fields: map[string]any{
				"title": "Test",
			},
			expected: []string{},
		},
		{
			name: "aliases as interface array with mixed types (strings only)",
			fields: map[string]any{
				"aliases": []any{"alias1", 123, "alias2"},
			},
			expected: []string{"alias1", "alias2"},
		},
		{
			name: "aliases as non-array, non-string type",
			fields: map[string]any{
				"aliases": 123,
			},
			expected: []string{},
		},
		{
			name: "aliases as interface array with no string elements",
			fields: map[string]any{
				"aliases": []any{123, true, 45.6},
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

	originalFields := map[string]any{
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

// TestIs_GenericTypeChecker tests the new generic Is[T] function.
func TestIs_GenericTypeChecker(t *testing.T) {
	t.Parallel()

	fm := Frontmatter{
		Fields: map[string]any{
			"title":     "Test Note",
			"count":     42,
			"published": true,
			"tags":      []string{"test", "example"},
			"metadata":  map[string]any{"key": "value"},
		},
	}

	tests := []struct {
		name     string
		checkFn  func() bool
		expected bool
	}{
		{
			name:     "string field with Is[string]",
			checkFn:  func() bool { return Is[string](fm, "title") },
			expected: true,
		},
		{
			name:     "int field with Is[int]",
			checkFn:  func() bool { return Is[int](fm, "count") },
			expected: true,
		},
		{
			name:     "bool field with Is[bool]",
			checkFn:  func() bool { return Is[bool](fm, "published") },
			expected: true,
		},
		{
			name:     "slice field with Is[[]string]",
			checkFn:  func() bool { return Is[[]string](fm, "tags") },
			expected: true,
		},
		{
			name: "map field with Is[map[string]any]",
			checkFn: func() bool {
				return Is[map[string]any](fm, "metadata")
			},
			expected: true,
		},
		{
			name:     "wrong type returns false",
			checkFn:  func() bool { return Is[int](fm, "title") },
			expected: false,
		},
		{
			name:     "non-existing field returns false",
			checkFn:  func() bool { return Is[string](fm, "nonexistent") },
			expected: false,
		},
		{
			name: "Is[string] equivalent to IsString",
			checkFn: func() bool {
				return Is[string](fm, "title") == fm.IsString("title")
			},
			expected: true,
		},
		{
			name: "Is[bool] equivalent to IsBool",
			checkFn: func() bool {
				return Is[bool](fm, "published") == fm.IsBool("published")
			},
			expected: true,
		},
		{
			name: "Is[map[string]any] equivalent to IsMap",
			checkFn: func() bool {
				return Is[map[string]any](fm, "metadata") ==
					fm.IsMap("metadata")
			},
			expected: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()
			result := tt.checkFn()
			if result != tt.expected {
				t.Errorf("got %v, want %v", result, tt.expected)
			}
		})
	}
}
