package domain

import (
	"context"
	"errors"
	"fmt"
	"sync"
)

// Singleton pattern implementation for PropertyBank using sync.Once.
var (
	propertyBankInstance *PropertyBank
	propertyBankOnce     sync.Once
	propertyBankMu       sync.RWMutex // Protects instance during testing
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

// PropertyBankInstance returns the singleton PropertyBank instance.
// Thread-safe initialization guaranteed by sync.Once.
// On first call, creates empty PropertyBank. Subsequent calls return same
// instance.
// Note: In production, PropertyBank should be loaded via SchemaLoader adapter
// and set using SetPropertyBankForTesting() or direct assignment during
// initialization.
func PropertyBankInstance() *PropertyBank {
	propertyBankMu.RLock()
	if propertyBankInstance != nil {
		defer propertyBankMu.RUnlock()
		return propertyBankInstance
	}
	propertyBankMu.RUnlock()

	propertyBankOnce.Do(func() {
		propertyBankMu.Lock()
		defer propertyBankMu.Unlock()
		// Create empty PropertyBank as default
		// In production, this will be replaced by loaded PropertyBank
		propertyBankInstance = &PropertyBank{
			Properties: make(map[string]Property),
		}
	})

	propertyBankMu.RLock()
	defer propertyBankMu.RUnlock()
	return propertyBankInstance
}

// SetPropertyBankForTesting allows setting a custom PropertyBank instance for
// testing.
// This enables test isolation without global state pollution.
// Should only be used in tests. Use ResetPropertyBankForTesting() in test
// cleanup.
func SetPropertyBankForTesting(pb *PropertyBank) {
	propertyBankMu.Lock()
	defer propertyBankMu.Unlock()

	propertyBankInstance = pb
}

// ResetPropertyBankForTesting resets the singleton instance for test isolation.
// Should be called in test cleanup (typically via defer).
func ResetPropertyBankForTesting() {
	propertyBankMu.Lock()
	defer propertyBankMu.Unlock()

	propertyBankOnce = sync.Once{}
	propertyBankInstance = nil
}
