// MetadataQueryPort defines the contract for index-based metadata queries.
package spi

import (
	"context"
	"fmt"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
)

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
