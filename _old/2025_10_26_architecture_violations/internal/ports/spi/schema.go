// Package spi defines service provider interface ports for hexagonal
// architecture.
//
// These interfaces define contracts that infrastructure adapters must implement
// to provide services to the domain layer. This package contains the driven
// ports
// that allow the domain to remain independent of external dependencies.
package spi

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

// SchemaLoaderPort provides schema loading operations for domain services.
//
// This port allows domain services to load schema definitions and property
// banks
// from external storage without importing infrastructure concerns directly,
// maintaining the hexagonal architecture boundaries.
//
// The adapter implementation is responsible for:
// - JSON parsing and PropertySpec discriminator logic
// - $ref resolution in property banks using JSON pointer syntax
// - Security validation for file path access
// - Converting parsed JSON to domain objects using domain constructors
//
// All methods return domain objects directly (no separate DTO types).
type SchemaLoaderPort interface {
	// LoadSchemas loads all schema definitions from the configured schemas
	// directory.
	// Returns fully constructed domain Schema objects with properties parsed
	// and validated.
	// The adapter handles JSON parsing, PropertySpec discriminator logic, and
	// domain object creation.
	//
	// Context can be used for cancellation and timeout control.
	// Returns error if schemas cannot be loaded, parsed, or validated.
	LoadSchemas(ctx context.Context) ([]domain.Schema, error)

	// LoadPropertyBank loads property bank definitions from the configured
	// schemas/properties directory.
	// Returns a pointer to a fully constructed domain PropertyBank object with
	// $ref
	// resolution completed.
	// The adapter handles JSON parsing, $ref resolution using JSON pointer
	// syntax, circular reference detection, and domain object creation.
	//
	// Context can be used for cancellation and timeout control.
	// Returns error if property banks cannot be loaded, parsed, $ref resolved,
	// or validated.
	LoadPropertyBank(ctx context.Context) (*domain.PropertyBank, error)
}

// SchemaRegistryPort provides read-only access to resolved schema definitions.
//
// This port exposes schema lookup capabilities to domain services while
// keeping registry implementation details within the SPI adapter layer.
// Implementations must ensure thread-safe access and honor hexagonal
// architecture boundaries by depending only on domain models and shared
// packages.
//
// All methods operate on fully resolved domain.Schema values (post-inheritance
// processing) without exposing adapter-specific types.
type SchemaRegistryPort interface {
	// Get retrieves a schema by name.
	// Returns the resolved schema and a boolean indicating if the schema was
	// found in the registry.
	Get(name string) (domain.Schema, bool)
}
