// Package property provides helpers for JSON serialization of domain Property
// definitions. Keeping this logic in the adapter layer maintains the domain's
// independence from encoding concerns.
package property

import (
	"encoding/json"
	"fmt"

	"github.com/JackMatanky/lithos/internal/domain"
)

const (
	propertyTypeString = "string"
	propertyTypeNumber = "number"
	propertyTypeDate   = "date"
	propertyTypeFile   = "file"
	propertyTypeBool   = "bool"
)

type propertyJSON struct {
	Name     string `json:"name"`
	Required bool   `json:"required"`
	Array    bool   `json:"array"`
	Type     string `json:"type"`
}

// MarshalProperty converts a domain Property into its JSON representation.
func MarshalProperty(p domain.Property) ([]byte, error) {
	payload, err := newPropertyMarshaler(p).payload()
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
	pj, err := u.decodeJSON()
	if err != nil {
		return domain.Property{}, err
	}

	spec, err := u.decodeSpec(pj.Type)
	if err != nil {
		return domain.Property{}, err
	}

	property := buildProperty(pj, spec)
	return validateProperty(property)
}

func (u propertyUnmarshaller) decodeJSON() (*propertyJSON, error) {
	var pj propertyJSON
	if err := json.Unmarshal(u.raw, &pj); err != nil {
		return nil, err
	}
	return &pj, nil
}

func (u propertyUnmarshaller) decodeSpec(
	typeName string,
) (domain.PropertySpec, error) {
	return buildSpecFromJSON(typeName, u.raw)
}

func buildProperty(
	pj *propertyJSON,
	spec domain.PropertySpec,
) domain.Property {
	return domain.Property{
		Name:     pj.Name,
		Required: pj.Required,
		Array:    pj.Array,
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

func buildSpecFromJSON(
	typeName string,
	data []byte,
) (domain.PropertySpec, error) {
	decoder, ok := decodePropertySpec[typeName]
	if !ok {
		return nil, fmt.Errorf("unknown property type: %s", typeName)
	}

	return decoder(data)
}

var decodePropertySpec = map[string]func([]byte) (domain.PropertySpec, error){
	propertyTypeString: decodeStringSpec,
	propertyTypeNumber: decodeNumberSpec,
	propertyTypeDate:   decodeDateSpec,
	propertyTypeFile:   decodeFileSpec,
	propertyTypeBool:   decodeBoolSpec,
}

func decodeStringSpec(data []byte) (domain.PropertySpec, error) {
	var spec domain.StringPropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func decodeNumberSpec(data []byte) (domain.PropertySpec, error) {
	var spec domain.NumberPropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func decodeDateSpec(data []byte) (domain.PropertySpec, error) {
	var spec domain.DatePropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func decodeFileSpec(data []byte) (domain.PropertySpec, error) {
	var spec domain.FilePropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func decodeBoolSpec(_ []byte) (domain.PropertySpec, error) {
	return domain.BoolPropertySpec{}, nil
}

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
