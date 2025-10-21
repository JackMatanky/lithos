// Package testutils provides shared test utilities and testdata paths
// for consistent access to test resources across the lithos project.
package testutils

import (
	"errors"
	"os"
	"path/filepath"
	"runtime"
)

// TestDataPaths provides constants for commonly used test data directory paths.
// Use the File() method to append specific filenames to directory paths.
type TestDataPaths struct {
	// Schema test data directories
	SchemaValid      string // schema/valid/
	SchemaInvalid    string // schema/invalid/
	SchemaProperties string // schema/properties/

	// Other test data directories
	Golden    string // golden/
	Templates string // templates/
	Notes     string // notes/
}

// NewTestDataPaths creates a TestDataPaths instance with all directory paths
// initialized.
func NewTestDataPaths() *TestDataPaths {
	return &TestDataPaths{
		// Schema directories
		SchemaValid:      "schema/valid",
		SchemaInvalid:    "schema/invalid",
		SchemaProperties: "schema/properties",

		// Other directories
		Golden:    "golden",
		Templates: "templates",
		Notes:     "notes",
	}
}

// File appends a filename to a directory path, returning the complete relative
// path.
func (p *TestDataPaths) File(directory, filename string) string {
	return filepath.Join(directory, filename)
}

// LoadTestData loads test data from a file in the testdata directory.
// The filename should be relative to the testdata root (e.g.,
// "schema/valid/user.json").
func LoadTestData(filename string) (string, error) {
	// Find the testdata directory relative to this file
	_, thisFile, _, ok := runtime.Caller(0)
	if !ok {
		return "", errors.New("failed to get caller information")
	}
	testdataDir := filepath.Join(filepath.Dir(thisFile), "..", "..", "testdata")
	path := filepath.Join(testdataDir, filename)
	data, err := os.ReadFile(path) // #nosec G304 - testdata files are safe
	if err != nil {
		return "", err
	}
	return string(data), nil
}

// LoadSchemaTestData loads test data from the testdata/schema directory.
// The filename should be relative to testdata/schema (e.g., "valid/user.json").
func LoadSchemaTestData(filename string) (string, error) {
	return LoadTestData(filepath.Join("schema", filename))
}

// GetTestDataPath returns the absolute path to a testdata file.
func GetTestDataPath(filename string) (string, error) {
	_, thisFile, _, ok := runtime.Caller(0)
	if !ok {
		return "", errors.New("failed to get caller information")
	}
	testdataDir := filepath.Join(filepath.Dir(thisFile), "..", "..", "testdata")
	path := filepath.Join(testdataDir, filename)
	absPath, err := filepath.Abs(path)
	if err != nil {
		return "", err
	}
	return absPath, nil
}

// GetSchemaTestDataPath returns the absolute path to a schema testdata file.
func GetSchemaTestDataPath(filename string) (string, error) {
	return GetTestDataPath(filepath.Join("schema", filename))
}
