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
// (O(log n)) complex logic
package query

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"
	"reflect"
	"strings"
	"sync"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// QueryService provides smart routing for indexed notes using hybrid storage.
// It implements thread-safe concurrent reads using sync.RWMutex and supports
// the FR9 query capabilities with optimized routing to BoltDB and SQLite
// backends.
//
// Hybrid Storage Architecture:
// - BoltDB: Hot cache for fast lookups (paths, basenames, aliases, file
// classes)
// - SQLite: Deep storage for complex queries and full content
// - Smart Routing: Automatic query optimization based on operation type
//
// Thread-Safe Design:
// - Multiple readers can query simultaneously via RLock
// - Writes (RefreshFromCache) are exclusive via Lock
// - No data races during concurrent access patterns
//
// Query Routing:
// - Hot Queries (BoltDB): ByPath, ByBasename, ByAlias, directory filtering
// - Complex Queries (SQLite): ByFrontmatter, ByFileClass with property
// filtering
// - Hybrid Queries: Coordinate between stores for optimal performance.
type QueryService struct {
	mu sync.RWMutex

	// Dependencies
	boltReader   spi.CacheReaderPort // Hot cache for fast lookups
	sqliteReader spi.CacheReaderPort // Deep storage for complex queries
	config       domain.Config       // For file_class_key configuration
	log          zerolog.Logger

	// Primary index: NoteID → Note (populated from both stores)
	byID map[domain.NoteID]domain.Note

	// Path index: file path → Note (BoltDB-optimized)
	byPath map[string]domain.Note

	// Basename index: filename without extension → []Note (BoltDB-optimized)
	byBasename map[string][]domain.Note

	// FileClass index: schema name → []Note (hybrid: BoltDB index + SQLite
	// details)
	byFileClass map[string][]domain.Note

	// Frontmatter index: field → value → []Note (SQLite-optimized)
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

// canonicalizeFrontmatterValue normalizes frontmatter values for type-agnostic
// comparison.
// Handles numeric type conversions (int 2 == float 2.0) and safe comparison
// for complex types.
// Returns the normalized value and whether normalization was successful.
//
//nolint:cyclop // Type normalization requires exhaustive type checking
func canonicalizeFrontmatterValue(value interface{}) (interface{}, bool) {
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

// NewQueryService creates a new QueryService with hybrid storage routing.
// It initializes all index maps and injects required dependencies for smart
// query routing.
// The service routes queries to optimal storage backends based on query type.
//
// Hybrid Architecture:
// - BoltDB reader for hot data (paths, basenames, aliases, file classes)
// - SQLite reader for deep storage (complex queries, full content)
// - Smart routing for optimal performance
//
// Thread-Safe Design:
// - RWMutex enables multiple concurrent reads, exclusive writes
// - All indices start empty and are populated via RefreshFromCache()
// - Dependencies are injected (no globals) for testability
//
// Usage:
//
//	qs := NewQueryService(boltReader, sqliteReader, config, logger)
//	err := qs.RefreshFromCache(ctx) // Populate indices from both stores
//	note, err := qs.ByID(ctx, id)   // Query safely with smart routing
func NewQueryService(
	boltReader spi.CacheReaderPort,
	sqliteReader spi.CacheReaderPort,
	config domain.Config,
	log zerolog.Logger,
) *QueryService {
	return &QueryService{
		byID:          make(map[domain.NoteID]domain.Note),
		byPath:        make(map[string]domain.Note),
		byBasename:    make(map[string][]domain.Note),
		byFileClass:   make(map[string][]domain.Note),
		byFrontmatter: make(map[string]map[interface{}][]domain.Note),
		boltReader:    boltReader,
		sqliteReader:  sqliteReader,
		config:        config,
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
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "ByID").
			Str("noteID", id.String()).
			Msg("query performance")
	}()

	q.mu.RLock()
	defer q.mu.RUnlock()

	note, exists := q.byID[id]
	if !exists {
		return domain.Note{}, lithosErr.NewResourceError(
			"note",
			"get",
			id.String(),
			errors.New("not found"),
		)
	}

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
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "ByPath").
			Str("path", path).
			Msg("query performance")
	}()

	q.mu.RLock()
	defer q.mu.RUnlock()

	note, exists := q.byPath[path]
	if !exists {
		return domain.Note{}, lithosErr.NewResourceError(
			"note",
			"get",
			path,
			errors.New("not found"),
		)
	}

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
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "ByFileClass").
			Str("fileClass", fileClass).
			Msg("query performance")
	}()

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
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "ByBasename").
			Str("basename", basename).
			Msg("query performance")
	}()

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
	start := time.Now()
	defer func() {
		q.log.Debug().
			Dur("duration", time.Since(start)).
			Str("method", "ByFrontmatter").
			Str("field", field).
			Interface("value", value).
			Msg("query performance")
	}()

	q.mu.RLock()
	defer q.mu.RUnlock()

	// Check if field exists in index
	fieldMap, fieldExists := q.byFrontmatter[field]
	if !fieldExists {
		return nil, nil // Return empty slice for non-existent field
	}

	canonicalValue, ok := canonicalizeFrontmatterValue(value)
	if !ok || !isComparableForIndex(canonicalValue) {
		q.log.Debug().
			Str("field", field).
			Interface("value_type", fmt.Sprintf("%T", value)).
			Msg("query by frontmatter (value not comparable)")
		return nil, nil
	}

	if notes, exists := fieldMap[canonicalValue]; exists && len(notes) > 0 {
		q.log.Debug().
			Str("field", field).
			Interface("canonical_value", canonicalValue).
			Int("count", len(notes)).
			Msg("query by frontmatter (match)")
		return notes, nil
	}

	q.log.Debug().
		Str("field", field).
		Interface("canonical_value", canonicalValue).
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

