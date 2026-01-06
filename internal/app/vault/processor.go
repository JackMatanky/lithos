package vault

import (
	"context"
	"errors"
	"os"
	"path/filepath"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/metrics"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// VaultProcessor handles vault-specific file processing orchestration.
// This component coordinates the complete pipeline from file discovery to
// cache-ready notes, including stats tracking, metadata building, and event
// publishing.
//
// Responsibilities:
//   - File type filtering (only .md files)
//   - Markdown processing + validation coordination
//   - Stats tracking (parse/validation success/failure)
//   - Metadata building for cache persistence
//   - Event publishing for processed notes
//
// Architecture:
//   - Uses MarkdownProcessor for core processing
//   - Focuses on vault-specific orchestration logic
//   - Provides clean interface for VaultIndexer
type VaultProcessor struct {
	processor *MarkdownProcessor
	config    domain.Config
	eventBus  events.EventBus
	log       zerolog.Logger
}

// NewVaultProcessor creates a new vault processor with the provided
// dependencies.
func NewVaultProcessor(
	processor *MarkdownProcessor,
	config domain.Config,
	eventBus events.EventBus,
	log zerolog.Logger,
) *VaultProcessor {
	return &VaultProcessor{
		processor: processor,
		config:    config,
		eventBus:  eventBus,
		log:       log,
	}
}

// ProcessFile handles complete vault file processing: filtering, parsing,
// validation,
// stats tracking, metadata building, and event publishing.
//
// Returns:
//   - domain.Note: Successfully processed note
//   - spi.CacheWriteMetadata: Cache metadata for persistence
//   - bool: true if processing succeeded, false otherwise
//
// This method orchestrates the vault-specific aspects of file processing,
// delegating core markdown operations to MarkdownProcessor.
func (p *VaultProcessor) ProcessFile(
	ctx context.Context,
	file *dto.VaultFile,
	stats *metrics.IndexStats,
) (domain.Note, spi.CacheWriteMetadata, bool) {
	// Filter: only .md files for frontmatter processing
	if file.Ext() != markdownExt {
		return domain.Note{}, spi.CacheWriteMetadata{}, false
	}

	// Use processor to parse and validate
	note, err := p.processor.ProcessFile(ctx, file.Path, file.Content)
	if err != nil {
		// Check if this is a validation error vs parse error
		var frontmatterErr *lithosErr.FrontmatterError
		if errors.As(err, &frontmatterErr) {
			stats.ValidationFailures++
		} else {
			stats.ParseFailures++
		}
		return domain.Note{}, spi.CacheWriteMetadata{}, false
	}

	// Success - validation passed (only count if validator exists)
	if p.processor.validator != nil {
		stats.ValidationSuccesses++
	}

	// Build metadata for caching
	indexTime := time.Now().UTC()
	metadata := p.buildCacheWriteMetadata(file, indexTime)

	// Publish note indexed event
	p.publishNoteIndexedEvent(ctx, note)

	return note, metadata, true
}

// buildCacheWriteMetadata builds cache write metadata for a vault file.
// This includes file modification time, size, and indexing timestamp.
func (p *VaultProcessor) buildCacheWriteMetadata(
	file *dto.VaultFile,
	indexTime time.Time,
) spi.CacheWriteMetadata {
	if file != nil && file.Info != nil {
		return spi.CacheWriteMetadata{
			ModifiedAt: file.Info.ModTime().UTC(),
			FileSize:   file.Info.Size(),
			IndexTime:  indexTime,
		}
	}
	if file != nil {
		return p.metadataFromPath(file.Path, indexTime)
	}
	return spi.CacheWriteMetadata{
		IndexTime:  indexTime,
		ModifiedAt: time.Time{},
		FileSize:   0,
	}
}

// metadataFromPath builds cache write metadata from a file path.
// Used when file info is not available in the VaultFile.
func (p *VaultProcessor) metadataFromPath(
	path string,
	indexTime time.Time,
) spi.CacheWriteMetadata {
	meta := spi.CacheWriteMetadata{
		IndexTime:  indexTime,
		ModifiedAt: time.Time{},
		FileSize:   0,
	}
	if path == "" {
		return meta
	}
	absolute := filepath.Join(p.config.VaultPath, path)
	info, err := os.Stat(absolute)
	if err != nil {
		return meta
	}
	meta.ModifiedAt = info.ModTime().UTC()
	meta.FileSize = info.Size()
	return meta
}

// publishNoteIndexedEvent publishes a note indexed event to the event bus.
func (p *VaultProcessor) publishNoteIndexedEvent(
	ctx context.Context,
	note domain.Note,
) {
	publishNoteIndexed(ctx, p.eventBus, p.log, note, time.Now())
}
