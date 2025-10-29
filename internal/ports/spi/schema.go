// Package spi defines Service Provider Interfaces (SPI) for external resource
// loading
// and persistence operations in the lithos configuration and schema engine.
//
// SPI ports define contracts for external dependencies that the domain
// requires,
// such as loading schemas from filesystem, databases, or remote services.
package spi

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

// SchemaPort defines the interface for loading schemas and property bank
// definitions
// from external storage systems.
//
// This port abstracts the schema loading responsibility from the domain,
// enabling different storage backends (filesystem, database, remote services)
// while maintaining
// consistent domain behavior.
//
// Architecture Reference: docs/architecture/components.md#schemaport
// Requirements: FR5 (Schema Loading), FR9 (Configuration Schema) from
// docs/prd/requirements.md
//
// Loading Process:
//  1. Load property bank first (required for $ref resolution)
//  2. Load all schema definitions from storage
//  3. Return raw schemas without inheritance resolution
//  4. SchemaResolver handles inheritance and $ref resolution separately
//
// Implementation Contract:
// - Load() returns raw schemas (Extends/Excludes/Properties as-is from storage)
//   - Property bank must be loaded before schemas to enable $ref resolution
//   - Duplicate schema names should be detected and reported as errors
//   - All loading errors should fail fast with descriptive messages
//   - Unknown JSON fields should be preserved (FR6 requirement)
type SchemaPort interface {
	// Load retrieves all schemas and the property bank from storage.
	//
	// Returns:
	//  - []Schema: Raw schema definitions (no inheritance resolution applied)
	//  - PropertyBank: Shared property definitions for $ref resolution
	//  - error: Loading, parsing, or validation errors
	//
	// The returned schemas contain Extends/Excludes/Properties exactly as
	// defined
	// in storage. The SchemaResolver component handles inheritance resolution
	// and $ref substitution in a separate processing step.
	//
	// Property bank is loaded exactly once per Load() call and contains all
	// shared property definitions that schemas can reference via $ref syntax.
	//
	// Loading errors include:
	//  - Missing property bank file
	//  - Malformed JSON in property bank or schema files
	//  - Duplicate schema names across all loaded schemas
	//  - File system access errors
	//
	// Context is used for cancellation and deadline propagation during
	// potentially long-running file system operations.
	Load(ctx context.Context) ([]domain.Schema, domain.PropertyBank, error)
}
