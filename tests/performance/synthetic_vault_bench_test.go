package performance

import (
	"context"
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

// BenchmarkSmallVault tests indexing performance on 250 notes.
func BenchmarkSmallVault(b *testing.B) {
	config := utils.SyntheticVaultConfig{
		ContactCount:      50,
		TaskCount:         100,
		OrganizationCount: 20,
		MeetingCount:      50,
		NoteCount:         30,
	}
	benchmarkVaultIndexing(b, config)
}

// BenchmarkMediumVault tests indexing performance on 2,500 notes.
func BenchmarkMediumVault(b *testing.B) {
	config := utils.SyntheticVaultConfig{
		ContactCount:      500,
		TaskCount:         1000,
		OrganizationCount: 200,
		MeetingCount:      500,
		NoteCount:         300,
	}
	benchmarkVaultIndexing(b, config)
}

// BenchmarkLargeVault tests indexing performance on 10,000 notes.
func BenchmarkLargeVault(b *testing.B) {
	if testing.Short() {
		b.Skip("skipping large vault benchmark in short mode")
	}
	benchmarkVaultIndexing(b, utils.DefaultLargeVaultConfig())
}

// BenchmarkMassiveVault tests indexing performance on 100,000 notes.
func BenchmarkMassiveVault(b *testing.B) {
	if testing.Short() {
		b.Skip("skipping massive vault benchmark in short mode")
	}
	benchmarkVaultIndexing(b, utils.MassiveVaultConfig())
}

func benchmarkVaultIndexing(b *testing.B, config utils.SyntheticVaultConfig) {
	// Setup (outside timer)
	ws := utils.NewWorkspaceBench(b)
	root := ws.Root()

	schemasDir := filepath.Join(root, "schemas")
	vaultDir := filepath.Join(root, "vault")
	cacheDir := filepath.Join(root, "cache")

	// Generate synthetic vault
	b.Logf("Generating synthetic vault with config: %+v", config)
	utils.GenerateSyntheticVaultBench(b, ws, config)

	totalNotes := config.ContactCount + config.TaskCount +
		config.OrganizationCount + config.MeetingCount + config.NoteCount
	b.Logf("Generated %d total notes", totalNotes)

	// Setup indexer stack
	domainConfig := domain.Config{
		VaultPath:        vaultDir,
		CacheDir:         cacheDir,
		SchemasDir:       schemasDir,
		PropertyBankFile: "property_bank.json",
		FileClassKey:     "file_class",
	}

	cfgInstance := domainConfig
	domain.SetInstanceForTesting(&cfgInstance)
	b.Cleanup(domain.ResetConfigForTesting)

	logger := zerolog.Nop() // Silent logger for benchmarks

	vaultReader := vaultAdapter.NewVaultReaderAdapter(domainConfig, logger)
	boltCacheDir := filepath.Join(cacheDir, "bolt")
	sqliteCacheDir := filepath.Join(cacheDir, "sqlite")
	boltConfig := domain.Config{CacheDir: boltCacheDir}
	sqliteConfig := domain.Config{CacheDir: sqliteCacheDir}

	boltWriter := json.NewJSONCacheWriter(boltConfig, logger)
	sqliteWriter := json.NewJSONCacheWriter(sqliteConfig, logger)
	cacheReader := json.NewJSONCacheReader(boltConfig, logger)

	schemaLoader := schemaadapter.NewSchemaLoaderAdapter(&domainConfig, &logger)
	schemaRegistry := schemaadapter.NewSchemaRegistryAdapter(logger)
	schemaEngine, err := schemaengine.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		logger,
		nil,
	)
	require.NoError(b, err)

	markdownParser := vaultAdapter.NewMarkdownParserAdapter(logger)
	frontmatterService := frontmatter.NewFrontmatterService(
		schemaEngine,
		logger,
		nil,
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
		domainConfig,
		logger,
		nil,
	)

	ctx := context.Background()

	// Reset timer before actual benchmark
	b.ResetTimer()

	// Run benchmark
	for range b.N {
		start := time.Now()
		stats, buildErr := indexer.Build(ctx)
		duration := time.Since(start)

		require.NoError(b, buildErr)
		require.Equal(
			b,
			totalNotes,
			stats.IndexedCount,
			"Should index all notes",
		)

		// Calculate throughput
		notesPerSec := float64(stats.IndexedCount) / duration.Seconds()
		b.ReportMetric(notesPerSec, "notes/sec")
		b.ReportMetric(float64(duration.Milliseconds()), "ms/total")
		b.ReportMetric(
			duration.Seconds()/float64(stats.IndexedCount)*1000,
			"ms/note",
		)
	}
}