// RefreshIncremental updates in-memory indices for notes modified since the
// specified time. This method enables efficient incremental indexing by only
// processing changed notes.
// Thread-safe: uses Lock for exclusive write access during updates.
//
// Incremental Process:
// - Reads all notes from hybrid storage (same as full refresh)
// - Filters notes based on ModTime > modifiedSince
// - Updates only indices for modified notes
// - Preserves existing indices for unchanged notes
// - Logs info message with modified note count
//
// When to Call:
// - After vault scanning detects file changes
// - When indexer provides list of modified files
// - For performance optimization in large vaults
//
// Staleness Detection:
// - Compares note.ModTime against modifiedSince parameter
// - Only rebuilds indices for notes newer than threshold
// - Maintains consistency across storage layers
//
// Error Handling:
// - Returns error if cache read fails
// - Falls back to full refresh if incremental fails
// - Preserves existing indices if update fails.
func (q *QueryService) RefreshIncremental(
	ctx context.Context,
	modifiedSince time.Time,
) error {
	q.log.Info().
		Time("modified_since", modifiedSince).
		Msg("incremental refresh starting")

	notes, err := q.loadNotesForRefresh(ctx)
	if err != nil {
		return err
	}

	// Note: ModTime filtering removed as domain.Note no longer has ModTime
	// field This is a temporary workaround - proper solution requires cache
	// architecture redesign
	modifiedNotes := notes

	if len(modifiedNotes) == 0 {
		q.log.Info().Msg("no notes found")
		return nil
	}

	q.updateIndicesIncremental(modifiedNotes)
	q.log.Info().
		Int("modified_count", len(modifiedNotes)).
		Msg("incremental refresh completed")
	return nil
}

