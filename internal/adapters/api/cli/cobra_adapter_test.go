package cli

import (
	"bytes"
	"errors"
	"os"
	"strings"
	"testing"

	templaterepo "github.com/jack/lithos/internal/adapters/spi/template"
	templatedomain "github.com/jack/lithos/internal/app/template"
	"github.com/jack/lithos/internal/ports/spi"
)

// mockFileSystemPort implements spi.FileSystemPort for testing.
type mockFileSystemPort struct {
	readFileFunc  func(path string) ([]byte, error)
	writeFileFunc func(path string, data []byte) error
	writtenFiles  map[string][]byte
}

// createTemplateEngine creates a template engine with concrete adapters for
// testing.
func createTemplateEngine() *templatedomain.TemplateEngine {
	templateParser := templatedomain.NewStaticTemplateParser()
	templateExecutor := templatedomain.NewGoTemplateExecutor()
	return templatedomain.NewTemplateEngine(templateParser, templateExecutor)
}

// createTemplateParser creates a template parser for testing.
func createTemplateParser() spi.TemplateParser {
	return templatedomain.NewStaticTemplateParser()
}

func newMockFileSystemPort() *mockFileSystemPort {
	return &mockFileSystemPort{
		writtenFiles: make(map[string][]byte),
	}
}

func (m *mockFileSystemPort) ReadFile(path string) ([]byte, error) {
	if m.readFileFunc != nil {
		return m.readFileFunc(path)
	}
	return nil, errors.New("mock not configured")
}

func (m *mockFileSystemPort) WriteFileAtomic(path string, data []byte) error {
	if m.writeFileFunc != nil {
		return m.writeFileFunc(path, data)
	}
	// Default behavior: store written data
	m.writtenFiles[path] = data
	return nil
}

func (m *mockFileSystemPort) Walk(root string, fn spi.WalkFunc) error {
	return errors.New("not implemented")
}

func TestCobraCLIAdapter_Execute_VersionCommand(t *testing.T) {
	mockFS := newMockFileSystemPort()
	templateParser := templatedomain.NewStaticTemplateParser()
	templateExecutor := templatedomain.NewGoTemplateExecutor()
	templateEngine := templatedomain.NewTemplateEngine(
		templateParser,
		templateExecutor,
	)
	templateRepo := templaterepo.NewTemplateFSAdapter(
		mockFS,
		createTemplateParser(),
	)
	adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Execute version command
	exitCode := adapter.Execute([]string{"version"})

	// Restore stdout
	_ = w.Close()
	os.Stdout = oldStdout

	// Read captured output
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	output := buf.String()

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output contains version
	if !strings.Contains(output, "Lithos version") {
		t.Errorf("Execute() output = %q, should contain version info", output)
	}
}

func TestCobraCLIAdapter_Execute_HelpCommand(t *testing.T) {
	mockFS := newMockFileSystemPort()
	templateEngine := createTemplateEngine()
	templateRepo := templaterepo.NewTemplateFSAdapter(
		mockFS,
		createTemplateParser(),
	)
	adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Execute help command
	exitCode := adapter.Execute([]string{"--help"})

	// Restore stdout
	_ = w.Close()
	os.Stdout = oldStdout

	// Read captured output
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	output := buf.String()

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output contains help info
	if !strings.Contains(output, "lithos") {
		t.Errorf("Execute() output = %q, should contain command info", output)
	}
}

func TestCobraCLIAdapter_Execute_InvalidCommand(t *testing.T) {
	mockFS := newMockFileSystemPort()
	templateEngine := createTemplateEngine()
	templateRepo := templaterepo.NewTemplateFSAdapter(
		mockFS,
		createTemplateParser(),
	)
	adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

	// Execute invalid command
	exitCode := adapter.Execute([]string{"invalid-command"})

	// Verify exit code is non-zero for invalid command
	if exitCode == 0 {
		t.Error("Execute() should return non-zero exit code for invalid " +
			"command")
	}
}

func TestCobraCLIAdapter_Execute_NoArgs(t *testing.T) {
	mockFS := newMockFileSystemPort()
	templateEngine := createTemplateEngine()
	templateRepo := templaterepo.NewTemplateFSAdapter(
		mockFS,
		createTemplateParser(),
	)
	adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Execute with no args (should show help)
	exitCode := adapter.Execute([]string{})

	// Restore stdout
	_ = w.Close()
	os.Stdout = oldStdout

	// Read captured output
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	output := buf.String()

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output contains help info
	if !strings.Contains(output, "lithos") {
		t.Errorf("Execute() output = %q, should contain command info", output)
	}
}

func TestCobraCLIAdapter_Execute_NewCommand_Success(t *testing.T) {
	expectedContent := []byte("Hello, {{.Name}}!")
	mockFS := newMockFileSystemPort()
	mockFS.readFileFunc = func(path string) ([]byte, error) {
		if path == "template.txt" {
			return expectedContent, nil
		}
		return nil, errors.New("file not found")
	}
	templateEngine := createTemplateEngine()
	templateRepo := templaterepo.NewTemplateFSAdapter(
		mockFS,
		createTemplateParser(),
	)
	adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Execute new command
	exitCode := adapter.Execute([]string{"new", "template.txt"})

	// Restore stdout
	_ = w.Close()
	os.Stdout = oldStdout

	// Read captured output
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	output := buf.String()

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output contains confirmation message
	if !strings.Contains(output, "Created template.md") {
		t.Errorf(
			"Execute() output = %q, should contain file creation confirmation",
			output,
		)
	}

	// Verify file was written with correct content
	if writtenData, exists := mockFS.writtenFiles["template.md"]; !exists {
		t.Error("Expected template.md to be written")
	} else if !strings.Contains(string(writtenData), "Hello, <no value>!") {
		t.Errorf(
			"Written file content = %q, should contain rendered template content",
			string(writtenData),
		)
	}
}

