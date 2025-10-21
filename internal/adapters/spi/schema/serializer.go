// Package schema provides helpers for JSON serialization of domain Property
// definitions. Keeping this logic in the adapter layer maintains the domain's
// independence from encoding concerns.
package schema

import (
	"encoding/json"
	"fmt"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// ----------------------------------------------------------
//                    Property Type Constants
// ----------------------------------------------------------

// Property type constants for JSON serialization/deserialization
// These mirror the domain constants but are used for adapter-specific JSON
// processing. Duplication is acceptable here as these are adapter concerns.
const (
	propertyTypeString = "string"
	propertyTypeNumber = "number"
	propertyTypeDate   = "date"
	propertyTypeFile   = "file"
	propertyTypeBool   = "bool"
)

// ----------------------------------------------------------
//                   Property JSON Structures
// ----------------------------------------------------------

// ----------------------------------------------------------
//                      Marshal Functions
// ----------------------------------------------------------

// ----------------------------------------------------------
//                     Unmarshal Functions
// ----------------------------------------------------------

// MarshalProperty serializes a domain Property to JSON data.
func MarshalProperty(property domain.Property) ([]byte, error) {
	marshaler := newPropertyMarshaler(property)
	payload, err := marshaler.payload()
	if err != nil {
		return nil, err
	}
	return json.Marshal(payload)
}

// UnmarshalProperty constructs a domain Property from JSON data.
func UnmarshalProperty(data []byte) (domain.Property, error) {
	return newPropertyUnmarshaller(data).unmarshal()
}

type propertyMarshaler struct {
	property domain.Property
}

func newPropertyMarshaler(p domain.Property) propertyMarshaler {
	return propertyMarshaler{property: p}
}

func (m propertyMarshaler) payload() (map[string]interface{}, error) {
	typeName, err := m.property.TypeName()
	if err != nil {
		return nil, err
	}

	payload := m.basePayload(typeName)
	if appendErr := m.appendSpecFields(payload, typeName); appendErr != nil {
		return nil, appendErr
	}

	return payload, nil
}

func (m propertyMarshaler) basePayload(typeName string) map[string]interface{} {
	return map[string]interface{}{
		"name":     m.property.Name,
		"required": m.property.Required,
		"array":    m.property.Array,
		"type":     typeName,
	}
}

func (m propertyMarshaler) appendSpecFields(
	payload map[string]interface{},
	typeName string,
) error {
	fields, err := specExtraFields(typeName, m.property.Spec)
	if err != nil {
		return err
	}

	mergeFields(payload, fields)
	return nil
}

type propertyUnmarshaller struct {
	raw []byte
}

func newPropertyUnmarshaller(raw []byte) propertyUnmarshaller {
	return propertyUnmarshaller{raw: raw}
}

func (u propertyUnmarshaller) unmarshal() (domain.Property, error) {
	pd, err := u.decodeJSON()
	if err != nil {
		return domain.Property{}, err
	}

	// Use type from the dedicated Type field
	typeStr := pd.Type
	if typeStr == "" {
		return domain.Property{}, fmt.Errorf(
			"property spec missing 'type' field",
		)
	}

	// Use Spec map directly for spec fields (type already extracted)
	spec, err := u.decodeSpec(typeStr, pd.Spec)
	if err != nil {
		return domain.Property{}, err
	}

	property := buildProperty(pd, spec)
	return validateProperty(property)
}

func (u propertyUnmarshaller) decodeJSON() (*propertyDTO, error) {
	var pd propertyDTO
	if err := json.Unmarshal(u.raw, &pd); err != nil {
		return nil, err
	}
	return &pd, nil
}

func (u propertyUnmarshaller) decodeSpec(
	typeName string,
	specMap map[string]interface{},
) (domain.PropertySpec, error) {
	return buildSpecFromMap(typeName, specMap)
}

func buildProperty(
	pd *propertyDTO,
	spec domain.PropertySpec,
) domain.Property {
	return domain.Property{
		Name:     pd.Name,
		Required: pd.Required,
		Array:    pd.Array,
		Spec:     spec,
	}
}

func validateProperty(property domain.Property) (domain.Property, error) {
	if err := property.Validate(); err != nil {
		return domain.Property{}, err
	}
	return property, nil
}

func mergeFields(
	payload map[string]interface{},
	fields map[string]interface{},
) {
	for key, value := range fields {
		payload[key] = value
	}
}

// ----------------------------------------------------------
//                 Spec Field Extraction Functions
// ----------------------------------------------------------

type specExtractor func(domain.PropertySpec) (map[string]interface{}, error)

var propertySpecExtractors = map[string]specExtractor{
	propertyTypeString: extractStringSpecFields,
	propertyTypeNumber: extractNumberSpecFields,
	propertyTypeDate:   extractDateSpecFields,
	propertyTypeFile:   extractFileSpecFields,
	propertyTypeBool:   extractBoolSpecFields,
}

func extractStringSpecFields(
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	payload, ok := materializeStringSpec(spec)
	if !ok {
		return nil, fmt.Errorf(
			"spec type mismatch for %s",
			propertyTypeString,
		)
	}
	return stringSpecFields(payload), nil
}

func extractNumberSpecFields(
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	payload, ok := materializeNumberSpec(spec)
	if !ok {
		return nil, fmt.Errorf(
			"spec type mismatch for %s",
			propertyTypeNumber,
		)
	}
	return numberSpecFields(payload), nil
}

func extractDateSpecFields(
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	payload, ok := materializeDateSpec(spec)
	if !ok {
		return nil, fmt.Errorf(
			"spec type mismatch for %s",
			propertyTypeDate,
		)
	}
	return dateSpecFields(payload), nil
}

func extractFileSpecFields(
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	payload, ok := materializeFileSpec(spec)
	if !ok {
		return nil, fmt.Errorf(
			"spec type mismatch for %s",
			propertyTypeFile,
		)
	}
	return fileSpecFields(payload), nil
}

func extractBoolSpecFields(
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	if _, ok := materializeBoolSpec(spec); !ok {
		return nil, fmt.Errorf(
			"spec type mismatch for %s",
			propertyTypeBool,
		)
	}
	return map[string]interface{}{}, nil
}

func specExtraFields(
	typeName string,
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	extractor, ok := propertySpecExtractors[typeName]
	if !ok {
		return nil, fmt.Errorf("unknown property type: %s", typeName)
	}

	return extractor(spec)
}

// ----------------------------------------------------------
//                    Spec Field Builders
// ----------------------------------------------------------

func stringSpecFields(spec domain.StringPropertySpec) map[string]interface{} {
	fields := make(map[string]interface{})
	if len(spec.Enum) > 0 {
		fields["enum"] = spec.Enum
	}
	if spec.Pattern != "" {
		fields["pattern"] = spec.Pattern
	}
	return fields
}

func numberSpecFields(spec domain.NumberPropertySpec) map[string]interface{} {
	fields := make(map[string]interface{})
	if spec.Min != nil {
		fields["min"] = *spec.Min
	}
	if spec.Max != nil {
		fields["max"] = *spec.Max
	}
	if spec.Step != nil {
		fields["step"] = *spec.Step
	}
	return fields
}

func dateSpecFields(spec domain.DatePropertySpec) map[string]interface{} {
	if spec.Format == "" {
		return map[string]interface{}{}
	}
	return map[string]interface{}{"format": spec.Format}
}

func fileSpecFields(spec domain.FilePropertySpec) map[string]interface{} {
	fields := make(map[string]interface{})
	if spec.FileClass != "" {
		fields["fileClass"] = spec.FileClass
	}
	if spec.Directory != "" {
		fields["directory"] = spec.Directory
	}
	return fields
}

// ----------------------------------------------------------
//                   Property Spec Decoders
// ----------------------------------------------------------

// buildSpecFromMap builds a PropertySpec from a map (already unmarshaled JSON).
func buildSpecFromMap(
	typeName string,
	specMap map[string]interface{},
) (domain.PropertySpec, error) {
	builder, ok := buildPropertySpec[typeName]
	if !ok {
		return nil, errors.NewValidationError(
			"type",
			fmt.Sprintf("unknown property type: %s", typeName),
			typeName,
		)
	}

	return builder(specMap)
}

var buildPropertySpec = map[string]func(map[string]interface{}) (domain.PropertySpec, error){
	propertyTypeString: buildStringSpec,
	propertyTypeNumber: buildNumberSpec,
	propertyTypeDate:   buildDateSpec,
	propertyTypeFile:   buildFileSpec,
	propertyTypeBool:   buildBoolSpec,
}

//nolint:unparam // Function always returns nil error by design
func buildStringSpec(spec map[string]interface{}) (domain.PropertySpec, error) {
	result := domain.StringPropertySpec{
		Enum:    []string{},
		Pattern: "",
	}

	if pattern, ok := spec["pattern"].(string); ok {
		result.Pattern = pattern
	}

	if enumInterface, hasEnum := spec["enum"]; hasEnum {
		result.Enum = extractEnumValues(enumInterface)
	}

	return result, nil
}

//nolint:unparam // Function always returns nil error by design
func buildNumberSpec(spec map[string]interface{}) (domain.PropertySpec, error) {
	result := domain.NumberPropertySpec{
		Min:  nil,
		Max:  nil,
		Step: nil,
	}

	if minVal, ok := spec["min"].(float64); ok {
		result.Min = &minVal
	}
	if maxVal, ok := spec["max"].(float64); ok {
		result.Max = &maxVal
	}
	if step, ok := spec["step"].(float64); ok {
		result.Step = &step
	}

	return result, nil
}

//nolint:unparam // Function always returns nil error by design
func buildDateSpec(spec map[string]interface{}) (domain.PropertySpec, error) {
	result := domain.DatePropertySpec{
		Format: "",
	}

	if format, ok := spec["format"].(string); ok {
		result.Format = format
	}

	return result, nil
}

//nolint:unparam // Function always returns nil error by design
func buildFileSpec(spec map[string]interface{}) (domain.PropertySpec, error) {
	result := domain.FilePropertySpec{
		FileClass: "",
		Directory: "",
	}

	if fileClass, ok := spec["fileClass"].(string); ok {
		result.FileClass = fileClass
	}
	if directory, ok := spec["directory"].(string); ok {
		result.Directory = directory
	}

	return result, nil
}

func buildBoolSpec(_ map[string]interface{}) (domain.PropertySpec, error) {
	return domain.BoolPropertySpec{}, nil
}

// extractEnumValues extracts string values from an enum interface.
func extractEnumValues(enumInterface interface{}) []string {
	var enum []string

	if enumSlice, isSlice := enumInterface.([]interface{}); isSlice {
		for _, item := range enumSlice {
			if strItem, isString := item.(string); isString {
				enum = append(enum, strItem)
			}
		}
	}

	return enum
}

// ----------------------------------------------------------
//                     Materialize Functions
// ----------------------------------------------------------

// materialize*Spec functions provide type-safe conversion from the PropertySpec
// interface to concrete types. While these functions appear duplicated, the
// duplication is necessary because each returns a different concrete type,
// enabling compile-time type safety. Attempts to use generics here would
// either violate govet rules or require unsafe type assertions.

func materializeStringSpec(
	spec domain.PropertySpec,
) (domain.StringPropertySpec, bool) {
	switch typed := spec.(type) {
	case domain.StringPropertySpec:
		return typed, true
	case *domain.StringPropertySpec:
		if typed == nil {
			return domain.StringPropertySpec{}, false
		}
		return *typed, true
	default:
		return domain.StringPropertySpec{}, false
	}
}

func materializeNumberSpec(
	spec domain.PropertySpec,
) (domain.NumberPropertySpec, bool) {
	switch typed := spec.(type) {
	case domain.NumberPropertySpec:
		return typed, true
	case *domain.NumberPropertySpec:
		if typed == nil {
			return domain.NumberPropertySpec{}, false
		}
		return *typed, true
	default:
		return domain.NumberPropertySpec{}, false
	}
}

func materializeDateSpec(
	spec domain.PropertySpec,
) (domain.DatePropertySpec, bool) {
	switch typed := spec.(type) {
	case domain.DatePropertySpec:
		return typed, true
	case *domain.DatePropertySpec:
		if typed == nil {
			return domain.DatePropertySpec{}, false
		}
		return *typed, true
	default:
		return domain.DatePropertySpec{}, false
	}
}

func materializeFileSpec(
	spec domain.PropertySpec,
) (domain.FilePropertySpec, bool) {
	switch typed := spec.(type) {
	case domain.FilePropertySpec:
		return typed, true
	case *domain.FilePropertySpec:
		if typed == nil {
			return domain.FilePropertySpec{}, false
		}
		return *typed, true
	default:
		return domain.FilePropertySpec{}, false
	}
}

func materializeBoolSpec(
	spec domain.PropertySpec,
) (domain.BoolPropertySpec, bool) {
	switch typed := spec.(type) {
	case domain.BoolPropertySpec:
		return typed, true
	case *domain.BoolPropertySpec:
		if typed == nil {
			return domain.BoolPropertySpec{}, false
		}
		return *typed, true
	default:
		return domain.BoolPropertySpec{}, false
	}
}
