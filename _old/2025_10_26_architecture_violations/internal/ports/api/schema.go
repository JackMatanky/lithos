// Package api defines application programming interface ports for hexagonal
// architecture.
//
// These interfaces define contracts that domain services must implement
// to provide application capabilities. This package contains the driving
// ports that allow external actors to interact with the domain layer.
package api

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// SchemaValidatorPort defines the interface for schema validation services.
//
// This port allows external actors to validate frontmatter data against
// schema definitions while maintaining hexagonal architecture boundaries.
// The domain service implementation is responsible for:
// - Schema lookup and retrieval from registry
// - Polymorphic property validation using PropertySpec interface
// - Required field and array constraint validation
// - Structured error reporting with field-level details
//
// All validation operations support context cancellation for long-running
// validation processes.
type SchemaValidatorPort interface {
	// Validate validates frontmatter data against a specific schema.
	//
	// The method looks up the schema by name from the registry, then validates
	// each frontmatter field against the schema's resolved properties (after
	// inheritance resolution). Uses PropertySpec polymorphism for type-specific
	// validation rules.
	//
	// Returns a Result containing ValidationResult with all field-level errors,
	// or an error if schema lookup fails or validation cannot be performed.
	//
	// Context can be used for cancellation and timeout control during
	// validation.
	Validate(
		ctx context.Context,
		schemaName string,
		frontmatter domain.Frontmatter,
	) errors.Result[errors.ValidationResult]
}
