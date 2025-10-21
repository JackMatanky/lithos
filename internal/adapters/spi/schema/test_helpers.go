// Package schema provides filesystem-based implementations of schema loading
// ports for hexagonal architecture.
//
// This file contains test helpers specific to the schema adapter tests.
// Shared test utilities are available in tests/utils package.
package schema

import (
	"path/filepath"

	testutils "github.com/JackMatanky/lithos/tests/utils"
)

// Mock implementations for testing (using shared test utilities)

type mockFileSystemPort = testutils.MockFileSystemPort
type mockConfigPort = testutils.MockConfigPort

func newMockFileSystemPort() *testutils.MockFileSystemPort {
	return testutils.NewMockFileSystemPort()
}

func newMockConfigPort(vaultPath string) *testutils.MockConfigPort {
	return testutils.NewMockConfigPort(vaultPath)
}

// Helper function to create adapter with mocks.
func createTestAdapter() (
	adapter *SchemaLoaderAdapter,
	fs *mockFileSystemPort,
	cfg *mockConfigPort,
) {
	fs = newMockFileSystemPort()
	cfg = newMockConfigPort("/test/vault")
	adapter = NewSchemaLoaderAdapter(fs, cfg)
	return adapter, fs, cfg
}

// Helper function to setup schema file.
//
//nolint:unparam // Return value may be useful for debugging in future tests
func setupSchemaFile(
	fs *mockFileSystemPort,
	cfg *mockConfigPort,
	filename, content string,
) string {
	schemaPath := filepath.Join(cfg.Config().SchemasDir, filename)
	fs.AddFile(schemaPath, []byte(content))
	fs.AddWalkPath(schemaPath)
	return schemaPath
}

// Helper function to setup property bank file.
//
//nolint:unparam // Return value may be useful for debugging in future tests
func setupPropertyBankFile(
	fs *mockFileSystemPort,
	cfg *mockConfigPort,
	filename, content string,
) string {
	propPath := filepath.Join(cfg.Config().SchemasDir, "properties", filename)
	fs.AddFile(propPath, []byte(content))
	fs.AddWalkPath(propPath)
	return propPath
}

// loadTestData is a convenience wrapper for the shared utility.
func loadTestData(filename string) (string, error) {
	return testutils.LoadSchemaTestData(filename)
}
