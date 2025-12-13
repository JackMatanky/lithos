package template

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"text/template"

	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// mockFileSystemPort implements spi.FileSystemPort for testing.
type mockFileSystemPort struct {
	readFileFunc func(path string) ([]byte, error)
}

// mockTemplateParser implements spi.TemplateParser for testing.
type mockTemplateParser struct{}

func (m *mockTemplateParser) Parse(
	ctx context.Context,
	content string,
) errors.Result[*template.Template] {
	// Return error for invalid syntax to test error handling
	if strings.Contains(content, "{{invalid syntax") {
		return errors.Err[*template.Template](
			fmt.Errorf("parse error: invalid syntax"),
		)
	}
	return errors.Ok(template.New("mock"))
}

func (m *mockFileSystemPort) ReadFile(path string) ([]byte, error) {
	if m.readFileFunc != nil {
		return m.readFileFunc(path)
	}
	return nil, nil
}

func (m *mockFileSystemPort) WriteFile(path string, data []byte) error {
	return nil
}

func (m *mockFileSystemPort) WriteFileAtomic(path string, data []byte) error {
	return nil
}

func (m *mockFileSystemPort) Walk(root string, fn spi.WalkFunc) error {
	return nil
}

func TestNewFSAdapter(t *testing.T) {
	mockFS := &mockFileSystemPort{}
	mockParser := &mockTemplateParser{}
	adapter := NewFSAdapter(mockFS, mockParser)

	if adapter == nil {
		t.Error("NewFSAdapter() returned nil")
	}
}

func TestFSAdapter_ListTemplates(t *testing.T) {
	mockFS := &mockFileSystemPort{}
	mockParser := &mockTemplateParser{}
	adapter := NewFSAdapter(mockFS, mockParser)
	ctx := context.Background()

	templates, err := adapter.List(ctx)
	if err != nil {
		t.Errorf("List() unexpected error = %v", err)
		return
	}

	// Currently returns empty list
	if len(templates) != 0 {
		t.Errorf("List() = %v, want empty slice", templates)
	}
}

func TestFSAdapter_Get(t *testing.T) {
	mockFS := &mockFileSystemPort{}
	mockParser := &mockTemplateParser{}
	adapter := NewFSAdapter(mockFS, mockParser)
	ctx := context.Background()

	_, err := adapter.Get(ctx, "nonexistent")
	if err == nil {
		t.Error(
			"GetTemplate() expected error for nonexistent template, got nil",
		)
	}
}

func TestFSAdapter_GetByPath_Success(t *testing.T) {
	mockFS := &mockFileSystemPort{
		readFileFunc: func(path string) ([]byte, error) {
			if path == "/path/to/test-template.txt" {
				return []byte("Hello, {{.Name}}!"), nil
			}
			return nil, fmt.Errorf("file not found")
		},
	}

	mockParser := &mockTemplateParser{}
	adapter := NewFSAdapter(mockFS, mockParser)
	ctx := context.Background()

	got, err := adapter.GetByPath(ctx, "/path/to/test-template.txt")

	if err != nil {
		t.Errorf("GetByPath() unexpected error = %v", err)
		return
	}

	if got == nil {
		t.Error("GetByPath() returned nil template")
		return
	}

	if got.Name != "test-template" {
		t.Errorf(
			"GetByPath() template name = %q, want %q",
			got.Name,
			"test-template",
		)
	}

	if got.Content != "Hello, {{.Name}}!" {
		t.Errorf(
			"GetByPath() template content = %q, want %q",
			got.Content,
			"Hello, {{.Name}}!",
		)
	}
}

func TestFSAdapter_GetByPath_FileReadError(t *testing.T) {
	mockFS := &mockFileSystemPort{
		readFileFunc: func(path string) ([]byte, error) {
			return nil, fmt.Errorf("file not found")
		},
	}

	mockParser := &mockTemplateParser{}
	adapter := NewFSAdapter(mockFS, mockParser)
	ctx := context.Background()

	_, err := adapter.GetByPath(ctx, "/invalid/path.txt")

	if err == nil {
		t.Error("GetByPath() expected error, got nil")
		return
	}

	if !containsString(err.Error(), "failed to read template file") {
		t.Errorf(
			"error = %v, want containing %q",
			err,
			"failed to read template file",
		)
	}
}

func TestFSAdapter_GetByPath_ParseError(t *testing.T) {
	mockFS := &mockFileSystemPort{
		readFileFunc: func(path string) ([]byte, error) {
			if path == "/path/to/invalid.txt" {
				return []byte("{{invalid syntax"), nil
			}
			return nil, fmt.Errorf("file not found")
		},
	}

	mockParser := &mockTemplateParser{}
	adapter := NewFSAdapter(mockFS, mockParser)
	ctx := context.Background()

	_, err := adapter.GetByPath(ctx, "/path/to/invalid.txt")

	if err == nil {
		t.Error("GetByPath() expected error, got nil")
		return
	}

	if !containsString(err.Error(), "failed to parse template") {
		t.Errorf(
			"error = %v, want containing %q",
			err,
			"failed to parse template",
		)
	}
}

// containsString checks if a string contains a substring (helper for tests).
func containsString(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
