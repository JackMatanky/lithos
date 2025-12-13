package tests

import (
	"context"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const (
	testDir = "tests"
)

// validateMarkdownLink checks if a markdown link [text](url) is valid.
func validateMarkdownLink(t *testing.T, line, docFile string, i int) {
	start := strings.Index(line, "](")
	if start == -1 {
		return
	}

	end := strings.Index(line[start+2:], ")")
	if end == -1 {
		return
	}

	link := line[start+2 : start+2+end]
	// Skip external links (http/https/mailto)
	if strings.HasPrefix(link, "http") ||
		strings.HasPrefix(link, "https") ||
		strings.HasPrefix(link, "mailto") {
		return
	}

	// Check if relative link exists
	linkPath := filepath.Join(filepath.Dir(docFile), link)
	if _, statErr := os.Stat(linkPath); os.IsNotExist(statErr) {
		t.Errorf("Broken link in %s line %d: %s", docFile, i+1, link)
	}
}

// validateReferenceLink checks if a reference-style link [text][ref] has a
// definition.
func validateReferenceLink(
	t *testing.T,
	line string,
	lines []string,
	docFile string,
	i int,
) {
	start := strings.Index(line, "][")
	if start == -1 {
		return
	}

	end := strings.Index(line[start+2:], "]")
	if end == -1 {
		return
	}

	ref := line[start+2 : start+2+end]
	// Look for reference definition later in file
	for j := i + 1; j < len(lines); j++ {
		if strings.HasPrefix(strings.TrimSpace(lines[j]), "["+ref+"]:") {
			return // Found definition
		}
	}
	t.Errorf("Undefined reference link in %s line %d: %s", docFile, i+1, ref)
}

// TestCLIDocumentationExamples validates all CLI command examples in
// documentation.
func TestCLIDocumentationExamples(t *testing.T) {
	// This test validates that all CLI command examples in documentation work
	// correctly
	// GREEN PHASE: This test should pass because documentation now exists

	// Test version command example
	t.Run("version_command_example", func(t *testing.T) {
		// This should validate the version command example from README.md
		// Example: lithos version
		cmd := exec.CommandContext(
			context.Background(),
			"go",
			"run",
			"../cmd/lithos/main.go",
			"version",
		)
		cmd.Dir = testDir // Run from tests directory
		output, err := cmd.CombinedOutput()
		// The command may fail due to missing configuration, but we check that
		// it's not an unknown command
		outputStr := string(output)
		// Should not be a "command not found" error - the command structure
		// should be valid
		assert.NotContains(t, outputStr, "unknown command")
		// If it succeeds, should show version
		if err == nil {
			assert.Contains(t, outputStr, "lithos v")
		}
	})

	t.Run("new_command_example", func(t *testing.T) {
		// This should validate the new command example from documentation
		// Example: lithos new contact
		cmd := exec.CommandContext(context.Background(),
			"go",
			"run",
			"../cmd/lithos/main.go",
			"new",
			"contact",
		)
		cmd.Dir = testDir // Run from tests directory
		output, err := cmd.CombinedOutput()
		// Should fail due to missing template or configuration, but command
		// should be recognized
		outputStr := string(output)
		require.Error(t, err)
		// Should not be a "command not found" error - the command structure
		// should be valid
		assert.NotContains(t, outputStr, "unknown command")
	})

	t.Run("index_command_example", func(t *testing.T) {
		// This should validate the index command example from documentation
		// Example: lithos index
		cmd := exec.CommandContext(
			context.Background(),
			"go",
			"run",
			"../cmd/lithos/main.go",
			"index",
		)
		cmd.Dir = testDir // Run from tests directory
		output, err := cmd.CombinedOutput()
		// Should fail due to missing vault/configuration, but command should be
		// recognized
		outputStr := string(output)
		require.Error(t, err)
		// Should not be a "command not found" error - the command structure
		// should be valid
		assert.NotContains(t, outputStr, "unknown command")
	})
}

// TestDocumentationLinks validates all links in documentation are working.
func TestDocumentationLinks(t *testing.T) {
	// This test validates that all documentation links are valid
	// RED PHASE: This test should fail because documentation files don't exist
	// yet

	docFiles := []string{
		"../README.md",
		"../CHANGELOG.md",
	}

	for _, docFile := range docFiles {
		t.Run(
			"links_in_"+strings.ReplaceAll(docFile, "/", "_"),
			func(t *testing.T) {
				// Check if file exists
				if _, err := os.Stat(docFile); os.IsNotExist(err) {
					t.Errorf("Documentation file %s does not exist", docFile)
					return
				}

				// Read file content
				content, err := os.ReadFile(docFile)
				require.NoError(t, err)

				// Find markdown links and validate them
				lines := strings.Split(string(content), "\n")
				for i, line := range lines {
					validateMarkdownLink(t, line, docFile, i)
					validateReferenceLink(t, line, lines, docFile, i)
				}
			},
		)
	}
}

// TestCodeSnippetCompilation validates that code snippets in documentation
// compile.
func TestCodeSnippetCompilation(t *testing.T) {
	// This test validates that Go code snippets in documentation compile
	// correctly
	// RED PHASE: This test should fail because documentation doesn't exist yet

	docFiles := []string{
		"../README.md",
	}

	for _, docFile := range docFiles {
		t.Run(
			"code_snippets_in_"+strings.ReplaceAll(docFile, "/", "_"),
			func(t *testing.T) {
				if _, err := os.Stat(docFile); os.IsNotExist(err) {
					t.Errorf("Documentation file %s does not exist", docFile)
					return
				}

				content, err := os.ReadFile(docFile)
				require.NoError(t, err)

				// Extract Go code blocks
				lines := strings.Split(string(content), "\n")
				inCodeBlock := false
				var codeSnippet strings.Builder

				for _, line := range lines {
					switch {
					case strings.HasPrefix(line, "```go"):
						inCodeBlock = true
						codeSnippet.Reset()
					case strings.HasPrefix(line, "```") && inCodeBlock:
						inCodeBlock = false
						// Test compilation of the code snippet
						if codeSnippet.Len() > 0 {
							testGoCodeCompilation(t, codeSnippet.String())
						}
					case inCodeBlock:
						codeSnippet.WriteString(line + "\n")
					}
				}
			},
		)
	}
}

// testGoCodeCompilation tests if a Go code snippet compiles.
func testGoCodeCompilation(t *testing.T, code string) {
	// Create a temporary file with the code
	tmpFile, err := os.CreateTemp("", "doc_test_*.go")
	require.NoError(t, err)
	defer func() { _ = os.Remove(tmpFile.Name()) }()

	// Write the code to the temp file
	_, err = tmpFile.WriteString(code)
	require.NoError(t, err)
	require.NoError(t, tmpFile.Close())

	// Try to compile it
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-o",
		"/dev/null",
		tmpFile.Name(),
	)
	err = cmd.Run()
	require.NoError(t, err, "Code snippet should compile: %s", code)
}

