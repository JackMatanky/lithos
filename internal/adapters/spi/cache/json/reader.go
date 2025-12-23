package json

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// Compile-time interface compliance check.
// This ensures JSONCacheReadAdapter implements CacheReaderPort correctly.
// Will fail to compile if the interface contract is not satisfied.
var _ spi.CacheReaderPort = (*JSONCacheReadAdapter)(nil)
var _ spi.MetadataQueryPort = (*JSONCacheReadAdapter)(nil)

// JSONCacheReadAdapter implements CacheReaderPort for filesystem-based
// note retrieval with unknown field preservation. It uses the CQRS read-side
// pattern to provide efficient querying and lazy loading of cached notes.
//
// Unknown Field Preservation (FR6):
//   - Preserves all JSON fields during deserialization using flexible
//     unmarshaling
//   - Ensures round-trip compatibility for user-defined fields in
//     Frontmatter.Fields
//   - Uses map[string]interface{} for unknown field storage
//
// Partial Failure Tolerance:
//   - List method continues processing when individual notes fail to load
//   - Logs warnings for unreadable files but returns partial results
//   - Maintains system availability even with corrupted cache entries
//
// Thread Safety:
//   - Safe for concurrent reads from multiple services (QueryService +
//     FrontmatterService)
//   - No shared mutable state beyond configuration
//   - Filesystem operations provide OS-level consistency guarantees
//
// See docs/architecture/components.md#jsoncachereadapter for implementation
// guidance.
type JSONCacheReadAdapter struct {
	config   domain.Config
	log      zerolog.Logger
	readFile func(string) ([]byte, error)
	walkDir  func(string, filepath.WalkFunc) error
}

// NewJSONCacheReader creates a new JSONCacheReadAdapter with the provided
// configuration and logger. The adapter implements read-side CQRS operations
// for cache retrieval and is thread-safe for concurrent access.
//
// Parameters:
//   - config: Application configuration containing CacheDir path
//   - log: Structured logger for operation tracking
//
// Returns:
//   - *JSONCacheReadAdapter: Configured adapter ready for cache operations
//
// Thread Safety: The returned adapter is safe for concurrent use.
func NewJSONCacheReader(
	config domain.Config,
	log zerolog.Logger,
) *JSONCacheReadAdapter {
	return &JSONCacheReadAdapter{
		config:   config,
		log:      log,
		readFile: os.ReadFile,
		walkDir:  filepath.Walk,
	}
}

// Read retrieves a single note from cache by ID.
// Returns ErrNotFound if note doesn't exist in cache.
// Preserves unknown JSON fields per FR6 requirement.
//
// Retrieval Behavior:
// - Returns domain.Note with preserved unknown fields
// - Uses JSON deserialization with flexible field handling
// - Optional in-memory caching for performance
//
// Error Conditions:
// - ErrNotFound: Note doesn't exist in cache
// - Wrapped errors: JSON parsing, file access, permission issues
//
// Thread-safe: Safe for concurrent calls.
// Context: Respects ctx cancellation, returns ctx.Err() if canceled.
// Errors: Wrapped with operation context and resource identifiers (FR9).
func (a *JSONCacheReadAdapter) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return domain.Note{}, ctx.Err()
	default:
	}

	// Construct file path
	filePath := cache.NoteFilePath(a.config.CacheDir, path)

	data, readErr := a.readFile(filePath)
	if readErr == nil {
		return a.unmarshalNote(path, filePath, data)
	}

	if !os.IsNotExist(readErr) {
		return domain.Note{}, lithosErr.NewCacheReadError(
			path,
			filePath,
			"read",
			readErr,
		)
	}

	legacyPath := cache.LegacyNoteFilePath(a.config.CacheDir, path)
	legacyData, legacyErr := a.readFile(legacyPath)
	switch {
	case legacyErr == nil:
		return a.unmarshalNote(path, legacyPath, legacyData)
	case os.IsNotExist(legacyErr):
		return domain.Note{}, lithosErr.ErrNotFound
	default:
		return domain.Note{}, lithosErr.NewCacheReadError(
			path,
			legacyPath,
			"read_legacy",
			legacyErr,
		)
	}
}

