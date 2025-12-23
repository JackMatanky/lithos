package performance

import (
	"context"
	"fmt"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/json"
	schemaadapter "github.com/JackMatanky/lithos/internal/adapters/spi/schema"
	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	schemaengine "github.com/JackMatanky/lithos/internal/app/schema"
	vaultService "github.com/JackMatanky/lithos/internal/app/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/require"
)

func TestComplexScaleIndexing(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping complex scale test in short mode")
	}

	// 1. Setup Workspace
	ws := utils.NewWorkspace(t)
	root := ws.Root()

	schemasDir := filepath.Join(root, "schemas")
	vaultDir := filepath.Join(root, "vault")
	cacheDir := filepath.Join(root, "cache")

	ws.MkdirAll("schemas", 0o750)
	ws.MkdirAll("vault", 0o750)
	ws.MkdirAll("cache", 0o750)

	// 2. Copy Complex Schemas
	schemaFiles := []string{"dir.json", "dir_contact.json", "task.json"}
	for _, f := range schemaFiles {
		utils.CopyFromTestdata(
			t,
			ws,
			filepath.Join("schemas", f),
			"vault",
			"schemas",
			f,
		)
	}
	utils.CopyFromTestdata(
		t,
		ws,
		"schemas/property_bank.json",
		"vault",
		"schemas",
		"property_bank.json",
	)

	// 3. Generate 1,000 Complex Notes
	// 500 Contacts
	ws.MkdirAll("vault/contacts", 0o750)
	for i := range 500 {
		filename := fmt.Sprintf("contact_%03d.md", i)
		uuid := fmt.Sprintf("550e8400-e29b-41d4-a716-%012d", i)
		content := fmt.Sprintf(`---
file_class: dir_contact
uuid: %s
title: Contact %d
name_first: Contact
name_last: %d
email_personal: contact%d@example.com
organization:
  - [[tech_corp]]
aliases:
  - Contact %d
---
# Contact %d
`, uuid, i, i, i, i, i)
		ws.WriteFile(
			filepath.Join("vault", "contacts", filename),
			[]byte(content),
			0o600,
		)
	}

	// 500 Tasks
	ws.MkdirAll("vault/tasks", 0o750)
	for i := range 500 {
		filename := fmt.Sprintf("task_%03d.md", i)
		uuid := fmt.Sprintf("661f8511-e30c-42d5-a817-%012d", i)
		content := fmt.Sprintf(`---
file_class: task
uuid: %s
title: Task %d
status: to_do
type: action_item
context: work
task_start: 2024-01-01
contact:
  - [[contact_%03d]]
---
# Task %d
`, uuid, i, i, i)
		ws.WriteFile(
			filepath.Join("vault", "tasks", filename),
			[]byte(content),
			0o600,
		)
	}

	// 4. Setup Indexer Stack
	config := domain.Config{
		VaultPath:        vaultDir,
		CacheDir:         cacheDir,
		SchemasDir:       schemasDir,
		PropertyBankFile: "property_bank.json",
	}
	logger := zerolog.New(zerolog.NewTestWriter(t)).With().Timestamp().Logger()

	vaultReader := vaultAdapter.NewVaultReaderAdapter(config, logger)
	boltCacheDir := filepath.Join(cacheDir, "bolt")
	sqliteCacheDir := filepath.Join(cacheDir, "sqlite")
	boltConfig := domain.Config{CacheDir: boltCacheDir}
	sqliteConfig := domain.Config{CacheDir: sqliteCacheDir}

	boltWriter := json.NewJSONCacheWriter(boltConfig, logger)
	sqliteWriter := json.NewJSONCacheWriter(sqliteConfig, logger)

	cacheReader := json.NewJSONCacheReader(boltConfig, logger)

	schemaLoader := schemaadapter.NewSchemaLoaderAdapter(&config, &logger)
	schemaRegistry := schemaadapter.NewSchemaRegistryAdapter(logger)
	schemaEngine, err := schemaengine.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		logger,
		nil,
	)
	require.NoError(t, err)

	markdownParser := vaultAdapter.NewMarkdownParserAdapter(logger)
	frontmatterService := frontmatter.NewFrontmatterService(
		schemaEngine,
		logger,
		nil,
	)

	indexer := vaultService.NewVaultIndexer(
		vaultReader,
		boltWriter,
		sqliteWriter,
		cacheReader,
		markdownParser,
		frontmatterService,
		schemaEngine,
		config,
		logger,
		nil,
	)

	// 5. Run Indexing
	ctx := context.Background()
	start := time.Now()
	stats, err := indexer.Build(ctx)
	duration := time.Since(start)

	// 6. Verify Results
	require.NoError(t, err)
	t.Logf("Indexed 1,000 realistic notes in %v", duration)
	t.Logf(
		"Stats: Scanned=%d, Indexed=%d",
		stats.ScannedCount,
		stats.IndexedCount,
	)

	// Should have scanned and indexed all files
	// 500 contacts + 500 tasks = 1000 notes
	require.Equal(t, 1000, stats.IndexedCount, "Should index all 1,000 notes")
}
