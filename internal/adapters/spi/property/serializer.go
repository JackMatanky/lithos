// Package property provides adapters for Property serialization.
// This adapter handles JSON marshaling/unmarshaling of Property instances,
// keeping serialization concerns out of the domain layer per hexagonal
// architecture.
package property

import (
	"encoding/json"
	"fmt"

	"github.com/JackMatanky/lithos/internal/domain"
)

// PropertySerializer handles JSON serialization of Property instances.
// This adapter implements the infrastructure concern of JSON
// marshaling/unmarshaling,
// allowing the domain Property model to remain pure business logic.
type PropertySerializer struct{}

// NewPropertySerializer creates a new PropertySerializer instance.
func NewPropertySerializer() *PropertySerializer {
	return &PropertySerializer{}
}

// MarshalJSON implements custom JSON marshaling for Property.
// Ensures the type discriminator is included in the output.
func (s *PropertySerializer) MarshalJSON(p domain.Property) ([]byte, error) {
	payload, err := s.propertyJSONPayload(p)
	if err != nil {
		return nil, err
	}

	return json.Marshal(payload)
}

// UnmarshalJSON implements custom JSON unmarshaling for Property.
// Uses discriminator-based unmarshaling to create the appropriate PropertySpec.
func (s *PropertySerializer) UnmarshalJSON(
	data []byte,
) (domain.Property, error) {
	pj, err := s.parsePropertyJSON(data)
	if err != nil {
		return domain.Property{}, err
	}

	var p domain.Property
	s.setPropertyFields(&p, pj)

	spec, err := s.buildSpecFromJSON(pj.Type, data)
	if err != nil {
		return domain.Property{}, err
	}

	p.Spec = spec

	if err := p.Validate(); err != nil {
		return domain.Property{}, err
	}

	return p, nil
}

// propertyJSON is a temporary struct for JSON unmarshaling of Property.
type propertyJSON struct {
	Name     string `json:"name"`
	Required bool   `json:"required"`
	Array    bool   `json:"array"`
	Type     string `json:"type"`
}

// parsePropertyJSON unmarshals the JSON data into a temporary propertyJSON
// struct.
func (s *PropertySerializer) parsePropertyJSON(
	data []byte,
) (*propertyJSON, error) {
	var pj propertyJSON
	if err := json.Unmarshal(data, &pj); err != nil {
		return nil, err
	}

	return &pj, nil
}

// setPropertyFields sets the common fields from propertyJSON to Property.
func (s *PropertySerializer) setPropertyFields(
	p *domain.Property,
	pj *propertyJSON,
) {
	p.Name = pj.Name
	p.Required = pj.Required
	p.Array = pj.Array
	// Type is derived from Spec, not stored in struct
}

// propertySpecDetails determines the JSON type discriminator and any
// spec-specific fields that need to be serialized.
func (s *PropertySerializer) propertySpecDetails(
	spec domain.PropertySpec,
) (typeName string, extraFields map[string]interface{}, err error) {
	normalized, normalizeErr := s.normalizeSpec(spec)
	if normalizeErr != nil {
		err = normalizeErr
		return
	}

	typeName, err = s.propertyTypeName(normalized)
	if err != nil {
		return
	}

	extraFields, err = s.specExtraFields(typeName, normalized)
	return
}

func (s *PropertySerializer) stringSpecFields(
	spec domain.StringPropertySpec,
) map[string]interface{} {
	var fields map[string]interface{}
	if len(spec.Enum) > 0 {
		fields = s.ensureFieldsMap(fields)
		fields["enum"] = spec.Enum
	}
	if spec.Pattern != "" {
		fields = s.ensureFieldsMap(fields)
		fields["pattern"] = spec.Pattern
	}

	return fields
}

func (s *PropertySerializer) numberSpecFields(
	spec domain.NumberPropertySpec,
) map[string]interface{} {
	var fields map[string]interface{}

	if spec.Min != nil {
		fields = s.ensureFieldsMap(fields)
		fields["min"] = *spec.Min
	}
	if spec.Max != nil {
		fields = s.ensureFieldsMap(fields)
		fields["max"] = *spec.Max
	}
	if spec.Step != nil {
		fields = s.ensureFieldsMap(fields)
		fields["step"] = *spec.Step
	}

	return fields
}

func (s *PropertySerializer) dateSpecFields(
	spec domain.DatePropertySpec,
) map[string]interface{} {
	if spec.Format == "" {
		return nil
	}

	fields := make(map[string]interface{}, 1)
	fields["format"] = spec.Format
	return fields
}

