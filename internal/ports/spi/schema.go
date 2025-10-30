package spi

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

// SchemaPort defines the interface for loading schemas and property bank
// definitions from external storage systems.
//
// This port abstracts the schema loading responsibility from the domain,
// enabling different storage backends (filesystem, database, remote services)
// while maintaining consistent domain behavior.
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
	// defined in storage. The SchemaResolver component handles inheritance
	// resolution and $ref substitution in a separate processing step.
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

// SchemaRegistryPort defines the interface for fast in-memory access to loaded
// and resolved schemas.
//
// Provides thread-safe registry access for schema lookups by FrontmatterService
// and QueryService.
// Populated by SchemaEngine at startup from SchemaPort.Load() results.
// Thread-safe for concurrent reads.
//
// Architecture Reference: docs/architecture/components.md#schemaregistryport
// Requirements: FR5 (Schema Loading), FR7 (Schema Registry) from
// docs/prd/requirements.md
//
// Implementation Contract:
// - GetSchema/GetProperty return SchemaError with ErrNotFound classification
// when lookups fail
// - HasSchema/HasProperty never error, return bool only
// - RegisterAll clears existing entries before registration (idempotent
// behavior)
// - RegisterAll stores defensive copies of schemas and properties
// (not original references)
// - GetSchema/GetProperty return defensive copies (not internal references)
// - Thread-safe for concurrent reads.
type SchemaRegistryPort interface {
	// GetSchema retrieves a schema by name from the registry.
	//
	// Returns SchemaError with ErrNotFound classification when schema doesn't
	// exist.
	// Returns defensive copy to prevent external mutation of registry state.
	//
	// Context is used for cancellation during potentially long-running
	// operations.
	GetSchema(ctx context.Context, name string) (domain.Schema, error)

	// GetProperty retrieves a property from the property bank by name.
	//
	// Returns SchemaError with ErrNotFound classification when property does
	// not exist.
	// Returns defensive copy to prevent external mutation of registry state.
	//
	// Context is used for cancellation during potentially long-running
	// operations.
	GetProperty(ctx context.Context, name string) (domain.Property, error)

	// HasSchema checks if a schema exists in the registry.
	//
	// Never errors, returns bool only for existence check.
	// Thread-safe for concurrent access.
	//
	// Context is used for cancellation during potentially long-running
	// operations.
	HasSchema(ctx context.Context, name string) bool

	// HasProperty checks if a property exists in the property bank.
	//
	// Never errors, returns bool only for existence check.
	// Thread-safe for concurrent access.
	//
	// Context is used for cancellation during potentially long-running
	// operations.
	HasProperty(ctx context.Context, name string) bool

	// RegisterAll registers all schemas and properties into the registry.
	//
	// Clears existing entries before registration (idempotent behavior).
	// Stores defensive copies to prevent external mutation of registry state.
	// Enables re-registration without stale data (aligns with FR9).
	//
	// Context is used for cancellation during potentially long-running
	// operations.
	RegisterAll(
		ctx context.Context,
		schemas []domain.Schema,
		bank domain.PropertyBank,
	) error
}
