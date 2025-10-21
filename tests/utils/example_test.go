package testutils

import (
	"testing"
)

// ExampleTestDataPaths demonstrates how to use the new directory-based
// TestDataPaths.
func ExampleTestDataPaths() {
	paths := NewTestDataPaths()

	// Load different files from the same directory
	completeUserPath := paths.File(paths.SchemaValid, "complete-user.json")
	noteSchemaPath := paths.File(paths.SchemaValid, "note.json")

	// Load files from properties directory
	bankPath := paths.File(paths.SchemaProperties, "bank.json")
	stringPath := paths.File(paths.SchemaProperties, "string.json")

	// Load invalid schemas
	malformedPath := paths.File(paths.SchemaInvalid, "malformed.json")

	// Use with LoadTestData function
	_, _ = LoadTestData(completeUserPath)
	_, _ = LoadTestData(bankPath)
	_, _ = LoadTestData(malformedPath)

	// Use with LoadSchemaTestData (strips the "schema/" prefix automatically)
	_, _ = LoadSchemaTestData(paths.File("valid", "complete-user.json"))
	_, _ = LoadSchemaTestData(paths.File("properties", "bank.json"))

	// Print paths to show the structure
	_ = completeUserPath // "schema/valid/complete-user.json"
	_ = noteSchemaPath   // "schema/valid/note.json"
	_ = bankPath         // "schema/properties/bank.json"
	_ = stringPath       // "schema/properties/string.json"
	_ = malformedPath    // "schema/invalid/malformed.json"
}

// TestExampleUsage shows how the new structure is less brittle.
func TestExampleUsage(t *testing.T) {
	paths := NewTestDataPaths()

	// Adding a new file to an existing directory is easy
	newFilePath := paths.File(paths.SchemaValid, "new-schema.json")
	expected := "schema/valid/new-schema.json"

	if newFilePath != expected {
		t.Errorf("Got %s, want %s", newFilePath, expected)
	}

	// No need to add new constants for every file
	// Just specify the directory and filename dynamically
	anotherFile := paths.File(paths.SchemaProperties, "dynamic-property.json")
	expectedAnother := "schema/properties/dynamic-property.json"

	if anotherFile != expectedAnother {
		t.Errorf("Got %s, want %s", anotherFile, expectedAnother)
	}
}
