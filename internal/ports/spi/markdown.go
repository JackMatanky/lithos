// Package spi defines Service Provider Interface (SPI) ports for external adapters.
// SPI ports are implemented by adapters and injected into the application layer,
// enabling the hexagonal architecture pattern where the domain defines contracts
// but does not depend on infrastructure implementations.
//
// This file defines the MarkdownParserPort for syntactic parsing of markdown
// frontmatter, keeping infrastructure concerns (goldmark) out of the domain layer.
package spi

import "context"

// MarkdownParserPort defines the contract for parsing markdown frontmatter.
// This port enables syntactic parsing of YAML frontmatter from markdown content
// while keeping goldmark and other parsing infrastructure out of the domain layer.
//
// Architecture Layer: Port (SPI)
// Responsibility: Syntactic validation and parsing
//
// The adapter implementing this port should:
// - Parse YAML frontmatter structure using appropriate parsing libraries
// - Validate YAML syntax and report structural errors with line numbers
// - Return parsed frontmatter as a map for domain layer consumption
// - Handle edge cases: missing frontmatter, empty frontmatter, malformed YAML
//
// The domain layer (FrontmatterService) consumes this port for:
// - Obtaining parsed frontmatter data for semantic validation
// - Delegating syntactic parsing concerns to the adapter layer
// - Maintaining clean separation between parsing and business logic
//
// Reference: docs/architecture/coding-standards.md - Validation Layer Separation
type MarkdownParserPort interface {
	// ParseFrontmatter extracts and parses YAML frontmatter from markdown content.
	//
	// This method performs syntactic validation of YAML structure and returns
	// the parsed frontmatter as a map. It does NOT perform semantic validation
	// (schema compliance, business rules) - that responsibility belongs to the
	// domain layer.
	//
	// Parameters:
	//   ctx: Context for cancellation and timeout control
	//   content: Raw markdown content as bytes, may or may not contain frontmatter
	//
	// Returns:
	//   map[string]any: Parsed frontmatter fields, empty map if no frontmatter found
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
	//   content := []byte("---\ntitle: My Note\ntags: [work, important]\n---\n# Content")
	//   result, err := parser.ParseFrontmatter(ctx, content)
	//   // result = map[string]any{"title": "My Note", "tags": []any{"work", "important"}}
	ParseFrontmatter(ctx context.Context, content []byte) (map[string]any, error)
}
