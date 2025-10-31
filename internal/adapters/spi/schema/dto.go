package schema

import (
	"context"
	"encoding/json"
	"fmt"
	"sort"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// propertySpecParsers map property type identifiers to domain spec decoders.
var propertySpecParsers = map[string]func(json.RawMessage) (domain.PropertySpec, error){
	"string": parseStringSpec,
	"number": parseNumberSpec,
	"bool":   parseBoolSpec,
	"date":   parseDateSpec,
	"file":   parseFileSpec,
}

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

func parseStringSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	if len(raw) == 0 {
		return domain.StringSpec{}, nil
	}
	var spec domain.StringSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func parseNumberSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	if len(raw) == 0 {
		return domain.NumberSpec{}, nil
	}
	var spec domain.NumberSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func parseBoolSpec(json.RawMessage) (domain.PropertySpec, error) {
	return domain.BoolSpec{}, nil
}

func parseDateSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	if len(raw) == 0 {
		return domain.DateSpec{}, nil
	}
	var spec domain.DateSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func parseFileSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	if len(raw) == 0 {
		return domain.FileSpec{}, nil
	}
	var spec domain.FileSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

// toDomain converts the DTO into a domain.PropertyBank.
func (dto propertyBankDTO) toDomain(path string) (domain.PropertyBank, error) {
	properties := make(map[string]domain.Property, len(dto.Properties))

	for id, raw := range dto.Properties {
		// PropertyBank entries are always full definitions, never refs
		prop, err := parsePropertyDef(id, raw, path, "property_bank")
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
		prop, err := parsePropertyDef(name, raw, path, dto.Name)
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
	ctx := context.Background() // TODO: Pass proper context from caller
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

// parsePropertyDef parses a full Property definition from JSON.
func decodePropertyDefinition(
	raw json.RawMessage,
) (typeName string, required, array bool, specRaw json.RawMessage, err error) {
	var header struct {
		Type     string `json:"type"`
		Required bool   `json:"required"`
		Array    bool   `json:"array"`
	}
	if headerErr := json.Unmarshal(raw, &header); headerErr != nil {
		return "", false, false, nil, headerErr
	}
	if header.Type == "" {
		return "", false, false, nil, fmt.Errorf(
			"type field is required for all properties",
		)
	}

	var fields map[string]json.RawMessage
	if fieldsErr := json.Unmarshal(raw, &fields); fieldsErr != nil {
		return "", false, false, nil, fieldsErr
	}
	delete(fields, "type")
	delete(fields, "required")
	delete(fields, "array")
	// Remove fields that shouldn't be in spec
	delete(fields, "name") // name comes from the map key
	delete(fields, "id")   // id is generated by domain

	specRaw, err = json.Marshal(fields)
	if err != nil {
		return "", false, false, nil, err
	}

	typeName = strings.ToLower(header.Type)
	required = header.Required
	array = header.Array
	return typeName, required, array, specRaw, nil
}

func parsePropertyDef(
	name string,
	raw json.RawMessage,
	path string,
	owner string,
) (domain.Property, error) {
	typeName, required, array, specRaw, decodeErr := decodePropertyDefinition(
		raw,
	)
	if decodeErr != nil {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("failed to parse property %q", name),
			owner,
			path,
			decodeErr,
		)
	}

	parser, ok := propertySpecParsers[typeName]
	if !ok {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("unsupported property type %q", typeName),
			owner,
			path,
			fmt.Errorf("no parser registered"),
		)
	}

	spec, err := parser(specRaw)
	if err != nil {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("invalid spec for property %q", name),
			owner,
			path,
			err,
		)
	}

	// Use domain.NewProperty to get proper ID generation and validation
	propertyPtr, err := domain.NewProperty(name, required, array, spec)
	if err != nil {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("invalid property definition for %q", name),
			owner,
			path,
			err,
		)
	}

	return *propertyPtr, nil
}

// propertyDefinitionError constructs a SchemaError with a consistent
// remediation hint for malformed property data.
func propertyDefinitionError(
	message, schemaName, path string,
	cause error,
) error {
	return domainerrors.NewSchemaErrorWithRemediation(
		message,
		schemaName,
		syntaxRemediation(path),
		cause,
	)
}