// TestLargeVaultPerformance is a non-benchmark test that validates performance
// targets.
func TestLargeVaultPerformance(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping large vault performance test in short mode")
	}

	ws := utils.NewWorkspace(t)
	root := ws.Root()

	schemasDir := filepath.Join(root, "schemas")
	vaultDir := filepath.Join(root, "vault")
	cacheDir := filepath.Join(root, "cache")

	// Generate 10,000 note vault
	config := utils.DefaultLargeVaultConfig()
	t.Logf("Generating large vault with %d notes...",
		config.ContactCount+config.TaskCount+config.OrganizationCount+
			config.MeetingCount+config.NoteCount)

	utils.GenerateSyntheticVault(t, ws, config)

	totalNotes := config.ContactCount + config.TaskCount +
		config.OrganizationCount + config.MeetingCount + config.NoteCount

	// Setup indexer
	domainConfig := domain.Config{
		VaultPath:        vaultDir,
		CacheDir:         cacheDir,
		SchemasDir:       schemasDir,
		PropertyBankFile: "property_bank.json",
		FileClassKey:     "file_class",
	}
	cfgInstance := domainConfig
	domain.SetInstanceForTesting(&cfgInstance)
	t.Cleanup(domain.ResetConfigForTesting)

	logger := zerolog.New(zerolog.NewTestWriter(t)).With().Timestamp().Logger()

	vaultReader := vaultAdapter.NewVaultReaderAdapter(domainConfig, logger)
	boltCacheDir := filepath.Join(cacheDir, "bolt")
	sqliteCacheDir := filepath.Join(cacheDir, "sqlite")
	boltConfig := domain.Config{CacheDir: boltCacheDir}
	sqliteConfig := domain.Config{CacheDir: sqliteCacheDir}

	boltWriter := json.NewJSONCacheWriter(boltConfig, logger)
	sqliteWriter := json.NewJSONCacheWriter(sqliteConfig, logger)
	cacheReader := json.NewJSONCacheReader(boltConfig, logger)

	schemaLoader := schemaadapter.NewSchemaLoaderAdapter(&domainConfig, &logger)
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
		domainConfig,
		logger,
		nil,
	)

	// Run indexing and measure
	ctx := context.Background()
	start := time.Now()
	stats, err := indexer.Build(ctx)
	duration := time.Since(start)

	require.NoError(t, err)
	require.Equal(t, totalNotes, stats.IndexedCount, "Should index all notes")

	// Calculate metrics
	notesPerSec := float64(stats.IndexedCount) / duration.Seconds()
	msPerNote := duration.Seconds() / float64(stats.IndexedCount) * 1000

	t.Logf("=== Performance Results ===")
	t.Logf("Total notes:       %d", totalNotes)
	t.Logf("Total duration:    %v", duration)
	t.Logf("Throughput:        %.2f notes/sec", notesPerSec)
	t.Logf("Latency per note:  %.4f ms", msPerNote)
	t.Logf("Scanned:           %d", stats.ScannedCount)
	t.Logf("Indexed:           %d", stats.IndexedCount)

	// Performance targets (adjust based on hardware)
	targetNotesPerSec := 100.0 // Minimum 100 notes/sec (reasonable baseline)
	if notesPerSec < targetNotesPerSec {
		t.Errorf("Performance below target: %.2f notes/sec (target: %.2f)",
			notesPerSec, targetNotesPerSec)
	}

	// Should complete 10k notes in under 120 seconds (2 minutes)
	maxDuration := 120 * time.Second
	if duration > maxDuration {
		t.Errorf("Indexing took too long: %v (max: %v)", duration, maxDuration)
	}
}
