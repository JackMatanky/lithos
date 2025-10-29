package api

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

// CommandPort defines the use case handler contract that CLI adapters call back
// to for business logic execution. CommandOrchestrator implements this
// interface, keeping orchestration logic squarely in the domain layer while the
// CLI adapter focuses on user interaction concerns.
//
// Hexagonal callback pattern:
//   - CommandOrchestrator passes itself to CLIPort.Start(ctx, handler)
//   - CLI adapter configures commands, parses input, and delegates to handler
//   - Handler executes domain workflows and returns results/errors to adapter
//
// Separation of concerns:
//   - CLI adapter: command parsing, flag handling, terminal UX
//   - CommandPort implementation: domain orchestration, validation, persistence
//
// Current methods (Epic 1):
//   - NewNote: create note from template (core happy-path workflow)
//
// Future methods (later epics):
//   - IndexVault: rebuild vault index and cache
//   - FindTemplates: list and fuzzy-find templates
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
	// Context usage:
	//   - ctx propagates cancellation/timeouts triggered by CLIPort adapters
	//   - Long-running operations must observe ctx.Done()
	//
	// Returns:
	//   - domain.Note on success for CLI presentation
	//   - error on failure (template not found, render/parse issues, validation
	//     failures, persistence problems, or context cancellation)
	NewNote(
		ctx context.Context,
		templateID domain.TemplateID,
	) (domain.Note, error)

	// Additional methods to be added in later stories:
	// - IndexVault(ctx context.Context) (IndexStats, error)
	// - FindTemplates(ctx context.Context, query string) ([]Template, error)
}
