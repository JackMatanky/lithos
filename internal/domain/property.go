package domain

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
)

// Property represents a DDD entity for schema property definitions with
// validation constraints.
// It defines how a frontmatter field should be validated in notes.
// As a DDD entity, Property has identity determined by its ID field.
//
// Reference: docs/architecture/data-models.md#property.
type Property struct {
	// ID is the unique identifier for this property entity, generated from
	// hash of (Name + Spec content) to ensure deterministic identity.
	ID string `json:"id"`

	// Name is the property identifier matching frontmatter key
	// (case-sensitive).
	Name string `json:"name"`

	// Required indicates whether property must be present in frontmatter.
	Required bool `json:"required"`

	// Array indicates whether property accepts multiple values vs single
	// scalar.
	Array bool `json:"array"`

	// Spec defines type-specific validation constraints.
	Spec PropertySpec `json:"spec"`
}

// NewProperty creates a new Property entity with auto-generated ID.
// The ID is generated using hash of (Name + Spec content) for deterministic
// identity.
// Returns error if the property definition is invalid.
func NewProperty(
	name string,
	required, array bool,
	spec PropertySpec,
) (*Property, error) {
	// Generate deterministic ID from name and spec content
	id := generatePropertyID(name, spec)

	property := Property{
		ID:       id,
		Name:     name,
		Required: required,
		Array:    array,
		Spec:     spec,
	}

	// Validate the property
	if err := property.Validate(context.Background()); err != nil {
		return nil, err
	}

	return &property, nil
}

// generatePropertyID creates a deterministic hash-based ID from property name
// and full spec content for maximum uniqueness.
func generatePropertyID(name string, spec PropertySpec) string {
	// Serialize the full spec to JSON for comprehensive content hashing
	specJSON, err := json.Marshal(spec)
	if err != nil {
		// Fallback to type-only if serialization fails (shouldn't happen)
		specJSON = []byte(fmt.Sprintf(`{"type":%q}`, spec.Type()))
	}

	// Include name and full spec content
	content := fmt.Sprintf("%s|%s", name, string(specJSON))
	hash := sha256.Sum256([]byte(content))
	return hex.EncodeToString(hash[:])
}

// Validate performs structural validation of the Property definition.
// It checks basic constraints and delegates type-specific validation to Spec.
func (p Property) Validate(ctx context.Context) error {
	if err := validatePropertyName(p.Name); err != nil {
		return err
	}

	// Validate the spec
	if err := ensurePropertySpec(p.Spec); err != nil {
		return err
	}
	return validatePropertySpec(ctx, p.Name, p.Spec)
}

// validatePropertyName checks that property name is not empty.
func validatePropertyName(name string) error {
	if name != "" {
		return nil
	}
	return fmt.Errorf("property name cannot be empty")
}

// ensurePropertySpec checks that property spec is not nil.
func ensurePropertySpec(spec PropertySpec) error {
	if spec != nil {
		return nil
	}
	return fmt.Errorf("property spec cannot be nil")
}

// validatePropertySpec delegates validation to the PropertySpec implementation.
func validatePropertySpec(
	ctx context.Context,
	name string,
	spec PropertySpec,
) error {
	if err := spec.Validate(ctx); err != nil {
		return fmt.Errorf("property %s: %w", name, err)
	}
	return nil
}
