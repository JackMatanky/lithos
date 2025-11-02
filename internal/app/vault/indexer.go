package vault

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/dto"
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
	// Returns IndexStats with operation metrics and any error encountered.
	Build(ctx context.Context) (IndexStats, error)
}

// VaultIndexer orchestrates the vault indexing workflow from scan to cache
// persistence. It implements the CQRS write-side pattern for indexing
// operations, coordinating vault scanning, frontmatter extraction/validation,
// note creation, and cache persistence.
//
// The indexer focuses solely on orchestration - it delegates scanning to
// VaultScannerPort, frontmatter processing to FrontmatterService, schema
// operations to SchemaEngine, and caching to CacheWriterPort and
// CacheReaderPort.
//
// Key Design Principles:
//   - Focused Service: Orchestrates workflow only, does not implement
//     scanning/caching/frontmatter processing
//
// - Resilient Error Handling: Frontmatter validation errors logged but don't
// abort
//
//	  entire indexing; cache failures logged as warnings, indexing continues
//	- Integrated Workflow: FrontmatterService integration enables validated
//	  frontmatter in indexed Notes
//
// Reference: docs/architecture/components.md#vaultindexer.
type VaultIndexer struct {
	vaultScanner       spi.VaultScannerPort
	cacheWriter        spi.CacheWriterPort
	cacheReader        spi.CacheReaderPort
	frontmatterService *frontmatter.FrontmatterService
	schemaEngine       *schema.SchemaEngine
	config             domain.Config
	log                zerolog.Logger
}

