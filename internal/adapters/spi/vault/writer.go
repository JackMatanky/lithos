package vault

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/moby/sys/atomicwriter"
	"github.com/rs/zerolog"
	"gopkg.in/yaml.v3"
)

const (
	// filePermissions defines the permissions for created files.
	filePermissions = 0o600
)

// VaultWriterAdapter implements VaultWriterPort for filesystem-based note
// persistence.
// It provides atomic write guarantees using moby/sys/atomicwriter and ensures
// all frontmatter fields are preserved per FR6 requirements.
//
// Reference: docs/architecture/components.md#vaultwriteradapter.
type VaultWriterAdapter struct {
	config domain.Config
	logger zerolog.Logger
}

// NewVaultWriterAdapter creates a new VaultWriterAdapter with the given
// configuration and logger.
// It implements VaultWriterPort for atomic note persistence to the filesystem.
//
// The adapter uses the vault path from config for all operations and logs
// operations at appropriate levels for observability.
func NewVaultWriterAdapter(
	config domain.Config,
	logger zerolog.Logger,
) spi.VaultWriterPort {
	return &VaultWriterAdapter{
		config: config,
		logger: logger,
	}
}

// serializeNote converts a note to markdown format with YAML frontmatter.
// Preserves all frontmatter fields per FR6 requirements.
func serializeNote(note domain.Note) ([]byte, error) {
	// Serialize frontmatter to YAML
	yamlData, err := yaml.Marshal(note.Frontmatter.Fields)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal frontmatter: %w", err)
	}

	// Create markdown content with frontmatter
	var content []byte
	if len(yamlData) > 0 {
		content = append([]byte("---\n"), yamlData...)
		content = append(content, []byte("---\n")...)
	}

	// Add content if present in frontmatter
	if contentField, ok := note.Frontmatter.Fields["content"].(string); ok {
		content = append(content, []byte(contentField)...)
	}

	return content, nil
}

// Persist writes note to vault with atomic guarantees.
// Creates parent directories if missing, serializes frontmatter to YAML,
// and writes to file using atomic temp-file + rename pattern.
// Preserves all frontmatter fields per FR6 requirements.
func (v *VaultWriterAdapter) Persist(
	ctx context.Context,
	note domain.Note,
	path string,
) error {
	// Resolve full path within vault
	fullPath := filepath.Join(v.config.VaultPath, path)

	// Create parent directories
	if err := os.MkdirAll(filepath.Dir(fullPath), 0o750); err != nil {
		v.logger.Error().
			Err(err).
			Str("path", fullPath).
			Msg("Failed to create parent directories")
		return fmt.Errorf(
			"failed to create directories for %s: %w",
			fullPath,
			err,
		)
	}

	// Serialize note to markdown format
	content, err := serializeNote(note)
	if err != nil {
		v.logger.Error().
			Err(err).
			Str("note_id", string(note.ID)).
			Msg("Failed to serialize note")
		return fmt.Errorf(
			"failed to serialize note %s: %w",
			note.ID,
			err,
		)
	}

	// Write atomically
	if writeErr := atomicwriter.WriteFile(fullPath, content, filePermissions); writeErr != nil {
		v.logger.Error().
			Err(writeErr).
			Str("path", fullPath).
			Str("note_id", string(note.ID)).
			Msg("Failed to write note atomically")
		return fmt.Errorf(
			"failed to persist note %s to %s: %w",
			note.ID,
			fullPath,
			writeErr,
		)
	}

	v.logger.Info().
		Str("path", fullPath).
		Str("note_id", string(note.ID)).
		Msg("Successfully persisted note")

	return nil
}

// Delete removes note from vault.
// Idempotent: returns nil if file doesn't exist.
// Used by CommandOrchestrator for note deletion.
func (v *VaultWriterAdapter) Delete(ctx context.Context, path string) error {
	// Resolve full path within vault
	fullPath := filepath.Join(v.config.VaultPath, path)

	// Remove file - os.Remove is idempotent (no error if file doesn't exist)
	if err := os.Remove(fullPath); err != nil && !os.IsNotExist(err) {
		v.logger.Error().
			Err(err).
			Str("path", fullPath).
			Msg("Failed to delete note")
		return fmt.Errorf("failed to delete note at %s: %w", fullPath, err)
	}

	v.logger.Info().
		Str("path", fullPath).
		Msg("Successfully deleted note")

	return nil
}
