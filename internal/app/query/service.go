// Package query provides fast in-memory lookups for indexed notes.
// It implements thread-safe concurrent reads using sync.RWMutex and supports
// the FR9 query capabilities: lookup by ID, path, basename, and schema.
//
// Thread-Safe Design:
// - Multiple readers can query simultaneously via RLock
// - Writes (RefreshFromCache) are exclusive via Lock
// - No data races during concurrent access patterns
//
// In-Memory Indices:
// - byID: Primary index for NoteID → Note lookups (O(1))
// - byPath: Path index for file path → Note lookups (O(1))
// - byBasename: Basename index for filename → []Note lookups (O(log n))
// - byFileClass: Schema index for fileClass → []Note lookups (O(log n))
// - byFrontmatter: Frontmatter index for field → value → []Note lookups
// (O(log n))
//
// complex logic
//
//nolint:cyclop // Query service implements multiple index types requiring
package query

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"
	"reflect"
	"strings"
	"sync"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// QueryService provides fast in-memory lookups for indexed notes.
// It implements thread-safe concurrent reads using sync.RWMutex and supports
// the FR9 query capabilities: lookup by ID, path, basename, schema, and
// frontmatter.
//
// Thread-Safe Design:
// - Multiple readers can query simultaneously via RLock
// - Writes (RefreshFromCache) are exclusive via Lock
// - No data races during concurrent access patterns
//
// In-Memory Indices:
// - byID: Primary index for NoteID → Note lookups (O(1))
// - byPath: Path index for file path → Note lookups (O(1))
// - byBasename: Basename index for filename → []Note lookups (O(log n))
// - byFileClass: Schema index for fileClass → []Note lookups (O(log n))
// - byFrontmatter: Frontmatter index for field → value → []Note lookups
// (O(log n)).
type QueryService struct {
	mu sync.RWMutex

	// Dependencies
	cacheReader spi.CacheReaderPort
	log         zerolog.Logger

	// Primary index: NoteID → Note
	byID map[domain.NoteID]domain.Note

	// Path index: file path → Note
	byPath map[string]domain.Note

	// Basename index: filename without extension → []Note
	byBasename map[string][]domain.Note

	// FileClass index: schema name → []Note
	byFileClass map[string][]domain.Note

	// Frontmatter index: field → value → []Note
	byFrontmatter map[string]map[interface{}][]domain.Note
}

// extractBasenameFromNoteID extracts the filename without extension from a
// NoteID.
// NoteID now contains the full vault-relative path, so we extract the basename.
// Handles both Unix (/) and Windows (\) path separators for cross-platform
// compatibility.
// Example: "projects/notes/meeting.md" → "meeting".
func extractBasenameFromNoteID(id domain.NoteID) string {
	path := string(id)
	// Normalize Windows backslashes to forward slashes for cross-platform
	// compatibility
	path = strings.ReplaceAll(path, "\\", "/")
	base := filepath.Base(path)
	// Remove extension if present
	if ext := filepath.Ext(base); ext != "" {
		base = strings.TrimSuffix(base, ext)
	}
	return base
}

// normalizeFrontmatterValue normalizes frontmatter values for type-agnostic
// comparison.
// Handles numeric type conversions (int 2 == float 2.0) and safe comparison
// for complex types.
// Returns the normalized value and whether normalization was successful.
//
//nolint:cyclop // Type normalization requires exhaustive type checking
func normalizeFrontmatterValue(value interface{}) (interface{}, bool) {
	switch v := value.(type) {
	case int:
		// Convert int to float64 for consistent numeric comparison
		return float64(v), true
	case int8:
		return float64(v), true
	case int16:
		return float64(v), true
	case int32:
		return float64(v), true
	case int64:
		return float64(v), true
	case uint:
		return float64(v), true
	case uint8:
		return float64(v), true
	case uint16:
		return float64(v), true
	case uint32:
		return float64(v), true
	case uint64:
		return float64(v), true
	case float32:
		return float64(v), true
	case float64:
		return v, true
	case string, bool:
		// Strings and booleans are already comparable
		return v, true
	default:
		// Complex types (arrays, maps) are not safely comparable
		// Return false to indicate normalization failed
		return nil, false
	}
}

// isComparableForIndex ensures the provided value can be safely used as a map
// key. Non-comparable types like slices and maps would panic if used directly.
func isComparableForIndex(value interface{}) bool {
	if value == nil {
		return false
	}
	val := reflect.ValueOf(value)
	return val.IsValid() && val.Type().Comparable()
}

