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

	// Initialize adapters
	schemaEngine, err := initSchemaEngine(ctx, cfg, log)
	if err != nil {
		return nil, nil, err
	}

	loadedSchemas := schemaEngine.SchemasSnapshot()

	// Initialize hybrid storage adapters
	storage, err := initStorage(cfg, log, loadedSchemas)
	if err != nil {
		return nil, nil, err
	}

	templateEngine := template.NewTemplateEngine(
		templateAdapter.NewTemplateLoaderAdapter(&cfg, &log),
		&cfg,
		&log,
	)

	markdownParser := vaultAdapter.NewMarkdownParserAdapter(log)
	frontmatterService := frontmatter.NewFrontmatterService(
		schemaEngine,
		markdownParser,
		log,
	)
	vaultIndexer := vault.NewVaultIndexer(
		vaultAdapter.NewVaultReaderAdapter(cfg, log),
		storage.boltWriter,
		storage.sqliteWriter,
		storage.cacheReader,
		markdownParser,
		frontmatterService,
		schemaEngine,
		cfg,
		log,
	)

	orchestrator := command.NewCLIComander(
		cli.NewCobraCLIAdapter(log),
		templateEngine,
		schemaEngine,
		vaultIndexer,
		vaultAdapter.NewVaultWriterAdapter(cfg, log),
		&cfg,
		&log,
	)

	return orchestrator, storage.cleanup, nil
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

func initSchemaEngine(
	ctx context.Context,
	cfg domain.Config,
	log zerolog.Logger,
) (*schema.SchemaEngine, error) {
	schemaLoader := schemaAdapter.NewSchemaLoaderAdapter(&cfg, &log)
	schemaRegistry := schemaAdapter.NewSchemaRegistryAdapter(log)
	schemaEngine, err := schema.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		log,
	)
	if err != nil {
		return nil, err
	}
	if loadErr := schemaEngine.Load(ctx); loadErr != nil {
		return nil, loadErr
	}
	return schemaEngine, nil
}