func (s *PropertySerializer) fileSpecFields(
	spec domain.FilePropertySpec,
) map[string]interface{} {
	var fields map[string]interface{}

	if spec.FileClass != "" {
		fields = s.ensureFieldsMap(fields)
		fields["fileClass"] = spec.FileClass
	}
	if spec.Directory != "" {
		fields = s.ensureFieldsMap(fields)
		fields["directory"] = spec.Directory
	}

	return fields
}

func (s *PropertySerializer) ensureFieldsMap(
	fields map[string]interface{},
) map[string]interface{} {
	if fields == nil {
		return make(map[string]interface{})
	}
	return fields
}

func (s *PropertySerializer) propertyJSONPayload(
	p domain.Property,
) (map[string]interface{}, error) {
	typeStr, extraFields, err := s.propertySpecDetails(p.Spec)
	if err != nil {
		return nil, err
	}

	result := map[string]interface{}{
		"name":     p.Name,
		"required": p.Required,
		"array":    p.Array,
		"type":     typeStr,
	}

	for key, value := range extraFields {
		result[key] = value
	}

	return result, nil
}

func (s *PropertySerializer) decodeStringSpec(
	data []byte,
) (domain.PropertySpec, error) {
	var spec domain.StringPropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func (s *PropertySerializer) decodeNumberSpec(
	data []byte,
) (domain.PropertySpec, error) {
	var spec domain.NumberPropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func (s *PropertySerializer) decodeDateSpec(
	data []byte,
) (domain.PropertySpec, error) {
	var spec domain.DatePropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func (s *PropertySerializer) decodeFileSpec(
	data []byte,
) (domain.PropertySpec, error) {
	var spec domain.FilePropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func (s *PropertySerializer) decodeBoolSpec(
	_ []byte,
) (domain.PropertySpec, error) {
	return domain.BoolPropertySpec{}, nil
}

func (s *PropertySerializer) determinePropertyType(
	spec domain.PropertySpec,
) string {
	normalized, err := s.normalizeSpec(spec)
	if err != nil {
		return ""
	}

	typeName, err := s.propertyTypeName(normalized)
	if err != nil {
		return ""
	}

	return typeName
}

func (s *PropertySerializer) buildSpecFromJSON(
	typeName string,
	data []byte,
) (domain.PropertySpec, error) {
	decoder, ok := decodePropertySpec[typeName]
	if !ok {
		return nil, fmt.Errorf("unknown property type: %s", typeName)
	}

	return decoder(data)
}

func (s *PropertySerializer) normalizeSpec(
	spec domain.PropertySpec,
) (domain.PropertySpec, error) {
	if spec == nil {
		return nil, fmt.Errorf("property spec cannot be nil")
	}

	if deref, handled, err := s.dereferencedSpec(spec); handled {
		return deref, err
	}

	return spec, nil
}

func (s *PropertySerializer) dereferencedSpec(
	spec domain.PropertySpec,
) (domain.PropertySpec, bool, error) {
	switch typed := spec.(type) {
	case *domain.StringPropertySpec:
		return s.dereferenceStringSpec(typed)
	case *domain.NumberPropertySpec:
		return s.dereferenceNumberSpec(typed)
	case *domain.DatePropertySpec:
		return s.dereferenceDateSpec(typed)
	case *domain.FilePropertySpec:
		return s.dereferenceFileSpec(typed)
	case *domain.BoolPropertySpec:
		return s.dereferenceBoolSpec(typed)
	default:
		return nil, false, nil
	}
}

func (s *PropertySerializer) dereferenceStringSpec(
	spec *domain.StringPropertySpec,
) (domain.PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("string property spec cannot be nil")
	}
	return *spec, true, nil
}

func (s *PropertySerializer) dereferenceNumberSpec(
	spec *domain.NumberPropertySpec,
) (domain.PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("number property spec cannot be nil")
	}
	return *spec, true, nil
}

func (s *PropertySerializer) dereferenceDateSpec(
	spec *domain.DatePropertySpec,
) (domain.PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("date property spec cannot be nil")
	}
	return *spec, true, nil
}

func (s *PropertySerializer) dereferenceFileSpec(
	spec *domain.FilePropertySpec,
) (domain.PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("file property spec cannot be nil")
	}
	return *spec, true, nil
}

func (s *PropertySerializer) dereferenceBoolSpec(
	spec *domain.BoolPropertySpec,
) (domain.PropertySpec, bool, error) {
	if spec == nil {
		return nil, true, fmt.Errorf("bool property spec cannot be nil")
	}
	return *spec, true, nil
}

