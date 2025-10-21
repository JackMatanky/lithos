// Package schema provides filesystem-based implementations of schema loading
// ports for hexagonal architecture.
//
// This file contains shared test helpers and mock implementations used
// across multiple test files in the schema package.
package schema

import (
	"errors"
	"path/filepath"
	"strings"

	"github.com/JackMatanky/lithos/internal/adapters/spi/config"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// Mock implementations for testing

type mockFileSystemPort struct {
	files     map[string][]byte
	walkPaths []string
	walkError error
	readError error
}

func newMockFileSystemPort() *mockFileSystemPort {
	return &mockFileSystemPort{
		files:     make(map[string][]byte),
		walkPaths: []string{},
		walkError: nil,
		readError: nil,
	}
}

func (m *mockFileSystemPort) ReadFile(path string) ([]byte, error) {
	if m.readError != nil {
		return nil, m.readError
	}

	if data, exists := m.files[path]; exists {
		return data, nil
	}
	return nil, errors.New("file not found")
}

func (m *mockFileSystemPort) WriteFileAtomic(path string, data []byte) error {
	m.files[path] = data
	return nil
}

func (m *mockFileSystemPort) Walk(root string, fn spi.WalkFunc) error {
	if m.walkError != nil {
		return m.walkError
	}

	for _, path := range m.walkPaths {
		isDir := !strings.HasSuffix(path, ".json")
		if err := fn(path, isDir); err != nil {
			return err
		}
	}
	return nil
}

func (m *mockFileSystemPort) addFile(path string, content []byte) {
	m.files[path] = content
}

func (m *mockFileSystemPort) addWalkPath(path string) {
	m.walkPaths = append(m.walkPaths, path)
}

type mockConfigPort struct {
	cfg *config.Config
}

func newMockConfigPort(vaultPath string) *mockConfigPort {
	cfg := config.NewConfig(vaultPath)
	return &mockConfigPort{cfg: cfg}
}

func (m *mockConfigPort) Config() *config.Config {
	return m.cfg
}

// Test data constants

const validSchemaJSON = `{
	"name": "user",
	"extends": "base",
	"excludes": ["internal_id"],
	"properties": {
		"email": {
			"type": "string",
			"required": true,
			"pattern": "^[\\w\\.-]+@[\\w\\.-]+\\.[A-Za-z]{2,}$"
		},
		"age": {
			"type": "number",
			"required": false,
			"min": 0,
			"max": 150
		},
		"active": {
			"type": "bool",
			"required": true
		}
	}
}`

const validPropertyBankJSON = `{
	"properties": {
		"common-email": {
			"type": "string",
			"required": true,
			"pattern": "^[\\w\\.-]+@[\\w\\.-]+\\.[A-Za-z]{2,}$"
		},
		"user-profile": {
			"type": "string",
			"required": false
		}
	}
}`

const malformedSchemaJSON = `{
	"name": "user",
	"properties": {
		"email": {
			"type": "string"
		}
	// missing closing brace
}`

// Helper function to create adapter with mocks.
func createTestAdapter() (*SchemaLoaderAdapter, *mockFileSystemPort, *mockConfigPort) {
	fs := newMockFileSystemPort()
	cfg := newMockConfigPort("/test/vault")
	adapter := NewSchemaLoaderAdapter(fs, cfg)
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
	fs.addFile(schemaPath, []byte(content))
	fs.addWalkPath(schemaPath)
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
	fs.addFile(propPath, []byte(content))
	fs.addWalkPath(propPath)
	return propPath
}
