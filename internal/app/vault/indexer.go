package vault

import (
	"context"
	"path/filepath"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/dto"
	"github.com/rs/zerolog"
)

// VaultIndexer orchestrates the vault indexing workflow from scan to cache
// persistence. It implements the CQRS write-side pattern for indexing
// operations, coordinating
// vault scanning, basic note creation, and cache persistence.
//
// The indexer focuses solely on orchestration - it delegates scanning to
// VaultScannerPort, caching to CacheWriterPort, and uses injected dependencies
// for all infrastructure concerns.
//
// Key Design Principles:
// - Focused Service: Orchestrates workflow only, does not implement
// scanning/caching - Resilient Error Handling: Cache failures logged as
// warnings, indexing continues - MVP Scope: Creates basic notes with file
// metadata, frontmatter parsing deferred
//
// Reference: docs/architecture/components.md#vaultindexer.
type VaultIndexer struct {
	vaultScanner spi.VaultScannerPort
	cacheWriter  spi.CacheWriterPort
	config       domain.Config
	log          zerolog.Logger
}

// IndexStats tracks metrics for vault indexing operations.
// Used for performance monitoring and NFR3 compliance.
//
// Fields:
// - ScannedCount: Total files scanned from vault
// - IndexedCount: Notes successfully persisted to cache
// - CacheFailures: Cache write errors (logged as warnings)
// - Duration: Total indexing time for performance tracking.
type IndexStats struct {
	ScannedCount  int
	IndexedCount  int
	CacheFailures int
	Duration      time.Duration
}

// NewVaultIndexer creates a new VaultIndexer with injected dependencies.
// Constructor follows dependency injection pattern for testability and
// flexibility.
//
// Parameters:
//   - vaultScanner: Port for scanning vault files
//   - cacheWriter: Port for persisting notes to cache
//   - config: Application configuration
//   - log: Structured logger for operation tracking
//
// Returns:
//   - *VaultIndexer: Configured indexer ready for vault operations
func NewVaultIndexer(
	vaultScanner spi.VaultScannerPort,
	cacheWriter spi.CacheWriterPort,
	config domain.Config,
	log zerolog.Logger,
) *VaultIndexer {
	return &VaultIndexer{
		vaultScanner: vaultScanner,
		cacheWriter:  cacheWriter,
		config:       config,
		log:          log,
	}
}

// Build orchestrates the complete vault indexing workflow.
// Implements the 3-step process: vault scan → basic note creation → cache
// persist.
//
// Workflow Steps:
// 1. Scan vault using scanFiles()
// 2. Process each file using processFile()
// 3. Log final statistics using logStats()
//
// Error Handling:
// - Vault scan failures: Return error immediately (abort indexing)
// - Cache write failures: Log warning, increment CacheFailures, continue
// processing
// - Partial success acceptable - index what we can
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//
// Returns:
//   - IndexStats: Metrics for the indexing operation
//   - error: Vault scan errors only (cache failures logged but don't abort)
//
// Thread-safe: Safe for concurrent calls (dependencies handle synchronization).
func (v *VaultIndexer) Build(ctx context.Context) (IndexStats, error) {
	startTime := time.Now()
	stats := IndexStats{
		ScannedCount:  0,
		IndexedCount:  0,
		CacheFailures: 0,
		Duration:      0,
	}

	// Step 1: Scan vault
	vaultFiles, err := v.scanFiles(ctx)
	if err != nil {
		return stats, err
	}
	stats.ScannedCount = len(vaultFiles)

	// Step 2: Process each file
	for i := range vaultFiles {
		v.processFile(ctx, vaultFiles[i], &stats)
	}

	stats.Duration = time.Since(startTime)

	// Step 3: Log summary
	v.logStats(stats)

	return stats, nil
}

// scanFiles performs vault scanning using the injected VaultScannerPort.
// Returns all vault files or an error if scanning fails.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//
// Returns:
//   - []dto.VaultFile: All files found in the vault
//   - error: Scanning failure (aborts indexing)
func (v *VaultIndexer) scanFiles(ctx context.Context) ([]dto.VaultFile, error) {
	return v.vaultScanner.ScanAll(ctx)
}

// processFile handles single file processing: filtering, note creation, and
// persistence.
// Updates stats for tracking and logging.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - vf: Vault file to process
//   - stats: IndexStats to update with processing results
func (v *VaultIndexer) processFile(
	ctx context.Context,
	vf dto.VaultFile,
	stats *IndexStats,
) {
	// Filter: only .md files for MVP
	if vf.Ext != ".md" {
		return
	}

	// Create basic Note with file metadata
	noteID := deriveNoteIDFromPath(vf.Path)
	note := domain.NewNote(noteID, domain.Frontmatter{
		FileClass: "",
		Fields:    map[string]interface{}{},
	})

	// Persist to cache
	if persistErr := v.cacheWriter.Persist(ctx, note); persistErr != nil {
		stats.CacheFailures++
		v.log.Warn().
			Err(persistErr).
			Str("noteID", string(noteID)).
			Msg("cache persist failed")

		// Continue - don't abort indexing
	} else {
		stats.IndexedCount++
	}
}

// logStats logs the final indexing statistics using structured logging.
// Provides metrics for NFR3 performance monitoring.
//
// Parameters:
//   - stats: Final IndexStats to log
func (v *VaultIndexer) logStats(stats IndexStats) {
	v.log.Info().
		Int("scanned", stats.ScannedCount).
		Int("indexed", stats.IndexedCount).
		Int("cache_failures", stats.CacheFailures).
		Dur("duration", stats.Duration).
		Msg("vault indexing complete")
}

// deriveNoteIDFromPath creates a NoteID from a file path.
// For MVP: Use relative path from vault root as NoteID.
//
// Example: "/vault/projects/foo.md" → "projects/foo"
//
// Future: May use UUID or content hash for NoteID.
func deriveNoteIDFromPath(path string) domain.NoteID {
	// For MVP: Use relative path from vault root
	// Remove .md extension and use as NoteID
	basename := filepath.Base(path)
	name := strings.TrimSuffix(basename, filepath.Ext(basename))
	return domain.NewNoteID(name)
}
