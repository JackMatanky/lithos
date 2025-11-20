package spi

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
)

// Package spi defines Service Provider Interfaces (ports) for hexagonal
// architecture. This package contains interfaces that abstract infrastructure
// concerns from domain logic.
//
// Ports included:
// - CacheWriterPort/CacheReaderPort: CQRS pattern for cache operations
// - VaultScannerPort: Vault file system scanning and enumeration
// - MarkdownParserPort: YAML frontmatter parsing from markdown content
// - MetadataQueryPort: Index-based metadata queries for notes
// - SchemaEnginePort: Schema loading and resolution operations
// - TemplatePort: Template loading and management
//
// Mock implementations are provided for all ports to support testing.
// Both ports must be thread-safe for concurrent access patterns typical in
// multi-service architectures (QueryService + FrontmatterService reading,
// indexing writing concurrently).
//
// Error handling follows docs/architecture/error-handling-strategy.md:
// - All errors wrapped with operation context and resource identifiers
// - Preserves error chains with fmt.Errorf("%w", err)
// - Uses structured error types from internal/shared/errors
//
// Functional Requirements:
// - FR6: Preserve unknown JSON fields during cache operations
// - FR9: Include operation context in all error messages
//
// See docs/architecture/components.md for detailed implementation guidance.

const (
	// PathQueryScopeFull matches an exact vault-relative path.
	PathQueryScopeFull PathQueryScope = "full"
	// PathQueryScopeBasename matches filename without extension across folders.
	PathQueryScopeBasename PathQueryScope = "basename"
	// PathQueryScopeFolder matches all notes under a vault-relative folder.
	PathQueryScopeFolder PathQueryScope = "folder"
)

// PathQueryScope enumerates the supported lookup scopes for PathQuery.
type PathQueryScope string

// PathQueryOptions convey how a PathQuery should interpret the provided value.
// Exactly one selector field must be populated; adapters return an error when
// none (or multiple) selectors are supplied to keep behavior deterministic.
type PathQueryOptions struct {
	// Value represents the path fragment to resolve. Expected format depends on
	// Scope: vault-relative path for Full, filename for Basename, directory for
	// Folder (trailing slash optional).
	Value string
	// Scope controls how Value should be matched. Defaults to Full if empty.
	Scope PathQueryScope
}

// CacheWriterPort defines the CQRS write-side contract for cache persistence.
// Implementations must provide atomic write guarantees and thread-safe
// concurrent access. All errors must be wrapped per error-handling-strategy.md
// with operation context and resource identifiers (FR9).
//
// CQRS Write-Side Responsibility:
// - Atomic persistence using temp-file + rename pattern (moby/sys/atomicwriter)
// - Cache directory creation (mkdir -p semantics)
// - Overwrite existing entries without corruption
// - Thread-safe concurrent writes from multiple indexers
//
// Implementation Expectations:
// - Use Config.CacheDir for directory location (default: .lithos/cache/)
// - One JSON file per note ({NoteID}.json)
// - Filesystem operations must be thread-safe (OS-level locking)
// - Context cancellation respected in all I/O operations
//
// Error Wrapping Requirements:
// - Wrap all errors with operation context: "cache write failed for note {id}
// at {path}"
// - Include resource identifiers (note ID, file path) in error messages
// - Preserve error chains for debugging
//
// See docs/architecture/components.md#cachewriterport for implementation
// guidance.
//
// Example usage:
//
//	writer := cacheWriterAdapter{}
//	ctx := context.Background()
//	note := domain.NewNote(domain.NewNoteID("note-123"), frontmatter)
//
//	if err := writer.Persist(ctx, note); err != nil {
//	    return fmt.Errorf("failed to cache note: %w", err)
//	}
type CacheWriterPort interface {
	// Persist atomically writes note to cache using temp-file + rename pattern.
	// Creates cache directory if missing. Overwrites existing cache entry.
	// Returns error wrapped with operation context if write fails.
	//
	// Atomic Write Semantics:
	// - Writes to temporary file first, then renames to final location
	// - Prevents partial reads during concurrent access
	// - Ensures cache consistency even during system crashes
	//
	// Cache Directory Behavior:
	// - Creates .lithos/cache/ directory if missing
	// - Uses Config.CacheDir for configurable location
	// - File naming: {NoteID}.json
	//
	// Thread-safe: Safe for concurrent calls.
	// Context: Respects ctx cancellation, returns ctx.Err() if canceled.
	// Errors: Wrapped with operation context and resource identifiers (FR9).
	Persist(ctx context.Context, note domain.Note, indexTime time.Time) error

	// Delete removes note from cache. Idempotent: returns nil if note doesn't
	// exist. Returns error wrapped with operation context if deletion fails
	// (e.g., permissions).
	//
	// Idempotent Semantics:
	// - Returns nil if note already deleted or never existed
	// - No error for "note not found" - treated as successful deletion
	// - Allows safe retry of delete operations
	//
	// Error Conditions:
	// - Permission denied on cache directory
	// - Filesystem errors during deletion
	// - Context cancellation during operation
	//
	// Thread-safe: Safe for concurrent calls.
	// Context: Respects ctx cancellation, returns ctx.Err() if canceled.
	// Errors: Wrapped with operation context and resource identifiers (FR9).
	Delete(ctx context.Context, id domain.NoteID) error
}

