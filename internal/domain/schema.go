// Package domain contains the core business logic models for Lithos.
// These models represent domain concepts and contain no infrastructure
// dependencies.
package domain

import (
	"encoding/json"
	"fmt"
	"regexp"
	"strings"

	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// Schema defines metadata class structure with property constraints and
// inheritance. Governs validation rules for notes of a given `fileClass`.
// Schemas are loaded from JSON files
// and resolved at application startup.
type Schema struct {
	// Name is the schema identifier matching `fileClass` frontmatter value.
	// Must be unique within vault. Used as key in schema registry map.
	Name string `json:"name"`

	// Extends is the parent schema name for inheritance chains.
	// References another schema by Name. Can form multi-level chains.
	// Empty string means no parent.
	Extends string `json:"extends,omitempty"`

	// Excludes are parent property names to remove from inherited schema.
	// Enables subtractive inheritance when child needs to narrow parent's
	// property set.
	// Applied after parent resolution, before child property merging.
	// Property names must match exactly (case-sensitive).
	Excludes []string `json:"excludes,omitempty"`

	// Properties are property definitions declared in this schema file.
	// Represents the delta/override for inherited schemas, or complete property
	// set for root schemas.
	// Properties with same name as parent override parent definition.
	Properties []Property `json:"properties"`

	// ResolvedProperties is the flattened property list after applying
	// inheritance resolution, exclusions, and merging. Computed by Builder
	// pattern during schema loading.
	// This is the authoritative property list used by Validator.
	// Never persisted to disk (always computed).
	ResolvedProperties []Property `json:"-"`
}

// NewSchema creates a new Schema with the given name and properties.
func NewSchema(name string, properties []Property) Schema {
	return Schema{
		Name:               name,
		Extends:            "",
		Excludes:           nil,
		Properties:         properties,
		ResolvedProperties: nil,
	}
}

// Validate checks if the schema definition itself is valid.
func (s *Schema) Validate() error {
	if err := validateSchemaName(s.Name); err != nil {
		return err
	}

	if err := validateSchemaExtends(s.Name, s.Extends); err != nil {
		return err
	}

	if err := validateSchemaExcludes(s.Excludes); err != nil {
		return err
	}

	return validateSchemaProperties(s.Properties)
}

// GetProperty returns the property with the given name, or nil if not found.
func (s *Schema) GetProperty(name string) *Property {
	for i := range s.Properties {
		if s.Properties[i].Name == name {
			return &s.Properties[i]
		}
	}
	return nil
}

// HasProperty checks if the schema has a property with the given name.
func (s *Schema) HasProperty(name string) bool {
	return s.GetProperty(name) != nil
}

// MarshalJSON implements custom JSON marshaling for Schema.
// Ensures properties are properly serialized with their type discriminators.
func (s *Schema) MarshalJSON() ([]byte, error) {
	// Create a map to control the output structure
	result := map[string]interface{}{
		"name":       s.Name,
		"properties": s.Properties,
	}

	if s.Extends != "" {
		result["extends"] = s.Extends
	}

	if len(s.Excludes) > 0 {
		result["excludes"] = s.Excludes
	}

	return json.Marshal(result)
}

// UnmarshalJSON implements custom JSON unmarshaling for Schema.
// Handles the properties array with discriminator-based unmarshaling.
func (s *Schema) UnmarshalJSON(data []byte) error {
	// Define a temporary struct for unmarshaling
	type schemaJSON struct {
		Name       string     `json:"name"`
		Extends    string     `json:"extends,omitempty"`
		Excludes   []string   `json:"excludes,omitempty"`
		Properties []Property `json:"properties"`
	}

	var sj schemaJSON
	if err := json.Unmarshal(data, &sj); err != nil {
		return err
	}

	s.Name = sj.Name
	s.Extends = sj.Extends
	s.Excludes = sj.Excludes
	s.Properties = sj.Properties

	return nil
}

var identifierRegexp = regexp.MustCompile(`^[a-zA-Z][a-zA-Z0-9_-]*$`)

func validateSchemaName(name string) error {
	trimmed := strings.TrimSpace(name)
	if trimmed == "" {
		return errors.NewValidationError("name", "cannot be empty", name)
	}

	if !isValidIdentifier(trimmed) {
		return errors.NewValidationError(
			"name",
			"must be valid identifier (letters, numbers, dash, underscore only)",
			name,
		)
	}

	return nil
}

func validateSchemaExtends(name, extends string) error {
	trimmed := strings.TrimSpace(extends)
	if trimmed == "" {
		return nil
	}

	if !isValidIdentifier(trimmed) {
		return errors.NewValidationError(
			"extends",
			"must be valid identifier (letters, numbers, dash, underscore only)",
			extends,
		)
	}

	if trimmed == name {
		return errors.NewValidationError(
			"extends",
			"cannot reference itself",
			extends,
		)
	}

	return nil
}

func validateSchemaExcludes(excludes []string) error {
	seen := make(map[string]struct{}, len(excludes))

	for _, exclude := range excludes {
		trimmed := strings.TrimSpace(exclude)
		if trimmed == "" {
			return errors.NewValidationError(
				"excludes",
				"property name cannot be empty",
				exclude,
			)
		}

		if !isValidIdentifier(trimmed) {
			return errors.NewValidationError(
				"excludes",
				fmt.Sprintf("invalid property name: %s", exclude),
				exclude,
			)
		}

		if _, exists := seen[trimmed]; exists {
			return errors.NewValidationError(
				"excludes",
				fmt.Sprintf("duplicate exclude property: %s", exclude),
				exclude,
			)
		}

		seen[trimmed] = struct{}{}
	}

	return nil
}

func validateSchemaProperties(properties []Property) error {
	encountered := make(map[string]struct{}, len(properties))

	for index, prop := range properties {
		if err := prop.Validate(); err != nil {
			return fmt.Errorf("property %d (%s): %w", index, prop.Name, err)
		}

		if _, exists := encountered[prop.Name]; exists {
			return errors.NewValidationError(
				"properties",
				fmt.Sprintf("duplicate property name: %s", prop.Name),
				prop.Name,
			)
		}

		encountered[prop.Name] = struct{}{}
	}

	return nil
}

func isValidIdentifier(value string) bool {
	return identifierRegexp.MatchString(value)
}
