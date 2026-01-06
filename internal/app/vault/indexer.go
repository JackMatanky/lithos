package vault

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync/atomic"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/metrics"
	"github.com/JackMatanky/lithos/internal/app/persistence"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
)

const (
	// markdownExt defines the file extension for markdown files.
	markdownExt = ".md"
)

// VaultIndexerInterface defines the contract for vault indexing operations.
// This interface allows for mocking in tests while maintaining clean
// architecture.
type VaultIndexerInterface interface {
	// Build performs a complete vault indexing operation.
	// Returns metrics.IndexStats with operation metrics and any error
	// encountered.
	Build(ctx context.Context) (metrics.IndexStats, error)
}

// VaultIndexer orchestrates the vault indexing workflow from scan to cache
// persistence. It implements the CQRS write-side pattern for indexing
// operations, coordinating vault scanning, note processing, and cache
// persistence.
//
// The indexer focuses solely on orchestration - it delegates scanning to
// VaultScannerPort, markdown processing to MarkdownProcessor, caching to
// CacheWriter, and schema operations to SchemaEngine.
//
// Key Design Principles:
//   - Focused Orchestration: Coordinates workflow, delegates implementation
//   - Simplified Dependencies: Uses helper components instead of raw ports
//
// - Resilient Error Handling: Parse/validation errors logged, indexing
// continues - Batch Performance: Uses ParallelWriter via CacheWriter for
// optimal throughput
//
// Reference: docs/architecture/components.md#vaultindexer.
type VaultIndexer struct {
	vaultScanner  spi.VaultScannerPort
	cacheReader   spi.CacheReaderPort
	processor     *VaultProcessor
	cacheWriter   *persistence.CacheWriter
	schemaEngine  *schema.SchemaEngine
	config        domain.Config
	log           zerolog.Logger
	eventBus      events.EventBus
	suppressCount atomic.Int32
}

// NewVaultIndexer creates a new VaultIndexer with injected dependencies.
// Constructor follows dependency injection pattern and creates helper
// components (MarkdownProcessor, CacheWriter) internally.
//
// Parameters:
//   - vaultScanner: Port for scanning vault files
//   - cacheReader: Port for reading cached notes
//   - boltWriter: Port for BoltDB cache (used by CacheWriter)
//   - sqliteWriter: Port for SQLite cache (used by CacheWriter)
//   - markdownParser: Port for markdown parsing (used by MarkdownProcessor)
//   - frontmatterService: Service for validation (used by MarkdownProcessor)
//   - schemaEngine: Engine for schema loading and resolution
//   - config: Application configuration
//   - log: Structured logger for operation tracking
//   - eventBus: Event bus for publishing/subscribing to events
//
// Returns:
//   - *VaultIndexer: Configured indexer ready for vault operations
func NewVaultIndexer(
	vaultScanner spi.VaultScannerPort,
	cacheReader spi.CacheReaderPort,
	boltWriter spi.CacheWriterPort,
	sqliteWriter spi.CacheWriterPort,
	markdownParser spi.MarkdownParserPort,
	frontmatterService *frontmatter.FrontmatterService,
	schemaEngine *schema.SchemaEngine,
	config domain.Config,
	log zerolog.Logger,
	eventBus events.EventBus,
) *VaultIndexer {
	// Create helper components
	markdownProcessor := NewMarkdownProcessor(
		markdownParser,
		frontmatterService,
		eventBus,
		log.With().Str("component", "MarkdownProcessor").Logger(),
	)
	processor := NewVaultProcessor(
		markdownProcessor,
		config,
		eventBus,
		log.With().Str("component", "VaultProcessor").Logger(),
	)
	cacheWriter := persistence.NewCacheWriter(
		boltWriter,
		sqliteWriter,
		eventBus,
		log.With().Str("component", "CacheWriter").Logger(),
	)

	indexer := &VaultIndexer{
		vaultScanner:  vaultScanner,
		cacheReader:   cacheReader,
		processor:     processor,
		cacheWriter:   cacheWriter,
		schemaEngine:  schemaEngine,
		config:        config,
		log:           log,
		eventBus:      eventBus,
		suppressCount: atomic.Int32{},
	}

	// Subscribe to vault-level events
	if eventBus != nil {
		_ = eventBus.Subscribe(
			"CommandIssued",
			indexer.handleCommandIssuedEvent,
		)
		_ = eventBus.Subscribe("NoteIndexed", indexer.handleNoteIndexedEvent)
	}

	return indexer
}

