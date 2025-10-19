package template

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"text/template"

	"github.com/jack/lithos/internal/ports/spi"
	"github.com/jack/lithos/internal/shared/errors"
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

func (m *mockTemplateParser) Execute(
	ctx context.Context,
	tmpl *template.Template,
	data interface{},
) errors.Result[string] {
	return errors.Ok("mocked")
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

func TestNewTemplateFSAdapter(t *testing.T) {
	mockFS := &mockFileSystemPort{}
	mockParser := &mockTemplateParser{}
	adapter := NewTemplateFSAdapter(mockFS, mockParser)

	if adapter == nil {
		t.Error("NewTemplateFSAdapter() returned nil")
	}
}

func TestTemplateFSAdapter_ListTemplates(t *testing.T) {
	mockFS := &mockFileSystemPort{}
	mockParser := &mockTemplateParser{}
	adapter := NewTemplateFSAdapter(mockFS, mockParser)
	ctx := context.Background()

	templates, err := adapter.ListTemplates(ctx)
	if err != nil {
		t.Errorf("ListTemplates() unexpected error = %v", err)
		return
	}

	// Currently returns empty list
	if len(templates) != 0 {
		t.Errorf("ListTemplates() = %v, want empty slice", templates)
	}
}

func TestTemplateFSAdapter_GetTemplate(t *testing.T) {
	mockFS := &mockFileSystemPort{}
	mockParser := &mockTemplateParser{}
	adapter := NewTemplateFSAdapter(mockFS, mockParser)
	ctx := context.Background()

	_, err := adapter.GetTemplate(ctx, "nonexistent")
	if err == nil {
		t.Error(
			"GetTemplate() expected error for nonexistent template, got nil",
		)
	}
}

func TestTemplateFSAdapter_GetTemplateByPath(t *testing.T) {
	tests := []struct {
		name        string
		content     string
		path        string
		wantName    string
		wantErr     bool
		errContains string
	}{
		{
			name:     "successful template loading",
			content:  "Hello, {{.Name}}!",
			path:     "/path/to/test-template.txt",
			wantName: "test-template",
			wantErr:  false,
		},
		{
			name:        "file read error",
			content:     "",
			path:        "/invalid/path.txt",
			wantErr:     true,
			errContains: "failed to read template file",
		},
		{
			name:        "template parse error",
			content:     "{{invalid syntax",
			path:        "/path/to/invalid.txt",
			wantErr:     true,
			errContains: "failed to parse template",
		},
		{
			name:        "template parse error",
			content:     "{{invalid syntax",
			path:        "/path/to/invalid.txt",
			wantErr:     true,
			errContains: "failed to parse template",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockFS := &mockFileSystemPort{
				readFileFunc: func(path string) ([]byte, error) {
					if tt.name == "file read error" {
						return nil, fmt.Errorf("file not found")
					}
					if path == tt.path {
						return []byte(tt.content), nil
					}
					return nil, fmt.Errorf("file not found")
				},
			}

			mockParser := &mockTemplateParser{}
			adapter := NewTemplateFSAdapter(mockFS, mockParser)
			ctx := context.Background()

			got, err := adapter.GetTemplateByPath(ctx, tt.path)

			if tt.wantErr {
				if err == nil {
					t.Errorf("GetTemplateByPath() expected error, got nil")
					return
				}
				if tt.errContains != "" {
					if !containsString(err.Error(), tt.errContains) {
						t.Errorf(
							"GetTemplateByPath() error = %v, expected to contain %q",
							err,
							tt.errContains,
						)
					}
				}
				return
			}

			if err != nil {
				t.Errorf("GetTemplateByPath() unexpected error = %v", err)
				return
			}

			if got == nil {
				t.Error("GetTemplateByPath() returned nil template")
				return
			}

			if got.Name != tt.wantName {
				t.Errorf(
					"GetTemplateByPath() template name = %q, want %q",
					got.Name,
					tt.wantName,
				)
			}

			if got.Content != tt.content {
				t.Errorf(
					"GetTemplateByPath() template content = %q, want %q",
					got.Content,
					tt.content,
				)
			}

			if got.FilePath != tt.path {
				t.Errorf(
					"GetTemplateByPath() template file path = %q, want %q",
					got.FilePath,
					tt.path,
				)
			}

			if got.Parsed == nil {
				t.Error("GetTemplateByPath() template parsed field is nil")
			}
		})
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