// List returns all notes currently in the cache.
// May return partial results with warnings if some notes fail to load.
// Preserves unknown JSON fields for all returned notes per FR6.
//
// Listing Behavior:
// - Returns []domain.Note with all cached notes
// - Partial failure tolerance: logs warnings but continues
// - No guaranteed ordering of results
// - Optional in-memory caching for performance
//
// Error Handling:
// - Returns error only for critical failures (directory access)
// - Individual note read failures logged as warnings
// - Preserves as many notes as possible
//
// Thread-safe: Safe for concurrent calls.
// Context: Respects ctx cancellation, returns ctx.Err() if canceled.
// Errors: Wrapped with operation context (FR9).
func (a *JSONCacheReadAdapter) List(
	ctx context.Context,
) ([]domain.Note, error) {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	// Ensure cache directory exists for graceful first access
	if err := cache.EnsureCacheDir(a.config.CacheDir); err != nil {
		return nil, lithosErr.NewCacheReadError(
			"",
			a.config.CacheDir,
			"ensure_cache_dir",
			err,
		)
	}

	var notes []domain.Note

	// Walk the cache directory
	walkErr := a.walkDir(
		a.config.CacheDir,
		func(path string, info os.FileInfo, err error) error {
			// Check for context cancellation during walk
			select {
			case <-ctx.Done():
				return ctx.Err()
			default:
			}

			if err != nil {
				return err
			}

			if !shouldIncludeFile(info, path) {
				return nil
			}

			if note, ok := a.processNoteFile(ctx, path); ok {
				notes = append(notes, note)
			}

			return nil
		},
	)

	if walkErr != nil {
		return nil, lithosErr.NewCacheReadError(
			"",
			a.config.CacheDir,
			"list",
			walkErr,
		)
	}

	// Log successful list operation
	a.log.Debug().
		Int("note_count", len(notes)).
		Str("cache_dir", a.config.CacheDir).
		Msg("cache list operation completed")

	return notes, nil
}

// shouldIncludeFile determines if a file should be included in cache listing.
// Only .json files are included, directories are excluded.
func shouldIncludeFile(info os.FileInfo, path string) bool {
	return !info.IsDir() && filepath.Ext(path) == ".json"
}

// extractPathFromCachePath extracts the note path from a cache file path.
// Removes the .json extension and decodes the filename.
func extractPathFromCachePath(path string) string {
	filename := filepath.Base(path)
	if decodedPath, ok := cache.DecodePathFromFilename(filename); ok {
		return decodedPath
	}
	return strings.TrimSuffix(filename, ".json")
}

// BasenameQuery finds notes by filename without extension via O(n) scanning.
func (a *JSONCacheReadAdapter) BasenameQuery(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	notes, err := a.List(ctx)
	if err != nil {
		return nil, err
	}

	var results []domain.Note
	for i := range notes {
		// Extract basename from path
		path := strings.ReplaceAll(notes[i].Path, "\\", "/")
		base := filepath.Base(path)
		if ext := filepath.Ext(base); ext != "" {
			base = strings.TrimSuffix(base, ext)
		}

		if base == basename {
			results = append(results, notes[i])
		}
	}
	return results, nil
}

// AliasQuery finds notes by frontmatter alias values via O(n) scanning.
func (a *JSONCacheReadAdapter) AliasQuery(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	notes, err := a.List(ctx)
	if err != nil {
		return nil, err
	}

	var results []domain.Note
	for i := range notes {
		if hasAlias(notes[i].Frontmatter, alias) {
			results = append(results, notes[i])
		}
	}
	return results, nil
}

// FileClassQuery finds notes by schema fileClass value via O(n) scanning.
func (a *JSONCacheReadAdapter) FileClassQuery(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	notes, err := a.List(ctx)
	if err != nil {
		return nil, err
	}

	var results []domain.Note
	for i := range notes {
		// Check fileClass field from frontmatter (mapped via config key in
		// domain)
		// But here we check the extracted FileClass field
		if notes[i].Frontmatter.FileClass() == fileClass {
			results = append(results, notes[i])
		}
	}
	return results, nil
}

