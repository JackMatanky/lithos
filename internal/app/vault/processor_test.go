package vault

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/adapters/spi/dto"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/metrics"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
)

// TestVaultProcessor_ProcessFile tests the ProcessFile method.
func TestVaultProcessor_ProcessFile(t *testing.T) {
	t.Run("non-markdown file filtered out", func(t *testing.T) {
		// Setup
		markdownParser := utils.NewMockMarkdownParserPort()
		frontmatterService := &frontmatter.FrontmatterService{}
		eventBus := utils.NewMockEventBus()
		logger := zerolog.Nop()
		config := domain.Config{VaultPath: "/tmp"}

		markdownProcessor := NewMarkdownProcessor(
			markdownParser,
			frontmatterService,
			eventBus,
			logger,
		)
		processor := NewVaultProcessor(
			markdownProcessor,
			config,
			eventBus,
			logger,
		)
		stats := &metrics.IndexStats{}

		file := &dto.VaultFile{
			Path: "test.txt",
		}

		// Execute
		note, metadata, result := processor.ProcessFile(
			context.Background(),
			file,
			stats,
		)

		// Verify
		assert.False(t, result)
		assert.Empty(t, note.Path)
		assert.Equal(t, spi.CacheWriteMetadata{}, metadata)
	})

	t.Run("successful markdown processing", func(t *testing.T) {
		// Setup
		markdownParser := utils.NewMockMarkdownParserPort()
		frontmatterService := &frontmatter.FrontmatterService{}
		eventBus := utils.NewMockEventBus()
		logger := zerolog.Nop()
		config := domain.Config{VaultPath: "/tmp"}

		// Mock successful parsing
		markdownParser.SetParseResult(map[string]any{
			"title": []interface{}{"Test Note"},
		}, nil)

		markdownProcessor := NewMarkdownProcessor(
			markdownParser,
			frontmatterService,
			eventBus,
			logger,
		)
		processor := NewVaultProcessor(
			markdownProcessor,
			config,
			eventBus,
			logger,
		)
		stats := &metrics.IndexStats{}

		file := &dto.VaultFile{
			Path:    "test.md",
			Content: []byte("---\ntitle: Test Note\n---\n\nContent here."),
		}

		// Execute
		resultNote, metadata, result := processor.ProcessFile(
			context.Background(),
			file,
			stats,
		)

		// Verify
		assert.True(t, result)
		assert.Equal(t, "test.md", resultNote.Path)
		assert.NotEmpty(t, metadata.IndexTime)
		assert.Equal(t, 1, stats.ValidationSuccesses)
	})
}