// Build orchestrates the complete vault indexing workflow.
// Implements the enhanced workflow: schema load → vault scan → frontmatter
// extraction/validation → note creation → cache persist.
//
// Workflow Steps:
// 1. Load schemas using SchemaEngine.Load()
// 2. Scan vault using scanFiles()
// 3. Process each file using processFile() (with frontmatter integration)
// 4. Log final statistics using logStats()
//
// Error Handling:
// - Schema load failures: Return error immediately (abort indexing)
// - Vault scan failures: Return error immediately (abort indexing)
// - Frontmatter validation failures: Log warning, increment ValidationFailures,
// continue
// - Cache write failures: Log warning, increment CacheFailures, continue
// - Partial success acceptable - index what we can
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//
// Returns:
//   - metrics.IndexStats: Metrics for the indexing operation
//   - error: Schema/scan errors only (validation/cache failures logged but don't
//     abort)
//
// Thread-safe: Safe for concurrent calls (dependencies handle synchronization).
func (v *VaultIndexer) Build(ctx context.Context) (metrics.IndexStats, error) {
	v.suppressCount.Add(1)
	defer v.suppressCount.Add(-1)

	startTime := time.Now()
	stats := metrics.IndexStats{
		ScannedCount:        0,
		IndexedCount:        0,
		ParseFailures:       0,
		CacheFailures:       0,
		ValidationSuccesses: 0,
		ValidationFailures:  0,
		Duration:            0,
	}

	// Step 1: Load schemas first (if schema engine is available)
	if v.schemaEngine != nil {
		if err := v.schemaEngine.Load(ctx); err != nil {
			return stats, err
		}
	}

	// Step 2: Scan vault
	vaultFiles, err := v.scanFiles(ctx)
	if err != nil {
		return stats, err
	}
	stats.ScannedCount = len(vaultFiles)

	// Step 3: Process files into notes and metadata
	notes := make([]domain.Note, 0, len(vaultFiles))
	metadataMap := make(map[string]spi.CacheWriteMetadata, len(vaultFiles))

	for i := range vaultFiles {
		if cancelErr := ctx.Err(); cancelErr != nil {
			stats.Duration = time.Since(startTime)
			return stats, cancelErr
		}

		note, metadata, processed := v.processor.ProcessFile(
			ctx,
			&vaultFiles[i],
			&stats,
		)
		if processed {
			notes = append(notes, note)
			metadataMap[note.Path] = metadata
		}
	}

	// Step 4: Batch write all notes to cache using CacheWriter
	stats.IndexedCount = len(notes)
	if len(notes) > 0 {
		if commitErr := v.cacheWriter.WriteBatch(ctx, notes, metadataMap); commitErr != nil {
			stats.CacheFailures++
			v.log.Error().
				Err(commitErr).
				Int("notes", len(notes)).
				Msg("batch cache write failed")
		}
	}

	stats.Duration = time.Since(startTime)

	// Step 4: Validate cache state
	validationResult, validationErr := v.validateCacheState(
		ctx,
		vaultFiles,
		nil,
	)
	if validationErr != nil {
		v.log.Warn().
			Err(validationErr).
			Msg("cache state validation failed")
	} else {
		v.logCacheValidationResult(validationResult)
	}

	// Step 5: Log summary
	v.logStats(stats)
	v.publishIndexingCompleteEvent(ctx, stats)

	return stats, nil
}

