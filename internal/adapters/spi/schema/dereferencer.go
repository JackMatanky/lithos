package schema

import (
	"context"
	"fmt"

	"github.com/JackMatanky/lithos/internal/domain"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// PropertyRef represents a reference to a property definition in the
// PropertyBank.
// This is an adapter layer structure used for $ref resolution.
type PropertyRef struct {
	Name string // The name to use for the property in the schema
	Ref  string // The reference to lookup in PropertyBank
}

// MixedProperty represents either a resolved Property or a PropertyRef that
// needs resolution.
type MixedProperty struct {
	Property    *domain.Property // nil if this is a PropertyRef
	PropertyRef *PropertyRef     // nil if this is a resolved Property
}

// PropertyDereferencer handles $ref replacement with PropertyBank lookups.
// It operates at the adapter layer, converting property references into
// concrete Property definitions from the PropertyBank.
//
// This component is part of the DDD architecture refactoring, splitting
// the original SchemaResolver into focused infrastructure components.
//
// Responsibilities:
//   - Handle $ref replacement with PropertyBank property lookups
//   - Pure infrastructure concern - JSON pointer resolution
//   - Error on missing $ref targets
//   - One-to-one mapping validation with PropertyBank
//
// Architecture Reference: docs/architecture/components.md#propertydereferencer.
type PropertyDereferencer struct{}

// NewPropertyDereferencer creates a new PropertyDereferencer instance.
// PropertyDereferencer has no dependencies and is pure infrastructure logic.
func NewPropertyDereferencer() *PropertyDereferencer {
	return &PropertyDereferencer{}
}

// DereferenceProperties converts a mixed list of Properties and PropertyRefs
// into a list of concrete Properties by resolving all $ref references.
//
// For PropertyRef entries, looks up the target in PropertyBank and creates
// a new Property using:
//   - Name from the PropertyRef (the key in the schema)
//   - Spec, Required, Array from the PropertyBank definition
//
// For Property entries, passes them through unchanged.
//
// Returns error if any $ref target is not found in PropertyBank.
//
// Context is used for cancellation during potentially long-running resolution.
func (d *PropertyDereferencer) DereferenceProperties(
	ctx context.Context,
	schemaName string,
	properties []MixedProperty,
	bank domain.PropertyBank,
) ([]domain.Property, error) {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	if len(properties) == 0 {
		return []domain.Property{}, nil
	}

	result := make([]domain.Property, 0, len(properties))

	for _, prop := range properties {
		// Check for cancellation during iteration
		if err := ctx.Err(); err != nil {
			return nil, err
		}

		switch {
		case prop.Property != nil:
			// Pass through concrete Properties unchanged
			result = append(result, *prop.Property)
		case prop.PropertyRef != nil:
			// Resolve $ref to concrete Property
			resolved, err := d.resolvePropertyRef(
				*prop.PropertyRef,
				bank,
				schemaName,
			)
			if err != nil {
				return nil, err
			}
			result = append(result, resolved)
		default:
			// Invalid MixedProperty - both fields nil
			continue
		}
	}

	return result, nil
}

// resolvePropertyRef converts a PropertyRef into a concrete Property by looking
// up
// the definition in the PropertyBank.
func (d *PropertyDereferencer) resolvePropertyRef(
	propRef PropertyRef,
	bank domain.PropertyBank,
	schemaName string,
) (domain.Property, error) {
	// Look up the property definition in the bank
	bankProp, exists := bank.Lookup(propRef.Ref)
	if !exists {
		return domain.Property{}, lithoserrors.NewSchemaErrorWithRemediation(
			fmt.Sprintf(
				"schema %s, property %s: $ref '%s' not found in property bank",
				schemaName,
				propRef.Name,
				propRef.Ref,
			),
			schemaName,
			fmt.Sprintf(
				"add property '%s' to property bank or fix $ref",
				propRef.Ref,
			),
			nil,
		)
	}

	// Create a new Property using:
	// - Name from the PropertyRef (the key in the schema)
	// - Spec, Required, Array from the PropertyBank definition
	// This allows the schema to use the ref's name while getting validation
	// rules from the bank
	propPtr, err := domain.NewProperty(
		propRef.Name,
		bankProp.Required,
		bankProp.Array,
		bankProp.Spec,
	)
	if err != nil {
		return domain.Property{}, err
	}
	return *propPtr, nil
}
