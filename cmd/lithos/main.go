package main

import (
	"context"
	"os"

	"github.com/JackMatanky/lithos/internal/adapters/api/cli"
	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/boltdb"
	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/sqlite"
	"github.com/JackMatanky/lithos/internal/adapters/spi/config"
	schemaAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/schema"
	templateAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/template"
	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/app/command"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/rs/zerolog"
)

type storageResources struct {
	boltWriter   *boltdb.BoltDBCacheWriteAdapter
	sqliteWriter *sqlite.SQLiteWriterAdapter
	cacheReader  *boltdb.BoltDBCacheReadAdapter
	cleanup      func()
}

// Container manages dependency injection and service lifecycle.
type Container struct {
	config domain.Config
	logger zerolog.Logger

	// Services with lazy initialization
	schemaEngine       *schema.SchemaEngine
	storage            *storageResources
	templateEngine     *template.TemplateEngine
	markdownParser     *vaultAdapter.MarkdownParserAdapter
	frontmatterService *frontmatter.FrontmatterService
	vaultIndexer       *vault.VaultIndexer
	cliOrchestrator    *command.CLIComander
}

// NewContainer creates a new dependency injection container.
func NewContainer(cfg domain.Config, log zerolog.Logger) *Container {
	return &Container{
		config:             cfg,
		logger:             log,
		schemaEngine:       nil,
		storage:            nil,
		templateEngine:     nil,
		markdownParser:     nil,
		frontmatterService: nil,
		vaultIndexer:       nil,
		cliOrchestrator:    nil,
	}
}

// SchemaEngine returns the schema engine service with lazy initialization.
func (c *Container) SchemaEngine() (*schema.SchemaEngine, error) {
	if c.schemaEngine == nil {
		schemaLoader := schemaAdapter.NewSchemaLoaderAdapter(
			&c.config,
			&c.logger,
		)
		schemaRegistry := schemaAdapter.NewSchemaRegistryAdapter(c.logger)
		engine, err := schema.NewSchemaEngine(
			schemaLoader,
			schemaRegistry,
			c.logger,
		)
		if err != nil {
			return nil, err
		}
		if loadErr := engine.Load(context.Background()); loadErr != nil {
			return nil, loadErr
		}
		c.schemaEngine = engine
	}
	return c.schemaEngine, nil
}

// Storage returns the storage resources with lazy initialization.
func (c *Container) Storage() (*storageResources, error) {
	if c.storage == nil {
		storage, err := initStorage(
			c.config,
			c.logger,
			nil,
		) // schemas loaded separately
		if err != nil {
			return nil, err
		}
		c.storage = storage
	}
	return c.storage, nil
}

// TemplateEngine returns the template engine service with lazy initialization.
func (c *Container) TemplateEngine() *template.TemplateEngine {
	if c.templateEngine == nil {
		c.templateEngine = template.NewTemplateEngine(
			templateAdapter.NewTemplateLoaderAdapter(&c.config, &c.logger),
			&c.config,
			&c.logger,
		)
	}
	return c.templateEngine
}

// MarkdownParser returns the markdown parser service.
func (c *Container) MarkdownParser() *vaultAdapter.MarkdownParserAdapter {
	if c.markdownParser == nil {
		c.markdownParser = vaultAdapter.NewMarkdownParserAdapter(c.logger)
	}
	return c.markdownParser
}

// FrontmatterService returns the frontmatter service with lazy initialization.
func (c *Container) FrontmatterService() (*frontmatter.FrontmatterService, error) {
	if c.frontmatterService == nil {
		schemaEngine, err := c.SchemaEngine()
		if err != nil {
			return nil, err
		}
		c.frontmatterService = frontmatter.NewFrontmatterService(
			schemaEngine,
			c.MarkdownParser(),
			c.logger,
		)
	}
	return c.frontmatterService, nil
}