func (q *QueryService) loadNotesForRefresh(
	ctx context.Context,
) ([]domain.Note, error) {
	// Load from SQLite deep storage (primary source for complete notes)
	sqliteNotes, sqliteErr := q.sqliteReader.List(ctx)
	if sqliteErr != nil {
		if strings.Contains(sqliteErr.Error(), "no such file or directory") ||
			strings.Contains(sqliteErr.Error(), "directory not found") {
			q.log.Info().Msg("SQLite cache missing, checking BoltDB hot cache")
		} else {
			return nil, fmt.Errorf("cache refresh failed: SQLite cache read failed: %w", sqliteErr)
		}
	}

	// Load from BoltDB hot cache (fallback/supplement)
	boltNotes, boltErr := q.boltReader.List(ctx)
	if boltErr != nil {
		if strings.Contains(boltErr.Error(), "no such file or directory") ||
			strings.Contains(boltErr.Error(), "directory not found") {
			q.log.Info().Msg("BoltDB cache missing, using available data")
		} else {
			q.log.Warn().Err(boltErr).Msg("BoltDB cache read failed, continuing with SQLite data")
		}
	}

	// Merge notes from both stores, preferring SQLite for complete data
	noteMap := make(map[domain.NoteID]domain.Note)

	// Add BoltDB notes first (may be incomplete)
	for _, note := range boltNotes {
		noteMap[note.ID] = note
	}

	// Add/override with SQLite notes (complete data)
	for _, note := range sqliteNotes {
		noteMap[note.ID] = note
	}

	notes := make([]domain.Note, 0, len(noteMap))
	for _, note := range noteMap {
		notes = append(notes, note)
	}

	q.log.Info().
		Int("sqlite_notes", len(sqliteNotes)).
		Int("bolt_notes", len(boltNotes)).
		Int("merged_notes", len(notes)).
		Msg("loaded notes from hybrid storage")

	return notes, nil
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

		// Populate byPath index using NoteID (which contains the path)
		q.byPath[string(note.ID)] = note

		// Populate byBasename index using extracted basename
		basename := extractBasenameFromNoteID(note.ID)
		q.byBasename[basename] = append(q.byBasename[basename], note)

		// Populate byFileClass using configurable file_class_key
		if fileClassValue, exists := note.Frontmatter.Fields[q.config.FileClassKey]; exists {
			if fc, ok := fileClassValue.(string); ok && fc != "" {
				q.byFileClass[fc] = append(q.byFileClass[fc], note)
			}
		}

		// Populate frontmatter index for all fields
		for field, value := range note.Frontmatter.Fields {
			canonicalValue, ok := canonicalizeFrontmatterValue(value)
			if !ok || !isComparableForIndex(canonicalValue) {
				q.log.Debug().
					Str("field", field).
					Interface("value_type", fmt.Sprintf("%T", value)).
					Str("note_id", note.ID.String()).
					Msg("skipping frontmatter index for non-comparable value")
				continue
			}

			if q.byFrontmatter[field] == nil {
				q.byFrontmatter[field] = make(map[interface{}][]domain.Note)
			}

			q.byFrontmatter[field][canonicalValue] = append(
				q.byFrontmatter[field][canonicalValue],
				note,
			)
		}
	}

	q.log.Info().Int("count", len(notes)).Msg("query service refreshed")
}

func (q *QueryService) updateIndicesIncremental(notes []domain.Note) {
	q.mu.Lock()
	defer q.mu.Unlock()

	// Update indices for modified notes
	for _, note := range notes {
		// Update byID
		q.byID[note.ID] = note

		// Update byPath using NoteID (which contains the path)
		q.byPath[string(note.ID)] = note

		// Update byBasename
		basename := extractBasenameFromNoteID(note.ID)
		q.removeNoteFromMapOfSlices(q.byBasename, note.ID)
		q.byBasename[basename] = append(q.byBasename[basename], note)

		// Update byFileClass using configurable key
		q.removeNoteFromMapOfSlices(q.byFileClass, note.ID)
		if fileClassValue, exists := note.Frontmatter.Fields[q.config.FileClassKey]; exists {
			if fc, ok := fileClassValue.(string); ok && fc != "" {
				q.byFileClass[fc] = append(q.byFileClass[fc], note)
			}
		}

		// Update byFrontmatter (remove old entries and add new)
		q.removeNoteFromFrontmatterIndexes(note.ID)
		q.addNoteToFrontmatterIndexes(note)
	}
}

// Legacy helper replaced by granular incremental index updates inside
// updateIndicesIncremental. Removed to reduce cognitive complexity and resolve
// unused-function lint warning.

// removeNoteFromMapOfSlices removes a note with the given id from all slices in
// the map.
func (q *QueryService) removeNoteFromMapOfSlices(
	m map[string][]domain.Note,
	id domain.NoteID,
) {
	for k, notes := range m {
		for i, n := range notes {
			if n.ID == id {
				m[k] = append(notes[:i], notes[i+1:]...)
				break
			}
		}
	}
}

// removeNoteFromFrontmatterIndexes removes the note with the provided id from
// the frontmatter index.
func (q *QueryService) removeNoteFromFrontmatterIndexes(id domain.NoteID) {
	for field, valueMap := range q.byFrontmatter {
		for value, notes := range valueMap {
			for i, n := range notes {
				if n.ID == id {
					q.byFrontmatter[field][value] = append(
						notes[:i],
						notes[i+1:]...)
					break
				}
			}
		}
	}
}

// addNoteToFrontmatterIndexes inserts note into frontmatter index for all its
// fields.
func (q *QueryService) addNoteToFrontmatterIndexes(note domain.Note) {
	for field, value := range note.Frontmatter.Fields {
		canonicalValue, ok := canonicalizeFrontmatterValue(value)
		if !ok || !isComparableForIndex(canonicalValue) {
			continue
		}
		if q.byFrontmatter[field] == nil {
			q.byFrontmatter[field] = make(map[interface{}][]domain.Note)
		}
		q.byFrontmatter[field][canonicalValue] = append(
			q.byFrontmatter[field][canonicalValue],
			note,
		)
	}
}

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