func (s *PropertySerializer) propertyTypeName(
	spec domain.PropertySpec,
) (string, error) {
	switch spec.(type) {
	case domain.StringPropertySpec:
		return propertyTypeString, nil
	case domain.NumberPropertySpec:
		return propertyTypeNumber, nil
	case domain.DatePropertySpec:
		return propertyTypeDate, nil
	case domain.FilePropertySpec:
		return propertyTypeFile, nil
	case domain.BoolPropertySpec:
		return propertyTypeBool, nil
	default:
		return "", fmt.Errorf("unknown property spec type: %T", spec)
	}
}

func (s *PropertySerializer) specExtraFields(
	typeName string,
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	extractor, ok := extractPropertyFields[typeName]
	if !ok {
		return nil, fmt.Errorf("unknown property type: %s", typeName)
	}

	return extractor(spec)
}

const (
	propertyTypeString = "string"
	propertyTypeNumber = "number"
	propertyTypeDate   = "date"
	propertyTypeFile   = "file"
	propertyTypeBool   = "bool"
)

var decodePropertySpec = map[string]func([]byte) (domain.PropertySpec, error){
	propertyTypeString: decodeStringSpec,
	propertyTypeNumber: decodeNumberSpec,
	propertyTypeDate:   decodeDateSpec,
	propertyTypeFile:   decodeFileSpec,
	propertyTypeBool:   decodeBoolSpec,
}

var extractPropertyFields = map[string]func(domain.PropertySpec) (map[string]interface{}, error){
	propertyTypeString: extractStringSpecFields,
	propertyTypeNumber: extractNumberSpecFields,
	propertyTypeDate:   extractDateSpecFields,
	propertyTypeFile:   extractFileSpecFields,
	propertyTypeBool:   extractBoolSpecFields,
}

// Helper functions for the map (need to be standalone for the map)
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

func extractStringSpecFields(
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	// This needs access to the serializer methods, but since it's in the map,
	// we need to make it work. For now, inline the logic.
	payload, ok := spec.(domain.StringPropertySpec)
	if !ok {
		return nil, fmt.Errorf("spec type mismatch for %s", propertyTypeString)
	}
	var fields map[string]interface{}
	if len(payload.Enum) > 0 {
		if fields == nil {
			fields = make(map[string]interface{})
		}
		fields["enum"] = payload.Enum
	}
	if payload.Pattern != "" {
		if fields == nil {
			fields = make(map[string]interface{})
		}
		fields["pattern"] = payload.Pattern
	}
	return fields, nil
}

func extractNumberSpecFields(
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	payload, ok := spec.(domain.NumberPropertySpec)
	if !ok {
		return nil, fmt.Errorf("spec type mismatch for %s", propertyTypeNumber)
	}
	var fields map[string]interface{}
	if payload.Min != nil {
		if fields == nil {
			fields = make(map[string]interface{})
		}
		fields["min"] = *payload.Min
	}
	if payload.Max != nil {
		if fields == nil {
			fields = make(map[string]interface{})
		}
		fields["max"] = *payload.Max
	}
	if payload.Step != nil {
		if fields == nil {
			fields = make(map[string]interface{})
		}
		fields["step"] = *payload.Step
	}
	return fields, nil
}

func extractDateSpecFields(
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	payload, ok := spec.(domain.DatePropertySpec)
	if !ok {
		return nil, fmt.Errorf("spec type mismatch for %s", propertyTypeDate)
	}
	if payload.Format == "" {
		return nil, nil
	}
	fields := make(map[string]interface{}, 1)
	fields["format"] = payload.Format
	return fields, nil
}

func extractFileSpecFields(
	spec domain.PropertySpec,
) (map[string]interface{}, error) {
	payload, ok := spec.(domain.FilePropertySpec)
	if !ok {
		return nil, fmt.Errorf("spec type mismatch for %s", propertyTypeFile)
	}
	var fields map[string]interface{}
	if payload.FileClass != "" {
		if fields == nil {
			fields = make(map[string]interface{})
		}
		fields["fileClass"] = payload.FileClass
	}
	if payload.Directory != "" {
		if fields == nil {
			fields = make(map[string]interface{})
		}
		fields["directory"] = payload.Directory
	}
	return fields, nil
}

func extractBoolSpecFields(
	domain.PropertySpec,
) (map[string]interface{}, error) {
	return map[string]interface{}{}, nil
}
