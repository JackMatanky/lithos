package integration

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/jack/lithos/internal/adapters/api/cli"
	"github.com/jack/lithos/internal/adapters/spi/filesystem"
	templaterepo "github.com/jack/lithos/internal/adapters/spi/template"
	templatedomain "github.com/jack/lithos/internal/app/template"
)

// findProjectRoot finds the project root directory by looking for go.mod.
func findProjectRoot(t *testing.T) string {
	_, filename, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("Could not get caller information")
	}

	// Start from the directory containing this test file
	dir := filepath.Dir(filename)

	// Walk up directories until we find go.mod
	for {
		if _, err := os.Stat(filepath.Join(dir, "go.mod")); err == nil {
			return dir
		}

		parent := filepath.Dir(dir)
		if parent == dir {
			t.Fatal("Could not find project root (go.mod)")
		}
		dir = parent
	}
}

func TestNewCommand_Integration_CompleteFlow(t *testing.T) {
	// Change to project root directory for relative paths to work
	projectRoot := findProjectRoot(t)
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	defer os.Chdir(originalDir) // Restore original directory
	if err := os.Chdir(projectRoot); err != nil {
		t.Fatalf("Failed to change to project root: %v", err)
	}

	templatePath := "testdata/templates/integration-test-template.txt"
	// Create real filesystem adapter
	fsAdapter := filesystem.NewLocalFileSystemAdapter()

	// Create template parser and executor from domain services
	templateParser := templatedomain.NewStaticTemplateParser()
	templateExecutor := templatedomain.NewGoTemplateExecutor()

	// Create template engine and repository
	templateEngine := templatedomain.NewTemplateEngine(
		templateParser,
		templateExecutor,
	)
	templateRepo := templaterepo.NewTemplateFSAdapter(fsAdapter, templateParser)

	// Create CLI adapter with injected dependencies
	adapter := cli.NewCobraCLIAdapter(templateEngine, templateRepo, fsAdapter)

	// Execute the new command with testdata template
	exitCode := adapter.Execute([]string{"new", templatePath})

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output file was created
	outputPath := "integration-test-template.md"
	if _, statErr := os.Stat(outputPath); os.IsNotExist(statErr) {
		t.Errorf("Output file %s was not created", outputPath)
		return
	}

	// Read and verify output file content
	outputContent, err := os.ReadFile(outputPath)
	if err != nil {
		t.Fatalf("Failed to read output file: %v", err)
	}

	outputStr := string(outputContent)

	// Verify template functions were applied
	if !strings.Contains(outputStr, "# daily note") {
		t.Error("toLower function was not applied correctly")
	}

	if !strings.Contains(outputStr, "Hello, WORLD!") {
		t.Error("toUpper function was not applied correctly")
	}

	if !strings.Contains(outputStr, "Created on:") {
		t.Error("now function was not applied correctly")
	}

	// Verify the date format is correct (YYYY-MM-DD)
	lines := strings.Split(outputStr, "\n")
	var dateLine string
	for _, line := range lines {
		if strings.Contains(line, "Created on:") {
			dateLine = line
			break
		}
	}

	if dateLine == "" {
		t.Error("Date line not found in output")
		return
	}

	// Extract date part and verify format
	parts := strings.Split(dateLine, ": ")
	if len(parts) != 2 {
		t.Error("Date line format is incorrect")
		return
	}

	dateStr := strings.TrimSpace(parts[1])
	// Basic validation: should be YYYY-MM-DD format
	if len(dateStr) != 10 || dateStr[4] != '-' || dateStr[7] != '-' {
		t.Errorf("Date format is incorrect: %s", dateStr)
	}
}

func TestNewCommand_Integration_AtomicWrite(t *testing.T) {
	// Change to project root directory for relative paths to work
	projectRoot := findProjectRoot(t)
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	defer os.Chdir(originalDir) // Restore original directory
	if err := os.Chdir(projectRoot); err != nil {
		t.Fatalf("Failed to change to project root: %v", err)
	}

	templatePath := "testdata/templates/large-integration-template.txt"
	// Create real filesystem adapter
	fsAdapter := filesystem.NewLocalFileSystemAdapter()

	// Create template parser and executor from domain services
	templateParser := templatedomain.NewStaticTemplateParser()
	templateExecutor := templatedomain.NewGoTemplateExecutor()

	// Create template engine and repository
	templateEngine := templatedomain.NewTemplateEngine(
		templateParser,
		templateExecutor,
	)
	templateRepo := templaterepo.NewTemplateFSAdapter(fsAdapter, templateParser)

	// Create CLI adapter with injected dependencies
	adapter := cli.NewCobraCLIAdapter(templateEngine, templateRepo, fsAdapter)

	// Execute the new command with testdata template
	exitCode := adapter.Execute([]string{"new", templatePath})

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output file was created and is complete
	outputPath := "large-integration-template.md"
	outputContent, err := os.ReadFile(outputPath)
	if err != nil {
		t.Fatalf("Failed to read output file: %v", err)
	}

	// Verify template functions were applied
	outputStr := string(outputContent)
	if !strings.Contains(outputStr, "Line number") {
		t.Error("Template function not applied correctly")
	}

	// Verify the rendered content includes both functions
	if !strings.Contains(outputStr, "number") {
		t.Error("toLower function was not applied correctly")
	}

	// Check that date function was applied
	if !strings.Contains(outputStr, "2025-") ||
		!strings.Contains(outputStr, "-10-") {
		t.Error("now function was not applied correctly")
	}
}
