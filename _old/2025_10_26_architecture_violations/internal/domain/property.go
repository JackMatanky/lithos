// Package domain contains the core business logic models for Lithos.
// These models represent domain concepts and contain no infrastructure
// dependencies.
package domain

import (
	"fmt"
	"strings"
	"sync"

	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

const (
	propertyTypeString = "string"
	propertyTypeNumber = "number"
	propertyTypeDate   = "date"
	propertyTypeFile   = "file"
	propertyTypeBool   = "bool"
)

// PropertySpec defines the interface for type-specific property specifications.
// Each implementation encapsulates configuration for a specific property type.
type PropertySpec interface {
	// PropertySpec is now a pure data interface - validation moved to services
}

// Property defines a single metadata property with type constraints and
// validation rules. Properties describe what data can be stored in frontmatter
// and how it should be validated.
type Property struct {
	// Name is the property identifier matching frontmatter key.
	// Case-sensitive, must be valid YAML key.
	Name string

	// Required indicates whether this property must be present in note
	// frontmatter.
	Required bool

	// Array indicates whether this property accepts multiple values (YAML
	// list). If true, values must be slices/arrays. If false, values must be
	// scalar.
	Array bool

	// Spec contains type-specific configuration and validation rules.
	// Exactly one spec type per property based on semantic type.
	Spec PropertySpec
}

// NewProperty creates a new Property with the given specification. Callers
// should invoke Validate on the resulting property before using it to ensure
// the configuration is valid.
func NewProperty(
	name string,
	required, array bool,
	spec PropertySpec,
) Property {
	return Property{
		Name:     name,
		Required: required,
		Array:    array,
		Spec:     spec,
	}
}

// TypeName returns the semantic type name (string/number/date/file/bool) for
// this property based on its spec.
func (p Property) TypeName() (string, error) {
	normalized, err := normalizeSpec(p.Spec)
	if err != nil {
		return "", err
	}

	return propertyTypeName(normalized)
}

// Property spec normalization helpers

func normalizeSpec(spec PropertySpec) (PropertySpec, error) {
	if spec == nil {
		return nil, fmt.Errorf("property spec cannot be nil")
	}

	if deref, handled, err := dereferencedSpec(spec); handled {
		return deref, err
	}

	return spec, nil
}

func dereferencedSpec(spec PropertySpec) (PropertySpec, bool, error) {
	switch typed := spec.(type) {
	case *StringPropertySpec:
		return dereferenceStringSpec(typed)
	case *NumberPropertySpec:
		return dereferenceNumberSpec(typed)
	case *DatePropertySpec:
		return dereferenceDateSpec(typed)
	case *FilePropertySpec:
		return dereferenceFileSpec(typed)
	case *BoolPropertySpec:
		return dereferenceBoolSpec(typed)
	default:
		return nil, false, nil
	}
}

func dereferenceStringSpec(
	spec *StringPropertySpec,
) (PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("string property spec cannot be nil")
	}
	return *spec, true, nil
}

func dereferenceNumberSpec(
	spec *NumberPropertySpec,
) (PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("number property spec cannot be nil")
	}
	return *spec, true, nil
}

func dereferenceDateSpec(spec *DatePropertySpec) (PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("date property spec cannot be nil")
	}
	return *spec, true, nil
}

func dereferenceFileSpec(spec *FilePropertySpec) (PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("file property spec cannot be nil")
	}
	return *spec, true, nil
}

func dereferenceBoolSpec(spec *BoolPropertySpec) (PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("bool property spec cannot be nil")
	}
	return *spec, true, nil
}

func propertyTypeName(spec PropertySpec) (string, error) {
	switch spec.(type) {
	case StringPropertySpec:
		return propertyTypeString, nil
	case NumberPropertySpec:
		return propertyTypeNumber, nil
	case DatePropertySpec:
		return propertyTypeDate, nil
	case FilePropertySpec:
		return propertyTypeFile, nil
	case BoolPropertySpec:
		return propertyTypeBool, nil
	default:
		return "", fmt.Errorf("unknown property spec type: %T", spec)
	}
}

// PropertyBank provides a library of reusable, pre-configured Property
// definitions that schemas can reference by name. Reduces duplication across
// schema definitions, ensures consistency for common properties.
type PropertyBank struct {
	// Properties contains named property definitions keyed by unique
	// identifier.
	// Keys should be descriptive names like "standard_title", "iso_date".
	Properties map[string]Property

	// Location is the path to property bank JSON file.
	// Default: "schemas/property_bank.json"
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
		return domainerrors.NewValidationError("name", "cannot be empty", name)
	}

	if trimmed != name {
		return domainerrors.NewValidationError(
			"name",
			"cannot have leading/trailing whitespace",
			name,
		)
	}

	pb.mu.Lock()
	defer pb.mu.Unlock()

	if _, exists := pb.Properties[name]; exists {
		return domainerrors.NewValidationError(
			"name",
			"property already exists",
			name,
		)
	}

	pb.Properties[name] = property
	return nil
}

// StringPropertySpec validates string values with optional enum and pattern
// constraints.
type StringPropertySpec struct {
	// Enum contains allowed values as fixed list. If non-empty, value must be
	// in list.
	// Empty list means no enum constraint (any string valid).
	Enum []string

	// Pattern is a regex pattern for custom string validation.
	// If non-empty, value must match pattern. Uses Go regexp package.
	Pattern string
}

// NumberPropertySpec validates numeric values with optional min/max/step
// constraints.
type NumberPropertySpec struct {
	// Min is the minimum allowed value (inclusive). Nil means no minimum
	// constraint.
	Min *float64

	// Max is the maximum allowed value (inclusive). Nil means no maximum
	// constraint.
	Max *float64

	// Step is the increment/decrement amount. If 1.0, implies integer values.
	// If nil, any precision allowed.
	Step *float64
}

// DatePropertySpec validates date/time values with format constraints.
type DatePropertySpec struct {
	// Format is the Go time layout string for parsing.
	// If empty, defaults to RFC3339.
	Format string
}

// FilePropertySpec validates file reference values with optional
// class/directory constraints.
type FilePropertySpec struct {
	// FileClass restricts valid file references to notes with specific
	// fileClass value. Supports negation via ^ prefix. Empty string means no
	// fileClass restriction.
	FileClass string

	// Directory restricts valid file references to notes within specific vault
	// directory path.
	// Path is relative to vault root. Supports negation via ^ prefix.
	Directory string
}

// BoolPropertySpec validates boolean values. No additional configuration
// needed.
type BoolPropertySpec struct{}