func TestCobraCLIAdapter_Execute_NewCommand_FileNotFound(t *testing.T) {
	mockFS := newMockFileSystemPort()
	mockFS.readFileFunc = func(path string) ([]byte, error) {
		return nil, errors.New("file not found")
	}
	templateEngine := createTemplateEngine()
	templateRepo := templaterepo.NewTemplateFSAdapter(
		mockFS,
		createTemplateParser(),
	)
	adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

	// Execute new command with non-existent file
	exitCode := adapter.Execute([]string{"new", "nonexistent.txt"})

	// Verify exit code is non-zero for error
	if exitCode == 0 {
		t.Error("Execute() should return non-zero exit code for file not found")
	}
}

func TestCobraCLIAdapter_Execute_NewCommand_NoArgs(t *testing.T) {
	mockFS := newMockFileSystemPort()
	templateEngine := createTemplateEngine()
	templateRepo := templaterepo.NewTemplateFSAdapter(
		mockFS,
		createTemplateParser(),
	)
	adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

	// Execute new command without args
	exitCode := adapter.Execute([]string{"new"})

	// Verify exit code is non-zero for missing args
	if exitCode == 0 {
		t.Error(
			"Execute() should return non-zero exit code for missing template-path",
		)
	}
}

func TestCobraCLIAdapter_Execute_NewCommand_WriteFailure(t *testing.T) {
	expectedContent := []byte("Hello, {{toLower \"WORLD\"}}!")
	mockFS := newMockFileSystemPort()
	mockFS.readFileFunc = func(path string) ([]byte, error) {
		if path == "template.txt" {
			return expectedContent, nil
		}
		return nil, errors.New("file not found")
	}
	mockFS.writeFileFunc = func(path string, data []byte) error {
		return errors.New("disk full")
	}

	templateEngine := createTemplateEngine()
	templateRepo := templaterepo.NewTemplateFSAdapter(
		mockFS,
		createTemplateParser(),
	)
	adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

	// Execute new command
	exitCode := adapter.Execute([]string{"new", "template.txt"})

	// Verify exit code is non-zero for write failure
	if exitCode == 0 {
		t.Error("Execute() should return non-zero exit code for write failure")
	}
}

func TestCobraCLIAdapter_Execute_NewCommand_WithFunctions(t *testing.T) {
	expectedContent := []byte("Hello, {{toLower \"WORLD\"}}!")
	mockFS := newMockFileSystemPort()
	mockFS.readFileFunc = func(path string) ([]byte, error) {
		if path == "template.txt" {
			return expectedContent, nil
		}
		return nil, errors.New("file not found")
	}

	templateEngine := createTemplateEngine()
	templateRepo := templaterepo.NewTemplateFSAdapter(
		mockFS,
		createTemplateParser(),
	)
	adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Execute new command
	exitCode := adapter.Execute([]string{"new", "template.txt"})

	// Restore stdout
	_ = w.Close()
	os.Stdout = oldStdout

	// Read captured output
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	output := buf.String()

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output contains confirmation message
	if !strings.Contains(output, "Created template.md") {
		t.Errorf(
			"Execute() output = %q, should contain file creation confirmation",
			output,
		)
	}

	// Verify file was written with correct content (template functions applied)
	if writtenData, exists := mockFS.writtenFiles["template.md"]; !exists {
		t.Error("Expected template.md to be written")
	} else if !strings.Contains(string(writtenData), "Hello, world!") {
		t.Errorf(
			"Written file content = %q, should contain rendered template with functions applied",
			string(writtenData),
		)
	}
}

func TestCobraCLIAdapter_Execute_NewCommand_FilenameGeneration(t *testing.T) {
	tests := []struct {
		name         string
		templatePath string
		expectedFile string
	}{
		{
			name:         "txt extension",
			templatePath: "mytemplate.txt",
			expectedFile: "mytemplate.md",
		},
		{
			name:         "md extension",
			templatePath: "mytemplate.md",
			expectedFile: "mytemplate.md",
		},
		{
			name:         "no extension",
			templatePath: "mytemplate",
			expectedFile: "mytemplate.md",
		},
		{
			name:         "path with directories",
			templatePath: "templates/notes/daily.txt",
			expectedFile: "daily.md",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockFS := newMockFileSystemPort()
			mockFS.readFileFunc = func(path string) ([]byte, error) {
				if path == tt.templatePath {
					return []byte("Simple template"), nil
				}
				return nil, errors.New("file not found")
			}

			templateEngine := createTemplateEngine()
			templateRepo := templaterepo.NewTemplateFSAdapter(
				mockFS,
				createTemplateParser(),
			)
			adapter := NewCobraCLIAdapter(templateEngine, templateRepo, mockFS)

			// Execute new command
			exitCode := adapter.Execute([]string{"new", tt.templatePath})

			// Verify exit code
			if exitCode != 0 {
				t.Errorf("Execute() exit code = %v, want 0", exitCode)
			}

			// Verify file was written with expected filename
			if _, exists := mockFS.writtenFiles[tt.expectedFile]; !exists {
				t.Errorf("Expected %s to be written", tt.expectedFile)
			}
		})
	}
}
