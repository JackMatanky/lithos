// Package main provides the entry point for the Lithos CLI application.
//
// Lithos is a tool for managing Obsidian vaults with schema-driven lookups,
// template rendering, and interactive input capabilities.
package main

import (
	"os"

	"github.com/jack/lithos/internal/adapters/api/cli"
)

func main() {
	adapter := cli.NewCobraCLIAdapter()
	os.Exit(adapter.Execute(os.Args[1:]))
}
