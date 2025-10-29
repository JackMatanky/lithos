package e2e

import (
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestLithosNew_StaticTemplate tests lithos new with static template.
func TestLithosNew_StaticTemplate(t *testing.T) {
	// Setup: Create temp vault
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	templatesDir := filepath.Join(tempDir, "templates")
	require.NoError(t, os.MkdirAll(templatesDir, 0o750))

	// Create vault config
	configContent := fmt.Sprintf(`{
  "vault_path": "%s",
  "templates_dir": "templates",
  "schemas_dir": "schemas",
  "cache_dir": ".cache",
  "log_level": "info"
}`, tempDir)
	configPath := filepath.Join(tempDir, "lithos.json")
	require.NoError(t, os.WriteFile(configPath, []byte(configContent), 0o600))

	// Create cache and schemas directories
	cacheDir := filepath.Join(tempDir, ".cache")
	require.NoError(t, os.MkdirAll(cacheDir, 0o750))
	schemasDir := filepath.Join(tempDir, "schemas")
	require.NoError(t, os.MkdirAll(schemasDir, 0o750))

	// Copy test template
	testDataPath := filepath.Join("..", "..", "testdata")
	templatePath := filepath.Join("templates", "static-template.md")
	srcTemplate := filepath.Join(testDataPath, templatePath)
	dstTemplate := filepath.Join(templatesDir, "static-template.md")
	copyFile(t, srcTemplate, dstTemplate)

	// Verify template file exists
	require.FileExists(t, dstTemplate)

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos new static-template
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"new",
		"static-template",
	)
	cmd.Dir = tempDir
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "command output: %s", output)

	// Verify: File exists
	notePath := filepath.Join(tempDir, "static-template.md")
	require.FileExists(t, notePath)

	// Verify: Content matches expected
	content, err := os.ReadFile(notePath)
	require.NoError(t, err)

	// Verify template functions executed
	assert.Contains(t, string(content), time.Now().Format("2006-01-02"))
}

// TestLithosNew_BasicNote tests lithos new with basic note template.
func TestLithosNew_BasicNote(t *testing.T) {
	// Setup: Create temp vault
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	templatesDir := filepath.Join(tempDir, "templates")
	require.NoError(t, os.MkdirAll(templatesDir, 0o750))

	// Copy test template
	testDataPath2 := filepath.Join("..", "..", "testdata")
	templatePath2 := filepath.Join("templates", "basic-note.md")
	srcTemplate := filepath.Join(testDataPath2, templatePath2)
	dstTemplate := filepath.Join(templatesDir, "basic-note.md")
	copyFile(t, srcTemplate, dstTemplate)

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos new basic-note
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"new",
		"basic-note",
	)
	cmd.Dir = tempDir
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "command output: %s", output)

	// Verify: File exists
	notePath := filepath.Join(tempDir, "basic-note.md")
	require.FileExists(t, notePath)

	// Verify: Content contains template functions
	content, err := os.ReadFile(notePath)
	require.NoError(t, err)

	contentStr := string(content)
	assert.Contains(
		t,
		contentStr,
		time.Now().Format("2006-01-02"),
	) // now() function
	assert.Contains(
		t,
		contentStr,
		"unknown",
	) // toLower applied to default "Unknown"
	assert.Contains(
		t,
		contentStr,
		"PERSONAL",
	) // toUpper applied to default "personal"
}

// TestLithosNew_TemplateNotFound tests error when template not found.
func TestLithosNew_TemplateNotFound(t *testing.T) {
	// Setup: Create temp vault with no templates
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Ensure templates directory exists but is empty
	templatesDir := filepath.Join(tempDir, "templates")
	require.NoError(t, os.MkdirAll(templatesDir, 0o750))

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos new nonexistent
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"new",
		"nonexistent",
	)
	cmd.Dir = tempDir
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	err = cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	exitErr := &exec.ExitError{}
	if errors.As(err, &exitErr) {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithosVersion tests the lithos version command.
func TestLithosVersion(t *testing.T) {
	// Setup: Create temp directory for binary
	tempDir, err := os.MkdirTemp("", "lithos-e2e-*")
	require.NoError(t, err)
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Build binary
	binaryPath := filepath.Join(tempDir, "lithos")
	cmd := exec.CommandContext(
		context.Background(),
		"go",
		"build",
		"-mod=readonly",
		"-o",
		binaryPath,
		"../../cmd/lithos",
	)
	require.NoError(t, cmd.Run())

	// Execute: lithos version
	cmd = exec.CommandContext(context.Background(), binaryPath, "version")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err)

	// Verify: Output contains version
	outputStr := string(output)
	assert.Contains(t, outputStr, "lithos v0.1.0")
}

// copyFile is a helper function to copy files during test setup.
func copyFile(t *testing.T, src, dst string) {
	t.Helper()

	srcFile, err := os.Open(src)
	require.NoError(t, err)
	defer func() { _ = srcFile.Close() }()

	dstFile, err := os.Create(dst)
	require.NoError(t, err)
	defer func() { _ = dstFile.Close() }()

	_, err = dstFile.ReadFrom(srcFile)
	require.NoError(t, err)
}
