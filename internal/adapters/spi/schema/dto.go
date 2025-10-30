package schema

import (
	"encoding/json"
	"fmt"
	"sort"

	"github.com/JackMatanky/lithos/internal/domain"
	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// propertySpecParsers maps property types to their concrete spec parser
// functions, enabling polymorphic decoding of the `spec` field.
var propertySpecParsers = map[domain.PropertyType]propertySpecParser{
	domain.PropertyTypeString: parseStringSpec,
	domain.PropertyTypeNumber: parseNumberSpec,
	domain.PropertyTypeBool:   parseBoolSpec,
	domain.PropertyTypeDate:   parseDateSpec,
	domain.PropertyTypeFile:   parseFileSpec,
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

// propertyEntry captures the results of decoding a single named property.
type propertyEntry struct {
	key      string
	property domain.Property
}

// propertyDTO captures the per-property definition fields excluding
// references; it is paired with propertyRefDTO to keep responsibilities small.
type propertyDTO struct {
	Name     string          `json:"name"`
	Required bool            `json:"required"`
	Array    bool            `json:"array"`
	Spec     json.RawMessage `json:"spec"`
}

// propertyRefDTO carries the optional `$ref` pointer for a property.
type propertyRefDTO struct {
	Ref string `json:"$ref"`
}

// propertySpecParser converts a raw JSON blob into a concrete
// domain.PropertySpec implementation.
type propertySpecParser func(json.RawMessage) (domain.PropertySpec, error)

// toDomain converts the DTO into a domain.PropertyBank while preserving
// detailed remediation information on failure.
func (dto propertyBankDTO) toDomain(path string) (domain.PropertyBank, error) {
	entries, err := dto.propertyEntries(path)
	if err != nil {
		return domain.PropertyBank{}, err
	}

	properties := make(map[string]domain.Property, len(entries))
	for _, entry := range entries {
		properties[entry.key] = entry.property
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

// propertyBankDTO.propertyEntries decodes each property definition and returns
// the resulting domain properties keyed by property identifier.
func (dto propertyBankDTO) propertyEntries(
	path string,
) ([]propertyEntry, error) {
	entries := make([]propertyEntry, 0, len(dto.Properties))
	for id, raw := range dto.Properties {
		property, err := parsePropertyDefinition(id, raw, path, id)
		if err != nil {
			return nil, err
		}
		entries = append(entries, propertyEntry{key: id, property: property})
	}
	return entries, nil
}

// schemaDTO.propertyEntries decodes the schema DTO's properties into a slice of
// entries, preserving their names for later sorting.
func (dto schemaDTO) propertyEntries(path string) ([]propertyEntry, error) {
	entries := make([]propertyEntry, 0, len(dto.Properties))
	for name, raw := range dto.Properties {
		property, err := parsePropertyDefinition(name, raw, path, dto.Name)
		if err != nil {
			return nil, err
		}
		entries = append(entries, propertyEntry{key: name, property: property})
	}
	return entries, nil
}

// toDomain materializes the schema DTO into a domain.Schema value, keeping the
// properties sorted deterministically by name.
func (dto schemaDTO) toDomain(path string) (domain.Schema, error) {
	entries, err := dto.propertyEntries(path)
	if err != nil {
		return domain.Schema{}, err
	}

	properties := make([]domain.Property, 0, len(entries))
	for _, entry := range entries {
		properties = append(properties, entry.property)
	}

	sort.Slice(properties, func(i, j int) bool {
		return properties[i].Name < properties[j].Name
	})

	return domain.Schema{
		Name:               dto.Name,
		Extends:            dto.Extends,
		Excludes:           dto.Excludes,
		Properties:         properties,
		ResolvedProperties: nil,
	}, nil
}

// parsePropertyDefinition converts a raw JSON definition into a
// domain.Property, handling shared error messaging concerns in one place.
func parsePropertyDefinition(
	name string,
	raw json.RawMessage,
	path string,
	owner string,
) (domain.Property, error) {
	var def propertyDTO
	if err := json.Unmarshal(raw, &def); err != nil {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("failed to parse property %q", name),
			owner,
			path,
			err,
		)
	}

	var ref propertyRefDTO
	if err := json.Unmarshal(raw, &ref); err != nil {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("failed to parse property %q", name),
			owner,
			path,
			err,
		)
	}

	var property domain.Property
	property.Name = name
	property.Required = def.Required
	property.Array = def.Array
	property.Ref = ref.Ref

	if def.Name != "" {
		property.Name = def.Name
	}

	if len(def.Spec) > 0 {
		specValue, err := parsePropertySpec(def.Spec)
		if err != nil {
			return domain.Property{}, propertyDefinitionError(
				fmt.Sprintf("invalid spec for property %q", name),
				owner,
				path,
				err,
			)
		}
		property.Spec = specValue
	}

	return property, nil
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

// parsePropertySpec converts a raw spec blob into the appropriate concrete
// PropertySpec using the propertySpecParsers registry.
func parsePropertySpec(raw json.RawMessage) (domain.PropertySpec, error) {
	var meta struct {
		Type domain.PropertyType `json:"type"`
	}

	if err := json.Unmarshal(raw, &meta); err != nil {
		return nil, err
	}

	parser, ok := propertySpecParsers[meta.Type]
	if !ok {
		return nil, fmt.Errorf("unsupported property type %q", meta.Type)
	}

	return parser(raw)
}

// parseStringSpec parses a string property specification.
func parseStringSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	var spec domain.StringSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

// parseNumberSpec parses a numeric property specification.
func parseNumberSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	var spec domain.NumberSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

// parseBoolSpec parses a boolean property specification.
func parseBoolSpec(json.RawMessage) (domain.PropertySpec, error) {
	return domain.BoolSpec{}, nil
}

// parseDateSpec parses a date property specification.
func parseDateSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	var spec domain.DateSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

// parseFileSpec parses a file reference property specification.
func parseFileSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	var spec domain.FileSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}