// NewVaultIndexer creates a new VaultIndexer with injected dependencies.
// Constructor follows dependency injection pattern for testability and
// flexibility.
//
// Parameters:
//   - vaultScanner: Port for scanning vault files
//   - cacheWriter: Port for persisting notes to cache
//   - cacheReader: Port for reading cached notes
//   - frontmatterService: Service for frontmatter extraction and validation
//   - schemaEngine: Engine for schema loading and resolution
//   - config: Application configuration
//   - log: Structured logger for operation tracking
//
// Returns:
//   - *VaultIndexer: Configured indexer ready for vault operations
func NewVaultIndexer(
	vaultScanner spi.VaultScannerPort,
	cacheWriter spi.CacheWriterPort,
	cacheReader spi.CacheReaderPort,
	frontmatterService *frontmatter.FrontmatterService,
	schemaEngine *schema.SchemaEngine,
	config domain.Config,
	log zerolog.Logger,
) *VaultIndexer {
	return &VaultIndexer{
		vaultScanner:       vaultScanner,
		cacheWriter:        cacheWriter,
		cacheReader:        cacheReader,
		frontmatterService: frontmatterService,
		schemaEngine:       schemaEngine,
		config:             config,
		log:                log,
	}
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
//   - IndexStats: Metrics for the indexing operation
//
// - error: Schema/scan errors only (validation/cache failures logged but don't
// abort)
//
// Thread-safe: Safe for concurrent calls (dependencies handle synchronization).
func (v *VaultIndexer) Build(ctx context.Context) (IndexStats, error) {
	startTime := time.Now()
	stats := IndexStats{
		ScannedCount:        0,
		IndexedCount:        0,
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

	// Step 3: Process each file
	for i := range vaultFiles {
		v.processFile(ctx, &vaultFiles[i], &stats)
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
	startTime := time.Now()
	stats := IndexStats{
		ScannedCount:        0,
		IndexedCount:        0,
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
	retainedNotes := v.reconcileDeletions(ctx, &stats)

	// Step 2: Scan modified files
	vaultFiles, err := v.scanModifiedFiles(ctx, since)
	if err != nil {
		return err
	}
	stats.ScannedCount = len(vaultFiles)

	// Step 3: Process each modified file
	for i := range vaultFiles {
		v.processFile(ctx, &vaultFiles[i], &stats)
	}

	stats.Duration = time.Since(startTime)

	// Step 4: Validate cache state
	validationResult, validationErr := v.validateCacheState(
		ctx,
		nil,
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
// removes
// orphaned cache entries (files deleted from vault but still cached).
// Ensures cache-vault consistency during incremental operations.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - stats: IndexStats to update with deletion failures
//
// Returns:
// - error: Critical errors that should abort refresh (e.g., vault scan
// failure).
func (v *VaultIndexer) reconcileDeletions(
	ctx context.Context,
	stats *IndexStats,
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
		relativePath := filepath.FromSlash(note.Path)
		absolutePath := filepath.Join(v.config.VaultPath, relativePath)

		_, statErr := os.Stat(absolutePath)
		if statErr == nil {
			retained = append(retained, note)
			continue
		}

		if os.IsNotExist(statErr) {
			if deleteErr := v.cacheWriter.Delete(ctx, note.ID); deleteErr != nil {
				stats.CacheFailures++
				v.log.Warn().
					Err(deleteErr).
					Str("noteID", string(note.ID)).
					Msg("failed to delete orphaned cache entry")
			} else {
				v.log.Debug().
					Str("noteID", string(note.ID)).
					Str("path", absolutePath).
					Msg("deleted orphaned cache entry")
			}
			continue
		}

		v.log.Warn().
			Err(statErr).
			Str("noteID", string(note.ID)).
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
//   - CacheValidationResult: Detailed results of cache state validation
//   - error: Critical validation errors (e.g., unable to access vault/cache)
func (v *VaultIndexer) validateCacheState(
	ctx context.Context,
	vaultFiles []dto.VaultFile,
	cachedNotes []domain.Note,
) (CacheValidationResult, error) {
	// Collect vault state
	vaultNoteIDs, totalVaultFiles, vaultErr := v.collectVaultState(
		ctx,
		vaultFiles,
	)
	if vaultErr != nil {
		return CacheValidationResult{}, fmt.Errorf(
			"failed to collect vault state for validation: %w",
			vaultErr,
		)
	}

	// Collect cache state
	cacheNoteIDs, totalCacheEntries, cachedNotes, cacheErr := v.collectCacheState(
		ctx,
		cachedNotes,
	)
	if cacheErr != nil {
		return CacheValidationResult{}, fmt.Errorf(
			"failed to collect cache state for validation: %w",
			cacheErr,
		)
	}

	// Find inconsistencies
	orphanedCount, missingCount, orphanedDetails, missingDetails, isConsistent :=
		v.findInconsistencies(
			vaultNoteIDs,
			cacheNoteIDs,
			cachedNotes,
		)

	result := CacheValidationResult{
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

// collectVaultState scans the vault and builds a map of NoteIDs for markdown
// files.
// Returns the NoteID map, total count, and any scanning error.
func (v *VaultIndexer) collectVaultState(
	ctx context.Context,
	cachedVault []dto.VaultFile,
) (
	vaultNoteIDs map[domain.NoteID]bool,
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

	vaultNoteIDs = make(map[domain.NoteID]bool)
	totalFiles = 0
	for i := range vaultFiles {
		vf := vaultFiles[i]
		if vf.Ext == markdownExt {
			noteID := deriveNoteIDFromPath(v.config.VaultPath, vf.Path)
			vaultNoteIDs[noteID] = true
			totalFiles++
		}
	}
	return vaultNoteIDs, totalFiles, nil
}

// collectCacheState retrieves all cached notes and builds a map of NoteIDs.
// Returns the NoteID map, total count, cached notes slice, and any listing
// error.
func (v *VaultIndexer) collectCacheState(
	ctx context.Context,
	preloaded []domain.Note,
) (
	cacheNoteIDs map[domain.NoteID]bool,
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

	cacheNoteIDs = make(map[domain.NoteID]bool)
	totalEntries = len(cachedNotes)
	for i := range cachedNotes {
		note := cachedNotes[i]
		cacheNoteIDs[note.ID] = true
	}
	return
}

// findInconsistencies compares vault and cache NoteID sets to identify
// orphaned and missing entries. Returns orphaned count, missing count,
// orphaned details, missing details, and consistency flag.
func (v *VaultIndexer) findInconsistencies(
	vaultNoteIDs map[domain.NoteID]bool,
	cacheNoteIDs map[domain.NoteID]bool,
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
		if !vaultNoteIDs[note.ID] {
			orphanedCount++
			orphanedDetails = append(orphanedDetails, string(note.ID))
			isConsistent = false
		}
	}

	// Find missing cache entries (in vault but not in cache)
	for noteID := range vaultNoteIDs {
		if !cacheNoteIDs[noteID] {
			missingCount++
			missingDetails = append(missingDetails, string(noteID))
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
//   - stats: IndexStats to update with processing results
func (v *VaultIndexer) processFile(
	ctx context.Context,
	file *dto.VaultFile,
	stats *IndexStats,
) {
	// Filter: only .md files for frontmatter processing
	if file.Ext != markdownExt {
		return
	}

	var noteFrontmatter domain.Frontmatter

	// If frontmatterService is available, use it for extraction and validation
	if v.frontmatterService != nil {
		noteFrontmatter = v.processFileWithFrontmatter(ctx, file, stats)
		if noteFrontmatter.Fields == nil {
			return // Processing failed, stats already updated
		}
		stats.ValidationSuccesses++
	} else {
		// Fallback: create basic empty frontmatter for backward compatibility
		noteFrontmatter = domain.Frontmatter{
			FileClass: "",
			Fields:    map[string]interface{}{},
		}
	}

	// Create Note with frontmatter (validated or basic)
	noteID := deriveNoteIDFromPath(v.config.VaultPath, file.Path)
	note := domain.NewNote(noteID, noteFrontmatter)

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
		Int("validation_successes", stats.ValidationSuccesses).
		Int("validation_failures", stats.ValidationFailures).
		Dur("duration", stats.Duration).
		Msg("vault indexing complete")
}

// logRefreshStats logs incremental refresh statistics using structured logging.
// Provides metrics for incremental update performance monitoring.
//
// Parameters:
//   - stats: Refresh IndexStats to log
//   - since: Timestamp used for the incremental scan
func (v *VaultIndexer) logRefreshStats(
	stats IndexStats,
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
//   - result: CacheValidationResult to log
func (v *VaultIndexer) logCacheValidationResult(result CacheValidationResult) {
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

// deriveNoteIDFromPath creates a NoteID from a file path.
// Preserves vault-relative path information to prevent collisions.
//
// Example: "/vault/projects/foo.md" → "projects/foo.md"
//
// Path Normalization:
// - Converts backslashes to forward slashes for consistency
// - Strips vault root prefix to get relative path
// - Preserves full relative path from vault root
// - Keeps .md extension for uniqueness and clarity.
func deriveNoteIDFromPath(vaultRoot, path string) domain.NoteID {
	// Normalize path separators to forward slashes for cross-platform
	// consistency
	normalizedPath := strings.ReplaceAll(path, "\\", "/")
	normalizedRoot := strings.ReplaceAll(vaultRoot, "\\", "/")

	// Strip vault root prefix to get relative path
	// Ensure root ends with separator for proper stripping
	if !strings.HasSuffix(normalizedRoot, "/") {
		normalizedRoot += "/"
	}

	var relativePath string
	if strings.HasPrefix(normalizedPath, normalizedRoot) {
		relativePath = strings.TrimPrefix(normalizedPath, normalizedRoot)
	} else {
		// Fallback: assume path is already relative
		relativePath = normalizedPath
	}

	return domain.NewNoteID(relativePath)
}

// logValidationError logs frontmatter validation errors with structured
// context.
// Used for tracking validation failures without aborting indexing.
//
// Parameters:
//   - filePath: Path of the file that failed validation
//   - err: The validation error that occurred
func (v *VaultIndexer) logValidationError(filePath string, err error) {
	v.log.Warn().
		Err(err).
		Str("filePath", filePath).
		Msg("frontmatter validation failed")
}

// processFileWithFrontmatter handles frontmatter extraction and validation.
// Helper method to reduce complexity in processFile.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - vf: Vault file to process
//   - stats: IndexStats to update with processing results
//
// Returns:
//   - domain.Frontmatter: Validated frontmatter or empty if processing failed
func (v *VaultIndexer) processFileWithFrontmatter(
	ctx context.Context,
	vf *dto.VaultFile,
	stats *IndexStats,
) domain.Frontmatter {
	// Extract frontmatter from file content
	extractedFM, extractErr := v.frontmatterService.Extract(vf.Content)
	if extractErr != nil {
		v.logValidationError(vf.Path, extractErr)
		stats.ValidationFailures++
		return domain.Frontmatter{} // Return empty to signal failure
	}

	// Get schema for validation if fileClass is present
	if extractedFM.FileClass != "" {
		schemaForValidation, schemaErr := v.getSchemaForValidation(
			ctx,
			extractedFM.FileClass,
		)
		if schemaErr != nil {
			v.logValidationError(vf.Path, schemaErr)
			stats.ValidationFailures++
			return domain.Frontmatter{} // Return empty to signal failure
		}

		// Validate frontmatter against schema
		validationErr := v.frontmatterService.Validate(
			ctx,
			extractedFM,
			schemaForValidation,
		)
		if validationErr != nil {
			v.logValidationError(vf.Path, validationErr)
			stats.ValidationFailures++
			return domain.Frontmatter{} // Return empty to signal failure
		}
	}

	return extractedFM
}

// getSchemaForValidation retrieves a schema for frontmatter validation.
// Helper method that wraps schema engine access with error handling.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - fileClass: The schema name to retrieve
//
// Returns:
//   - domain.Schema: The resolved schema for validation
//   - error: Schema retrieval error if schema not found or engine unavailable
func (v *VaultIndexer) getSchemaForValidation(
	ctx context.Context,
	fileClass string,
) (domain.Schema, error) {
	if v.schemaEngine == nil {
		return domain.Schema{}, fmt.Errorf("schema engine not available")
	}
	// Use generic Get method from SchemaEngine
	return schema.Get[domain.Schema](v.schemaEngine, ctx, fileClass)
}
