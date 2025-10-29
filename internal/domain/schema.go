// Package domain contains the core business entities and domain logic for
// lithos.
// Schema represents a validation schema for note frontmatter fields.
package domain

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/JackMatanky/lithos/internal/shared/errors"
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
	// Examples: "contact", "project", "daily-note", "meeting-note"
	// Must be unique across all schemas in the system.
	Name string `json:"name" yaml:"name"`

	// Extends is the optional parent schema name for inheritance chains.
	// Can form multi-level inheritance (e.g., "fleeting-note" extends
	// "base-note" extends "note").
	// Empty string means no parent schema.
	// Example: "meeting-note" extends "base-note"
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
	Properties []Property `json:"properties" yaml:"properties"`
}

// UnmarshalJSON implements custom JSON unmarshaling for Schema.
// It converts the properties object (map[string]interface{}) to a slice of
// Property.
func (s *Schema) UnmarshalJSON(data []byte) error {
	// Define a temporary struct for unmarshaling
	type schemaAlias struct {
		Name       string                 `json:"name"`
		Extends    string                 `json:"extends,omitempty"`
		Excludes   []string               `json:"excludes,omitempty"`
		Properties map[string]interface{} `json:"properties"`
	}

	var alias schemaAlias
	if err := json.Unmarshal(data, &alias); err != nil {
		return err
	}

	s.Name = alias.Name
	s.Extends = alias.Extends
	s.Excludes = alias.Excludes

	// Convert properties map to slice
	s.Properties = make([]Property, 0, len(alias.Properties))
	for propName, propValue := range alias.Properties {
		var prop Property
		prop.Name = propName

		// Marshal the property value back to JSON for unmarshaling into
		// Property
		propData, err := json.Marshal(propValue)
		if err != nil {
			return fmt.Errorf(
				"failed to marshal property %s: %w",
				propName,
				err,
			)
		}

		if err := json.Unmarshal(propData, &prop); err != nil {
			return fmt.Errorf(
				"failed to unmarshal property %s: %w",
				propName,
				err,
			)
		}

		s.Properties = append(s.Properties, prop)
	}

	return nil
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
func (s Schema) Validate(ctx context.Context) error {
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
func (s Schema) validateName() error {
	if s.Name == "" {
		return errors.NewSchemaErrorWithRemediation(
			"schema name cannot be empty",
			"",
			"provide a unique schema name matching expected fileClass values",
			nil,
		)
	}
	return nil
}

// validateExcludesConstraint ensures excludes is only used with extends.
// Single Responsibility: Excludes/Extends constraint validation only.
func (s Schema) validateExcludesConstraint() error {
	if len(s.Excludes) > 0 && s.Extends == "" {
		return errors.NewSchemaErrorWithRemediation(
			"excludes can only be set when extends is not empty",
			s.Name,
			"either set extends to parent schema name or remove excludes",
			nil,
		)
	}
	return nil
}

// validateProperties validates each property and checks for duplicates.
func (s Schema) validateProperties(ctx context.Context) error {
	// Track property names to detect duplicates
	seen := make(map[string]bool)

	for _, prop := range s.Properties {
		// Check for cancellation during property validation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		// Check for duplicate property names
		if seen[prop.Name] {
			return errors.NewSchemaErrorWithRemediation(
				fmt.Sprintf("duplicate property name: %s", prop.Name),
				s.Name,
				"ensure all property names within a schema are unique",
				nil,
			)
		}
		seen[prop.Name] = true

		// Validate individual property
		if err := prop.Validate(ctx); err != nil {
			return errors.NewSchemaErrorWithRemediation(
				fmt.Sprintf("property %s validation failed", prop.Name),
				s.Name,
				"fix property definition according to architecture constraints",
				err,
			)
		}
	}

	return nil
}

// NewSchema creates a new Schema with validation.
// Returns error if the schema definition is invalid.
func NewSchema(
	name, extends string,
	excludes []string,
	properties []Property,
) (*Schema, error) {
	schema := Schema{
		Name:       name,
		Extends:    extends,
		Excludes:   excludes,
		Properties: properties,
	}

	if err := schema.Validate(context.Background()); err != nil {
		return nil, err
	}

	return &schema, nil
}
