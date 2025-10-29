package main

import (
	"context"
	"os"

	"github.com/JackMatanky/lithos/internal/adapters/api/cli"
	"github.com/JackMatanky/lithos/internal/adapters/spi/config"
	templateAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/template"
	"github.com/JackMatanky/lithos/internal/app/command"
	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/shared/logger"
)

func main() {
	ctx := context.Background()

	// Layer 1: Infrastructure Layer
	// Initialize logger with default level (info)
	log := logger.New(os.Stdout, "info")

	// Create config adapter
	configAdapter := config.NewViperAdapter(log)

	// Load config with fatal error handling
	cfg, err := configAdapter.Load(ctx)
	if err != nil {
		log.Fatal().Err(err).Msg("failed to load configuration")
	}

	// Update logger from config
	log = logger.New(os.Stdout, cfg.LogLevel)

	// Layer 2: SPI Adapters
	// Create TemplateLoaderAdapter
	templateLoader := templateAdapter.NewTemplateLoaderAdapter(&cfg, &log)

	// Layer 3: Domain Services
	// Create TemplateEngine
	templateEngine := template.NewTemplateEngine(templateLoader, &cfg, &log)

	// Layer 4: API Adapters
	// Create CobraCLIAdapter
	cliAdapter := cli.NewCobraCLIAdapter(log)

	// Layer 5: CommandOrchestrator
	// Create CommandOrchestrator
	orchestrator := command.NewCommandOrchestrator(
		cliAdapter,
		templateEngine,
		&cfg,
		&log,
	)

	// Start application
	if runErr := orchestrator.Run(ctx); runErr != nil {
		log.Fatal().Err(runErr).Msg("application failed")
	}
}
