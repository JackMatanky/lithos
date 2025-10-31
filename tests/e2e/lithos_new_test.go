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

	testutils "github.com/JackMatanky/lithos/tests/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestLithosNew_StaticTemplate tests lithos new with static template.
func TestLithosNew_StaticTemplate(t *testing.T) {
	// Setup workspace
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	ws.MkdirAll("templates", 0o750)
	ws.MkdirAll("schemas", 0o750)
	ws.MkdirAll(".cache", 0o750)

	// Create vault config
	configContent := fmt.Sprintf(`{
  "vault_path": "%s",
  "templates_dir": "templates",
  "schemas_dir": "schemas",
  "cache_dir": ".cache",
  "log_level": "info"
}`, tempDir)
	ws.WriteFile("lithos.json", []byte(configContent), 0o600)

	// Copy test template
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("templates", "static_template.md"),
		"templates",
		"static_template.md",
	)
	dstTemplate := ws.Path("templates", "static_template.md")

	// Copy required property bank
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("schemas", "property_bank.json"),
		"schemas",
		"property_bank.json",
	)

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

	// Execute: lithos new static_template
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"new",
		"static_template",
	)
	cmd.Dir = tempDir
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "command output: %s", output)

	// Verify: File exists
	notePath := ws.Path("static_template.md")
	require.FileExists(t, notePath)

	// Verify: Content matches expected
	content, err := os.ReadFile(notePath)
	require.NoError(t, err)

	// Verify template functions executed
	assert.Contains(t, string(content), time.Now().Format("2006-01-02"))
}

// TestLithosNew_BasicNote tests lithos new with basic note template.
func TestLithosNew_BasicNote(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	ws.MkdirAll("templates", 0o750)
	ws.MkdirAll("schemas", 0o750)

	// Copy test template
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("templates", "basic_note.md"),
		"templates",
		"basic_note.md",
	)

	// Copy required property bank
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("schemas", "property_bank.json"),
		"schemas",
		"property_bank.json",
	)

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

	// Execute: lithos new basic_note
	cmd = exec.CommandContext(
		context.Background(),
		binaryPath,
		"new",
		"basic_note",
	)
	cmd.Dir = tempDir
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "command output: %s", output)

	// Verify: File exists
	notePath := ws.Path("basic_note.md")
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
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	// Ensure templates directory exists but is empty
	ws.MkdirAll("templates", 0o750)

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
	err := cmd.Run()

	// Verify: Command fails with exit code 1
	require.Error(t, err)
	exitErr := &exec.ExitError{}
	if errors.As(err, &exitErr) {
		require.Equal(t, 1, exitErr.ExitCode())
	}
}

// TestLithosVersion tests the lithos version command.
func TestLithosVersion(t *testing.T) {
	ws := testutils.NewWorkspace(t)
	tempDir := ws.Root()

	// Setup minimal vault structure with property bank and a schema
	ws.MkdirAll("schemas", 0o750)
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("schemas", "property_bank.json"),
		"schemas",
		"property_bank.json",
	)
	testutils.CopyFromTestdata(
		t,
		ws,
		filepath.Join("schemas", "base_note.json"),
		"schemas",
		"base_note.json",
	)

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
	cmd.Dir = tempDir
	cmd.Env = append(os.Environ(), fmt.Sprintf("LITHOS_VAULT_PATH=%s", tempDir))
	cmd.Env = append(cmd.Env,
		fmt.Sprintf("LITHOS_SCHEMAS_DIR=%s", ws.Path("schemas")),
	)
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "command output: %s", output)

	// Verify: Output contains version
	outputStr := string(output)
	assert.Contains(t, outputStr, "lithos v0.1.0")
}
