// Package cli provides CLI adapters for the hexagonal architecture.
//
// This package contains adapters that translate command-line interactions
// into domain requests and present results back to users.
package cli

import (
	"fmt"
	"os"

	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/spf13/cobra"
)

// CobraCLIAdapter implements a CLI adapter using the Cobra framework.
// It provides a structured command-line interface for the Lithos application.
type CobraCLIAdapter struct {
	rootCmd        *cobra.Command
	templateEngine *template.TemplateEngine
	templateRepo   spi.TemplateRepositoryPort
	fileSystemPort spi.FileSystemPort
}

// NewCobraCLIAdapter creates a new CobraCLIAdapter instance with
// the root command and subcommands configured.
func NewCobraCLIAdapter(
	templateEngine *template.TemplateEngine,
	templateRepo spi.TemplateRepositoryPort,
	fileSystemPort spi.FileSystemPort,
) *CobraCLIAdapter {
	adapter := &CobraCLIAdapter{
		rootCmd:        &cobra.Command{},
		templateEngine: templateEngine,
		templateRepo:   templateRepo,
		fileSystemPort: fileSystemPort,
	}
	adapter.setupCommands()
	return adapter
}

// Execute runs the CLI application with the provided command-line arguments.
// It returns an exit code suitable for use with os.Exit().
func (a *CobraCLIAdapter) Execute(args []string) int {
	a.rootCmd.SetArgs(args)

	if err := a.rootCmd.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		return 1
	}

	return 0
}

// setupCommands initializes the root command and registers all subcommands.
func (a *CobraCLIAdapter) setupCommands() {
	a.setupRootCommand()
	a.registerCommands()
}

// setupRootCommand configures the root command with basic metadata.
func (a *CobraCLIAdapter) setupRootCommand() {
	a.rootCmd = &cobra.Command{
		Use:   "lithos",
		Short: "Lithos - Obsidian vault management tool",
		Long: `Lithos is a command-line tool for managing Obsidian vaults with
schema-driven lookups, template rendering, and interactive input capabilities.`,
	}
}

// setupVersionCommand creates and returns the version command.
func (a *CobraCLIAdapter) setupVersionCommand() *cobra.Command {
	return &cobra.Command{
		Use:   "version",
		Short: "Print the version number of Lithos",
		Long:  `Print the version number of Lithos and exit.`,
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println("Lithos version 0.1.0")
		},
	}
}

// setupNewCommand creates and returns the new command.
func (a *CobraCLIAdapter) setupNewCommand() *cobra.Command {
	return NewCommand(a.templateEngine, a.templateRepo, a.fileSystemPort)
}

// registerCommands adds all subcommands to the root command.
func (a *CobraCLIAdapter) registerCommands() {
	a.rootCmd.AddCommand(a.setupVersionCommand())
	a.rootCmd.AddCommand(a.setupNewCommand())
}
