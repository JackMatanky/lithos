// Package spi defines Service Provider Interface (SPI) ports for external adapters.
// SPI ports are implemented by adapters and injected into the application layer,
// enabling the hexagonal architecture pattern where the domain defines contracts
// but does not depend on infrastructure implementations.
//
// This file defines the MetadataQueryPort for index-based metadata queries,
// enabling efficient lookups by basename, alias, and fileClass without requiring
// full cache iteration or complex query execution.
package spi

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

// MetadataQueryPort defines the contract for index-based metadata queries.
// This port enables efficient lookups of notes by metadata fields that are
// commonly indexed (basename, alias, fileClass) without requiring full cache
// iteration or complex query execution.
//
// Architecture Layer: Port (SPI)
// Responsibility: Index-based metadata queries
//
// The adapter implementing this port should:
//   - Provide O(1) or O(log n) lookup performance for indexed fields
//   - Handle duplicate entries gracefully (return all matches)
//   - Support context cancellation for long-running operations
//   - Return empty slices (not nil) when no matches found
//   - Implement proper error handling for infrastructure failures
//
// The application layer (QueryService) consumes this port for:
//   - Fast metadata-based note lookups
//   - Index-based query routing (vs full-text or complex queries)
//   - Efficient duplicate handling and collision resolution
//
// Reference: docs/architecture/components.md#metadataqueryport
type MetadataQueryPort interface {
	// ByBasename finds notes by filename without extension.
	// This enables fast lookups for notes with the same base filename,
	// which is common when notes have the same title but different paths.
	//
	// Parameters:
	//   ctx: Context for cancellation and timeout control
	//   basename: Filename without extension (e.g., "meeting-notes" from "meeting-notes.md")
	//
	// Returns:
	//   []domain.Note: All notes with matching basename (empty slice if none found)
	//   error: Context cancellation or infrastructure errors
	//
	// Behavior:
	//   - Returns all notes with matching basename (handles duplicates)
	//   - Empty slice (not nil) when no matches found
	//   - Context cancellation returns context.Canceled or context.DeadlineExceeded
	//   - Infrastructure errors wrapped with meaningful context
	//
	// Example:
	//   notes, err := port.ByBasename(ctx, "meeting-notes")
	//   // Returns all notes named "meeting-notes.md" across different directories
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
	//   []domain.Note: All notes containing the alias (empty slice if none found)
	//   error: Context cancellation or infrastructure errors
	//
	// Behavior:
	//   - Searches aliases array in frontmatter for exact matches
	//   - Returns all notes containing the alias (multiple notes can have same alias)
	//   - Empty slice (not nil) when no matches found
	//   - Context cancellation returns context.Canceled or context.DeadlineExceeded
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
	//   []domain.Note: All notes with matching fileClass (empty slice if none found)
	//   error: Context cancellation or infrastructure errors
	//
	// Behavior:
	//   - Searches fileClass field in frontmatter for exact matches
	//   - Returns all notes with matching fileClass
	//   - Empty slice (not nil) when no matches found
	//   - Context cancellation returns context.Canceled or context.DeadlineExceeded
	//   - Infrastructure errors wrapped with meaningful context
	//
	// Example:
	//   notes, err := port.ByFileClass(ctx, "meeting")
	//   // Returns all notes with fileClass: "meeting" in frontmatter
	ByFileClass(ctx context.Context, fileClass string) ([]domain.Note, error)
}
