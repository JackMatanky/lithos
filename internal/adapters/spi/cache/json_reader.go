package cache

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

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
	id domain.NoteID,
) (domain.Note, error) {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return domain.Note{}, ctx.Err()
	default:
	}

	// Construct file path
	path := noteFilePath(a.config.CacheDir, id)

	data, readErr := a.readFile(path)
	if readErr == nil {
		return a.unmarshalNote(id, path, data)
	}

	if !os.IsNotExist(readErr) {
		return domain.Note{}, lithosErr.NewCacheReadError(
			string(id),
			path,
			"read",
			readErr,
		)
	}

	legacyPath := legacyNoteFilePath(a.config.CacheDir, id)
	legacyData, legacyErr := a.readFile(legacyPath)
	switch {
	case legacyErr == nil:
		return a.unmarshalNote(id, legacyPath, legacyData)
	case os.IsNotExist(legacyErr):
		return domain.Note{}, lithosErr.ErrNotFound
	default:
		return domain.Note{}, lithosErr.NewCacheReadError(
			string(id),
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
	if err := EnsureCacheDir(a.config.CacheDir); err != nil {
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

// extractNoteIDFromPath extracts the note ID from a cache file path.
// Removes the .json extension from the filename.
func extractNoteIDFromPath(path string) domain.NoteID {
	filename := filepath.Base(path)
	if id, ok := decodeNoteIDFromFilename(filename); ok {
		return id
	}
	return domain.NoteID(strings.TrimSuffix(filename, ".json"))
}

// processNoteFile processes a single cache file and returns the note if
// successful.
// Logs warnings for read failures but doesn't fail the entire operation.
func (a *JSONCacheReadAdapter) processNoteFile(
	ctx context.Context,
	path string,
) (domain.Note, bool) {
	noteID := extractNoteIDFromPath(path)

	note, readErr := a.Read(ctx, noteID)
	if readErr != nil {
		a.log.Warn().
			Err(readErr).
			Str("path", path).
			Str("note_id", string(noteID)).
			Msg("failed to read cache file during list operation")
		return domain.Note{}, false
	}

	return note, true
}

func (a *JSONCacheReadAdapter) unmarshalNote(
	id domain.NoteID,
	path string,
	data []byte,
) (domain.Note, error) {
	var note domain.Note
	if unmarshalErr := json.Unmarshal(data, &note); unmarshalErr != nil {
		return domain.Note{}, lithosErr.NewCacheReadError(
			string(id),
			path,
			"unmarshal",
			unmarshalErr,
		)
	}

	a.log.Debug().
		Str("note_id", string(id)).
		Str("path", path).
		Msg("cache read successful")

	return note, nil
}

// ByBasename finds notes by filename without extension via O(n) scanning.
func (a *JSONCacheReadAdapter) ByBasename(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	notes, err := a.List(ctx)
	if err != nil {
		return nil, err
	}

	var results []domain.Note
	for _, note := range notes {
		// Extract basename from NoteID (path)
		path := string(note.ID)
		path = strings.ReplaceAll(path, "\\", "/")
		base := filepath.Base(path)
		if ext := filepath.Ext(base); ext != "" {
			base = strings.TrimSuffix(base, ext)
		}

		if base == basename {
			results = append(results, note)
		}
	}
	return results, nil
}

// ByAlias finds notes by frontmatter alias values via O(n) scanning.
func (a *JSONCacheReadAdapter) ByAlias(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	notes, err := a.List(ctx)
	if err != nil {
		return nil, err
	}

	var results []domain.Note
	for _, note := range notes {
		if hasAlias(note.Frontmatter, alias) {
			results = append(results, note)
		}
	}
	return results, nil
}

// ByFileClass finds notes by schema fileClass value via O(n) scanning.
func (a *JSONCacheReadAdapter) ByFileClass(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	notes, err := a.List(ctx)
	if err != nil {
		return nil, err
	}

	var results []domain.Note
	for _, note := range notes {
		// Check fileClass field from frontmatter (mapped via config key in
		// domain)
		// But here we check the extracted FileClass field
		if note.Frontmatter.FileClass == fileClass {
			results = append(results, note)
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
	for _, note := range notes {
		path := string(note.ID)
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
			results = append(results, note)
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
	for _, note := range notes {
		// Check tags field
		if tags, ok := note.Frontmatter.Fields["tags"]; ok {
			if containsTag(tags, tag) {
				results = append(results, note)
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
	for _, note := range notes {
		if val, ok := note.Frontmatter.Fields[field]; ok {
			// Simple string comparison for now
			if fmt.Sprintf("%v", val) == value {
				results = append(results, note)
			}
		}
	}
	return results, nil
}

// Helpers

func hasAlias(fm domain.Frontmatter, alias string) bool {
	val, ok := fm.Fields["aliases"]
	if !ok {
		return false
	}
	switch v := val.(type) {
	case string:
		return v == alias
	case []string:
		for _, s := range v {
			if s == alias {
				return true
			}
		}
	case []interface{}:
		for _, s := range v {
			if str, valid := s.(string); valid && str == alias {
				return true
			}
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
