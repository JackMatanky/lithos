package domain

import (
	"context"
	"fmt"
)

// Property represents a schema property definition with validation constraints.
// It defines how a frontmatter field should be validated in notes.
//
// Reference: docs/architecture/data-models.md#property.
type Property struct {
	// Name is the property identifier matching frontmatter key
	// (case-sensitive).
	Name string `json:"name"`

	// Required indicates whether property must be present in frontmatter.
	Required bool `json:"required"`

	// Array indicates whether property accepts multiple values vs single
	// scalar.
	Array bool `json:"array"`

	// Ref is a JSON pointer reference to a property in the PropertyBank.
	// When present, this property inherits all attributes from the referenced
	// property.
	// Mutually exclusive with Spec - only one can be set.
	Ref string `json:"$ref,omitempty"`

	// Spec defines type-specific validation constraints.
	// Mutually exclusive with Ref - only one can be set.
	Spec PropertySpec `json:"spec,omitempty"`
}

// Validate performs structural validation of the Property definition.
// It checks basic constraints and delegates type-specific validation to Spec.
func (p Property) Validate(ctx context.Context) error {
	if err := validatePropertyName(p.Name); err != nil {
		return err
	}

	// Either Ref or Spec must be set, but not both
	if p.Ref != "" && p.Spec != nil {
		return fmt.Errorf("property cannot have both $ref and spec")
	}
	if p.Ref == "" && p.Spec == nil {
		return fmt.Errorf("property must have either $ref or spec")
	}

	// If using $ref, no further validation needed here
	if p.Ref != "" {
		return nil
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
