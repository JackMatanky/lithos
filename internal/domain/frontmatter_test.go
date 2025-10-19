package domain

import (
	"testing"
)

func TestNewFrontmatter(t *testing.T) {
	tests := []struct {
		name     string
		fields   map[string]interface{}
		expected Frontmatter
	}{
		{
			name: "with fileClass",
			fields: map[string]interface{}{
				"fileClass": "contact",
				"name":      "John Doe",
				"email":     "john@example.com",
			},
			expected: Frontmatter{
				FileClass: "contact",
				Fields: map[string]interface{}{
					"fileClass": "contact",
					"name":      "John Doe",
					"email":     "john@example.com",
				},
			},
		},
		{
			name: "without fileClass",
			fields: map[string]interface{}{
				"title":  "My Note",
				"author": "Jane Smith",
			},
			expected: Frontmatter{
				FileClass: "",
				Fields: map[string]interface{}{
					"title":  "My Note",
					"author": "Jane Smith",
				},
			},
		},
		{
			name:   "empty fields",
			fields: map[string]interface{}{},
			expected: Frontmatter{
				FileClass: "",
				Fields:    map[string]interface{}{},
			},
		},
		{
			name:   "nil fields",
			fields: nil,
			expected: Frontmatter{
				FileClass: "",
				Fields:    nil,
			},
		},
		{
			name: "fileClass is not string",
			fields: map[string]interface{}{
				"fileClass": 123,
				"title":     "Note with invalid fileClass",
			},
			expected: Frontmatter{
				FileClass: "",
				Fields: map[string]interface{}{
					"fileClass": 123,
					"title":     "Note with invalid fileClass",
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := NewFrontmatter(tt.fields)

			if result.FileClass != tt.expected.FileClass {
				t.Errorf(
					"FileClass = %q, want %q",
					result.FileClass,
					tt.expected.FileClass,
				)
			}

			// Compare Fields map
			if len(result.Fields) != len(tt.expected.Fields) {
				t.Errorf(
					"Fields length = %d, want %d",
					len(result.Fields),
					len(tt.expected.Fields),
				)
				return
			}

			for key, expectedValue := range tt.expected.Fields {
				if actualValue, exists := result.Fields[key]; !exists {
					t.Errorf("Fields[%q] missing", key)
				} else if actualValue != expectedValue {
					t.Errorf("Fields[%q] = %v, want %v", key, actualValue, expectedValue)
				}
			}
		})
	}
}

func TestExtractFileClass(t *testing.T) {
	tests := []struct {
		name     string
		fields   map[string]interface{}
		expected string
	}{
		{
			name: "fileClass present and string",
			fields: map[string]interface{}{
				"fileClass": "contact",
				"name":      "John Doe",
			},
			expected: "contact",
		},
		{
			name: "fileClass present but not string",
			fields: map[string]interface{}{
				"fileClass": 123,
				"name":      "John Doe",
			},
			expected: "",
		},
		{
			name: "fileClass missing",
			fields: map[string]interface{}{
				"name":  "John Doe",
				"email": "john@example.com",
			},
			expected: "",
		},
		{
			name:     "empty fields",
			fields:   map[string]interface{}{},
			expected: "",
		},
		{
			name:     "nil fields",
			fields:   nil,
			expected: "",
		},
		{
			name: "fileClass is empty string",
			fields: map[string]interface{}{
				"fileClass": "",
				"title":     "Empty fileClass",
			},
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := extractFileClass(tt.fields)
			if result != tt.expected {
				t.Errorf(
					"extractFileClass() = %q, want %q",
					result,
					tt.expected,
				)
			}
		})
	}
}

func TestFrontmatterSchemaName(t *testing.T) {
	tests := []struct {
		name      string
		fileClass string
		expected  string
	}{
		{
			name:      "with schema name",
			fileClass: "contact",
			expected:  "contact",
		},
		{
			name:      "empty schema name",
			fileClass: "",
			expected:  "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			f := Frontmatter{
				FileClass: tt.fileClass,
				Fields:    map[string]interface{}{},
			}

			result := f.SchemaName()
			if result != tt.expected {
				t.Errorf("SchemaName() = %q, want %q", result, tt.expected)
			}
		})
	}
}

func TestFrontmatterStruct(t *testing.T) {
	// Test that Frontmatter struct can be created and accessed
	fields := map[string]interface{}{
		"fileClass": "project",
		"title":     "Test Project",
		"status":    "active",
	}

	frontmatter := Frontmatter{
		FileClass: "project",
		Fields:    fields,
	}

	if frontmatter.FileClass != "project" {
		t.Errorf("FileClass = %q, want %q", frontmatter.FileClass, "project")
	}

	if len(frontmatter.Fields) != 3 {
		t.Errorf("Fields length = %d, want %d", len(frontmatter.Fields), 3)
	}

	if frontmatter.Fields["title"] != "Test Project" {
		t.Errorf(
			"Fields[title] = %v, want %v",
			frontmatter.Fields["title"],
			"Test Project",
		)
	}
}
