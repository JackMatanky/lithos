// Package domain contains the core business logic models for Lithos.
// These models represent domain concepts and contain no infrastructure
// dependencies.
package domain

import (
	"fmt"
	"strings"
	"sync"

	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// PropertyBank provides a library of reusable, pre-configured Property
// definitions that schemas can reference by name. Reduces duplication across
// schema definitions, ensures consistency for common properties.
type PropertyBank struct {
	// Properties contains named property definitions keyed by unique
	// identifier.
	// Keys should be descriptive names like "standard_title", "iso_date".
	Properties map[string]Property

	// Location is the path to property bank directory containing JSON files.
	// Default: "schemas/properties/"
	Location string
	mu       sync.RWMutex
}

// NewPropertyBank creates a new PropertyBank with the given location.
func NewPropertyBank(location string) PropertyBank {
	trimmed := strings.TrimSpace(location)
	if trimmed == "" {
		trimmed = "schemas/properties/"
	}

	return PropertyBank{
		Properties: make(map[string]Property),
		Location:   trimmed,
		mu:         sync.RWMutex{},
	}
}

// RegisterProperty adds a reusable property definition to the bank.
// Returns an error if a property with the same name already exists.
func (pb *PropertyBank) RegisterProperty(name string, property Property) error {
	trimmed := strings.TrimSpace(name)
	if trimmed == "" {
		return errors.NewValidationError("name", "cannot be empty", name)
	}

	if trimmed != name {
		return errors.NewValidationError(
			"name",
			"cannot have leading/trailing whitespace",
			name,
		)
	}

	if err := property.Validate(); err != nil {
		return fmt.Errorf("invalid property definition: %w", err)
	}

	pb.mu.Lock()
	defer pb.mu.Unlock()

	if _, exists := pb.Properties[name]; exists {
		return errors.NewValidationError(
			"name",
			"property already exists",
			name,
		)
	}

	pb.Properties[name] = property
	return nil
}

// GetProperty returns the property with the given name and a boolean indicating
// whether the property exists.
func (pb *PropertyBank) GetProperty(name string) (Property, bool) {
	pb.mu.RLock()
	defer pb.mu.RUnlock()

	property, exists := pb.Properties[name]
	return property, exists
}

// HasProperty checks if the property bank has a property with the given name.
func (pb *PropertyBank) HasProperty(name string) bool {
	_, exists := pb.GetProperty(name)
	return exists
}

// Validate checks if the property bank definition itself is valid.
func (pb *PropertyBank) Validate() error {
	pb.mu.RLock()
	defer pb.mu.RUnlock()

	if strings.TrimSpace(pb.Location) == "" {
		return errors.NewValidationError(
			"location",
			"cannot be empty",
			pb.Location,
		)
	}

	// Validate all registered properties
	for name, property := range pb.Properties {
		if err := property.Validate(); err != nil {
			return fmt.Errorf("property '%s' is invalid: %w", name, err)
		}
	}

	return nil
}
