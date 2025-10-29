package api

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

// CommandPort defines the use case handler contract that CLI adapters call back
// to for business logic execution. Implemented by CommandOrchestrator to
// orchestrate
// domain services for each command.
//
// This interface follows hexagonal callback pattern where the CLI adapter
// receives the handler via CLIPort.Start() and calls handler methods based on
// parsed commands.
//
// Current methods (Epic 1):
// - NewNote: Create note from template
//
// Future methods (later epics):
// - IndexVault: Rebuild vault index and cache
// - FindTemplates: List and fuzzy-find templates
//
// Reference: docs/architecture/components.md#api-port-interfaces - CommandPort
// (v0.6.4).
type CommandPort interface {
	// NewNote orchestrates the complete note creation workflow:
	// 1. Load and render template via TemplateEngine
	// 2. Extract and validate frontmatter via FrontmatterService
	// 3. Generate NoteID from frontmatter fields
	// 4. Create Note domain object
	// 5. Persist to vault via VaultWriterPort
	// 6. Update cache via CacheWriterPort
	// 7. Return Note for CLI display
	//
	// Returns Note on success, error on failure (template not found, parse
	// error,
	// validation error, write error, etc.).
	NewNote(
		ctx context.Context,
		templateID domain.TemplateID,
	) (domain.Note, error)

	// Additional methods to be added in later stories:
	// - IndexVault(ctx context.Context) (IndexStats, error)
	// - FindTemplates(ctx context.Context, query string) ([]Template, error)
}
