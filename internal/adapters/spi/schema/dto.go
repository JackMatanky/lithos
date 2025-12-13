package schema

import (
	"context"
	"encoding/json"
	"fmt"
	"sort"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
)

// schemaDTO mirrors the on-disk schema JSON document structure.
type schemaDTO struct {
	Name       string                     `json:"name"`
	Extends    string                     `json:"extends"`
	Excludes   []string                   `json:"excludes"`
	Properties map[string]json.RawMessage `json:"properties"`
}

// propertyBankDTO represents the JSON layout of the property bank document.
type propertyBankDTO struct {
	Properties map[string]json.RawMessage `json:"properties"`
}

// propertyRefDTO carries the optional `$ref` pointer for a property.
type propertyRefDTO struct {
	Ref string `json:"$ref"`
}

// Normalize returns the canonical property identifier for this reference.
func (r propertyRefDTO) Normalize() string {
	if r.Ref == "" {
		return ""
	}

	const propertiesPrefix = "#/properties/"
	if strings.HasPrefix(r.Ref, propertiesPrefix) {
		return r.Ref[len(propertiesPrefix):]
	}

	if strings.HasPrefix(r.Ref, "#/") {
		if idx := strings.LastIndex(r.Ref, "/"); idx > 1 && idx < len(r.Ref)-1 {
			return r.Ref[idx+1:]
		}
	}

	return r.Ref
}

// toDomain converts the DTO into a domain.PropertyBank.
func (dto propertyBankDTO) toDomain(
	ctx context.Context,
	path string,
) (domain.PropertyBank, error) {
	properties := make(map[string]domain.Property, len(dto.Properties))

	for id, raw := range dto.Properties {
		// PropertyBank entries are always full definitions, never refs
		prop, err := parsePropertyDef(ctx, id, raw, path, "property_bank")
		if err != nil {
			return domain.PropertyBank{}, err
		}
		properties[id] = prop
	}

	bankPtr, err := domain.NewPropertyBank(properties)
	if err != nil {
		return domain.PropertyBank{}, propertyDefinitionError(
			fmt.Sprintf("invalid property bank at %s", path),
			"property_bank",
			path,
			err,
		)
	}

	return *bankPtr, nil
}

// toDomain materializes the schema DTO into a domain.Schema value.
// It resolves $ref references immediately using the PropertyBank.
func (dto schemaDTO) toDomain(
	ctx context.Context,
	path string,
	bank domain.PropertyBank,
) (domain.Schema, error) {
	// Create mixed slice of Properties and PropertyRefs
	properties := make([]MixedProperty, 0, len(dto.Properties))

	// Sort property keys for deterministic ordering
	keys := make([]string, 0, len(dto.Properties))
	for key := range dto.Properties {
		keys = append(keys, key)
	}
	sort.Strings(keys)

	for _, name := range keys {
		raw := dto.Properties[name]

		// Check if this is a $ref property
		var ref propertyRefDTO
		if err := json.Unmarshal(raw, &ref); err == nil && ref.Ref != "" {
			// This is a $ref - create PropertyRef for immediate resolution
			normalizedRef := ref.Normalize()
			properties = append(properties, MixedProperty{
				Property: nil,
				PropertyRef: &PropertyRef{
					Name: name,
					Ref:  normalizedRef,
				},
			})
			continue
		}

		// Parse as full Property definition
		prop, err := parsePropertyDef(ctx, name, raw, path, dto.Name)
		if err != nil {
			return domain.Schema{}, err
		}
		properties = append(properties, MixedProperty{
			Property:    &prop,
			PropertyRef: nil,
		})
	}

	// Resolve $refs immediately using PropertyDereferencer
	dereferencer := NewPropertyDereferencer()
	resolvedProperties, err := dereferencer.DereferenceProperties(
		ctx,
		dto.Name,
		properties,
		bank,
	)
	if err != nil {
		return domain.Schema{}, err
	}

	return domain.Schema{
		Name:               dto.Name,
		Extends:            dto.Extends,
		Excludes:           dto.Excludes,
		Properties:         resolvedProperties, // Original properties (resolved)
		ResolvedProperties: resolvedProperties, // Same as Properties for now
	}, nil
}