// CacheReaderPort defines the CQRS read-side contract for cache retrieval.
// Implementations must provide thread-safe concurrent reads and preserve
// unknown JSON fields (FR6 requirement). All errors must be wrapped per
// error-handling-strategy.md.
//
// CQRS Read-Side Responsibility:
// - Efficient retrieval of cached notes
// - Optional in-memory memoization for performance
// - Lazy loading to minimize memory usage
// - Thread-safe concurrent reads from multiple services
//
// Implementation Expectations:
// - JSON deserialization with unknown field preservation
// - Optional sync.RWMutex for in-memory caching
// - Return ErrNotFound when note doesn't exist in cache
// - Context cancellation respected in all operations
//
// Unknown Field Preservation (FR6):
// - Must preserve all JSON fields during deserialization
// - Use flexible unmarshaling (map[string]interface{}) for unknown fields
// - Ensure round-trip compatibility for user-defined fields
//
// Error Wrapping Requirements:
// - Wrap all errors with operation context: "cache read failed for note {id}"
// - Include resource identifiers in error messages
// - Preserve error chains for debugging
//
// See docs/architecture/components.md#cachereaderport for implementation
// guidance.
//
// Example usage:
//
//	reader := cacheReaderAdapter{}
//	ctx := context.Background()
//	id := domain.NewNoteID("note-123")
//
//	note, err := reader.Read(ctx, id)
//	if err != nil {
//	    return fmt.Errorf("failed to read cached note: %w", err)
//	}
//
//	notes, err := reader.List(ctx)
//	if err != nil {
//	    return fmt.Errorf("failed to list cached notes: %w", err)
//	}
type CacheReaderPort interface {
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
	Read(ctx context.Context, id domain.NoteID) (domain.Note, error)

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
	List(ctx context.Context) ([]domain.Note, error)
}

