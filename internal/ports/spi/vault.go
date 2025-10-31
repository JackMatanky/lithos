package spi

import (
	"context"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/dto"
)

// VaultReaderPort defines the contract for reading individual vault files at
// the business level. This port abstracts filesystem operations into
// domain-level file reading operations, enabling services to read single files
// without knowing filesystem details.
//
// The port provides single file access for validation and lookups. All
// operations include proper error context per FR9 requirements.
//
// Interface Segregation Principle: Separated from VaultScannerPort to avoid
// forcing single-file dependencies on services that only need scanning.
//
// Reference: docs/architecture/components.md#vaultreaderport.
type VaultReaderPort interface {
	// Read performs single file read for validation and lookups.
	// Validates path is within vault to prevent directory traversal attacks.
	// Used by FrontmatterService for FileSpec validation.
	// Returns VaultFile DTO with metadata and content.
	// Errors include operation context and file paths per FR9.
	Read(ctx context.Context, path string) (dto.VaultFile, error)
}

// VaultScannerPort defines the contract for scanning vault files at the
// business level. This port abstracts filesystem scanning operations into
// domain-level scanning operations, enabling the indexing service to scan
// notes without knowing filesystem details.
//
// The port supports both full vault scans (for initial indexing) and
// incremental scans (for performance optimization with large vaults). All
// operations include proper error context per FR9 requirements.
//
// Interface Segregation Principle: Separated from VaultReaderPort to avoid
// forcing scanning dependencies on services that only need single file access.
//
// Reference: docs/architecture/components.md#vaultscannerport.
type VaultScannerPort interface {
	// ScanAll performs a full vault scan for initial index build.
	// Returns all files in the vault as VaultFile DTOs for indexing.
	// Ignores cache directories (.lithos/) to prevent re-indexing cached data.
	// Used by VaultIndexer.Build() for complete vault indexing.
	// Errors include operation context and file paths per FR9.
	ScanAll(ctx context.Context) ([]dto.VaultFile, error)

	// ScanModified performs incremental scan for large vault optimization.
	// Returns only files modified since the given timestamp for performance.
	// Future optimization for NFR4 (large vault performance).
	// MVP can use ScanAll for both initial and incremental builds.
	// Errors include operation context and file paths per FR9.
	ScanModified(ctx context.Context, since time.Time) ([]dto.VaultFile, error)
}

// VaultWriterPort defines the contract for writing notes to vault with atomic
// guarantees. This port abstracts filesystem write operations into domain-level
// persistence operations, enabling the CommandOrchestrator to persist notes
// without knowing filesystem details.
//
// The port provides CQRS write-side operations with atomic guarantees, ensuring
// all-or-nothing semantics for note persistence. Operations include proper
// error context per FR9 requirements and preserve all frontmatter fields per
// FR6.
//
// Reference: docs/architecture/components.md#vaultwriterport.
type VaultWriterPort interface {
	// Persist writes note to vault with atomic guarantees
	// Creates parent directories if missing
	// Overwrites existing file without mutating note content
	// Preserves all frontmatter fields (FR6)
	// Used by CommandOrchestrator.NewNote() and dual-write pattern
	// Errors include operation context and file paths per FR9
	Persist(ctx context.Context, note domain.Note, path string) error

	// Delete removes note from vault
	// Idempotent: returns nil if file doesn't exist
	// Used by CommandOrchestrator for note deletion
	// Errors include operation context and file paths per FR9
	Delete(ctx context.Context, path string) error
}
