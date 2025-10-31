package domain

import (
	"context"
	"fmt"
)

// PropertyKind identifies the concrete type of an IProperty implementation.
type PropertyKind string

// Property kind constants describe whether an IProperty is a concrete
// definition or a reference to the property bank.
//
//nolint:decorder // Typed constants require PropertyKind to be defined first.
const (
	// PropertyKindDefinition marks an inline property definition.
	PropertyKindDefinition PropertyKind = "definition"
	// PropertyKindReference marks a property reference into the property bank.
	PropertyKindReference PropertyKind = "reference"
)

// IProperty is the common interface for Property and PropertyRef.
// It defines the contract for property validation, name access, and type
// identification.
type IProperty interface {
	GetName() string
	Type() PropertyKind
	Validate(ctx context.Context) error
}

// PropertyRef represents a reference to a property defined in the PropertyBank.
// It allows schemas to reuse common property definitions by referencing them.
//
// PropertyRef is resolved during schema resolution to obtain the full property
// definition from the PropertyBank.
//
// Reference: docs/architecture/data-models.md#property.
type PropertyRef struct {
	// Name is the property identifier as it appears in frontmatter
	// (case-sensitive).
	Name string `json:"name"`

	// Ref is the property identifier in the PropertyBank.
	// This is normalized from JSON pointer format (e.g., "#/properties/foo")
	// to simple identifiers (e.g., "foo").
	Ref string `json:"$ref"`
}

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

	// Spec defines type-specific validation constraints.
	Spec PropertySpec `json:"spec"`
}

// GetName returns the property name for PropertyRef.
func (p PropertyRef) GetName() string {
	return p.Name
}

// Type returns the kind identifier for PropertyRef.
func (p PropertyRef) Type() PropertyKind {
	return PropertyKindReference
}

// Validate performs structural validation of the PropertyRef.
// It checks that both Name and Ref are non-empty.
func (p PropertyRef) Validate(ctx context.Context) error {
	if err := validatePropertyName(p.Name); err != nil {
		return err
	}
	if p.Ref == "" {
		return fmt.Errorf("property reference must have non-empty $ref")
	}
	return nil
}

// GetName returns the property name for Property.
func (p Property) GetName() string {
	return p.Name
}

// Type returns the kind identifier for Property.
func (p Property) Type() PropertyKind {
	return PropertyKindDefinition
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
