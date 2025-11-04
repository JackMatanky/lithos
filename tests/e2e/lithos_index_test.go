package e2e

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestEndToEndCLIWorkflow tests the complete `lithos index` command workflow.
func TestEndToEndCLIWorkflow(t *testing.T) {
	// Setup workspace
	ws := utils.NewWorkspace(t)
	tempDir := ws.Root()
	vaultDir := filepath.Join(tempDir, "vault")

	// Create complex test vault with all edge cases
	createComplexTestVault(t, vaultDir)

	// Copy schemas to a location the CLI can find during startup
	ws.MkdirAll("schemas", 0o755)
	testSchemasDir := ws.Path("schemas")

	// Ensure test schemas exist
	ws.MkdirAll("schemas", 0o755)

	srcPropertyBank := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Set environment variables before building
	require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
	defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

	// Build lithos binary
	binaryPath := utils.BuildLithosBinary(t, tempDir)

	// Execute index command
	output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)
	require.NoError(t, err, "CLI index command should succeed")

	// Verify output contains expected elements
	assert.Contains(t, output, "✓ Vault indexed successfully")
	assert.Contains(t, output, "Statistics:")
	assert.Contains(t, output, "Scanned:")
	assert.Contains(t, output, "Indexed:")
	assert.Contains(t, output, "Duration:")

	// Parse and verify statistics
	utils.VerifyStatistics(t, output)
}

// createComplexTestVault creates a test vault with all edge cases for
// comprehensive testing.
func createComplexTestVault(t *testing.T, vaultDir string) {
	t.Helper()

	// Create directory structure with nested folders
	dirs := []string{
		"projects/active",
		"projects/archive",
		"ideas/brainstorm",
		"meetings/2025",
		"meetings/2024",
		"templates",
		"assets/images",
		"assets/documents",
	}

	for _, dir := range dirs {
		require.NoError(
			t,
			os.MkdirAll(filepath.Join(vaultDir, dir), 0o755),
		)
	}

	// Create files with duplicate basenames across directories
	duplicateBasenameContent := []struct {
		dir      string
		filename string
		title    string
		content  string
	}{
		{
			"projects/active",
			"meeting.md",
			"Active Project Meeting",
			"Content for active project",
		},
		{
			"projects/archive",
			"meeting.md",
			"Archived Project Meeting",
			"Content for archived project",
		},
		{
			"ideas/brainstorm",
			"meeting.md",
			"Brainstorm Meeting",
			"Content for brainstorm",
		},
		{"meetings/2025", "meeting.md", "2025 Meeting", "Content for 2025"},
		{"meetings/2024", "meeting.md", "2024 Meeting", "Content for 2024"},
	}

	for _, item := range duplicateBasenameContent {
		content := fmt.Sprintf(
			`---\ntitle: \"%s\"\ndate: \"2025-01-01\"\n---\n\n# %s\n\n%s\n`,
			item.title,
			item.title,
			item.content,
		)

		path := filepath.Join(vaultDir, item.dir, item.filename)
		require.NoError(
			t,
			os.WriteFile(path, []byte(content), 0o644),
		)
	}

	// Create files with different extensions and types
	mixedFiles := []struct {
		dir      string
		filename string
		content  string
	}{
		{
			"templates",
			"note-template.md",
			"# Note Template\\n\\nTemplate content",
		},
		{
			"templates",
			"meeting-template.md",
			"# Meeting Template\\n\\nMeeting template content",
		},
		{"assets/documents", "readme.txt", "This is a text document"},
		{"assets/documents", "data.json", `{"key": "value", "number": 42}`},
	}

	for _, item := range mixedFiles {
		path := filepath.Join(vaultDir, item.dir, item.filename)
		require.NoError(
			t,
			os.WriteFile(path, []byte(item.content), 0o644),
		)
	}

	// Create large binary file to test memory efficiency (1MB)
	largeBinaryPath := filepath.Join(
		vaultDir,
		"assets/images", //nolint:gocritic // filepath.Join with subpath strings is acceptable
		"large-image.jpg",
	)
	largeData := make([]byte, 1024*1024) // 1MB
	for i := range largeData {
		largeData[i] = byte(i % 256)
	}
	require.NoError(
		t,
		os.WriteFile(largeBinaryPath, largeData, 0o644),
	)

	// Create file with invalid frontmatter for error handling
	invalidFrontmatterPath := filepath.Join(
		vaultDir,
		"projects/active", //nolint:gocritic // filepath.Join with subpath strings is acceptable
		"invalid.md",
	)
	invalidContent := `
		---
		invalid: yaml: content:
		---
		# Invalid Frontmatter

		This file has invalid YAML frontmatter.
	`
	require.NoError(
		t,
		os.WriteFile(invalidFrontmatterPath, []byte(invalidContent), 0o644),
	)

	// Create file without frontmatter
	noFrontmatterPath := filepath.Join(
		vaultDir,
		"ideas/brainstorm", //nolint:gocritic // filepath.Join with subpath strings is acceptable
		"plain.md",
	)
	plainContent := `# Plain Markdown\n\nThis file has no frontmatter.`
	require.NoError(
		t,
		os.WriteFile(noFrontmatterPath, []byte(plainContent), 0o644),
	)
}
