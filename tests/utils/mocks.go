// Package testutils provides shared test utilities and mock implementations
// used across multiple test packages in the lithos project.
package testutils

import (
	"errors"
	"strings"

	"github.com/JackMatanky/lithos/internal/adapters/spi/config"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// MockFileSystemPort implements spi.FileSystemPort for testing.
// It provides an in-memory filesystem with configurable error injection.
type MockFileSystemPort struct {
	files         map[string][]byte
	walkPaths     []string
	walkError     error
	readError     error
	readFileFunc  func(path string) ([]byte, error)
	writeFileFunc func(path string, data []byte) error
}

// NewMockFileSystemPort creates a new mock filesystem port.
func NewMockFileSystemPort() *MockFileSystemPort {
	return &MockFileSystemPort{
		files:         make(map[string][]byte),
		walkPaths:     []string{},
		walkError:     nil,
		readError:     nil,
		readFileFunc:  nil,
		writeFileFunc: nil,
	}
}

// ReadFile implements spi.FileSystemPort.ReadFile.
func (m *MockFileSystemPort) ReadFile(path string) ([]byte, error) {
	if m.readFileFunc != nil {
		return m.readFileFunc(path)
	}

	if m.readError != nil {
		return nil, m.readError
	}

	if data, exists := m.files[path]; exists {
		return data, nil
	}
	return nil, errors.New("file not found")
}

// WriteFile implements spi.FileSystemPort.WriteFile.
func (m *MockFileSystemPort) WriteFile(path string, data []byte) error {
	m.files[path] = data
	return nil
}

// WriteFileAtomic implements spi.FileSystemPort.WriteFileAtomic.
func (m *MockFileSystemPort) WriteFileAtomic(path string, data []byte) error {
	if m.writeFileFunc != nil {
		return m.writeFileFunc(path, data)
	}
	m.files[path] = data
	return nil
}

// Walk implements spi.FileSystemPort.Walk.
func (m *MockFileSystemPort) Walk(root string, fn spi.WalkFunc) error {
	if m.walkError != nil {
		return m.walkError
	}

	for _, path := range m.walkPaths {
		isDir := !strings.HasSuffix(path, ".json") &&
			!strings.HasSuffix(path, ".md")
		if err := fn(path, isDir); err != nil {
			return err
		}
	}
	return nil
}

// AddFile adds a file to the mock filesystem.
func (m *MockFileSystemPort) AddFile(path string, content []byte) {
	m.files[path] = content
}

// AddWalkPath adds a path to be returned during Walk operations.
func (m *MockFileSystemPort) AddWalkPath(path string) {
	m.walkPaths = append(m.walkPaths, path)
}

// SetWalkError sets an error to be returned by Walk operations.
func (m *MockFileSystemPort) SetWalkError(err error) {
	m.walkError = err
}

// SetReadError sets an error to be returned by ReadFile operations.
func (m *MockFileSystemPort) SetReadError(err error) {
	m.readError = err
}

// GetWrittenFiles returns the map of files written during testing.
func (m *MockFileSystemPort) GetWrittenFiles() map[string][]byte {
	return m.files
}

// SetReadFileFunc sets a custom function for ReadFile operations.
func (m *MockFileSystemPort) SetReadFileFunc(
	fn func(path string) ([]byte, error),
) {
	m.readFileFunc = fn
}

// SetWriteFileFunc sets a custom function for WriteFileAtomic operations.
func (m *MockFileSystemPort) SetWriteFileFunc(
	fn func(path string, data []byte) error,
) {
	m.writeFileFunc = fn
}

// MockConfigPort implements spi.ConfigPort for testing.
type MockConfigPort struct {
	cfg *config.Config
}

// NewMockConfigPort creates a new mock config port with the specified vault
// path.
func NewMockConfigPort(vaultPath string) *MockConfigPort {
	cfg := config.NewConfig(vaultPath)
	return &MockConfigPort{cfg: cfg}
}

// Config implements spi.ConfigPort.Config.
func (m *MockConfigPort) Config() *config.Config {
	return m.cfg
}