type MetadataQueryPort interface {
	// ByBasename finds notes by filename without extension.
	// This enables fast lookups for notes with the same base filename,
	// which is common when notes have the same title but different paths.
	//
	// Parameters:
	//   ctx: Context for cancellation and timeout control
	// basename: Filename without extension (e.g., "meeting-notes" from
	// "meeting-notes.md")
	//
	// Returns:
	// []domain.Note: All notes with matching basename (empty slice if none
	// found)
	//   error: Context cancellation or infrastructure errors
	//
	// Behavior:
	//   - Returns all notes with matching basename (handles duplicates)
	//   - Empty slice (not nil) when no matches found
	// - Context cancellation returns context.Canceled or
	// context.DeadlineExceeded
	//   - Infrastructure errors wrapped with meaningful context
	//
	// Example:
	//   notes, err := port.ByBasename(ctx, "meeting-notes")
	// // Returns all notes named "meeting-notes.md" across different
	// directories
	ByBasename(ctx context.Context, basename string) ([]domain.Note, error)

	// ByAlias finds notes by frontmatter alias values.
	// Aliases enable multiple names to refer to the same note, supporting
	// flexible note referencing and linking patterns.
	//
	// Parameters:
	//   ctx: Context for cancellation and timeout control
	//   alias: Alias value to search for in frontmatter aliases array
	//
	// Returns:
	// []domain.Note: All notes containing the alias (empty slice if none found)
	//   error: Context cancellation or infrastructure errors
	//
	// Behavior:
	//   - Searches aliases array in frontmatter for exact matches
	// - Returns all notes containing the alias (multiple notes can have same
	// alias)
	//   - Empty slice (not nil) when no matches found
	// - Context cancellation returns context.Canceled or
	// context.DeadlineExceeded
	//   - Infrastructure errors wrapped with meaningful context
	//
	// Example:
	//   notes, err := port.ByAlias(ctx, "project-alpha")
	//   // Returns all notes that have "project-alpha" in their aliases array
	ByAlias(ctx context.Context, alias string) ([]domain.Note, error)

	// ByFileClass finds notes by schema fileClass value.
	// FileClass determines which schema validates the note's frontmatter,
	// enabling efficient grouping and validation of notes by type.
	//
	// Parameters:
	//   ctx: Context for cancellation and timeout control
	//   fileClass: Schema name to search for in frontmatter fileClass field
	//
	// Returns:
	// []domain.Note: All notes with matching fileClass (empty slice if none
	// found)
	//   error: Context cancellation or infrastructure errors
	//
	// Behavior:
	//   - Searches fileClass field in frontmatter for exact matches
	//   - Returns all notes with matching fileClass
	//   - Empty slice (not nil) when no matches found
	// - Context cancellation returns context.Canceled or
	// context.DeadlineExceeded
	//   - Infrastructure errors wrapped with meaningful context
	//
	// Example:
	//   notes, err := port.ByFileClass(ctx, "meeting")
	//   // Returns all notes with fileClass: "meeting" in frontmatter
	ByFileClass(ctx context.Context, fileClass string) ([]domain.Note, error)

	// PathQuery finds notes using a flexible path selector. Callers supply
	// PathQueryOptions to specify whether the lookup should match a full path,
	// a basename shared across folders, or all notes within a folder.
	//
	// Parameters:
	//   ctx: Context for cancellation and timeout control
	//   opts: Selector describing which scope/value to match
	//
	// Returns:
	// []domain.Note: All notes matching the selector (empty slice if none
	// found)
	//   error: Context cancellation, validation failures, or infrastructure
	//   errors
	//
	// Behavior:
	//   - Full scope requires exact vault-relative path match
	//   - Basename scope returns every note whose filename matches
	//   - Folder scope returns all notes under the provided directory
	//   - Empty slice (not nil) when no matches found
	//   - Validation errors returned when opts invalid
	//
	// Example:
	//   notes, err := port.PathQuery(ctx, PathQueryOptions{
	//       Scope: PathQueryScopeFolder,
	//       Value: "projects/",
	//   })
	PathQuery(ctx context.Context, opts PathQueryOptions) ([]domain.Note, error)

	// TagQuery finds notes containing a specific tag.
	//
	// Parameters:
	//   ctx: Context for cancellation
	//   tag: The tag to search for (e.g., "todo", "urgent")
	//
	// Returns:
	//   []domain.Note: Notes containing the tag
	//   error: Infrastructure errors
	TagQuery(ctx context.Context, tag string) ([]domain.Note, error)

	// FrontmatterQuery finds notes where a specific frontmatter field matches a
	// value.
	// This supports dynamic queries against arbitrary frontmatter fields.
	//
	// Parameters:
	//   ctx: Context for cancellation
	// field: The frontmatter field name (dot notation supported for nested
	// fields)
	//   value: The value to match (string representation)
	//
	// Returns:
	//   []domain.Note: Notes matching the query
	//   error: Infrastructure errors
	FrontmatterQuery(
		ctx context.Context,
		field, value string,
	) ([]domain.Note, error)
}

// Validate normalises the options and ensures a usable scope/value pair.
func (o PathQueryOptions) Validate() (PathQueryOptions, error) {
	scope := o.Scope
	if scope == "" {
		scope = PathQueryScopeFull
	}
	value := strings.TrimSpace(o.Value)
	if value == "" {
		return PathQueryOptions{}, fmt.Errorf("path query value is required")
	}
	return PathQueryOptions{
		Scope: scope,
		Value: value,
	}, nil
}
