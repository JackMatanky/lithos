// Package cli provides CLI adapter implementations for Lithos.
// This package contains the CobraCLIAdapter which implements the CLIPort
// interface using the Cobra CLI framework.
//
//nolint:godoclint // package godoc is sufficient
package cli

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/api"
	domainErrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"github.com/spf13/cobra"
)

const separatorWidth = 80

// CobraCLIAdapter implements the CLIPort interface using Cobra CLI framework.
// It provides a command-line interface for Lithos with root, version, and new
// commands.
type CobraCLIAdapter struct {
	handler api.CommandPort
	log     zerolog.Logger
}

// NewCobraCLIAdapter creates a new CobraCLIAdapter with the provided logger.
// The adapter implements hexagonal architecture by accepting a CommandPort
// handler
// during Start() to delegate business logic execution.
//
//nolint:gocritic // zerolog.Logger is heavy but follows project pattern
func NewCobraCLIAdapter(
	log zerolog.Logger,
) *CobraCLIAdapter {
	return &CobraCLIAdapter{
		handler: nil, // Will be set during Start()
		log:     log,
	}
}

// Start begins the CLI event loop and command processing.
// The CLI adapter receives the CommandPort handler (typically
// CommandOrchestrator)
// and uses it to delegate business logic execution.
//
// The adapter is responsible for:
// - Setting up command definitions and flags
// - Parsing command-line arguments
// - Handling user input validation
// - Formatting and displaying results
// - Error handling and exit codes
//
// The handler is responsible for:
// - Executing business logic
// - Orchestrating domain services
// - Returning domain objects/errors
//
// Parameters:
//   - ctx: Context for cancellation and timeout control
//   - handler: Domain service implementing CommandPort for business logic
//
// Returns:
//   - error: Any startup or execution errors from the CLI framework
func (a *CobraCLIAdapter) Start(
	ctx context.Context,
	handler api.CommandPort,
) error {
	a.handler = handler
	rootCmd := a.buildRootCommand()
	return rootCmd.ExecuteContext(ctx)
}

// buildRootCommand creates the root command with subcommands.
// This method follows SRP by focusing solely on root command structure.
func (a *CobraCLIAdapter) buildRootCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:           "lithos",
		Short:         "Template-driven markdown note generator for Obsidian vaults",
		SilenceUsage:  true,
		SilenceErrors: true,
	}
	cmd.AddCommand(a.buildVersionCommand())
	cmd.AddCommand(a.buildNewCommand())
	return cmd
}

// buildVersionCommand creates the version subcommand.
// This method follows SRP by focusing solely on version command creation.
func (a *CobraCLIAdapter) buildVersionCommand() *cobra.Command {
	return &cobra.Command{
		Use:   "version",
		Short: "Print version information",
		RunE: func(cmd *cobra.Command, args []string) error {
			cmd.Println("lithos v0.1.0")
			return nil
		},
	}
}

// buildNewCommand creates the new subcommand with flags.
// This method follows SRP by focusing solely on new command structure.
func (a *CobraCLIAdapter) buildNewCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "new [template-id]",
		Short: "Create a new note from template",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			return a.handleNewCommand(cmd, args)
		},
	}
	cmd.Flags().BoolP("view", "v", false, "Display note content after creation")
	return cmd
}

// handleNewCommand orchestrates the note creation workflow.
// This method follows SRP by coordinating the new command execution.
func (a *CobraCLIAdapter) handleNewCommand(
	cmd *cobra.Command,
	args []string,
) error {
	if len(args) == 0 {
		return fmt.Errorf("template-id required")
	}

	templateID := domain.NewTemplateID(args[0])
	note, err := a.handler.NewNote(cmd.Context(), templateID)
	if err != nil {
		return a.formatError(err)
	}

	return a.displayNoteCreated(cmd, note)
}

// displayNoteCreated formats and displays the note creation success message.
// This method follows SRP by focusing solely on output formatting.
func (a *CobraCLIAdapter) displayNoteCreated(
	cmd *cobra.Command,
	note domain.Note,
) error {
	cmd.Printf("✓ Created: %s.md\n", note.ID)

	if viewFlag, _ := cmd.Flags().GetBool("view"); viewFlag {
		cmd.Println(strings.Repeat("=", separatorWidth))
		// TODO: Read and display note content
		cmd.Println(strings.Repeat("=", separatorWidth))
	}

	return nil
}

// formatError converts domain errors to user-friendly CLI error messages.
// This method follows SRP by focusing solely on error formatting.
func (a *CobraCLIAdapter) formatError(err error) error {
	var resourceErr *domainErrors.ResourceError
	if errors.As(err, &resourceErr) {
		return fmt.Errorf(
			"template '%s' not found in %s",
			resourceErr.Target(),
			resourceErr.Resource(),
		)
	}

	var templateErr *domainErrors.TemplateError
	if errors.As(err, &templateErr) {
		return fmt.Errorf(
			"template error in '%s': %s",
			templateErr.TemplateID(),
			templateErr.Error(),
		)
	}

	return fmt.Errorf("error: %s", err.Error())
}
