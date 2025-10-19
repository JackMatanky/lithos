package main

import (
	"os"

	"github.com/JackMatanky/lithos/internal/adapters/api/cli"
	"github.com/JackMatanky/lithos/internal/adapters/spi/filesystem"
)

func main() {
	// Create FileSystemPort adapter
	fsAdapter := filesystem.NewLocalFileSystemAdapter()

	// Create CLI adapter with dependencies
	cliAdapter := cli.New(fsAdapter)

	// Execute CLI and exit with appropriate code
	exitCode := cliAdapter.Execute(os.Args[1:])
	os.Exit(exitCode)
}