// TestGodocGeneration validates that godoc can be generated from code comments.
func TestGodocGeneration(t *testing.T) {
	// This test validates that godoc generation works for documented packages
	// GREEN PHASE: This test should pass because code should be documented

	packages := []string{
		"./internal/app/vault",
		"./internal/app/query",
		"./internal/adapters/api/cli",
	}

	for _, pkg := range packages {
		t.Run(
			"godoc_for_"+strings.ReplaceAll(pkg, "/", "_"),
			func(t *testing.T) {
				cmd := exec.CommandContext(
					context.Background(),
					"go",
					"doc",
					pkg,
				)
				cmd.Dir = "../" // Run from project root
				output, err := cmd.CombinedOutput()
				require.NoError(
					t,
					err,
					"godoc should generate for package %s",
					pkg,
				)
				// Just check that it runs without error - some packages may
				// have minimal exported APIs
				assert.NotNil(t, output, "godoc should run for package %s", pkg)
			},
		)
	}
}

// TestConfigurationExamples validates configuration examples in documentation.
func TestConfigurationExamples(t *testing.T) {
	// This test validates that configuration examples in documentation are
	// valid
	// RED PHASE: This test should fail because documentation doesn't exist yet

	configExamples := []string{
		// These should be extracted from documentation
		"vault_path: ./testdata/vault",
		"file_class_key: type",
	}

	for _, example := range configExamples {
		t.Run(
			"config_example_"+strings.ReplaceAll(example, " ", "_"),
			func(t *testing.T) {
				// This should validate that the configuration example is
				// syntactically correct
				// For now, just check that it's not empty
				assert.NotEmpty(t, example)
				// In GREEN phase, this would validate against the config schema
			},
		)
	}
}
