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
// operations, coordinating vault scanning, frontmatter parsing/validation,
// note creation, and cache persistence.
//
// The indexer focuses solely on orchestration - it delegates scanning to
// VaultScannerPort, frontmatter parsing to MarkdownParserPort, validation
// to FrontmatterService, schema operations to SchemaEngine, and caching to
// CacheWriterPort and CacheReaderPort.
//
// Key Design Principles:
//   - Focused Service: Orchestrates workflow only, does not implement
//     scanning/caching/frontmatter processing
//   - Resilient Error Handling: Frontmatter validation errors logged but don't
//     abort entire indexing; cache failures logged as warnings, indexing
//     continues
//   - Integrated Workflow: MarkdownParserPort + FrontmatterService enables
//     validated frontmatter in indexed Notes
//
// Reference: docs/architecture/components.md#vaultindexer.
type VaultIndexer struct {
	vaultScanner       spi.VaultScannerPort
	boltWriter         spi.CacheWriterPort
	sqliteWriter       spi.CacheWriterPort
	cacheReader        spi.CacheReaderPort
	markdownParserPort spi.MarkdownParserPort
	frontmatterService *frontmatter.FrontmatterService
	schemaEngine       *schema.SchemaEngine
	config             domain.Config
	log                zerolog.Logger
	eventBus           events.EventBus
	suppressCount      atomic.Int32
}