// Refresh performs incremental vault indexing for large vault optimization.
// Processes modified files and handles deletion reconciliation.
//
// Workflow Steps:
// 1. Load schemas using SchemaEngine.Load() (if schema engine available)
// 2. Perform deletion reconciliation by comparing current vault state with
// cache
// 3. Scan modified files using scanModifiedFiles()
// 4. Process each modified file using processFile()
// 5. Log incremental update statistics
//
// Error Handling:
// - Schema load failures: Return error immediately (abort refresh)
// - Vault scan failures: Return error immediately (abort refresh)
// - Cache write/delete failures: Log warning, increment CacheFailures, continue
// processing
// - Partial success acceptable - update what we can
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - since: Only process files modified after this timestamp
//
// Returns:
//   - error: Schema/scan errors only (cache failures logged but don't abort)
//
// Thread-safe: Safe for concurrent calls (dependencies handle synchronization).
func (v *VaultIndexer) Refresh(ctx context.Context, since time.Time) error {
	v.suppressCount.Add(1)
	defer v.suppressCount.Add(-1)

	startTime := time.Now()
	stats := metrics.IndexStats{
		ScannedCount:        0,
		IndexedCount:        0,
		ParseFailures:       0,
		CacheFailures:       0,
		ValidationSuccesses: 0,
		ValidationFailures:  0,
		Duration:            0,
	}

	// Step 1: Load schemas first (if schema engine is available)
	if v.schemaEngine != nil {
		if err := v.schemaEngine.Load(ctx); err != nil {
			return fmt.Errorf("schema loading failed: %w", err)
		}
	}

	// Step 2: Perform deletion reconciliation
	retainedNotes := v.reconcileDeletions(ctx)

	// Step 3: Scan modified files
	vaultFiles, err := v.scanModifiedFiles(ctx, since)
	if err != nil {
		return err
	}
	stats.ScannedCount = len(vaultFiles)

	// Step 4: Process modified files into notes and metadata
	notes := make([]domain.Note, 0, len(vaultFiles))
	metadataMap := make(map[string]spi.CacheWriteMetadata, len(vaultFiles))

	for i := range vaultFiles {
		if cancelErr := ctx.Err(); cancelErr != nil {
			return cancelErr
		}

		note, metadata, processed := v.processor.ProcessFile(
			ctx,
			&vaultFiles[i],
			&stats,
		)
		if processed {
			notes = append(notes, note)
			metadataMap[note.Path] = metadata
		}
	}

	// Step 5: Batch write all modified notes to cache
	stats.IndexedCount = len(notes)
	if len(notes) > 0 {
		if commitErr := v.cacheWriter.WriteBatch(ctx, notes, metadataMap); commitErr != nil {
			stats.CacheFailures++
			v.log.Error().
				Err(commitErr).
				Int("notes", len(notes)).
				Msg("refresh batch cache write failed")
		}
	}

	stats.Duration = time.Since(startTime)

	// Step 4: Validate cache state
	validationResult, validationErr := v.validateCacheState(
		ctx,
		vaultFiles,
		retainedNotes,
	)
	if validationErr != nil {
		v.log.Warn().
			Err(validationErr).
			Msg("cache state validation failed during refresh")
	} else {
		v.logCacheValidationResult(validationResult)
	}

	// Step 5: Log incremental update
	v.logRefreshStats(stats, since)

	return nil
}

// reconcileDeletions compares current vault state with cache entries and
// removes orphaned cache entries (files deleted from vault but still cached).
// Ensures cache-vault consistency during incremental operations.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//
// Returns:
//   - []domain.Note: Notes that still exist in vault (retained)
func (v *VaultIndexer) reconcileDeletions(ctx context.Context) []domain.Note {
	cachedNotes, listErr := v.cacheReader.List(ctx)
	if listErr != nil {
		v.log.Warn().
			Err(listErr).
			Msg("failed to list cached notes for reconciliation, skipping deletion reconciliation")
		return nil
	}

	var retained []domain.Note
	var orphanedPaths []string

	for i := range cachedNotes {
		note := cachedNotes[i]
		absolutePath := filepath.Join(v.config.VaultPath, note.Path)

		_, statErr := os.Stat(absolutePath)
		if statErr == nil {
			retained = append(retained, note)
			continue
		}

		if os.IsNotExist(statErr) {
			orphanedPaths = append(orphanedPaths, note.Path)
			v.log.Debug().
				Str("notePath", note.Path).
				Str("path", absolutePath).
				Msg("detected orphaned cache entry")
			continue
		}

		v.log.Warn().
			Err(statErr).
			Str("notePath", note.Path).
			Str("path", absolutePath).
			Msg("failed to stat note path during reconciliation")
		retained = append(retained, note)
	}

	// Batch delete orphaned entries from cache
	if len(orphanedPaths) > 0 {
		if deleteErr := v.cacheWriter.DeleteBatch(ctx, orphanedPaths); deleteErr != nil {
			v.log.Warn().
				Err(deleteErr).
				Int("orphanedCount", len(orphanedPaths)).
				Msg("failed to delete orphaned cache entries")
		} else {
			v.log.Info().
				Int("deletedCount", len(orphanedPaths)).
				Msg("deleted orphaned cache entries")
		}
	}

	return retained
}

