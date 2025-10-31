package frontmatter

import (
	"context"
	"fmt"
	"strings"

	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/dto"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// ValidationResult captures the outcome of validating a note's frontmatter.
type ValidationResult struct {
	Valid      bool
	Errors     []errors.ValidationError
	NotePath   string
	SchemaName string
}

// FrontmatterService loads notes and ensures they are eligible for downstream
// schema validation logic (markdown-only guard).
type FrontmatterService struct {
	vaultReader spi.VaultReaderPort
	log         zerolog.Logger
}

// NewFrontmatterService constructs a FrontmatterService with its dependencies.
func NewFrontmatterService(
	vaultReader spi.VaultReaderPort,
	log zerolog.Logger,
) (*FrontmatterService, error) {
	if vaultReader == nil {
		return nil, fmt.Errorf("vaultReader cannot be nil")
	}
	return &FrontmatterService{
		vaultReader: vaultReader,
		log:         log,
	}, nil
}

// ValidateNote verifies the note is markdown before handing it to downstream
// validation workflows.
func (s *FrontmatterService) ValidateNote(
	ctx context.Context,
	notePath string,
	schemaName string,
) (ValidationResult, error) {
	s.log.Info().
		Str("notePath", notePath).
		Str("schemaName", schemaName).
		Msg("validating note frontmatter")

	file, err := s.vaultReader.Read(ctx, notePath)
	if err != nil {
		return ValidationResult{}, fmt.Errorf(
			"failed to load note %s: %w",
			notePath,
			err,
		)
	}

	if markdownErr := ensureMarkdownVaultFile(file); markdownErr != nil {
		valErr := errors.NewValidationError(
			"",
			markdownErr.Error(),
			file.Ext,
			markdownErr,
		)
		return ValidationResult{
			Valid:      false,
			Errors:     []errors.ValidationError{*valErr},
			NotePath:   notePath,
			SchemaName: schemaName,
		}, nil
	}

	return ValidationResult{
		Valid:      true,
		Errors:     nil,
		NotePath:   notePath,
		SchemaName: schemaName,
	}, nil
}

// ensureMarkdownVaultFile returns an error if the supplied file does not look
// like markdown content (extension or MIME mismatch).
func ensureMarkdownVaultFile(file dto.VaultFile) error {
	ext := strings.ToLower(file.Ext)
	if ext == ".md" || ext == ".markdown" {
		return nil
	}
	if strings.EqualFold(file.MimeType, "text/markdown") {
		return nil
	}
	return fmt.Errorf("unsupported file type %q; expected markdown", file.Ext)
}