// NewVaultIndexer creates a new VaultIndexer with injected dependencies.
// Constructor follows dependency injection pattern for testability and
// flexibility.
//
// Parameters:
//   - vaultScanner: Port for scanning vault files
//   - boltWriter: Port for persisting notes to BoltDB (hot cache)
//   - sqliteWriter: Port for persisting notes to SQLite (deep storage)
//   - cacheReader: Port for reading cached notes
//   - markdownParserPort: Port for parsing markdown frontmatter
//   - frontmatterService: Service for frontmatter validation
//   - schemaEngine: Engine for schema loading and resolution
//   - config: Application configuration
//   - log: Structured logger for operation tracking
//
// Returns:
//   - *VaultIndexer: Configured indexer ready for vault operations
func NewVaultIndexer(
	vaultScanner spi.VaultScannerPort,
	boltWriter spi.CacheWriterPort,
	sqliteWriter spi.CacheWriterPort,
	cacheReader spi.CacheReaderPort,
	markdownParserPort spi.MarkdownParserPort,
	frontmatterService *frontmatter.FrontmatterService,
	schemaEngine *schema.SchemaEngine,
	config domain.Config,
	log zerolog.Logger,
	eventBus events.EventBus,
) *VaultIndexer {
	indexer := &VaultIndexer{
		vaultScanner:       vaultScanner,
		boltWriter:         boltWriter,
		sqliteWriter:       sqliteWriter,
		cacheReader:        cacheReader,
		markdownParserPort: markdownParserPort,
		frontmatterService: frontmatterService,
		schemaEngine:       schemaEngine,
		config:             config,
		log:                log,
		eventBus:           eventBus,
		suppressCount:      atomic.Int32{},
	}
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

	// Create Unit of Work for transactional writes
	uow := NewCacheUnitOfWork(v.boltWriter, v.sqliteWriter)
	if beginErr := uow.Begin(); beginErr != nil {
		return stats, fmt.Errorf("failed to begin transaction: %w", beginErr)
	}

	// Step 3: Process each file
	for i := range vaultFiles {
		if cancelErr := ctx.Err(); cancelErr != nil {
			stats.Duration = time.Since(startTime)
			return stats, cancelErr
		}
		v.processFile(ctx, &vaultFiles[i], uow, &stats)
	}

	// Commit transaction
	if commitErr := uow.Commit(ctx); commitErr != nil {
		stats.CacheFailures++ // Log as generic failure if commit fails
		v.log.Error().Err(commitErr).Msg("transaction commit failed")
		// If rollback is handled in Commit (it is), we assume data is
		// consistent (rolled back)
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

	// Create Unit of Work
	uow := NewCacheUnitOfWork(v.boltWriter, v.sqliteWriter)
	if beginErr := uow.Begin(); beginErr != nil {
		return fmt.Errorf("failed to begin transaction: %w", beginErr)
	}

	// Step 2: Perform deletion reconciliation
	retainedNotes := v.reconcileDeletions(ctx, uow, &stats)

	// Step 2: Scan modified files
	vaultFiles, err := v.scanModifiedFiles(ctx, since)
	if err != nil {
		return err
	}
	stats.ScannedCount = len(vaultFiles)

	// Step 3: Process each modified file
	for i := range vaultFiles {
		if cancelErr := ctx.Err(); cancelErr != nil {
			return cancelErr
		}
		v.processFile(ctx, &vaultFiles[i], uow, &stats)
	}

	// Commit transaction
	if commitErr := uow.Commit(ctx); commitErr != nil {
		v.log.Error().Err(commitErr).Msg("refresh transaction commit failed")
		// Should we return error? The original code returned nil for cache
		// failures.
		// But UoW failure means nothing was written.
		// We'll log and return nil to match existing contract (schema/scan
		// errors only abort).
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
//   - stats: metrics.IndexStats to update with deletion failures
//
// Returns:
//   - error: Critical errors that should abort refresh (e.g., vault scan
//
// failure).
func (v *VaultIndexer) reconcileDeletions(
	ctx context.Context,
	uow *CacheUnitOfWork,
	stats *metrics.IndexStats,
) []domain.Note {
	cachedNotes, listErr := v.cacheReader.List(ctx)
	if listErr != nil {
		v.log.Warn().
			Err(listErr).
			Msg("failed to list cached notes for reconciliation, skipping deletion reconciliation")
		return nil // Don't abort refresh for cache read failures
	}

	var retained []domain.Note
	for i := range cachedNotes {
		note := cachedNotes[i]
		absolutePath := filepath.Join(v.config.VaultPath, note.Path)

		_, statErr := os.Stat(absolutePath)
		if statErr == nil {
			retained = append(retained, note)
			continue
		}

		if os.IsNotExist(statErr) {
			if deleteErr := uow.AddDelete(note.Path); deleteErr != nil {
				stats.CacheFailures++
				v.log.Warn().
					Err(deleteErr).
					Str("notePath", note.Path).
					Msg("failed to stage delete for orphaned cache entry")
			} else {
				v.log.Debug().
					Str("notePath", note.Path).
					Str("path", absolutePath).
					Msg("staged delete for orphaned cache entry")
			}
			continue
		}

		v.log.Warn().
			Err(statErr).
			Str("notePath", note.Path).
			Str("path", absolutePath).
			Msg("failed to stat note path during reconciliation")
		retained = append(retained, note)
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

// processFile handles single file processing: filtering, frontmatter
// extraction/validation,
// note creation, and persistence.
// Updates stats for tracking and logging.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - vf: Vault file to process
//   - stats: metrics.IndexStats to update with processing results
func (v *VaultIndexer) processFile(
	ctx context.Context,
	file *dto.VaultFile,
	uow *CacheUnitOfWork,
	stats *metrics.IndexStats,
) {
	// Filter: only .md files for frontmatter processing
	if file.Ext() != markdownExt {
		return
	}

	// Parse note using markdown parser
	note, err := v.markdownParserPort.ParseNote(ctx, file.Path, file.Content)
	if err != nil {
		stats.ParseFailures++
		v.log.Error().
			Err(err).
			Str("path", file.Path).
			Msg("failed to parse note")
		return
	}

	// If frontmatterService is available, validate the parsed frontmatter
	if v.frontmatterService != nil {
		if validationErr := v.frontmatterService.IsSchemaCompliant(
			ctx, note.Path,
			note.Frontmatter,
		); validationErr != nil {
			stats.ValidationFailures++
			v.log.Warn().
				Err(validationErr).
				Str("path", file.Path).
				Msg("frontmatter validation failed")
			return
		}
		stats.ValidationSuccesses++
	}

	// Persist to cache via Unit of Work
	// We generate indexTime here to ensure consistency
	indexTime := time.Now().UTC()
	metadata := v.buildCacheWriteMetadata(file, indexTime)
	if persistErr := uow.AddWrite(note, metadata); persistErr != nil {
		stats.CacheFailures++
		v.log.Warn().
			Err(persistErr).
			Str("notePath", note.Path).
			Msg("cache write staging failed")

		// Continue - don't abort indexing
	} else {
		stats.IndexedCount++
		v.publishNoteIndexedEvent(ctx, note)
	}
}

func (v *VaultIndexer) buildCacheWriteMetadata(
	file *dto.VaultFile,
	indexTime time.Time,
) spi.CacheWriteMetadata {
	if file != nil && file.Info != nil {
		return spi.CacheWriteMetadata{
			ModifiedAt: file.Info.ModTime().UTC(),
			FileSize:   file.Info.Size(),
			IndexTime:  indexTime,
		}
	}
	if file != nil {
		return v.metadataFromPath(file.Path, indexTime)
	}
	return spi.CacheWriteMetadata{
		IndexTime:  indexTime,
		ModifiedAt: time.Time{},
		FileSize:   0,
	}
}

func (v *VaultIndexer) metadataFromPath(
	path string,
	indexTime time.Time,
) spi.CacheWriteMetadata {
	meta := spi.CacheWriteMetadata{
		IndexTime:  indexTime,
		ModifiedAt: time.Time{},
		FileSize:   0,
	}
	if path == "" {
		return meta
	}
	absolute := filepath.Join(v.config.VaultPath, path)
	info, err := os.Stat(absolute)
	if err != nil {
		return meta
	}
	meta.ModifiedAt = info.ModTime().UTC()
	meta.FileSize = info.Size()
	return meta
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

func (v *VaultIndexer) publishNoteIndexedEvent(
	ctx context.Context,
	note domain.Note,
) {
	if v.eventBus == nil {
		return
	}
	event, err := domain.NewNoteIndexedEvent(note, time.Now())
	if err != nil {
		v.log.Warn().
			Err(err).
			Str("path", note.Path).
			Msg("failed to create note indexed event")
		return
	}
	if publishErr := v.eventBus.Publish(ctx, event); publishErr != nil {
		v.log.Warn().
			Err(publishErr).
			Str("path", note.Path).
			Msg("failed to publish note indexed event")
	}
}

func (v *VaultIndexer) publishIndexingCompleteEvent(
	ctx context.Context,
	stats metrics.IndexStats,
) {
	if v.eventBus == nil {
		return
	}
	summary := domain.VaultIndexingSummary{
		ScannedCount:        stats.ScannedCount,
		IndexedCount:        stats.IndexedCount,
		ParseFailures:       stats.ParseFailures,
		CacheFailures:       stats.CacheFailures,
		ValidationSuccesses: stats.ValidationSuccesses,
		ValidationFailures:  stats.ValidationFailures,
	}
	event, err := domain.NewVaultIndexingCompleteEvent(
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
	if publishErr := v.eventBus.Publish(ctx, event); publishErr != nil {
		v.log.Warn().
			Err(publishErr).
			Msg("failed to publish vault indexing complete event")
	}
}

func (v *VaultIndexer) handleCommandIssuedEvent(
	ctx context.Context,
	event domain.DomainEvent,
) error {
	commandEvent, ok := event.(*domain.CommandIssuedEvent)
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
	if v.frontmatterService != nil {
		if err := v.frontmatterService.IsSchemaCompliant(ctx, note.Path, note.Frontmatter); err != nil {
			return err
		}
	}
	uow := NewCacheUnitOfWork(v.boltWriter, v.sqliteWriter)
	if beginErr := uow.Begin(); beginErr != nil {
		return beginErr
	}
	indexTime := time.Now().UTC()
	metadata := v.metadataFromPath(note.Path, indexTime)
	if persistErr := uow.AddWrite(note, metadata); persistErr != nil {
		return persistErr
	}
	if commitErr := uow.Commit(ctx); commitErr != nil {
		return commitErr
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
