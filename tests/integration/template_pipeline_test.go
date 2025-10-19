package integration

import (
	"context"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/jack/lithos/internal/adapters/api/cli"
	"github.com/jack/lithos/internal/adapters/spi/filesystem"
	"github.com/jack/lithos/internal/app/template"
	"github.com/jack/lithos/internal/domain"
	"github.com/jack/lithos/internal/ports/spi"
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

// TestTemplatePipelineIntegration tests the complete template processing
// pipeline
// from template file reading through parsing, execution, and output generation.
// This integration test uses real files from testdata/ and compares against
// golden files.
func TestTemplatePipelineIntegration(t *testing.T) {
	// Setup test environment
	ctx := context.Background()

	// Create real filesystem adapter for integration testing
	fsAdapter := filesystem.NewLocalFileSystemAdapter()

	// Test static template with custom functions
	t.Run("static_template_with_functions", func(t *testing.T) {
		// Get the project root directory (where go.mod is)
		projectRoot := findProjectRoot(t)
		templatePath := filepath.Join(
			projectRoot,
			"testdata",
			"templates",
			"static-template.md",
		)
		expectedOutputPath := filepath.Join(
			projectRoot,
			"testdata",
			"golden",
			"static-template-expected.md",
		)

		// Verify test files exist
		if _, err := os.Stat(templatePath); os.IsNotExist(err) {
			t.Fatalf("Template file does not exist: %s", templatePath)
		}
		if _, err := os.Stat(expectedOutputPath); os.IsNotExist(err) {
			t.Fatalf(
				"Expected output file does not exist: %s",
				expectedOutputPath,
			)
		}

		// Read expected output
		expectedBytes, err := os.ReadFile(expectedOutputPath)
		if err != nil {
			t.Fatalf("Failed to read expected output: %v", err)
		}
		expectedOutput := string(expectedBytes)

		// Create temporary output file
		tempDir := t.TempDir()
		outputFileName := "static-template.md"
		expectedOutputFile := filepath.Join(tempDir, outputFileName)

		// Execute the new command (this will create the output file)
		cmd := cli.NewCommand(fsAdapter)
		cmd.SetArgs([]string{templatePath})

		// Capture the command execution - we need to redirect output
		// For integration testing, we'll call the command logic directly
		err = executeNewCommand(
			ctx,
			fsAdapter,
			templatePath,
			expectedOutputFile,
		)
		if err != nil {
			t.Fatalf("Command execution failed: %v", err)
		}

		// Read the actual output
		actualBytes, err := os.ReadFile(expectedOutputFile)
		if err != nil {
			t.Fatalf("Failed to read actual output: %v", err)
		}
		actualOutput := string(actualBytes)

		// Compare outputs (allowing for date differences in golden file)
		if !compareTemplateOutputs(expectedOutput, actualOutput) {
			t.Errorf(
				"Template output mismatch:\nExpected:\n%s\nActual:\n%s",
				expectedOutput,
				actualOutput,
			)
		}
	})
}

// executeNewCommand executes the new command logic directly for testing.
func executeNewCommand(
	ctx context.Context,
	fs spi.FileSystemPort,
	templatePath, outputPath string,
) error {
	// Read the template file
	content, err := fs.ReadFile(templatePath)
	if err != nil {
		return err
	}

	// Parse the template
	parser := template.NewStaticTemplateParser()
	parseResult := parser.Parse(ctx, string(content))
	if parseResult.IsErr() {
		return parseResult.Error()
	}

	parsedTemplate := parseResult.Value()

	// Create domain template object
	templateName := filepath.Base(templatePath)
	if ext := filepath.Ext(templateName); ext != "" {
		templateName = templateName[:len(templateName)-len(ext)]
	}

	tmpl := &domain.Template{
		FilePath: templatePath,
		Name:     templateName,
		Content:  string(content),
		Parsed:   parsedTemplate,
	}

	// Execute the template
	executor := template.NewGoTemplateExecutor()
	renderedContent, err := executor.Execute(ctx, tmpl, nil)
	if err != nil {
		return err
	}

	// Write rendered content to file
	return fs.WriteFileAtomic(outputPath, []byte(renderedContent))
}

// compareTemplateOutputs compares template outputs, allowing for dynamic
// content like dates.
func compareTemplateOutputs(expected, actual string) bool {
	expectedLines := strings.Split(strings.TrimSpace(expected), "\n")
	actualLines := strings.Split(strings.TrimSpace(actual), "\n")

	if len(expectedLines) != len(actualLines) {
		return false
	}

	for i, expectedLine := range expectedLines {
		actualLine := actualLines[i]

		// Allow date lines to differ (for now function)
		if strings.HasPrefix(expectedLine, "Created: ") &&
			strings.HasPrefix(actualLine, "Created: ") {
			continue
		}

		// Exact match for other lines
		if expectedLine != actualLine {
			return false
		}
	}

	return true
}
