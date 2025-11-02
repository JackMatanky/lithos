package main

import (
	"context"
	"os"

	"github.com/JackMatanky/lithos/internal/adapters/api/cli"
	"github.com/JackMatanky/lithos/internal/adapters/spi/cache"
	"github.com/JackMatanky/lithos/internal/adapters/spi/config"
	schemaAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/schema"
	templateAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/template"
	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/app/command"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/shared/logger"
)

func main() {
	ctx := context.Background()
	log := logger.New(os.Stdout, "info")
	configAdapter := config.NewViperAdapter(log)
	cfg, err := configAdapter.Load(ctx)
	if err != nil {
		log.Fatal().Err(err).Msg("failed to load configuration")
	}
	log = logger.New(os.Stdout, cfg.LogLevel)
	templateLoader := templateAdapter.NewTemplateLoaderAdapter(&cfg, &log)
	schemaLoader := schemaAdapter.NewSchemaLoaderAdapter(&cfg, &log)
	schemaRegistry := schemaAdapter.NewSchemaRegistryAdapter(log)
	schemaEngine, err := schema.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		log,
	)
	if err != nil {
		log.Fatal().Err(err).Msg("failed to create schema engine")
	}
	if loadErr := schemaEngine.Load(ctx); loadErr != nil {
		log.Fatal().Err(loadErr).Msg("failed to load schemas")
	}
	templateEngine := template.NewTemplateEngine(templateLoader, &cfg, &log)
	vaultScanner := vaultAdapter.NewVaultReaderAdapter(cfg, log)
	cacheWriter := cache.NewJSONCacheWriter(cfg, log)
	cacheReader := cache.NewJSONCacheReader(cfg, log)
	frontmatterService := frontmatter.NewFrontmatterService(schemaEngine, log)
	vaultIndexer := vault.NewVaultIndexer(
		vaultScanner,
		cacheWriter,
		cacheReader,
		frontmatterService,
		schemaEngine,
		cfg,
		log,
	)
	vaultWriter := vaultAdapter.NewVaultWriterAdapter(cfg, log)
	cliAdapter := cli.NewCobraCLIAdapter(log)
	orchestrator := command.NewCommandOrchestrator(
		cliAdapter,
		templateEngine,
		schemaEngine,
		vaultIndexer,
		vaultWriter,
		&cfg,
		&log,
	)
	if runErr := orchestrator.Run(ctx); runErr != nil {
		log.Fatal().Err(runErr).Msg("application failed")
	}
}
