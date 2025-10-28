package main

import (
	"os"

	"github.com/rs/zerolog"
	"github.com/spf13/cobra"
)

func main() {
	// Initialize logger
	//nolint:exhaustruct // ConsoleWriter has many optional fields
	logger := zerolog.New(zerolog.ConsoleWriter{Out: os.Stderr}).
		With().
		Timestamp().
		Logger()

	// Create root command
	rootCmd := &cobra.Command{
		Use:   "lithos",
		Short: "Lithos - A CLI tool for managing notes and templates",
		Long:  `Lithos is a CLI tool for managing notes, templates, and schemas in a vault-based system.`,
	}

	// Add subcommands here as they are implemented

	// Execute root command
	if err := rootCmd.Execute(); err != nil {
		logger.Fatal().Err(err).Msg("Command execution failed")
	}
}
