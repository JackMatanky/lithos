package domain

import (
	"context"
	"errors"
	"fmt"
)

// PropertyBank represents a singleton registry of reusable Property
// definitions.
// It enables schemas to reference shared property definitions via $ref syntax,
// reducing duplication and ensuring consistency for common properties.
//
// PropertyBank is loaded once at startup from a single JSON file and represents
// singleton semantics per application lifecycle. Only one PropertyBank instance
// exists per application lifecycle (loaded once at startup).
//
// Schemas reference properties using JSON pointer syntax:
//
//	{"$ref": "#/properties/standard_title"}
//
// PropertyBank loaded from schemas/property_bank.json by SchemaLoader adapter.
//
// Reference: docs/architecture/data-models.md#propertybank.
type PropertyBank struct {
	// Properties contains named property definitions keyed by unique
	// identifier. Properties cannot reference other properties (no nested $ref
	// in PropertyBank itself).
	Properties map[string]Property `json:"properties"`
}

// NewPropertyBank creates a new PropertyBank with validation.
// It validates that all property IDs are non-empty and delegates property
// validation to each Property.Validate().
//
// Returns (*PropertyBank, nil) for valid input.
// Returns (nil, error) for validation failures with informative error messages.
func NewPropertyBank(properties map[string]Property) (*PropertyBank, error) {
	if err := validatePropertyIDs(properties); err != nil {
		return nil, err
	}

	if err := validatePropertyDefinitions(context.Background(), properties); err != nil {
		return nil, err
	}

	return &PropertyBank{
		Properties: cloneProperties(properties),
	}, nil
}

// Lookup returns a property by ID from the bank.
// Returns (Property, true) if found, (zero Property, false) if not found.
// Returns a copy to preserve immutability.
func (pb *PropertyBank) Lookup(id string) (Property, bool) {
	prop, exists := pb.Properties[id]
	return prop, exists
}

// validatePropertyIDs checks that all property IDs are non-empty strings.
func validatePropertyIDs(properties map[string]Property) error {
	var errs []error
	for id := range properties {
		if id == "" {
			errs = append(errs, fmt.Errorf("property ID cannot be empty"))
		}
	}
	return errors.Join(errs...)
}

// validatePropertyDefinitions validates each property definition by delegating
// to Property.Validate().
func validatePropertyDefinitions(
	ctx context.Context,
	properties map[string]Property,
) error {
	var errs []error
	for id, prop := range properties {
		if err := (&prop).Validate(ctx); err != nil {
			errs = append(errs, fmt.Errorf("property %s: %w", id, err))
		}
	}
	return errors.Join(errs...)
}

// cloneProperties creates a defensive copy of the properties map.
func cloneProperties(properties map[string]Property) map[string]Property {
	dst := make(map[string]Property, len(properties))
	for id, prop := range properties {
		dst[id] = prop
	}
	return dst
}
