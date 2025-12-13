// MarkdownParserPort defines the contract for parsing markdown content.
package spi

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

type MarkdownParserPort interface {
	// ParseFrontmatter extracts and parses YAML frontmatter from markdown
	// content.
	//
	// This method performs syntactic validation of YAML structure and returns
	// the parsed frontmatter as a map. It does NOT perform semantic validation
	// (schema compliance, business rules) - that responsibility belongs to the
	// domain layer.
	//
	// Parameters:
	//   ctx: Context for cancellation and timeout control
	// content: Raw markdown content as bytes, may or may not contain
	// frontmatter
	//
	// Returns:
	// map[string]any: Parsed frontmatter fields, empty map if no frontmatter
	// found
	//   error: Parsing errors (malformed YAML, syntax errors) with line numbers
	//
	// Behavior:
	//   - Missing frontmatter: Returns empty map, no error
	//   - Empty frontmatter: Returns empty map, no error
	//   - Valid YAML: Returns parsed fields as map
	//   - Invalid YAML: Returns error with line number information
	//   - Context cancellation: Returns context error
	//
	// Example:
	// content := []byte("---\ntitle: My Note\ntags: [work, important]\n---\n#
	// Content")
	//   result, err := parser.ParseFrontmatter(ctx, content)
	// // result = map[string]any{"title": "My Note", "tags": []any{"work",
	// "important"}}
	ParseFrontmatter(
		ctx context.Context,
		content []byte,
	) (map[string]any, error)

	// ParseNote parses complete markdown content and constructs a domain Note.
	// Performs full AST parsing to extract frontmatter, links, headings, tags,
	// and tasks. Constructs Note with parsed metadata and empty Backlinks.
	//
	// Parameters:
	//   ctx: Context for cancellation and timeout control
	//   path: Vault-relative path for the note (used as identifier)
	//   content: Raw markdown content as bytes
	//
	// Returns:
	//   domain.Note: Fully constructed note with parsed metadata
	//   error: Parsing errors or validation failures
	//
	// Behavior:
	//   - Parses frontmatter, links, headings, tags, and tasks from content
	//   - Constructs Note using NewNote factory with validation
	//   - Backlinks start empty (populated during enrichment phase)
	//   - Context cancellation: Returns context error
	ParseNote(
		ctx context.Context,
		path string,
		content []byte,
	) (domain.Note, error)
}
