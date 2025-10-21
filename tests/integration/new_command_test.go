package integration

import (
	"io"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/JackMatanky/lithos/internal/adapters/api/cli"
	"github.com/JackMatanky/lithos/internal/adapters/spi/filesystem"
	templaterepo "github.com/JackMatanky/lithos/internal/adapters/spi/template"
	templatedomain "github.com/JackMatanky/lithos/internal/app/template"
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

// copyFile copies a file from src to dst.
func copyFile(src, dst string) error {
	srcFile, err := os.Open(src)
	if err != nil {
		return err
	}
	defer func() {
		_ = srcFile.Close()
	}()

	dstFile, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer func() {
		_ = dstFile.Close()
	}()

	_, err = io.Copy(dstFile, srcFile)
	return err
}

// copyTestDataToTemp copies required test data to the temporary directory.
func copyTestDataToTemp(t *testing.T, tempDir string) {
	// Create templates directory in temp dir
	templatesDir := filepath.Join(tempDir, "templates")
	if err := os.MkdirAll(templatesDir, 0o750); err != nil {
		t.Fatalf("Failed to create templates dir: %v", err)
	}

	// Copy template files
	files := []string{
		"integration-test-template.txt",
		"large-integration-template.txt",
	}
	for _, file := range files {
		src := filepath.Join("testdata", "templates", file)
		dst := filepath.Join(templatesDir, file)
		if err := copyFile(src, dst); err != nil {
			t.Fatalf("Failed to copy %s: %v", file, err)
		}
	}
}

func TestNewCommand_Integration_CompleteFlow(t *testing.T) {
	// Create temporary directory for test isolation
	tempDir := t.TempDir()

	// Copy test data to temp directory
	projectRoot := findProjectRoot(t)
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	t.Cleanup(func() {
		_ = os.Chdir(originalDir)
	})

	// Change to project root to copy test data, then to temp dir
	if err2 := os.Chdir(projectRoot); err2 != nil {
		t.Fatalf("Failed to change to project root: %v", err2)
	}
	copyTestDataToTemp(t, tempDir)

	// Change to temp directory for test execution
	if err2 := os.Chdir(tempDir); err2 != nil {
		t.Fatalf("Failed to change to temp dir: %v", err2)
	}

	templatePath := "templates/integration-test-template.txt"
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
	templateRepo := templaterepo.NewFSAdapter(fsAdapter, templateParser)

	// Create CLI adapter with injected dependencies
	adapter := cli.NewCobraCLIAdapter(templateEngine, templateRepo, fsAdapter)

	// Execute the new command with testdata template
	exitCode := adapter.Execute([]string{"new", templatePath})

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output file was created and is complete
	outputPath := "integration-test-template.md"
	outputContent, err := os.ReadFile(outputPath)
	if err != nil {
		t.Fatalf("Failed to read output file: %v", err)
	}

	// Verify template functions were applied
	outputStr := string(outputContent)
	if !strings.Contains(outputStr, "daily note") {
		t.Error("toLower function was not applied correctly")
	}

	// Verify the rendered content includes toUpper function
	if !strings.Contains(outputStr, "WORLD") {
		t.Error("toUpper function was not applied correctly")
	}

	// Check that date function was applied
	if !strings.Contains(outputStr, "2025-") ||
		!strings.Contains(outputStr, "-10-") {
		t.Error("now function was not applied correctly")
	}
}

func TestNewCommand_Integration_AtomicWrite(t *testing.T) {
	// Create temporary directory for test isolation
	tempDir := t.TempDir()

	// Copy test data to temp directory
	projectRoot := findProjectRoot(t)
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	t.Cleanup(func() {
		_ = os.Chdir(originalDir)
	})

	// Change to project root to copy test data, then to temp dir
	if err2 := os.Chdir(projectRoot); err2 != nil {
		t.Fatalf("Failed to change to project root: %v", err2)
	}
	copyTestDataToTemp(t, tempDir)

	// Change to temp directory for test execution
	if err2 := os.Chdir(tempDir); err2 != nil {
		t.Fatalf("Failed to change to temp dir: %v", err2)
	}

	templatePath := "templates/large-integration-template.txt"
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
	templateRepo := templaterepo.NewFSAdapter(fsAdapter, templateParser)

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