// NewQueryService creates a new QueryService with thread-safe in-memory
// indices.
// It initializes all index maps and injects required dependencies.
// The service is ready for concurrent access immediately after construction.
//
// Thread-Safe Design:
// - RWMutex enables multiple concurrent reads, exclusive writes
// - All indices start empty and are populated via RefreshFromCache()
// - Dependencies are injected (no globals) for testability
//
// Usage:
//
//	qs := NewQueryService(cacheReader, logger)
//	err := qs.RefreshFromCache(ctx) // Populate indices
//	note, err := qs.ByID(ctx, id)   // Query safely
func NewQueryService(
	cacheReader spi.CacheReaderPort,
	log zerolog.Logger,
) *QueryService {
	return &QueryService{
		byID:          make(map[domain.NoteID]domain.Note),
		byPath:        make(map[string]domain.Note),
		byBasename:    make(map[string][]domain.Note),
		byFileClass:   make(map[string][]domain.Note),
		byFrontmatter: make(map[string]map[interface{}][]domain.Note),
		cacheReader:   cacheReader,
		log:           log,
		// mu is initialized to zero value (unlocked state)
		mu: sync.RWMutex{},
	}
}

// ByID retrieves a note by its NoteID.
// Returns the note if found, or ResourceError if not found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns ResourceError for missing notes (single result lookup)
// - Logs debug message with NoteID for troubleshooting
// - O(1) lookup performance via map access.
func (q *QueryService) ByID(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	q.mu.RLock()
	defer q.mu.RUnlock()

	note, exists := q.byID[id]
	if !exists {
		return domain.Note{}, domainerrors.NewResourceError(
			"note",
			"get",
			id.String(),
			errors.New("not found"),
		)
	}

	q.log.Debug().Str("noteID", id.String()).Msg("query by ID")
	return note, nil
}

// ByPath retrieves a note by its file path.
// Returns the note if found, or ResourceError if not found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns ResourceError for missing notes (single result lookup)
// - Logs debug message with path for troubleshooting
// - O(1) lookup performance via map access.
func (q *QueryService) ByPath(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	q.mu.RLock()
	defer q.mu.RUnlock()

	note, exists := q.byPath[path]
	if !exists {
		return domain.Note{}, domainerrors.NewResourceError(
			"note",
			"get",
			path,
			errors.New("not found"),
		)
	}

	q.log.Debug().Str("path", path).Msg("query by path")
	return note, nil
}

// ByFileClass retrieves all notes matching a schema name (fileClass).
// Returns a slice of notes if any match, or empty slice if none found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching schemas (collection
// lookup)
// - Logs debug message with fileClass and result count
// - O(log n) lookup performance via map access.
func (q *QueryService) ByFileClass(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	return q.queryByField(q.byFileClass, "fileClass", fileClass)
}

// ByBasename retrieves all notes matching a filename basename (without
// extension).
// Returns a slice of notes if any match, or empty slice if none found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching basenames (collection
// lookup)
// - Basename is extracted from NoteID (full path) by removing directory path
// and file extension
// - Logs debug message with basename and result count
// - O(log n) lookup performance via map access.
//
// Example: NoteID "projects/notes/meeting.md" matches basename "meeting".
func (q *QueryService) ByBasename(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	return q.queryByField(q.byBasename, "basename", basename)
}

// ByFrontmatter retrieves all notes matching a frontmatter field value.
// Returns a slice of notes if any match, or empty slice if none found.
// Thread-safe: uses RLock to allow concurrent reads.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching field/value pairs
// - Supports type-agnostic queries (int 2 matches float 2.0)
// - Handles safe comparison for primitive types only
// - Logs debug message with field, value, and result count
// - O(log n) lookup performance via nested map access.
//
// Usage Examples:
//
//	notes := queryService.ByFrontmatter("author", "John Doe")
//	notes := queryService.ByFrontmatter("tags", "project-x")
//	notes := queryService.ByFrontmatter("status", "draft")
//	notes := queryService.ByFrontmatter("priority", 2) // matches float 2.0
func (q *QueryService) ByFrontmatter(
	ctx context.Context,
	field string,
	value interface{},
) ([]domain.Note, error) {
	q.mu.RLock()
	defer q.mu.RUnlock()

	// Check if field exists in index
	fieldMap, fieldExists := q.byFrontmatter[field]
	if !fieldExists {
		return nil, nil // Return empty slice for non-existent field
	}

	// Try direct lookup first (for exact matches)
	if isComparableForIndex(value) {
		if notes, exists := fieldMap[value]; exists && len(notes) > 0 {
			q.log.Debug().
				Str("field", field).
				Interface("value", value).
				Int("count", len(notes)).
				Msg("query by frontmatter (direct match)")
			return notes, nil
		}
	} else {
		q.log.Debug().
			Str("field", field).
			Interface("value_type", fmt.Sprintf("%T", value)).
			Msg("query by frontmatter (value not comparable)")
	}

	// Try normalized lookup for type-agnostic matching
	if normalizedValue, ok := normalizeFrontmatterValue(value); ok {
		if !isComparableForIndex(normalizedValue) {
			q.log.Debug().
				Str("field", field).
				Interface("value", value).
				Msg("query by frontmatter (normalized value not comparable)")
			return nil, nil
		}
		if notes, exists := fieldMap[normalizedValue]; exists &&
			len(notes) > 0 {
			q.log.Debug().
				Str("field", field).
				Interface("original_value", value).
				Interface("normalized_value", normalizedValue).
				Int("count", len(notes)).
				Msg("query by frontmatter (normalized match)")
			return notes, nil
		}
	}

	// No matches found
	q.log.Debug().
		Str("field", field).
		Interface("value", value).
		Msg("query by frontmatter (no matches)")
	return nil, nil
}

