// Package main provides the entry point for the Lithos CLI application.
//
// Lithos is a tool for managing Obsidian vaults with schema-driven lookups,
// template rendering, and interactive input capabilities.
package main

import (
	"os"

	"github.com/JackMatanky/lithos/internal/adapters/api/cli"
	"github.com/JackMatanky/lithos/internal/adapters/spi/filesystem"
	templaterepo "github.com/JackMatanky/lithos/internal/adapters/spi/template"
	templatedomain "github.com/JackMatanky/lithos/internal/app/template"
)

func main() {
	// Create filesystem adapter
	fileSystemPort := filesystem.NewLocalFileSystemAdapter()

	// Create template parser and executor from domain services
	templateParser := templatedomain.NewStaticTemplateParser()
	templateExecutor := templatedomain.NewGoTemplateExecutor()

	// Create template engine with injected dependencies
	templateEngine := templatedomain.NewTemplateEngine(
		templateParser,
		templateExecutor,
	)

	// Create template repository adapter
	templateRepo := templaterepo.NewFSAdapter(
		fileSystemPort,
		templateParser,
	)

	// Create CLI adapter with injected dependencies
	adapter := cli.NewCobraCLIAdapter(
		templateEngine,
		templateRepo,
		fileSystemPort,
	)
	os.Exit(adapter.Execute(os.Args[1:]))
}
