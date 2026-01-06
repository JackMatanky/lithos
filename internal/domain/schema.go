package domain

import (
	"context"
	"errors"
	"fmt"

	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
)

// Schema defines metadata structure with property constraints and inheritance.
// Governs validation rules for notes of a given fileClass.
type Schema struct {
	// Name is the schema identifier matching fileClass frontmatter value.
	Name string `json:"name" yaml:"name"`

	// Extends is the optional parent schema name for inheritance chains.
	Extends string `json:"extends,omitempty" yaml:"extends,omitempty"`

	// Excludes lists parent property names to exclude from inheritance.
	Excludes []string `json:"excludes,omitempty" yaml:"excludes,omitempty"`

	// Properties defines the property validation rules for this schema.
	Properties []Property `json:"properties" yaml:"properties"`

	// Resolved contains the flattened property set after inheritance.
	Resolved []Property `json:"resolved_properties,omitempty" yaml:"resolved_properties,omitempty"`
}

// NewSchema creates a new Schema with defensive copies.
func NewSchema(
	name, extends string,
	excludes []string,
	properties []Property,
) (*Schema, error) {
	excludesCopy := make([]string, len(excludes))
	copy(excludesCopy, excludes)

	propertiesCopy := make([]Property, len(properties))
	copy(propertiesCopy, properties)

	schema := Schema{
		Name:       name,
		Extends:    extends,
		Excludes:   excludesCopy,
		Properties: propertiesCopy,
		Resolved:   nil,
	}

	if err := schema.Validate(context.Background()); err != nil {
		return nil, err
	}

	return &schema, nil
}

// Validate performs structural validation of the schema definition.
func (s *Schema) Validate(ctx context.Context) error {
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	if err := s.validateName(); err != nil {
		return err
	}

	if err := s.validateExcludesConstraint(); err != nil {
		return err
	}

	return s.validateProperties(ctx)
}

func (s *Schema) validateName() error {
	if s.Name == "" {
		return lithosErr.NewSchemaErrorWithRemediation(
			"schema name cannot be empty",
			"",
			"provide a unique schema name matching expected fileClass values",
			nil,
		)
	}
	return nil
}

func (s *Schema) validateExcludesConstraint() error {
	if len(s.Excludes) > 0 && s.Extends == "" {
		return lithosErr.NewSchemaErrorWithRemediation(
			"excludes can only be set when extends is not empty",
			s.Name,
			"either set extends to parent schema name or remove excludes",
			nil,
		)
	}
	return nil
}

func (s *Schema) validateProperties(ctx context.Context) error {
	seen := make(map[string]bool)
	var errs []error

	for _, prop := range s.Properties {
		if err := ctx.Err(); err != nil {
			return err
		}

		if err := s.validateUniquePropertyName(prop.Name, seen); err != nil {
			errs = append(errs, err)
			continue
		}

		if err := prop.Validate(ctx); err != nil {
			errs = append(
				errs,
				fmt.Errorf("property %s: %w", prop.Name, err),
			)
		}
	}

	if len(errs) > 0 {
		return errors.Join(errs...)
	}
	return nil
}

func (s *Schema) validateUniquePropertyName(
	name string,
	seen map[string]bool,
) error {
	if seen[name] {
		return lithosErr.NewSchemaErrorWithRemediation(
			fmt.Sprintf("duplicate property name: %s", name),
			s.Name,
			"ensure all property names within a schema are unique",
			nil,
		)
	}
	seen[name] = true
	return nil
}