// VaultIndexer returns the vault indexer service with lazy initialization.
func (c *Container) VaultIndexer() (*vault.VaultIndexer, error) {
	if c.vaultIndexer != nil {
		return c.vaultIndexer, nil
	}

	storage, err := c.Storage()
	if err != nil {
		return nil, err
	}

	frontmatterSvc, err := c.FrontmatterService()
	if err != nil {
		return nil, err
	}

	schemaEngine, err := c.SchemaEngine()
	if err != nil {
		return nil, err
	}

	c.vaultIndexer = vault.NewVaultIndexer(
		vaultAdapter.NewVaultReaderAdapter(c.config, c.logger),
		storage.boltWriter,
		storage.sqliteWriter,
		storage.cacheReader,
		c.MarkdownParser(),
		frontmatterSvc,
		schemaEngine,
		c.config,
		c.logger,
	)
	return c.vaultIndexer, nil
}

// CLIOrchestrator returns the CLI orchestrator service with lazy
// initialization.
func (c *Container) CLIOrchestrator() (*command.CLIComander, error) {
	if c.cliOrchestrator == nil {
		vaultIndexer, err := c.VaultIndexer()
		if err != nil {
			return nil, err
		}

		c.cliOrchestrator = command.NewCLIComander(
			cli.NewCobraCLIAdapter(c.logger),
			c.TemplateEngine(),
			nil, // schemaEngine - can be added if needed
			vaultIndexer,
			vaultAdapter.NewVaultWriterAdapter(c.config, c.logger),
			&c.config,
			&c.logger,
		)
	}
	return c.cliOrchestrator, nil
}

// Cleanup releases all resources managed by the container.
func (c *Container) Cleanup() {
	if c.storage != nil {
		c.storage.cleanup()
	}
}

func main() {
	if err := run(context.Background()); err != nil {
		log := logger.New(os.Stdout, "info")
		log.Fatal().Err(err).Msg("application failed")
	}
}

func run(ctx context.Context) error {
	log := logger.New(os.Stdout, "info")
	orchestrator, cleanup, err := buildOrchestrator(ctx, log)
	if err != nil {
		return err
	}
	defer cleanup()

	return orchestrator.Run(ctx)
}

func buildOrchestrator(
	ctx context.Context,
	log zerolog.Logger,
) (*command.CLIComander, func(), error) {
	cfg, err := config.NewViperAdapter(log).Load(ctx)
	if err != nil {
		return nil, nil, err
	}
	log = logger.New(os.Stdout, cfg.LogLevel)

	// Use dependency injection container
	container := NewContainer(cfg, log)

	// Get the CLI orchestrator from the container
	orchestrator, err := container.CLIOrchestrator()
	if err != nil {
		return nil, nil, err
	}

	return orchestrator, container.Cleanup, nil
}

func initStorage(
	cfg domain.Config,
	log zerolog.Logger,
	schemas []domain.Schema,
) (*storageResources, error) {
	boltDB, err := boltdb.Open(cfg)
	if err != nil {
		return nil, err
	}

	cleanup := func() {
		_ = boltDB.Close()
	}

	boltWriter, err := boltdb.NewBoltDBCacheWriter(cfg, log, boltDB)
	if err != nil {
		cleanup()
		return nil, err
	}

	viewMigrator := sqlite.NewSchemaViewMigrator(schemas, cfg.FileClassKey, log)

	sqliteWriter, err := sqlite.NewSQLiteWriterAdapter(cfg, log, viewMigrator)
	if err != nil {
		cleanup()
		return nil, err
	}

	baseCleanup := cleanup
	cleanup = func() {
		_ = sqliteWriter.Close()
		baseCleanup()
	}

	cacheReader, err := boltdb.NewBoltDBCacheReadAdapter(cfg, log, boltDB)
	if err != nil {
		cleanup()
		return nil, err
	}

	return &storageResources{
		boltWriter:   boltWriter,
		sqliteWriter: sqliteWriter,
		cacheReader:  cacheReader,
		cleanup:      cleanup,
	}, nil
}
