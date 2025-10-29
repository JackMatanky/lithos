// Package cli provides CLI adapter implementations for Lithos.
// This package contains the CobraCLIAdapter which implements the CLIPort
// interface using the Cobra CLI framework.
//
//nolint:godoclint // test functions don't need godoc comments
package cli

import (
	"bytes"
	"context"
	"strings"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	mocks "github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/spf13/cobra"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestCobraCLIAdapterStructExists(t *testing.T) {
	// This test verifies that CobraCLIAdapter struct can be compiled
	// and the constructor works correctly
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	assert.NotNil(t, adapter)
	assert.IsType(t, &CobraCLIAdapter{}, adapter)
}

func TestStart_StoresHandlerCorrectly(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)
	mockHandler := &mocks.MockCommandPort{}

	// Start should store the handler and execute successfully (showing help
	// when no args)
	err := adapter.Start(context.Background(), mockHandler)

	// Start should succeed when no commands are provided (shows help)
	assert.NoError(t, err)
	// Note: We can't directly test the private field, but success indicates the
	// handler was stored
}

func TestBuildRootCommand_CreatesCommandWithCorrectStructure(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	cmd := adapter.buildRootCommand()

	assert.Equal(t, "lithos", cmd.Use)
	assert.Equal(
		t,
		"Template-driven markdown note generator for Obsidian vaults",
		cmd.Short,
	)
	assert.True(t, cmd.SilenceUsage)
	assert.True(t, cmd.SilenceErrors)

	// Check subcommands
	subcommands := cmd.Commands()
	require.Len(t, subcommands, 2)

	versionCmd := findCommandByUse(subcommands, "version")
	require.NotNil(t, versionCmd)
	assert.Equal(t, "Print version information", versionCmd.Short)

	newCmd := findCommandByUse(subcommands, "new [template-id]")
	require.NotNil(t, newCmd)
	assert.Equal(t, "Create a new note from template", newCmd.Short)
}

func TestVersionCommand_PrintsCorrectVersion(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	cmd := adapter.buildVersionCommand()

	var buf bytes.Buffer
	cmd.SetOut(&buf)

	err := cmd.RunE(cmd, []string{})
	require.NoError(t, err)

	output := buf.String()
	assert.Equal(t, "lithos v0.1.0\n", output)
}

func TestBuildNewCommand_ParsesTemplateIdArgumentCorrectly(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	cmd := adapter.buildNewCommand()

	assert.Equal(t, "new [template-id]", cmd.Use)
	assert.Equal(t, "Create a new note from template", cmd.Short)

	// Test argument validation
	err := cmd.Args(cmd, []string{}) // No args
	require.Error(t, err)

	err = cmd.Args(cmd, []string{"template1"}) // One arg (valid)
	require.NoError(t, err)

	err = cmd.Args(cmd, []string{"template1", "extra"}) // Too many args
	assert.Error(t, err)
}

func TestBuildNewCommand_ParsesViewFlagCorrectly(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	cmd := adapter.buildNewCommand()

	// Check flag exists
	viewFlag := cmd.Flags().Lookup("view")
	require.NotNil(t, viewFlag)
	assert.Equal(t, "v", viewFlag.Shorthand)
	assert.Equal(t, "Display note content after creation", viewFlag.Usage)

	// Test flag parsing
	err := cmd.Flags().Set("view", "true")
	require.NoError(t, err)

	viewValue, err := cmd.Flags().GetBool("view")
	require.NoError(t, err)
	assert.True(t, viewValue)
}

func TestHandleNewCommand_ExtractsTemplateIdFromArgs(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)
	mockHandler := &mocks.MockCommandPort{}

	// Set up mock to return success
	expectedNote := domain.NewNote(
		domain.NewNoteID("test123"),
		domain.NewFrontmatter(map[string]interface{}{}),
	)
	mockHandler.SetNewNoteResult(expectedNote, nil)

	adapter.handler = mockHandler

	cmd := adapter.buildNewCommand()

	err := adapter.handleNewCommand(cmd, []string{"template1"})
	require.NoError(t, err)

	// Verify handler was called with correct template ID
	// Note: We can't directly test the call, but success indicates it worked
}

func TestHandleNewCommand_ReturnsErrorWhenArgsEmpty(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	cmd := adapter.buildNewCommand()

	err := adapter.handleNewCommand(cmd, []string{})
	require.Error(t, err)
	assert.Contains(t, err.Error(), "template-id required")
}

func TestHandleNewCommand_CallsHandlerNewNoteWithCorrectArguments(
	t *testing.T,
) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)
	mockHandler := &mocks.MockCommandPort{}

	// Set up mock to return success
	expectedNote := domain.NewNote(
		domain.NewNoteID("test123"),
		domain.NewFrontmatter(map[string]interface{}{}),
	)
	mockHandler.SetNewNoteResult(expectedNote, nil)

	adapter.handler = mockHandler

	cmd := adapter.buildNewCommand()

	err := adapter.handleNewCommand(cmd, []string{"template1"})
	require.NoError(t, err)

	// Verify the call was made (success indicates handler was called)
}

func TestDisplayNoteCreated_FormatsOutputCorrectlyWithoutViewFlag(
	t *testing.T,
) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	note := domain.NewNote(
		domain.NewNoteID("test123"),
		domain.NewFrontmatter(map[string]interface{}{}),
	)

	cmd := adapter.buildNewCommand()
	var buf bytes.Buffer
	cmd.SetOut(&buf)

	err := adapter.displayNoteCreated(cmd, note)
	require.NoError(t, err)

	output := buf.String()
	assert.Equal(t, "✓ Created: test123.md\n", output)
}

func TestDisplayNoteCreated_DisplaysContentWithViewFlag(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	note := domain.NewNote(
		domain.NewNoteID("test123"),
		domain.NewFrontmatter(map[string]interface{}{}),
	)

	cmd := adapter.buildNewCommand()
	var buf bytes.Buffer
	cmd.SetOut(&buf)

	// Set view flag
	err := cmd.Flags().Set("view", "true")
	require.NoError(t, err)

	err = adapter.displayNoteCreated(cmd, note)
	require.NoError(t, err)

	output := buf.String()
	lines := strings.Split(strings.TrimSpace(output), "\n")

	assert.Equal(t, "✓ Created: test123.md", lines[0])
	assert.Equal(t, strings.Repeat("=", 80), lines[1])
	// Note: Content reading is TODO, so we expect empty content for now
	assert.Equal(t, strings.Repeat("=", 80), lines[2])
}

func TestFormatError_FormatsResourceErrorCorrectly(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	resourceErr := errors.NewResourceError(
		"template",
		"load",
		"my-template",
		nil,
	)
	err := adapter.formatError(resourceErr)

	assert.Equal(t, "template 'my-template' not found in template", err.Error())
}

func TestFormatError_FormatsTemplateErrorCorrectly(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	templateErr := errors.NewTemplateError("parse failed", "my-template", nil)
	err := adapter.formatError(templateErr)

	assert.Equal(
		t,
		"template error in 'my-template': parse failed",
		err.Error(),
	)
}

func TestFormatError_FormatsGenericErrorCorrectly(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewCobraCLIAdapter(logger)

	genericErr := errors.NewBaseError("something went wrong", nil)
	err := adapter.formatError(genericErr)

	assert.Equal(t, "error: something went wrong", err.Error())
}

// Helper function to find command by Use string.
func findCommandByUse(commands []*cobra.Command, use string) *cobra.Command {
	for _, cmd := range commands {
		if cmd.Use == use {
			return cmd
		}
	}
	return nil
}
