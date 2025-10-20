// Package domain contains the core business logic models for Lithos.
// These models represent domain concepts and contain no infrastructure
// dependencies.
package domain

import (
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
	Name string

	// Extends is the parent schema name for inheritance chains.
	// References another schema by Name. Can form multi-level chains.
	// Empty string means no parent.
	Extends string

	// Excludes are parent property names to remove from inherited schema.
	// Enables subtractive inheritance when child needs to narrow parent's
	// property set.
	// Applied after parent resolution, before child property merging.
	// Property names must match exactly (case-sensitive).
	Excludes []string

	// Properties are property definitions declared in this schema file.
	// Represents the delta/override for inherited schemas, or complete property
	// set for root schemas.
	// Properties with same name as parent override parent definition.
	Properties []Property

	// ResolvedProperties is the flattened property list after applying
	// inheritance resolution, exclusions, and merging. Computed by Builder
	// pattern during schema loading.
	// This is the authoritative property list used by Validator.
	// Never persisted to disk (always computed).
	ResolvedProperties []Property
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

// NewSchemaWithExtends creates a new Schema with inheritance configuration.
// This helper is intended for use by adapters after parsing external data.
func NewSchemaWithExtends(
	name, extends string,
	excludes []string,
	properties []Property,
) Schema {
	return Schema{
		Name:               name,
		Extends:            extends,
		Excludes:           excludes,
		Properties:         properties,
		ResolvedProperties: nil, // Will be computed by SchemaRegistry
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

// SetResolvedProperties updates the resolved properties after inheritance
// resolution.
// This method is intended for use by SchemaRegistry during the build process.
func (s *Schema) SetResolvedProperties(resolved []Property) {
	s.ResolvedProperties = resolved
}

// GetResolvedProperties returns the computed properties after inheritance
// resolution. Returns Properties if ResolvedProperties is nil (for schemas
// without inheritance).
func (s *Schema) GetResolvedProperties() []Property {
	if s.ResolvedProperties != nil {
		return s.ResolvedProperties
	}
	return s.Properties
}

var identifierRegexp = regexp.MustCompile(`^[a-zA-Z][a-zA-Z0-9_-]*$`)

// trimIdentifier removes leading/trailing whitespace from an identifier.
func trimIdentifier(value string) string {
	return strings.TrimSpace(value)
}

// checkIdentifierNotEmpty validates that an identifier is not empty after
// trimming.
func checkIdentifierNotEmpty(fieldName, trimmed, original string) error {
	if trimmed == "" {
		return createEmptyIdentifierError(fieldName, original)
	}
	return nil
}

// createEmptyIdentifierError creates an empty validation error for identifiers.
func createEmptyIdentifierError(fieldName, original string) error {
	return errors.NewValidationError(fieldName, "cannot be empty", original)
}

// checkIdentifierFormat validates that an identifier matches the required
// format.
func checkIdentifierFormat(fieldName, trimmed, original string) error {
	if !isValidIdentifier(trimmed) {
		return createIdentifierFormatError(fieldName, original)
	}
	return nil
}

// createIdentifierFormatError creates a format validation error for
// identifiers.
func createIdentifierFormatError(fieldName, original string) error {
	return errors.NewValidationError(
		fieldName,
		"must be valid identifier (letters, numbers, dash, underscore only)",
		original,
	)
}

// checkSelfReference validates that an identifier does not reference itself.
func checkSelfReference(fieldName, value, selfName, original string) error {
	if value == selfName {
		return errors.NewValidationError(
			fieldName,
			"cannot reference itself",
			original,
		)
	}
	return nil
}

// checkDuplicateInList validates that a value is not already in the provided
// map.
func checkDuplicateInList(
	fieldName, value, original string,
	seen map[string]struct{},
) error {
	if _, exists := seen[value]; exists {
		return createDuplicateError(fieldName, original)
	}
	return nil
}

// createDuplicateError creates a duplicate validation error for excludes.
func createDuplicateError(fieldName, original string) error {
	return errors.NewValidationError(
		fieldName,
		fmt.Sprintf("duplicate exclude property: %s", original),
		original,
	)
}

// validateSingleExclude validates a single exclude entry and adds it to the
// seen map.
func validateSingleExclude(exclude string, seen map[string]struct{}) error {
	trimmed := trimIdentifier(exclude)

	if err := checkIdentifierNotEmpty("excludes", trimmed, exclude); err != nil {
		return err
	}

	if err := checkIdentifierFormat("excludes", trimmed, exclude); err != nil {
		return err
	}

	if err := checkDuplicateInList("excludes", trimmed, exclude, seen); err != nil {
		return err
	}

	seen[trimmed] = struct{}{}
	return nil
}

// validateSingleProperty validates a single property and checks for duplicates.
func validateSingleProperty(
	index int,
	prop Property,
	encountered map[string]struct{},
) error {
	if err := validatePropertySelf(index, prop); err != nil {
		return err
	}

	return checkPropertyDuplicate(prop.Name, encountered)
}

// validatePropertySelf validates the property itself.
func validatePropertySelf(index int, prop Property) error {
	if err := prop.Validate(); err != nil {
		return fmt.Errorf("property %d (%s): %w", index, prop.Name, err)
	}
	return nil
}

// checkPropertyDuplicate checks if a property name is already encountered and
// adds it to the map.
func checkPropertyDuplicate(
	name string,
	encountered map[string]struct{},
) error {
	if _, exists := encountered[name]; exists {
		return errors.NewValidationError(
			"properties",
			fmt.Sprintf("duplicate property name: %s", name),
			name,
		)
	}
	encountered[name] = struct{}{}
	return nil
}

func validateSchemaName(name string) error {
	trimmed := trimIdentifier(name)

	if err := checkIdentifierNotEmpty("name", trimmed, name); err != nil {
		return err
	}

	return checkIdentifierFormat("name", trimmed, name)
}

func validateSchemaExtends(name, extends string) error {
	trimmed := trimIdentifier(extends)
	if trimmed == "" {
		return nil
	}

	if err := checkIdentifierFormat("extends", trimmed, extends); err != nil {
		return err
	}

	return checkSelfReference("extends", trimmed, name, extends)
}

func validateSchemaExcludes(excludes []string) error {
	seen := make(map[string]struct{}, len(excludes))

	for _, exclude := range excludes {
		if err := validateSingleExclude(exclude, seen); err != nil {
			return err
		}
	}

	return nil
}

func validateSchemaProperties(properties []Property) error {
	encountered := make(map[string]struct{}, len(properties))

	for index, prop := range properties {
		if err := validateSingleProperty(index, prop, encountered); err != nil {
			return err
		}
	}

	return nil
}

func isValidIdentifier(value string) bool {
	return identifierRegexp.MatchString(value)
}
