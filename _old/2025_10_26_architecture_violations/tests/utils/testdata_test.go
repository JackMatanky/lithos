package testutils

import (
	"path/filepath"
	"strings"
	"testing"
)

func TestLoadTestData(t *testing.T) {
	// Test loading schema test data
	data, err := LoadTestData("schema/valid/complete-user.json")
	if err != nil {
		t.Fatalf("Failed to load test data: %v", err)
	}

	if !strings.Contains(data, `"name": "user"`) {
		t.Error("Test data does not contain expected content")
	}
}

func TestLoadSchemaTestData(t *testing.T) {
	// Test the convenience wrapper
	data, err := LoadSchemaTestData("valid/complete-user.json")
	if err != nil {
		t.Fatalf("Failed to load schema test data: %v", err)
	}

	if !strings.Contains(data, `"name": "user"`) {
		t.Error("Schema test data does not contain expected content")
	}
}

func TestGetTestDataPath(t *testing.T) {
	path, err := GetTestDataPath("schema/valid/complete-user.json")
	if err != nil {
		t.Fatalf("Failed to get test data path: %v", err)
	}

	if !strings.HasSuffix(
		path,
		filepath.Join("testdata", "schema", "valid", "complete-user.json"),
	) {
		t.Errorf("Path does not end with expected suffix: %s", path)
	}
}

func TestGetSchemaTestDataPath(t *testing.T) {
	path, err := GetSchemaTestDataPath("valid/complete-user.json")
	if err != nil {
		t.Fatalf("Failed to get schema test data path: %v", err)
	}

	if !strings.HasSuffix(
		path,
		filepath.Join("testdata", "schema", "valid", "complete-user.json"),
	) {
		t.Errorf("Path does not end with expected suffix: %s", path)
	}
}

func TestNewTestDataPaths(t *testing.T) {
	paths := NewTestDataPaths()

	// Test that directory paths are properly initialized
	if paths.SchemaValid != "schema/valid" {
		t.Errorf("SchemaValid = %s, want %s", paths.SchemaValid, "schema/valid")
	}

	if paths.SchemaProperties != "schema/properties" {
		t.Errorf(
			"SchemaProperties = %s, want %s",
			paths.SchemaProperties,
			"schema/properties",
		)
	}

	if paths.SchemaInvalid != "schema/invalid" {
		t.Errorf(
			"SchemaInvalid = %s, want %s",
			paths.SchemaInvalid,
			"schema/invalid",
		)
	}

	if paths.Golden != "golden" {
		t.Errorf("Golden = %s, want %s", paths.Golden, "golden")
	}
}

func TestTestDataPathsFileMethod(t *testing.T) {
	paths := NewTestDataPaths()

	// Test the File method
	completeUserPath := paths.File(paths.SchemaValid, "complete-user.json")
	expected := filepath.Join(paths.SchemaValid, "complete-user.json")
	if completeUserPath != expected {
		t.Errorf("File method returned %s, want %s", completeUserPath, expected)
	}

	// Test with properties
	bankPath := paths.File(paths.SchemaProperties, "bank.json")
	expectedBank := filepath.Join(paths.SchemaProperties, "bank.json")
	if bankPath != expectedBank {
		t.Errorf("File method returned %s, want %s", bankPath, expectedBank)
	}
}

func TestTestDataPathsWithLoaders(t *testing.T) {
	paths := NewTestDataPaths()

	// Test that paths work with the loader functions
	completeUserPath := paths.File(paths.SchemaValid, "complete-user.json")
	data, err := LoadTestData(completeUserPath)
	if err != nil {
		t.Fatalf("Failed to load test data using paths: %v", err)
	}

	if !strings.Contains(data, `"name": "user"`) {
		t.Error("Loaded data does not contain expected content")
	}

	// Test with LoadSchemaTestData convenience function
	data2, err := LoadSchemaTestData(paths.File("valid", "complete-user.json"))
	if err != nil {
		t.Fatalf("Failed to load schema test data using paths: %v", err)
	}

	if !strings.Contains(data2, `"name": "user"`) {
		t.Error("Schema test data does not contain expected content")
	}
}