// PathQuery finds notes using a flexible path selector via O(n) scanning.
func (a *JSONCacheReadAdapter) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	normalized, err := opts.Validate()
	if err != nil {
		return nil, err
	}

	notes, err := a.List(ctx)
	if err != nil {
		return nil, err
	}

	var results []domain.Note
	for i := range notes {
		path := notes[i].Path
		match := false

		switch normalized.Scope {
		case spi.PathQueryScopeFull:
			match = path == normalized.Value
		case spi.PathQueryScopeBasename:
			base := filepath.Base(path)
			if ext := filepath.Ext(base); ext != "" {
				base = strings.TrimSuffix(base, ext)
			}
			match = base == normalized.Value
		case spi.PathQueryScopeFolder:
			match = strings.HasPrefix(path, normalized.Value)
		}

		if match {
			results = append(results, notes[i])
		}
	}
	return results, nil
}

// TagQuery finds notes containing a specific tag via O(n) scanning.
func (a *JSONCacheReadAdapter) TagQuery(
	ctx context.Context,
	tag string,
) ([]domain.Note, error) {
	notes, err := a.List(ctx)
	if err != nil {
		return nil, err
	}

	var results []domain.Note
	for i := range notes {
		// Check tags field
		if tags, ok := notes[i].Frontmatter.Get("tags"); ok {
			if containsTag(tags, tag) {
				results = append(results, notes[i])
			}
		}
	}
	return results, nil
}

// FrontmatterQuery finds notes where a specific frontmatter field matches a
// value via O(n) scanning.
func (a *JSONCacheReadAdapter) FrontmatterQuery(
	ctx context.Context,
	field, value string,
) ([]domain.Note, error) {
	notes, err := a.List(ctx)
	if err != nil {
		return nil, err
	}

	var results []domain.Note
	for i := range notes {
		if val, ok := notes[i].Frontmatter.Get(field); ok {
			// Simple string comparison for now
			if fmt.Sprintf("%v", val) == value {
				results = append(results, notes[i])
			}
		}
	}
	return results, nil
}

// processNoteFile processes a single cache file and returns the note if
// successful.
// Logs warnings for read failures but doesn't fail the entire operation.
func (a *JSONCacheReadAdapter) processNoteFile(
	ctx context.Context,
	path string,
) (domain.Note, bool) {
	notePath := extractPathFromCachePath(path)

	note, readErr := a.Read(ctx, notePath)
	if readErr != nil {
		a.log.Warn().
			Err(readErr).
			Str("file_path", path).
			Str("note_path", notePath).
			Msg("failed to read cache file during list operation")
		return domain.Note{}, false
	}

	return note, true
}

func (a *JSONCacheReadAdapter) unmarshalNote(
	notePath string,
	filePath string,
	data []byte,
) (domain.Note, error) {
	var note domain.Note
	if unmarshalErr := json.Unmarshal(data, &note); unmarshalErr != nil {
		return domain.Note{}, lithosErr.NewCacheReadError(
			notePath,
			filePath,
			"unmarshal",
			unmarshalErr,
		)
	}

	a.log.Debug().
		Str("note_path", notePath).
		Str("file_path", filePath).
		Msg("cache read successful")

	return note, nil
}

// Helpers

func hasAlias(fm domain.Frontmatter, alias string) bool {
	aliases := fm.Aliases()
	for _, a := range aliases {
		if a == alias {
			return true
		}
	}
	return false
}

func containsTag(tags interface{}, tag string) bool {
	switch v := tags.(type) {
	case string:
		return containsTagString(v, tag)
	case []string:
		return containsTagSlice(v, tag)
	case []interface{}:
		return containsTagInterface(v, tag)
	}
	return false
}

func containsTagString(v, tag string) bool {
	parts := strings.Split(v, ",")
	for _, p := range parts {
		if strings.TrimSpace(p) == tag {
			return true
		}
	}
	return false
}

func containsTagSlice(v []string, tag string) bool {
	for _, t := range v {
		if t == tag {
			return true
		}
	}
	return false
}

func containsTagInterface(v []interface{}, tag string) bool {
	for _, t := range v {
		if str, ok := t.(string); ok && str == tag {
			return true
		}
	}
	return false
}
