package vault

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/dto"
	"github.com/rs/zerolog"
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
// operations to SchemaEngine, and caching to CacheWriterPort.
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
	frontmatterService *frontmatter.FrontmatterService,
	schemaEngine *schema.SchemaEngine,
	config domain.Config,
	log zerolog.Logger,
) *VaultIndexer {
	return &VaultIndexer{
		vaultScanner:       vaultScanner,
		cacheWriter:        cacheWriter,
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
		v.processFile(ctx, vaultFiles[i], &stats)
	}

	stats.Duration = time.Since(startTime)

	// Step 4: Log summary
	v.logStats(stats)

	return stats, nil
}

// Refresh performs incremental vault indexing for large vault optimization.
// Only processes files modified since the specified timestamp.
//
// Workflow Steps:
// 1. Scan modified files using scanModifiedFiles()
// 2. Process each modified file using processFile()
// 3. Log incremental update statistics
//
// Error Handling:
// - Vault scan failures: Return error immediately (abort refresh)
// - Cache write failures: Log warning, increment CacheFailures, continue
// processing
// - Partial success acceptable - update what we can
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - since: Only process files modified after this timestamp
//
// Returns:
//   - error: Vault scan errors only (cache failures logged but don't abort)
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

	// Step 1: Scan modified files
	vaultFiles, err := v.scanModifiedFiles(ctx, since)
	if err != nil {
		return err
	}
	stats.ScannedCount = len(vaultFiles)

	// Step 2: Process each modified file
	for i := range vaultFiles {
		v.processFile(ctx, vaultFiles[i], &stats)
	}

	stats.Duration = time.Since(startTime)

	// Step 3: Log incremental update
	v.logRefreshStats(stats, since)

	return nil
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
// processing,
// note creation, and persistence.
// Updates stats for tracking and logging.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - vf: Vault file to process
//   - stats: IndexStats to update with processing results
func (v *VaultIndexer) processFile(
	ctx context.Context,
	file dto.VaultFile,
	stats *IndexStats,
) {
	// Filter: only .md files for frontmatter processing
	if file.Ext != ".md" {
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
	vf dto.VaultFile,
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
