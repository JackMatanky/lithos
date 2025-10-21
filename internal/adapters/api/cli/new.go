// Package cli provides CLI command implementations for the Lithos application.
// This file contains the implementation of the 'new' command for template
// processing.
package cli

import (
	"context"
	"fmt"

	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/spf13/cobra"
)

// NewCommand creates and returns the 'new' command for template processing.
// The command reads a template file, parses it, executes it, and writes to
// file.
func NewCommand(
	templateEngine *template.TemplateEngine,
	templateRepo spi.TemplateRepositoryPort,
	fileSystemPort spi.FileSystemPort,
) *cobra.Command {
	return &cobra.Command{
		Use:   "new <template-path>",
		Short: "Create a new item from a template",
		Long:  `Create a new item by reading and processing a template file.`,
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			return executeNewCommand(
				args[0],
				templateEngine,
				templateRepo,
				fileSystemPort,
			)
		},
	}
}

// executeNewCommand handles the core logic for the new command.
func executeNewCommand(
	templatePath string,
	templateEngine *template.TemplateEngine,
	templateRepo spi.TemplateRepositoryPort,
	fileSystemPort spi.FileSystemPort,
) error {
	ctx := context.Background()

	// Get parsed template from repository
	tmpl, err := templateRepo.GetByPath(ctx, templatePath)
	if err != nil {
		return fmt.Errorf(
			"failed to load template %q: %w",
			templatePath,
			err,
		)
	}

	// Execute parsed template
	renderedContent, err := templateEngine.ExecuteParsedTemplate(ctx, tmpl)
	if err != nil {
		return fmt.Errorf(
			"failed to execute template %q: %w",
			templatePath,
			err,
		)
	}

	// Write output file
	return writeOutputFile(tmpl.Name, renderedContent, fileSystemPort)
}

// writeOutputFile writes the rendered content to a markdown file.
func writeOutputFile(
	templateName string,
	content string,
	fileSystemPort spi.FileSystemPort,
) error {
	outputFileName := templateName + ".md"

	err := fileSystemPort.WriteFileAtomic(outputFileName, []byte(content))
	if err != nil {
		return fmt.Errorf(
			"failed to write output file %q: %w",
			outputFileName,
			err,
		)
	}

	fmt.Printf("Created %s\n", outputFileName)
	return nil
}
