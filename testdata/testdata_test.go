package testdata_test

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/goccy/go-yaml"
)

func TestTestDataStructureExists(t *testing.T) {
	tests := []struct {
		name string
		path string
	}{
		{"testdata directory", "."},
		{"vault directory", "vault"},
		{"golden directory", "golden"},
		{"templates directory", "vault/templates"},
		{"schemas directory", "vault/schemas"},
		{"notes directory", "vault/notes"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if _, err := os.Stat(tt.path); os.IsNotExist(err) {
				t.Errorf("Directory %s does not exist", tt.path)
			}
		})
	}
}

func TestSampleVaultFilesExist(t *testing.T) {
	tests := []struct {
		name string
		path string
	}{
		{"basic template", "vault/templates/basic-note.md"},
		{"note schema", "vault/schemas/note.json"},
		{"sample note 1", "vault/notes/sample-note-1.md"},
		{"sample note 2", "vault/notes/sample-note-2.md"},
		{"golden output", "golden/basic-note-expected.md"},
		{"golden input params", "golden/basic-note-input-params.json"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if _, err := os.Stat(tt.path); os.IsNotExist(err) {
				t.Errorf("File %s does not exist", tt.path)
			}
		})
	}
}

func TestTemplateFileValidSyntax(t *testing.T) {
	templatePath := "vault/templates/basic-note.md"

	content, err := os.ReadFile(templatePath)
	if err != nil {
		t.Fatalf("Failed to read template file: %v", err)
	}

	templateStr := string(content)

	// Verify template has expected template functions (as strings, not parsing)
	expectedFunctions := []string{"now", "toLower", "toUpper"}
	for _, fn := range expectedFunctions {
		if !strings.Contains(templateStr, fn) {
			t.Errorf("Template does not contain expected function: %s", fn)
		}
	}

	// Verify template has expected variable placeholders
	expectedVars := []string{
		".title", ".author", ".content", ".project", ".tags",
	}
	for _, v := range expectedVars {
		if !strings.Contains(templateStr, v) {
			t.Errorf("Template does not contain expected variable: %s", v)
		}
	}

	// Test basic template syntax (without custom functions) by checking for proper delimiters
	openBraces := strings.Count(templateStr, "{{")
	closeBraces := strings.Count(templateStr, "}}")
	if openBraces != closeBraces {
		t.Errorf("Template has mismatched braces: %d open, %d close", openBraces, closeBraces)
	}
}

func TestSchemaFileValidJSON(t *testing.T) {
	schemaPath := "vault/schemas/note.json"

	content, err := os.ReadFile(schemaPath)
	if err != nil {
		t.Fatalf("Failed to read schema file: %v", err)
	}

	// Test that schema is valid JSON
	var schema map[string]interface{}
	if unmarshalErr := json.Unmarshal(content, &schema); unmarshalErr != nil {
		t.Errorf("Schema JSON is invalid: %v", unmarshalErr)
	}

	// Verify schema has required structure
	if name, ok := schema["name"]; !ok || name != "note" {
		t.Errorf("Schema missing or incorrect 'name' field")
	}

	properties, hasProperties := schema["properties"]
	if !hasProperties {
		t.Errorf("Schema missing 'properties' field")
		return
	}

	props, isMap := properties.(map[string]interface{})
	if !isMap {
		t.Errorf("Schema 'properties' field is not an object")
		return
	}

	// Verify required properties exist
	requiredProps := []string{"title", "author", "content"}
	for _, prop := range requiredProps {
		if _, exists := props[prop]; !exists {
			t.Errorf("Schema missing required property: %s", prop)
		}
	}
}

func TestSampleNotesValidFrontmatter(t *testing.T) {
	noteFiles := []string{
		"vault/notes/sample-note-1.md",
		"vault/notes/sample-note-2.md",
	}

	for _, noteFile := range noteFiles {
		t.Run(filepath.Base(noteFile), func(t *testing.T) {
			validateNoteFile(t, noteFile)
		})
	}
}

func validateNoteFile(t *testing.T, noteFile string) {
	t.Helper()

	content, err := os.ReadFile(noteFile)
	if err != nil {
		t.Fatalf("Failed to read note file: %v", err)
	}

	contentStr := string(content)

	// Check for frontmatter delimiters
	if !strings.HasPrefix(contentStr, "---\n") {
		t.Errorf("Note file does not start with frontmatter delimiter")
	}

	// Extract frontmatter
	parts := strings.Split(contentStr, "---\n")
	if len(parts) < 3 {
		t.Errorf("Note file does not have proper frontmatter structure")
		return
	}

	frontmatter := parts[1]

	// Test that frontmatter is valid YAML
	var fm map[string]interface{}
	if yamlErr := yaml.Unmarshal([]byte(frontmatter), &fm); yamlErr != nil {
		t.Errorf("Frontmatter YAML is invalid: %v", yamlErr)
	}

	// Verify required frontmatter fields
	requiredFields := []string{"fileClass", "title", "author", "content"}
	for _, field := range requiredFields {
		if _, hasField := fm[field]; !hasField {
			t.Errorf("Frontmatter missing required field: %s", field)
		}
	}

	// Verify fileClass is "note"
	if fileClass, hasFileClass := fm["fileClass"]; !hasFileClass || fileClass != "note" {
		t.Errorf("Frontmatter fileClass should be 'note', got: %v", fileClass)
	}
}

func TestGoldenFileAccessible(t *testing.T) {
	goldenPath := "golden/basic-note-expected.md"

	content, err := os.ReadFile(goldenPath)
	if err != nil {
		t.Fatalf("Failed to read golden file: %v", err)
	}

	// Verify golden file has expected structure
	contentStr := string(content)

	// Should start with a title
	if !strings.HasPrefix(contentStr, "# ") {
		t.Errorf("Golden file does not start with expected title format")
	}

	// Should contain expected sections
	expectedSections := []string{"Created:", "Author:", "Tags:", "## Content", "## Notes"}
	for _, section := range expectedSections {
		if !strings.Contains(contentStr, section) {
			t.Errorf("Golden file missing expected section: %s", section)
		}
	}
}

func TestGoldenInputParamsValidJSON(t *testing.T) {
	paramsPath := "golden/basic-note-input-params.json"

	content, err := os.ReadFile(paramsPath)
	if err != nil {
		t.Fatalf("Failed to read golden input params file: %v", err)
	}

	// Test that params file is valid JSON
	var params map[string]interface{}
	if unmarshalErr := json.Unmarshal(content, &params); unmarshalErr != nil {
		t.Errorf("Golden input params JSON is invalid: %v", unmarshalErr)
	}

	// Verify required structure
	requiredFields := []string{"description", "template", "parameters"}
	for _, field := range requiredFields {
		if _, ok := params[field]; !ok {
			t.Errorf("Golden input params missing required field: %s", field)
		}
	}
}

func TestCrossplatformPaths(t *testing.T) {
	// Test that all paths use filepath.Join for cross-platform compatibility
	testPaths := []string{
		".",
		filepath.Join("vault"),
		filepath.Join("vault", "templates"),
		filepath.Join("vault", "schemas"),
		filepath.Join("vault", "notes"),
		filepath.Join("golden"),
	}

	for _, testPath := range testPaths {
		t.Run(testPath, func(t *testing.T) {
			if _, err := os.Stat(testPath); os.IsNotExist(err) {
				t.Errorf("Cross-platform path %s does not exist", testPath)
			}
		})
	}
}
