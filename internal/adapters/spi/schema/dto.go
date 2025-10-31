package schema

import (
	"encoding/json"
	"fmt"
	"sort"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// propertySpecDTOFactory maps property type strings to their concrete spec DTO
// constructors, enabling polymorphic decoding based on type.
var propertySpecDTOFactory = map[string]func() propertySpecDTO{
	"string": func() propertySpecDTO { return &stringSpecDTO{} },
	"number": func() propertySpecDTO { return &numberSpecDTO{} },
	"bool":   func() propertySpecDTO { return &boolSpecDTO{} },
	"date":   func() propertySpecDTO { return &dateSpecDTO{} },
	"file":   func() propertySpecDTO { return &fileSpecDTO{} },
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

// propertyEntry captures the results of decoding a single named property or
// property reference.
type propertyEntry struct {
	key   string
	value domain.IProperty
}

// propertyDTO captures the per-property definition fields excluding
// references; it is paired with propertyRefDTO to keep responsibilities small.
// The Type field is parsed as a string and mapped to domain.PropertyType.
// Extra captures all type-specific fields (pattern, min, max, format, etc.)
// that will be parsed into the appropriate PropertySpec.
type propertyDTO struct {
	Type     string                 `json:"type"`
	Required bool                   `json:"required"`
	Array    bool                   `json:"array"`
	Extra    map[string]interface{} `json:"-"` // Type-specific fields
}

// propertyRefDTO carries the optional `$ref` pointer for a property.
type propertyRefDTO struct {
	Ref string `json:"$ref"`
}

// propertySpecDTO defines the interface for type-specific property spec DTOs.
// Each concrete DTO implements:
// - parse(): unmarshals from propertyDTO.Extra map
// - toDomain(): converts to domain.PropertySpec.
type propertySpecDTO interface {
	parse(extra map[string]interface{}) error
	toDomain() (domain.PropertySpec, error)
}

// stringSpecDTO captures string-specific validation fields.
type stringSpecDTO struct {
	Enum    []string `json:"enum,omitempty"`
	Pattern string   `json:"pattern,omitempty"`
}

// numberSpecDTO captures numeric validation fields.
type numberSpecDTO struct {
	Min  *float64 `json:"min,omitempty"`
	Max  *float64 `json:"max,omitempty"`
	Step *float64 `json:"step,omitempty"`
}

// boolSpecDTO captures boolean spec (no fields).
type boolSpecDTO struct{}

// dateSpecDTO captures date-specific validation fields.
type dateSpecDTO struct {
	Format string `json:"format,omitempty"`
}

// fileSpecDTO captures file reference validation fields.
type fileSpecDTO struct {
	FileClass string `json:"file_class,omitempty"`
	Directory string `json:"directory,omitempty"`
}

// toDomain converts the propertyDTO to a domain.Property by parsing the
// property spec using the Type field and Extra fields.
// The name parameter comes from the map key in the properties object.
func (d propertyDTO) toDomain(name string) (domain.Property, error) {
	// Get the appropriate spec DTO factory for this type
	factory, ok := propertySpecDTOFactory[d.Type]
	if !ok {
		return domain.Property{}, fmt.Errorf(
			"unsupported property type %q",
			d.Type,
		)
	}

	// Create spec DTO and parse extra fields
	specDTO := factory()
	if err := specDTO.parse(d.Extra); err != nil {
		return domain.Property{}, fmt.Errorf(
			"failed to parse %s spec: %w",
			d.Type,
			err,
		)
	}

	// Convert spec DTO to domain PropertySpec
	spec, err := specDTO.toDomain()
	if err != nil {
		return domain.Property{}, fmt.Errorf(
			"failed to convert %s spec to domain: %w",
			d.Type,
			err,
		)
	}

	return domain.Property{
		Name:     name,
		Required: d.Required,
		Array:    d.Array,
		Spec:     spec,
	}, nil
}

// Normalize returns the canonical property identifier by stripping JSON Pointer
// prefixes from the $ref value.
//
// Schemas may encode $ref using either raw identifiers (e.g., "standard_title")
// or JSON Pointer paths (e.g., "#/properties/standard_title"). This method
// normalizes both formats to the raw identifier for consistent lookups.
//
// Examples:
//   - "#/properties/standard_title" → "standard_title"
//   - "#/definitions/foo" → "foo"
//   - "standard_title" → "standard_title" (unchanged)
//
// This is an adapter-layer concern because JSON Pointer syntax is a wire format
// detail. The domain layer should only see normalized identifiers.
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

// toDomain converts the propertyRefDTO to a domain.PropertyRef.
// The name parameter comes from the map key in the properties object.
func (r propertyRefDTO) toDomain(name string) domain.PropertyRef {
	return domain.PropertyRef{
		Name: name,
		Ref:  r.Normalize(),
	}
}

func (d *stringSpecDTO) parse(extra map[string]interface{}) error {
	// Marshal map to JSON then unmarshal into struct
	jsonData, err := json.Marshal(extra)
	if err != nil {
		return err
	}
	return json.Unmarshal(jsonData, d)
}

func (d stringSpecDTO) toDomain() (domain.PropertySpec, error) {
	return domain.StringSpec{
		Enum:    d.Enum,
		Pattern: d.Pattern,
	}, nil
}

func (d *numberSpecDTO) parse(extra map[string]interface{}) error {
	jsonData, err := json.Marshal(extra)
	if err != nil {
		return err
	}
	return json.Unmarshal(jsonData, d)
}

func (d numberSpecDTO) toDomain() (domain.PropertySpec, error) {
	return domain.NumberSpec{
		Min:  d.Min,
		Max:  d.Max,
		Step: d.Step,
	}, nil
}

func (d *boolSpecDTO) parse(extra map[string]interface{}) error {
	// No fields to parse
	return nil
}

func (d boolSpecDTO) toDomain() (domain.PropertySpec, error) {
	return domain.BoolSpec{}, nil
}

func (d *dateSpecDTO) parse(extra map[string]interface{}) error {
	jsonData, err := json.Marshal(extra)
	if err != nil {
		return err
	}
	return json.Unmarshal(jsonData, d)
}

func (d dateSpecDTO) toDomain() (domain.PropertySpec, error) {
	return domain.DateSpec{
		Format: d.Format,
	}, nil
}

func (d *fileSpecDTO) parse(extra map[string]interface{}) error {
	jsonData, err := json.Marshal(extra)
	if err != nil {
		return err
	}
	return json.Unmarshal(jsonData, d)
}

func (d fileSpecDTO) toDomain() (domain.PropertySpec, error) {
	return domain.FileSpec{
		FileClass: d.FileClass,
		Directory: d.Directory,
	}, nil
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
func parsePropertyDef(
	name string,
	raw json.RawMessage,
	path string,
	owner string,
) (domain.Property, error) {
	// Parse property attributes (type, required, array)
	var def propertyDTO
	if err := json.Unmarshal(raw, &def); err != nil {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("failed to parse property %q", name),
			owner,
			path,
			err,
		)
	}

	// Validate that type field is present
	if def.Type == "" {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("property %q missing required 'type' field", name),
			owner,
			path,
			fmt.Errorf("type field is required for all properties"),
		)
	}

	// Unmarshal into map to extract type-specific fields
	var allFields map[string]interface{}
	if err := json.Unmarshal(raw, &allFields); err != nil {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("failed to parse property %q", name),
			owner,
			path,
			err,
		)
	}

	// Extract type-specific fields (everything except type, required, array)
	def.Extra = make(map[string]interface{})
	for k, v := range allFields {
		if k != "type" && k != "required" && k != "array" {
			def.Extra[k] = v
		}
	}

	// Use propertyDTO.toDomain() to convert to domain.Property
	prop, err := def.toDomain(name)
	if err != nil {
		return domain.Property{}, propertyDefinitionError(
			fmt.Sprintf("invalid spec for property %q", name),
			owner,
			path,
			err,
		)
	}

	return prop, nil
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
