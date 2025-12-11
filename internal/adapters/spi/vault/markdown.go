package vault

import (
	"context"
	"fmt"
	"io"
	"sync"

	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/yuin/goldmark"
	"github.com/yuin/goldmark/parser"
	"go.abhg.dev/goldmark/frontmatter"
)

// Ensure MarkdownParserAdapter implements MarkdownParserPort interface.
var _ spi.MarkdownParserPort = (*MarkdownParserAdapter)(nil)

// MarkdownParserAdapter implements MarkdownParserPort for syntactic markdown
// frontmatter parsing. This adapter uses goldmark with frontmatter extension
// to handle YAML parsing while keeping infrastructure concerns out of the
// domain layer.
//
// Architecture Layer: SPI Adapter (Infrastructure)
// Location: internal/adapters/spi/vault/markdown.go
//
// Responsibilities:
//   - Syntactic validation: YAML structure, delimiter detection
//   - Parsing infrastructure: goldmark integration and configuration
//   - Error handling: Structured errors with line number information
//
// - Edge case handling: Missing frontmatter, malformed YAML, context
// cancellation
//
// Does NOT handle:
// - Semantic validation: Schema compliance, business rules (domain
// responsibility)
//   - Content rendering: Only parses frontmatter, ignores markdown content
//   - Field transformation: Returns raw parsed data as-is
//
// Reference: docs/architecture/coding-standards.md - Validation Layer
// Separation.
type MarkdownParserAdapter struct {
	markdown goldmark.Markdown
	log      zerolog.Logger
	parserMu sync.Mutex
}

// NewMarkdownParserAdapter creates a new MarkdownParserAdapter with goldmark
// configuration optimized for frontmatter extraction.
//
// The adapter is configured with:
//   - Frontmatter extension supporting YAML and TOML formats
//   - SetMetadata mode for extracting frontmatter into parser context
//   - Thread-safe goldmark instance with mutex protection
//
// Parameters:
//
//	logger: Structured logger for observability and error tracking
//
// Returns:
//
//	*MarkdownParserAdapter: Configured adapter ready for frontmatter parsing
func NewMarkdownParserAdapter(logger zerolog.Logger) *MarkdownParserAdapter {
	return &MarkdownParserAdapter{
		markdown: goldmark.New(
			goldmark.WithExtensions(
				&frontmatter.Extender{
					Formats: frontmatter.DefaultFormats, // YAML and TOML support
					Mode:    frontmatter.SetMetadata,    // Extract to parser context
				},
			),
		),
		log:      logger,
		parserMu: sync.Mutex{},
	}
}

// ParseFrontmatter extracts and parses YAML frontmatter from markdown content.
// This method performs syntactic validation of YAML structure and returns
// the parsed frontmatter as a map for domain layer consumption.
//
// Implementation:
//   - Uses goldmark with frontmatter extension for robust parsing
//
// - Handles edge cases: missing frontmatter, empty frontmatter, malformed YAML
//   - Provides structured error messages with line number information
//   - Supports context cancellation for long-running operations
//   - Thread-safe with mutex protection for goldmark parser
//
// Error Scenarios:
// - Context cancellation: Returns context.Canceled or context.DeadlineExceeded
//   - Malformed YAML: Returns parsing error with line number information
//   - Invalid delimiters: Returns structured error with position details
//   - Goldmark failures: Returns wrapped error with additional context
//
// Success Scenarios:
//   - Valid frontmatter: Returns parsed fields as map[string]any
//   - Missing frontmatter: Returns empty map (not an error)
//   - Empty frontmatter: Returns empty map (not an error)
//
// Example:
//
//	content := []byte("---\ntitle: My Note\ntags: [work]\n---\n# Content")
//	result, err := adapter.ParseFrontmatter(ctx, content)
//	// result = map[string]any{"title": "My Note", "tags": []any{"work"}}
func (a *MarkdownParserAdapter) ParseFrontmatter(
	ctx context.Context,
	content []byte,
) (map[string]any, error) {
	// Check context cancellation before processing
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	// Log parsing attempt for observability
	a.log.Debug().
		Int("content_length", len(content)).
		Msg("parsing markdown frontmatter")

	// Parse markdown with frontmatter using goldmark
	result, err := a.parseWithGoldmark(ctx, content)
	if err != nil {
		a.log.Error().
			Err(err).
			Int("content_length", len(content)).
			Msg("frontmatter parsing failed")
		return nil, fmt.Errorf("frontmatter parsing failed: %w", err)
	}

	// Log successful parsing
	a.log.Debug().
		Int("fields_count", len(result)).
		Msg("frontmatter parsed successfully")

	return result, nil
}

// parseWithGoldmark performs the actual parsing using goldmark with frontmatter
// extension. This helper method isolates goldmark-specific logic and provides
// proper error handling for parsing failures.
//
// The method uses goldmark's parser context to safely extract frontmatter
// without interference from code blocks or other markdown constructs that
// might contain similar delimiter patterns.
//
// Returns:
//   - map[string]any: Parsed frontmatter fields, empty map if no frontmatter
//   - error: Parsing errors with additional context and line information
func (a *MarkdownParserAdapter) parseWithGoldmark(
	ctx context.Context,
	content []byte,
) (map[string]any, error) {
	// Create parser context for frontmatter extraction
	// Context isolates frontmatter parsing from markdown rendering
	parserCtx := parser.NewContext()

	// Check context again before expensive parsing operation
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	// Parse the markdown content with thread safety
	// The frontmatter extension populates the context with extracted data
	a.parserMu.Lock()
	convertErr := a.markdown.Convert(
		content,
		io.Discard,
		parser.WithContext(parserCtx),
	)
	a.parserMu.Unlock()

	if convertErr != nil {
		return nil, fmt.Errorf("goldmark conversion failed: %w", convertErr)
	}

	// Extract frontmatter data from parser context
	frontmatterData := frontmatter.Get(parserCtx)
	if frontmatterData == nil {
		// No frontmatter found - return empty map (not an error)
		return make(map[string]any), nil
	}

	// Decode frontmatter.Data into standard map[string]any
	result := make(map[string]any)
	if err := frontmatterData.Decode(&result); err != nil {
		return nil, fmt.Errorf("frontmatter decode failed: %w", err)
	}

	return result, nil
}
