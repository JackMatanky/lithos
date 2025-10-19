// Package cli provides CLI command implementations for the Lithos application.
// This file contains the implementation of the 'new' command for template
// processing.
package cli

import (
	"context"
	"fmt"
	"path/filepath"

	"github.com/jack/lithos/internal/app/template"
	"github.com/jack/lithos/internal/domain"
	"github.com/jack/lithos/internal/ports/spi"
	"github.com/jack/lithos/internal/shared/errors"
	"github.com/spf13/cobra"
)

// NewCommand creates and returns the 'new' command for template processing.
// The command reads a template file, parses it, executes it, and writes to
// file.
func NewCommand(fileSystemPort spi.FileSystemPort) *cobra.Command {
	return &cobra.Command{
		Use:   "new <template-path>",
		Short: "Create a new item from a template",
		Long:  `Create a new item by reading and processing a template file.`,
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			return executeNewCommand(args[0], fileSystemPort)
		},
	}
}

// executeNewCommand handles the core logic for the new command.
func executeNewCommand(
	templatePath string,
	fileSystemPort spi.FileSystemPort,
) error {
	ctx := context.Background()

	// Read and parse template
	tmpl, err := readAndParseTemplate(ctx, templatePath, fileSystemPort)
	if err != nil {
		return err
	}

	// Execute template
	renderedContent, err := executeTemplate(ctx, tmpl)
	if err != nil {
		return err
	}

	// Write output file
	return writeOutputFile(tmpl.Name, renderedContent, fileSystemPort)
}

// readAndParseTemplate reads template content and parses it.
func readAndParseTemplate(
	ctx context.Context,
	templatePath string,
	fileSystemPort spi.FileSystemPort,
) (*domain.Template, error) {
	// Read the template file
	content, err := fileSystemPort.ReadFile(templatePath)
	if err != nil {
		return nil, fmt.Errorf(
			"failed to read template file %q: %w",
			templatePath,
			err,
		)
	}

	// Parse the template
	parser := template.NewStaticTemplateParser()
	parseResult := parser.Parse(ctx, string(content))
	if parseResult.IsErr() {
		return nil, errors.Wrap(
			parseResult.Error(),
			"failed to parse template",
		)
	}

	// Extract template name from path
	templateName := filepath.Base(templatePath)
	if ext := filepath.Ext(templateName); ext != "" {
		templateName = templateName[:len(templateName)-len(ext)]
	}

	return &domain.Template{
		FilePath: templatePath,
		Name:     templateName,
		Content:  string(content),
		Parsed:   parseResult.Value(),
	}, nil
}

// executeTemplate renders the template with the executor.
func executeTemplate(
	ctx context.Context,
	tmpl *domain.Template,
) (string, error) {
	executor := template.NewGoTemplateExecutor()
	renderedContent, err := executor.Execute(ctx, tmpl, nil)
	if err != nil {
		return "", errors.Wrap(err, "failed to execute template")
	}
	return renderedContent, nil
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