// validateCacheState verifies that cache contents accurately reflect current
// vault state. Performs comprehensive consistency checks to ensure cache-vault
// synchronization.
// Returns detailed validation results for debugging cache management issues.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//
// Returns:
//   - metrics.CacheValidationResult: Detailed results of cache state validation
//   - error: Critical validation errors (e.g., unable to access vault/cache)
func (v *VaultIndexer) validateCacheState(
	ctx context.Context,
	vaultFiles []dto.VaultFile,
	retainedNotes []domain.Note,
) (metrics.CacheValidationResult, error) {
	snapshot := vaultFiles
	if len(retainedNotes) > 0 {
		var buildErr error
		snapshot, buildErr = v.buildVaultSnapshot(
			ctx,
			vaultFiles,
			retainedNotes,
		)
		if buildErr != nil {
			return metrics.CacheValidationResult{}, fmt.Errorf(
				"failed to build vault snapshot: %w",
				buildErr,
			)
		}
	}

	// Collect vault state
	vaultNotePaths, totalVaultFiles, vaultErr := v.collectVaultState(
		ctx,
		snapshot,
	)
	if vaultErr != nil {
		return metrics.CacheValidationResult{}, fmt.Errorf(
			"failed to collect vault state for validation: %w",
			vaultErr,
		)
	}

	// Collect cache state
	cacheNotePaths, totalCacheEntries, cachedNotes, cacheErr := v.collectCacheState(
		ctx,
		retainedNotes,
	)
	if cacheErr != nil {
		return metrics.CacheValidationResult{}, fmt.Errorf(
			"failed to collect cache state for validation: %w",
			cacheErr,
		)
	}

	// Find inconsistencies
	orphanedCount, missingCount, orphanedDetails, missingDetails, isConsistent :=
		v.findInconsistencies(
			vaultNotePaths,
			cacheNotePaths,
			cachedNotes,
		)

	result := metrics.CacheValidationResult{
		TotalVaultFiles:    totalVaultFiles,
		TotalCacheEntries:  totalCacheEntries,
		OrphanedCacheFiles: orphanedCount,
		MissingCacheFiles:  missingCount,
		OrphanedDetails:    orphanedDetails,
		MissingDetails:     missingDetails,
		IsConsistent:       isConsistent,
	}

	return result, nil
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

// collectVaultState scans the vault and builds a map of note paths for markdown
// files.
// Returns the path map, total count, and any scanning error.
func (v *VaultIndexer) collectVaultState(
	ctx context.Context,
	cachedVault []dto.VaultFile,
) (
	vaultNotePaths map[string]bool,
	totalFiles int,
	err error,
) {
	var vaultFiles []dto.VaultFile
	if cachedVault != nil {
		vaultFiles = cachedVault
	} else {
		vaultFiles, err = v.scanFiles(ctx)
		if err != nil {
			return
		}
	}

	vaultNotePaths = make(map[string]bool)
	totalFiles = 0
	for i := range vaultFiles {
		vf := vaultFiles[i]
		if vf.Ext() == markdownExt {
			vaultNotePaths[vf.Path] = true
			totalFiles++
		}
	}
	return vaultNotePaths, totalFiles, nil
}

// collectCacheState retrieves all cached notes and builds a map of note paths.
// Returns the path map, total count, cached notes slice, and any listing
// error.
func (v *VaultIndexer) collectCacheState(
	ctx context.Context,
	preloaded []domain.Note,
) (
	cacheNotePaths map[string]bool,
	totalEntries int,
	cachedNotes []domain.Note,
	err error,
) {
	if preloaded != nil {
		cachedNotes = preloaded
	} else {
		cachedNotes, err = v.cacheReader.List(ctx)
		if err != nil {
			return
		}
	}

	cacheNotePaths = make(map[string]bool)
	totalEntries = len(cachedNotes)
	for i := range cachedNotes {
		note := cachedNotes[i]
		cacheNotePaths[note.Path] = true
	}
	return
}

func (v *VaultIndexer) buildVaultSnapshot(
	ctx context.Context,
	scanned []dto.VaultFile,
	retained []domain.Note,
) ([]dto.VaultFile, error) {
	if len(retained) == 0 {
		return scanned, nil
	}

	snapshot := make([]dto.VaultFile, 0, len(scanned)+len(retained))
	seen := make(map[string]struct{}, len(scanned)+len(retained))

	for i := range scanned {
		vf := scanned[i]
		snapshot = append(snapshot, vf)
		seen[filepath.Clean(vf.Path)] = struct{}{}
	}

	for i := range retained {
		if err := ctx.Err(); err != nil {
			return nil, err
		}

		note := retained[i]
		absolute := filepath.Join(v.config.VaultPath, note.Path)
		cleanPath := filepath.Clean(absolute)

		if _, exists := seen[cleanPath]; exists {
			continue
		}

		info, err := os.Stat(absolute)
		if err != nil {
			if os.IsNotExist(err) {
				continue
			}
			return nil, err
		}
		if strings.ToLower(filepath.Ext(absolute)) != markdownExt {
			continue
		}

		vaultFile, err := dto.NewVaultFile(
			absolute,
			v.config.VaultPath,
			info,
			nil,
		)
		if err != nil {
			v.log.Warn().
				Err(err).
				Str("path", absolute).
				Msg("failed to create VaultFile")
			continue
		}
		snapshot = append(snapshot, vaultFile)
		seen[cleanPath] = struct{}{}
	}

	return snapshot, nil
}

// findInconsistencies compares vault and cache path sets to identify
// orphaned and missing entries. Returns orphaned count, missing count,
// orphaned details, missing details, and consistency flag.
func (v *VaultIndexer) findInconsistencies(
	vaultNotePaths map[string]bool,
	cacheNotePaths map[string]bool,
	cachedNotes []domain.Note,
) (
	orphanedCount int,
	missingCount int,
	orphanedDetails []string,
	missingDetails []string,
	isConsistent bool,
) {
	orphanedDetails = []string{}
	missingDetails = []string{}
	isConsistent = true

	// Find orphaned cache entries (in cache but not in vault)
	for i := range cachedNotes {
		note := cachedNotes[i]
		if !vaultNotePaths[note.Path] {
			orphanedCount++
			orphanedDetails = append(orphanedDetails, note.Path)
			isConsistent = false
		}
	}

	// Find missing cache entries (in vault but not in cache)
	for notePath := range vaultNotePaths {
		if !cacheNotePaths[notePath] {
			missingCount++
			missingDetails = append(missingDetails, notePath)
			isConsistent = false
		}
	}

	return orphanedCount,
		missingCount,
		orphanedDetails,
		missingDetails,
		isConsistent
}

// scanModifiedFiles performs incremental vault scanning for modified files.
// Returns only files changed since the specified timestamp.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - since: Only return files modified after this timestamp
//
// Returns:
//   - []dto.VaultFile: Modified files found in the vault
//   - error: Scanning failure (aborts refresh)
func (v *VaultIndexer) scanModifiedFiles(
	ctx context.Context,
	since time.Time,
) ([]dto.VaultFile, error) {
	return v.vaultScanner.ScanModified(ctx, since)
}

// logStats logs the final indexing statistics using structured logging.
// Provides metrics for NFR3 performance monitoring.
//
// Parameters:
//   - stats: Final metrics.IndexStats to log
func (v *VaultIndexer) logStats(stats metrics.IndexStats) {
	v.log.Info().
		Int("scanned", stats.ScannedCount).
		Int("indexed", stats.IndexedCount).
		Int("cache_failures", stats.CacheFailures).
		Int("validation_successes", stats.ValidationSuccesses).
		Int("validation_failures", stats.ValidationFailures).
		Dur("duration", stats.Duration).
		Msg("vault indexing complete")
}

func (v *VaultIndexer) publishIndexingCompleteEvent(
	ctx context.Context,
	stats metrics.IndexStats,
) {
	if v.eventBus == nil {
		return
	}
	summary := events.VaultIndexingSummary{
		ScannedCount:        stats.ScannedCount,
		IndexedCount:        stats.IndexedCount,
		ParseFailures:       stats.ParseFailures,
		CacheFailures:       stats.CacheFailures,
		ValidationSuccesses: stats.ValidationSuccesses,
		ValidationFailures:  stats.ValidationFailures,
	}
	event, err := events.NewVaultIndexingCompleteEvent(
		summary,
		stats.Duration,
		time.Now(),
	)
	if err != nil {
		v.log.Warn().
			Err(err).
			Msg("failed to create vault indexing complete event")
		return
	}
	if publishErr := events.PublishSync(ctx, v.eventBus, event); publishErr != nil {
		v.log.Warn().
			Err(publishErr).
			Msg("failed to publish vault indexing complete event")
	}
}

func (v *VaultIndexer) handleCommandIssuedEvent(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	commandEvent, ok := event.(*events.CommandIssuedEvent)
	if !ok {
		return nil
	}
	if commandEvent.Command() != "IndexVault" {
		return nil
	}
	v.log.Info().Msg("received IndexVault command event")
	_, err := v.Build(ctx)
	return err
}

func (v *VaultIndexer) handleNoteIndexedEvent(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	if v.suppressCount.Load() > 0 {
		return nil
	}
	noteEvent, ok := event.(*domain.NoteIndexedEvent)
	if !ok {
		return nil
	}
	return v.applyNoteEvent(ctx, noteEvent.Note())
}

func (v *VaultIndexer) applyNoteEvent(
	ctx context.Context,
	note domain.Note,
) error {
	// Use CacheWriter to persist single note
	indexTime := time.Now().UTC()
	metadata := spi.CacheWriteMetadata{
		IndexTime:  indexTime,
		ModifiedAt: time.Time{},
		FileSize:   0,
	}
	if note.Path != "" {
		absolute := filepath.Join(v.config.VaultPath, note.Path)
		info, err := os.Stat(absolute)
		if err == nil {
			metadata.ModifiedAt = info.ModTime().UTC()
			metadata.FileSize = info.Size()
		}
	}
	metadataMap := map[string]spi.CacheWriteMetadata{
		note.Path: metadata,
	}

	if err := v.cacheWriter.WriteBatch(ctx, []domain.Note{note}, metadataMap); err != nil {
		v.log.Error().
			Err(err).
			Str("path", note.Path).
			Msg("failed to apply note indexed event to caches")
		return err
	}

	v.log.Debug().
		Str("path", note.Path).
		Msg("applied note indexed event to caches")
	return nil
}

// logRefreshStats logs incremental refresh statistics using structured logging.
// Provides metrics for incremental update performance monitoring.
//
// Parameters:
//   - stats: Refresh metrics.IndexStats to log
//   - since: Timestamp used for the incremental scan
func (v *VaultIndexer) logRefreshStats(
	stats metrics.IndexStats,
	since time.Time,
) {
	v.log.Info().
		Time("since", since).
		Int("scanned", stats.ScannedCount).
		Int("indexed", stats.IndexedCount).
		Int("cache_failures", stats.CacheFailures).
		Int("validation_successes", stats.ValidationSuccesses).
		Int("validation_failures", stats.ValidationFailures).
		Dur("duration", stats.Duration).
		Msg("vault refresh complete")
}

// logCacheValidationResult logs cache validation results using structured
// logging. Provides visibility into cache-vault consistency for debugging and
// monitoring.
//
// Parameters:
//   - result: metrics.CacheValidationResult to log
func (v *VaultIndexer) logCacheValidationResult(
	result metrics.CacheValidationResult,
) {
	if result.IsConsistent {
		v.log.Info().
			Int("vault_files", result.TotalVaultFiles).
			Int("cache_entries", result.TotalCacheEntries).
			Msg("cache state validation: consistent")
	} else {
		v.log.Warn().
			Int("vault_files", result.TotalVaultFiles).
			Int("cache_entries", result.TotalCacheEntries).
			Int("orphaned_cache", result.OrphanedCacheFiles).
			Int("missing_cache", result.MissingCacheFiles).
			Strs("orphaned_details", result.OrphanedDetails).
			Strs("missing_details", result.MissingDetails).
			Msg("cache state validation: inconsistencies found")
	}
}