// RefreshFromCache rebuilds all in-memory indices from the persistent cache.
// This method should be called during app startup and when cache is
// invalidated.
// Thread-safe: uses Lock for exclusive write access during rebuild.
//
// Rebuild Process:
// - Reads all notes from CacheReaderPort
// - Handles missing cache directory gracefully (fresh installations)
// - Clears existing indices to prevent stale data
// - Populates all indices (byID, byPath, byBasename, byFileClass,
// byFrontmatter)
// - Logs info message with total note count
//
// When to Call:
// - Application startup after cache initialization
// - Cache invalidation events
// - Manual cache refresh operations
//
// Error Handling:
// - Returns error if cache read fails (except missing directory)
// - Handles missing cache directory as empty cache (fresh installation)
// - Preserves existing indices if rebuild fails.
func (q *QueryService) RefreshFromCache(ctx context.Context) error {
	q.log.Info().Msg("refreshing query service from cache")

	notes, err := q.loadNotesForRefresh(ctx)
	if err != nil {
		return err
	}

	q.rebuildIndices(notes)
	return nil
}

func (q *QueryService) loadNotesForRefresh(
	ctx context.Context,
) ([]domain.Note, error) {
	notes, err := q.cacheReader.List(ctx)
	if err == nil {
		return notes, nil
	}

	if strings.Contains(err.Error(), "no such file or directory") ||
		strings.Contains(err.Error(), "directory not found") {
		q.log.Info().
			Msg("cache directory missing, treating as fresh installation")
		return []domain.Note{}, nil
	}

	return nil, fmt.Errorf("cache refresh failed: %w", err)
}

func (q *QueryService) rebuildIndices(notes []domain.Note) {
	q.mu.Lock()
	defer q.mu.Unlock()

	// Clear existing indices
	q.byID = make(map[domain.NoteID]domain.Note)
	q.byPath = make(map[string]domain.Note)
	q.byBasename = make(map[string][]domain.Note)
	q.byFileClass = make(map[string][]domain.Note)
	q.byFrontmatter = make(map[string]map[interface{}][]domain.Note)

	// Populate indices from cache
	for _, note := range notes {
		q.byID[note.ID] = note

		// Populate byPath index using the Path field
		q.byPath[note.Path] = note

		// Populate byBasename index using extracted basename
		basename := extractBasenameFromNoteID(note.ID)
		q.byBasename[basename] = append(q.byBasename[basename], note)

		if note.Frontmatter.FileClass != "" {
			q.byFileClass[note.Frontmatter.FileClass] = append(
				q.byFileClass[note.Frontmatter.FileClass],
				note,
			)
		}

		// Populate frontmatter index for all fields
		for field, value := range note.Frontmatter.Fields {
			if q.byFrontmatter[field] == nil {
				q.byFrontmatter[field] = make(map[interface{}][]domain.Note)
			}

			// Store with original value for exact matches
			if isComparableForIndex(value) {
				q.byFrontmatter[field][value] = append(
					q.byFrontmatter[field][value],
					note,
				)
			} else {
				q.log.Debug().
					Str("field", field).
					Interface("value_type", fmt.Sprintf("%T", value)).
					Str("note_id", note.ID.String()).
					Msg("skipping frontmatter index for non-comparable value")
			}

			// Also store with normalized value for type-agnostic queries
			if normalizedValue, ok := normalizeFrontmatterValue(value); ok &&
				normalizedValue != value && isComparableForIndex(normalizedValue) {
				q.byFrontmatter[field][normalizedValue] = append(
					q.byFrontmatter[field][normalizedValue],
					note,
				)
			}
		}
	}

	q.log.Info().Int("count", len(notes)).Msg("query service refreshed")
}

// queryByField performs a thread-safe lookup in a note index map.
// Returns a slice of notes if any match, or empty slice if none found.
// Used by ByFileClass, ByBasename, and other index-based query methods.
func (q *QueryService) queryByField(
	index map[string][]domain.Note,
	fieldName, fieldValue string,
) ([]domain.Note, error) {
	q.mu.RLock()
	defer q.mu.RUnlock()

	notes, exists := index[fieldValue]
	if !exists || len(notes) == 0 {
		return nil, nil // Return empty slice, not error
	}

	q.log.Debug().
		Str(fieldName, fieldValue).
		Int("count", len(notes)).
		Msg(fmt.Sprintf("query by %s", fieldName))
	return notes, nil
}
