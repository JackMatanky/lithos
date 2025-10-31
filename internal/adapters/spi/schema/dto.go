package schema

import (
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

// propertyEntry captures the results of decoding a single named property or
// property reference.
type propertyEntry struct {
	key   string
	value domain.IProperty
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

func (r propertyRefDTO) toDomain(name string) domain.PropertyRef {
	return domain.PropertyRef{
		Name: name,
		Ref:  r.Normalize(),
	}
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

// toDomain converts the DTO into a domain.PropertyBank while preserving
// detailed remediation information on failure.
func (dto propertyBankDTO) toDomain(path string) (domain.PropertyBank, error) {
	entries, err := dto.propertyEntries(path)
	if err != nil {
		return domain.PropertyBank{}, err
	}

	properties := make(map[string]domain.Property, len(entries))
	for _, entry := range entries {
		// PropertyBank should only contain full Property definitions, not refs
		if prop, ok := entry.value.(domain.Property); ok {
			properties[entry.key] = prop
		}
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
// PropertyBank should only contain full Property definitions, never
// PropertyRefs.
func (dto propertyBankDTO) propertyEntries(
	path string,
) ([]propertyEntry, error) {
	entries := make([]propertyEntry, 0, len(dto.Properties))
	for id, raw := range dto.Properties {
		// PropertyBank entries are always full definitions, never refs
		prop, err := parsePropertyDef(id, raw, path, "property_bank")
		if err != nil {
			return nil, err
		}
		entries = append(entries, propertyEntry{key: id, value: prop})
	}
	return entries, nil
}

// schemaDTO.propertyEntries decodes the schema DTO's properties into a slice of
// entries, preserving their names for later sorting.
// Schemas can contain either Property or PropertyRef.
func (dto schemaDTO) propertyEntries(path string) ([]propertyEntry, error) {
	entries := make([]propertyEntry, 0, len(dto.Properties))
	for name, raw := range dto.Properties {
		// Schemas can have either full properties or refs
		value, err := parseProperty(name, raw, path, dto.Name)
		if err != nil {
			return nil, err
		}
		entries = append(entries, propertyEntry{key: name, value: value})
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

	properties := make([]domain.IProperty, 0, len(entries))
	for _, entry := range entries {
		properties = append(properties, entry.value)
	}

	// Sort by name using IProperty interface
	sort.Slice(properties, func(i, j int) bool {
		return properties[i].GetName() < properties[j].GetName()
	})

	return domain.Schema{
		Name:               dto.Name,
		Extends:            dto.Extends,
		Excludes:           dto.Excludes,
		Properties:         properties,
		ResolvedProperties: nil,
	}, nil
}

// parsePropertyRef parses a PropertyRef from JSON.
func parsePropertyRef(
	name string,
	raw json.RawMessage,
	path string,
	owner string,
) (domain.PropertyRef, error) {
	var ref propertyRefDTO
	if err := json.Unmarshal(raw, &ref); err != nil {
		return domain.PropertyRef{}, propertyDefinitionError(
			fmt.Sprintf("failed to parse property ref %q", name),
			owner,
			path,
			err,
		)
	}

	return ref.toDomain(name), nil
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
	if err = json.Unmarshal(raw, &header); err != nil {
		return
	}
	if header.Type == "" {
		err = fmt.Errorf("type field is required for all properties")
		return
	}

	var fields map[string]json.RawMessage
	if err = json.Unmarshal(raw, &fields); err != nil {
		return
	}
	delete(fields, "type")
	delete(fields, "required")
	delete(fields, "array")

	specRaw, err = json.Marshal(fields)
	typeName = strings.ToLower(header.Type)
	required = header.Required
	array = header.Array
	return
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

	return domain.Property{
		Name:     name,
		Required: required,
		Array:    array,
		Spec:     spec,
	}, nil
}

// parseProperty parses either a PropertyRef or Property from JSON by checking
// for the presence of $ref field.
//
// The name parameter comes from the map key in the properties object.
// For example, in {"properties": {"title": {...}}}, name would be "title".
//
// Returns either domain.Property or domain.PropertyRef as domain.IProperty.
func parseProperty(
	name string,
	raw json.RawMessage,
	path string,
	owner string,
) (domain.IProperty, error) {
	// Check if this is a $ref property by looking for $ref field
	var ref propertyRefDTO
	if err := json.Unmarshal(raw, &ref); err != nil {
		return nil, propertyDefinitionError(
			fmt.Sprintf("failed to parse property %q", name),
			owner,
			path,
			err,
		)
	}

	// MVP: $ref is mutually exclusive - if present, parse as PropertyRef
	if ref.Ref != "" {
		return parsePropertyRef(name, raw, path, owner)
	}

	// Otherwise parse as full Property definition
	return parsePropertyDef(name, raw, path, owner)
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
