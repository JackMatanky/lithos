package domain

import (
	"context"
	"errors"
	"fmt"

	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
)

// Schema defines metadata structure with property constraints and inheritance.
// Governs validation rules for notes of a given fileClass.
//
// Architecture Reference: docs/architecture/data-models.md#schema
//
// Design:
//   - Rich domain model with structural validation behavior
//   - Inheritance via Extends/Excludes (resolved at startup by SchemaResolver)
//   - Properties stored as delta/override for inherited schemas
//   - Name matches fileClass frontmatter value for schema lookup
//
// Inheritance:
//   - Extends: Optional parent schema name for inheritance chains
//   - Excludes: Parent property names to exclude (only when Extends set)
//   - Properties: Delta/override for inherited, complete set for root schemas
//
// Validation:
//   - Schema.Validate() checks structural integrity
//   - Property validation delegated to each Property.Validate()
//   - Inheritance resolution handled by separate SchemaResolver service
type Schema struct {
	// Name is the schema identifier matching fileClass frontmatter value.
	// Examples: "contact", "project", "daily-note", "meeting_note"
	// Must be unique across all schemas in the system.
	Name string `json:"name" yaml:"name"`

	// Extends is the optional parent schema name for inheritance chains.
	// Can form multi-level inheritance (e.g., "fleeting-note" extends
	// "base-note" extends "note").
	// Empty string means no parent schema.
	// Example: "meeting_note" extends "base-note"
	Extends string `json:"extends,omitempty" yaml:"extends,omitempty"`

	// Excludes lists parent property names to exclude from inheritance.
	// Only applicable when Extends is not empty.
	// Enables subtractive inheritance to remove unwanted parent properties.
	// Example: ["internal_id", "legacy_field"]
	Excludes []string `json:"excludes,omitempty" yaml:"excludes,omitempty"`

	// Properties defines the property validation rules for this schema.
	// For inherited schemas: represents delta/override properties
	// For root schemas: represents the complete property set
	// Property names must be unique within the schema
	// Contains only Property entities (no PropertyRef in domain layer)
	Properties []Property `json:"properties" yaml:"properties"`

	// ResolvedProperties contains the flattened property set after inheritance
	// resolution and $ref substitution. Populated by SchemaResolver.
	// This is the final property set used for validation and consumption.
	// Empty until SchemaResolver.Resolve() is called.
	// Always contains only Property (no PropertyRef).
	ResolvedProperties []Property `json:"resolved_properties,omitempty" yaml:"resolved_properties,omitempty"` //nolint:lll // field tags required for JSON/YAML
}

// NewSchema creates a new Schema with defensive copies to preserve
// immutability.
// Returns error if the schema definition is invalid.
//
// Defensive copies prevent external modification of schema state after
// construction, ensuring schema definitions remain stable throughout their
// lifetime.
func NewSchema(
	name, extends string,
	excludes []string,
	properties []Property,
) (*Schema, error) {
	// Defensive copy of excludes slice
	excludesCopy := make([]string, len(excludes))
	copy(excludesCopy, excludes)

	// Defensive copy of properties slice
	propertiesCopy := make([]Property, len(properties))
	copy(propertiesCopy, properties)

	schema := Schema{
		Name:               name,
		Extends:            extends,
		Excludes:           excludesCopy,
		Properties:         propertiesCopy,
		ResolvedProperties: nil,
	}

	if err := schema.Validate(context.Background()); err != nil {
		return nil, err
	}

	return &schema, nil
}

// Validate performs structural validation of the schema definition.
//
// Validation Rules:
//   - Name must not be empty (required for schema lookup)
//   - Excludes can only be set when Extends is not empty
//   - Each Property must pass its own validation
//   - Property names must be unique within the schema
//
// Returns SchemaError for structural issues with schema definition.
// Does not validate inheritance chains - that's handled by SchemaResolver.
//
// Context is used for cancellation during potentially expensive validation.
func (s *Schema) Validate(ctx context.Context) error {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	if err := s.validateName(); err != nil {
		return err
	}

	if err := s.validateExcludesConstraint(); err != nil {
		return err
	}

	if err := s.validateProperties(ctx); err != nil {
		return err
	}

	return nil
}

// validateName ensures schema name is not empty.
func (s *Schema) validateName() error {
	if s.Name == "" {
		return lithosErr.NewSchemaErrorWithRemediation(
			"schema name cannot be empty",
			"",
			"provide a unique schema name matching expected fileClass values",
			nil,
		)
	}
	return nil
}

// validateExcludesConstraint ensures excludes is only used with extends.
func (s *Schema) validateExcludesConstraint() error {
	if len(s.Excludes) > 0 && s.Extends == "" {
		return lithosErr.NewSchemaErrorWithRemediation(
			"excludes can only be set when extends is not empty",
			s.Name,
			"either set extends to parent schema name or remove excludes",
			nil,
		)
	}
	return nil
}

// validateProperties validates all properties with a single pass through the
// list.
// Checks for duplicates and validates each property, aggregating all errors.
// Note: This may result in duplicate validation calls if properties were
// already validated during NewProperty construction, but ensures safety.
func (s *Schema) validateProperties(ctx context.Context) error {
	seen := make(map[string]bool)
	var errs []error

	for _, prop := range s.Properties {
		// Check for cancellation
		if err := ctx.Err(); err != nil {
			return err
		}

		// Check for duplicate property name
		if err := s.validateUniquePropertyName(prop.Name, seen); err != nil {
			errs = append(errs, err)
			continue
		}

		// Validate the property
		if err := prop.Validate(ctx); err != nil {
			errs = append(
				errs,
				fmt.Errorf("property %s: %w", prop.Name, err),
			)
		}
	}

	if len(errs) > 0 {
		return errors.Join(errs...)
	}
	return nil
}

// validateUniquePropertyName checks if a property name is duplicate and marks
// it as seen.
func (s *Schema) validateUniquePropertyName(
	name string,
	seen map[string]bool,
) error {
	if seen[name] {
		return lithosErr.NewSchemaErrorWithRemediation(
			fmt.Sprintf("duplicate property name: %s", name),
			s.Name,
			"ensure all property names within a schema are unique",
			nil,
		)
	}
	seen[name] = true
	return nil
}
