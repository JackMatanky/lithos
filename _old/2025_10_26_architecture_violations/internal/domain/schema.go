// Package domain contains the core business logic models for Lithos.
// These models represent domain concepts and contain no infrastructure
// dependencies.
package domain

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
