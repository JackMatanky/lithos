package integration

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/jack/lithos/internal/adapters/spi/filesystem"
	templaterepo "github.com/jack/lithos/internal/adapters/spi/template"
	templatedomain "github.com/jack/lithos/internal/app/template"
	"github.com/jack/lithos/internal/ports/spi"
)

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

		// Execute the new command logic directly for testing
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
	// Create template parser and executor from domain services
	templateParser := templatedomain.NewStaticTemplateParser()
	templateExecutor := templatedomain.NewGoTemplateExecutor()

	// Create template engine and repository
	templateEngine := templatedomain.NewTemplateEngine(
		templateParser,
		templateExecutor,
	)
	templateRepo := templaterepo.NewFSAdapter(fs, templateParser)

	// Get parsed template from repository
	tmpl, err := templateRepo.GetTemplateByPath(ctx, templatePath)
	if err != nil {
		return err
	}

	// Execute parsed template
	renderedContent, err := templateEngine.ExecuteParsedTemplate(ctx, tmpl)
	if err != nil {
		return err
	}

	// Write the output file
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
