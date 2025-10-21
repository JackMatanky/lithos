// Package domain contains the core business logic models for Lithos.
// These models represent domain concepts and contain no infrastructure
// dependencies.
package domain

import (
	"errors"
	"fmt"
	"reflect"
	"regexp"
	"strings"

	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

const (
	propertyTypeString = "string"
	propertyTypeNumber = "number"
	propertyTypeDate   = "date"
	propertyTypeFile   = "file"
	propertyTypeBool   = "bool"
)

var propertyNamePattern = regexp.MustCompile(`^[a-zA-Z][a-zA-Z0-9_-]*$`)

// PropertySpec defines the interface for type-specific property validation.
// Each implementation encapsulates validation logic for a specific property
// type.
type PropertySpec interface {
	// Validate checks if the given value conforms to this property
	// specification.
	// Returns nil if valid, or a ValidationError if invalid.
	Validate(value interface{}) error
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

// Validate checks if the property definition itself is valid.
func (p Property) Validate() error {
	if strings.TrimSpace(p.Name) == "" {
		return domainerrors.NewValidationError(
			"name",
			"cannot be empty",
			p.Name,
		)
	}

	if !propertyNamePattern.MatchString(p.Name) {
		return domainerrors.NewValidationError(
			"name",
			"must be valid YAML key (letters, numbers, dash, underscore only)",
			p.Name,
		)
	}

	if p.Spec == nil {
		return domainerrors.NewValidationError("spec", "cannot be nil", nil)
	}

	normalized, err := normalizeSpec(p.Spec)
	if err != nil {
		return err
	}

	_, typeErr := propertyTypeName(normalized)
	return typeErr
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

// ValidateValue ensures the provided value conforms to the property's array
// semantics and delegates to the spec for type-specific validation.
func (p Property) ValidateValue(value interface{}) error {
	if p.Spec == nil {
		return domainerrors.NewValidationError("spec", "cannot be nil", nil)
	}

	if p.Array {
		return p.validateArrayValue(value)
	}

	return p.Spec.Validate(value)
}

func (p Property) validateArrayValue(value interface{}) error {
	if value == nil {
		return nil
	}

	v := reflect.ValueOf(value)
	if v.Kind() != reflect.Slice && v.Kind() != reflect.Array {
		return domainerrors.NewValidationError(
			"value",
			"must be array or slice",
			value,
		)
	}

	for i := range v.Len() {
		elem := v.Index(i).Interface()
		if err := p.Spec.Validate(elem); err != nil {
			var validationErr domainerrors.ValidationError
			if errors.As(err, &validationErr) {
				field := fmt.Sprintf("value[%d].%s", i, validationErr.Field)
				return domainerrors.NewValidationError(
					field,
					validationErr.Message,
					validationErr.Value,
				)
			}
			return err
		}
	}

	return nil
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
