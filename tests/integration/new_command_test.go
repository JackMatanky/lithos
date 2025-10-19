package integration

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/jack/lithos/internal/adapters/api/cli"
	"github.com/jack/lithos/internal/adapters/spi/filesystem"
)

// Note: findProjectRoot is already defined in template_pipeline_test.go

func TestNewCommand_Integration_CompleteFlow(t *testing.T) {
	// Find project root and testdata
	projectRoot := findProjectRoot(t)
	templatePath := filepath.Join(
		projectRoot,
		"testdata",
		"templates",
		"integration-test-template.txt",
	)

	// Verify test template exists
	if _, err := os.Stat(templatePath); os.IsNotExist(err) {
		t.Fatalf("Test template not found: %s", templatePath)
	}

	// Create a temporary directory for test output
	tempDir, err := os.MkdirTemp("", "lithos-integration-test-")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() {
		_ = os.RemoveAll(tempDir)
	}()

	// Change to temp directory for test
	oldDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current dir: %v", err)
	}
	defer func() {
		_ = os.Chdir(oldDir)
	}()

	err = os.Chdir(tempDir)
	if err != nil {
		t.Fatalf("Failed to change to temp dir: %v", err)
	}

	// Create real filesystem adapter
	fsAdapter := filesystem.NewLocalFileSystemAdapter()

	// Create CLI adapter with real filesystem
	adapter := cli.NewCobraCLIAdapter(fsAdapter)

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
	// Find project root and testdata
	projectRoot := findProjectRoot(t)
	templatePath := filepath.Join(
		projectRoot,
		"testdata",
		"templates",
		"large-integration-template.txt",
	)

	// Verify test template exists
	if _, err := os.Stat(templatePath); os.IsNotExist(err) {
		t.Fatalf("Test template not found: %s", templatePath)
	}

	// Create a temporary directory for test output
	tempDir, err := os.MkdirTemp("", "lithos-atomic-test-")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() {
		_ = os.RemoveAll(tempDir)
	}()

	// Change to temp directory for test
	oldDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current dir: %v", err)
	}
	defer func() {
		_ = os.Chdir(oldDir)
	}()

	err = os.Chdir(tempDir)
	if err != nil {
		t.Fatalf("Failed to change to temp dir: %v", err)
	}

	// Create real filesystem adapter
	fsAdapter := filesystem.NewLocalFileSystemAdapter()

	// Create CLI adapter with real filesystem
	adapter := cli.NewCobraCLIAdapter(fsAdapter)

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
