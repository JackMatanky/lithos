// Package domain contains the core business logic models for Lithos.
// These models represent domain concepts and contain no infrastructure
// dependencies.
package domain

import (
	"encoding/json"
	"fmt"
	"math"
	"regexp"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/shared/errors"
)

const (
	propertyTypeString = "string"
	propertyTypeNumber = "number"
	propertyTypeDate   = "date"
	propertyTypeFile   = "file"
	propertyTypeBool   = "bool"
)

type (
	specDecoder        func([]byte) (PropertySpec, error)
	specFieldExtractor func(PropertySpec) (map[string]interface{}, error)
)

var (
	decodePropertySpec = map[string]specDecoder{
		propertyTypeString: decodeStringSpec,
		propertyTypeNumber: decodeNumberSpec,
		propertyTypeDate:   decodeDateSpec,
		propertyTypeFile:   decodeFileSpec,
		propertyTypeBool:   decodeBoolSpec,
	}

	extractPropertyFields = map[string]specFieldExtractor{
		propertyTypeString: extractStringSpecFields,
		propertyTypeNumber: extractNumberSpecFields,
		propertyTypeDate:   extractDateSpecFields,
		propertyTypeFile:   extractFileSpecFields,
		propertyTypeBool:   extractBoolSpecFields,
	}
)

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
	Name string `json:"name"`

	// Required indicates whether this property must be present in note
	// frontmatter.
	Required bool `json:"required"`

	// Array indicates whether this property accepts multiple values (YAML
	// list).
	// If true, frontmatter value must be array. If false, value must be scalar.
	Array bool `json:"array"`

	// Spec contains type-specific configuration and validation rules.
	// Exactly one spec type per property based on semantic type.
	Spec PropertySpec `json:"-"`

	// Type is the discriminator field for JSON unmarshaling.
	// Must be one of: "string", "number", "date", "file", "bool"
	Type string `json:"type"`
}

// StringPropertySpec validates string values with optional enum and pattern
// constraints.
type StringPropertySpec struct {
	// Enum contains allowed values as fixed list. If non-empty, value must be
	// in list.
	// Empty list means no enum constraint (any string valid).
	Enum []string `json:"enum,omitempty"`

	// Pattern is a regex pattern for custom string validation.
	// If non-empty, value must match pattern. Uses Go regexp package.
	Pattern string `json:"pattern,omitempty"`
}

// Validate implements PropertySpec for StringPropertySpec.
func (s StringPropertySpec) Validate(value interface{}) error {
	str, ok := value.(string)
	if !ok {
		return errors.NewValidationError("value", "must be string", value)
	}

	// Check enum constraint first
	if len(s.Enum) > 0 {
		found := false
		for _, allowed := range s.Enum {
			if str == allowed {
				found = true
				break
			}
		}
		if !found {
			return errors.NewValidationError(
				"value",
				fmt.Sprintf("must be one of: %v", s.Enum),
				value,
			)
		}
	}

	// Check pattern constraint
	if s.Pattern != "" {
		matched, err := regexp.MatchString(s.Pattern, str)
		if err != nil {
			return errors.NewValidationError(
				"pattern",
				"invalid regex pattern",
				s.Pattern,
			)
		}
		if !matched {
			return errors.NewValidationError(
				"value",
				fmt.Sprintf("must match pattern: %s", s.Pattern),
				value,
			)
		}
	}

	return nil
}

// NumberPropertySpec validates numeric values with optional min/max/step
// constraints.
type NumberPropertySpec struct {
	// Min is the minimum allowed value (inclusive). Nil means no minimum
	// constraint.
	Min *float64 `json:"min,omitempty"`

	// Max is the maximum allowed value (inclusive). Nil means no maximum
	// constraint.
	Max *float64 `json:"max,omitempty"`

	// Step is the increment/decrement amount. If 1.0, implies integer values.
	// If nil, any precision allowed.
	Step *float64 `json:"step,omitempty"`
}

// Validate implements PropertySpec for NumberPropertySpec.
func (n NumberPropertySpec) Validate(value interface{}) error {
	num, ok := value.(float64)
	if !ok {
		return errors.NewValidationError("value", "must be number", value)
	}

	if err := validateNumberBounds(n.Min, n.Max, num, value); err != nil {
		return err
	}

	return validateStepConstraint(n.Step, num, value)
}

// DatePropertySpec validates date/time values with format constraints.
type DatePropertySpec struct {
	// Format is the Go time layout string for parsing.
	// If empty, defaults to RFC3339.
	Format string `json:"format,omitempty"`
}

// Validate implements PropertySpec for DatePropertySpec.
func (d DatePropertySpec) Validate(value interface{}) error {
	str, ok := value.(string)
	if !ok {
		return errors.NewValidationError("value", "must be string", value)
	}

	format := d.Format
	if format == "" {
		format = time.RFC3339
	}

	_, err := time.Parse(format, str)
	if err != nil {
		return errors.NewValidationError(
			"value",
			fmt.Sprintf("must be valid date in format: %s", format),
			value,
		)
	}

	return nil
}

// FilePropertySpec validates file reference values with optional
// class/directory constraints.
type FilePropertySpec struct {
	// FileClass restricts valid file references to notes with specific
	// fileClass value. Supports negation via ^ prefix. Empty string means no
	// fileClass restriction.
	FileClass string `json:"fileClass,omitempty"`

	// Directory restricts valid file references to notes within specific vault
	// directory path.
	// Path is relative to vault root. Supports negation via ^ prefix.
	Directory string `json:"directory,omitempty"`
}

// Validate implements PropertySpec for FilePropertySpec.
// Note: This is a simplified validation for MVP. Full validation requires vault
// index lookup.
func (f FilePropertySpec) Validate(value interface{}) error {
	str, ok := value.(string)
	if !ok {
		return errors.NewValidationError("value", "must be string", value)
	}

	// Basic validation: check if it's a valid file path or wikilink format
	if str == "" {
		return errors.NewValidationError("value", "cannot be empty", value)
	}

	// For MVP, we only do basic format validation
	// Full validation would check against vault index for fileClass/directory
	// constraints
	// This will be implemented in the SchemaValidator domain service

	return nil
}

// BoolPropertySpec validates boolean values. No additional configuration
// needed.
type BoolPropertySpec struct{}

// Validate implements PropertySpec for BoolPropertySpec.
func (b BoolPropertySpec) Validate(value interface{}) error {
	_, ok := value.(bool)
	if !ok {
		return errors.NewValidationError("value", "must be boolean", value)
	}
	return nil
}

// NewProperty creates a new Property with the given specification.
func NewProperty(
	name string,
	required, array bool,
	spec PropertySpec,
) Property {
	typeName := determinePropertyType(spec)

	return Property{
		Name:     name,
		Required: required,
		Array:    array,
		Spec:     spec,
		Type:     typeName,
	}
}

// MarshalJSON implements custom JSON marshaling for Property.
// Ensures the type discriminator is included in the output.
func (p Property) MarshalJSON() ([]byte, error) {
	payload, err := propertyJSONPayload(p)
	if err != nil {
		return nil, err
	}

	return json.Marshal(payload)
}

// UnmarshalJSON implements custom JSON unmarshaling for Property.
// Uses discriminator-based unmarshaling to create the appropriate PropertySpec.
func (p *Property) UnmarshalJSON(data []byte) error {
	// Define a temporary struct to capture the discriminator and common fields
	type propertyJSON struct {
		Name     string `json:"name"`
		Required bool   `json:"required"`
		Array    bool   `json:"array"`
		Type     string `json:"type"`
	}

	var pj propertyJSON
	if err := json.Unmarshal(data, &pj); err != nil {
		return err
	}

	// Set common fields
	p.Name = pj.Name
	p.Required = pj.Required
	p.Array = pj.Array
	p.Type = pj.Type

	spec, err := buildSpecFromJSON(pj.Type, data)
	if err != nil {
		return err
	}

	p.Spec = spec

	return nil
}

// Validate checks if the property definition itself is valid.
func (p Property) Validate() error {
	if strings.TrimSpace(p.Name) == "" {
		return errors.NewValidationError("name", "cannot be empty", p.Name)
	}

	// Basic YAML key validation (no special chars except dash/underscore)
	if matched, _ := regexp.MatchString(`^[a-zA-Z][a-zA-Z0-9_-]*$`, p.Name); !matched {
		return errors.NewValidationError(
			"name",
			"must be valid YAML key (letters, numbers, dash, underscore only)",
			p.Name,
		)
	}

	if p.Spec == nil {
		return errors.NewValidationError("spec", "cannot be nil", nil)
	}

	return nil
}

// propertySpecDetails determines the JSON type discriminator and any
// spec-specific fields that need to be serialized.
func propertySpecDetails(
	spec PropertySpec,
) (typeName string, extraFields map[string]interface{}, err error) {
	normalized, normalizeErr := normalizeSpec(spec)
	if normalizeErr != nil {
		err = normalizeErr
		return
	}

	typeName, err = propertyTypeName(normalized)
	if err != nil {
		return
	}

	extraFields, err = specExtraFields(typeName, normalized)
	return
}

func stringSpecFields(spec StringPropertySpec) map[string]interface{} {
	var fields map[string]interface{}
	if len(spec.Enum) > 0 {
		fields = ensureFieldsMap(fields)
		fields["enum"] = spec.Enum
	}
	if spec.Pattern != "" {
		fields = ensureFieldsMap(fields)
		fields["pattern"] = spec.Pattern
	}

	return fields
}

func numberSpecFields(spec NumberPropertySpec) map[string]interface{} {
	var fields map[string]interface{}

	if spec.Min != nil {
		fields = ensureFieldsMap(fields)
		fields["min"] = *spec.Min
	}
	if spec.Max != nil {
		fields = ensureFieldsMap(fields)
		fields["max"] = *spec.Max
	}
	if spec.Step != nil {
		fields = ensureFieldsMap(fields)
		fields["step"] = *spec.Step
	}

	return fields
}

func dateSpecFields(spec DatePropertySpec) map[string]interface{} {
	if spec.Format == "" {
		return nil
	}

	fields := make(map[string]interface{}, 1)
	fields["format"] = spec.Format
	return fields
}

func fileSpecFields(spec FilePropertySpec) map[string]interface{} {
	var fields map[string]interface{}

	if spec.FileClass != "" {
		fields = ensureFieldsMap(fields)
		fields["fileClass"] = spec.FileClass
	}
	if spec.Directory != "" {
		fields = ensureFieldsMap(fields)
		fields["directory"] = spec.Directory
	}

	return fields
}

func ensureFieldsMap(fields map[string]interface{}) map[string]interface{} {
	if fields == nil {
		return make(map[string]interface{})
	}
	return fields
}

func propertyJSONPayload(p Property) (map[string]interface{}, error) {
	typeStr, extraFields, err := propertySpecDetails(p.Spec)
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

func decodeStringSpec(data []byte) (PropertySpec, error) {
	var spec StringPropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func decodeNumberSpec(data []byte) (PropertySpec, error) {
	var spec NumberPropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func decodeDateSpec(data []byte) (PropertySpec, error) {
	var spec DatePropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func decodeFileSpec(data []byte) (PropertySpec, error) {
	var spec FilePropertySpec
	if err := json.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func decodeBoolSpec(_ []byte) (PropertySpec, error) {
	return BoolPropertySpec{}, nil
}

func determinePropertyType(spec PropertySpec) string {
	normalized, err := normalizeSpec(spec)
	if err != nil {
		return ""
	}

	typeName, err := propertyTypeName(normalized)
	if err != nil {
		return ""
	}

	return typeName
}

func buildSpecFromJSON(
	typeName string,
	data []byte,
) (PropertySpec, error) {
	decoder, ok := decodePropertySpec[typeName]
	if !ok {
		return nil, fmt.Errorf("unknown property type: %s", typeName)
	}

	return decoder(data)
}

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

func specExtraFields(
	typeName string,
	spec PropertySpec,
) (map[string]interface{}, error) {
	extractor, ok := extractPropertyFields[typeName]
	if !ok {
		return nil, fmt.Errorf("unknown property type: %s", typeName)
	}

	return extractor(spec)
}

func extractStringSpecFields(
	spec PropertySpec,
) (map[string]interface{}, error) {
	payload, ok := spec.(StringPropertySpec)
	if !ok {
		return nil, fmt.Errorf("spec type mismatch for %s", propertyTypeString)
	}
	return stringSpecFields(payload), nil
}

func extractNumberSpecFields(
	spec PropertySpec,
) (map[string]interface{}, error) {
	payload, ok := spec.(NumberPropertySpec)
	if !ok {
		return nil, fmt.Errorf("spec type mismatch for %s", propertyTypeNumber)
	}
	return numberSpecFields(payload), nil
}

func extractDateSpecFields(spec PropertySpec) (map[string]interface{}, error) {
	payload, ok := spec.(DatePropertySpec)
	if !ok {
		return nil, fmt.Errorf("spec type mismatch for %s", propertyTypeDate)
	}
	return dateSpecFields(payload), nil
}

func extractFileSpecFields(spec PropertySpec) (map[string]interface{}, error) {
	payload, ok := spec.(FilePropertySpec)
	if !ok {
		return nil, fmt.Errorf("spec type mismatch for %s", propertyTypeFile)
	}
	return fileSpecFields(payload), nil
}

func extractBoolSpecFields(PropertySpec) (map[string]interface{}, error) {
	return map[string]interface{}{}, nil
}

func isBelowMin(minValue *float64, value float64) bool {
	return minValue != nil && value < *minValue
}

func isAboveMax(maxValue *float64, value float64) bool {
	return maxValue != nil && value > *maxValue
}

func validateNumberBounds(
	minValue *float64,
	maxValue *float64,
	value float64,
	raw interface{},
) error {
	if isBelowMin(minValue, value) {
		return errors.NewValidationError(
			"value",
			fmt.Sprintf("must be >= %v", *minValue),
			raw,
		)
	}

	if isAboveMax(maxValue, value) {
		return errors.NewValidationError(
			"value",
			fmt.Sprintf("must be <= %v", *maxValue),
			raw,
		)
	}

	return nil
}

func validateStepConstraint(
	step *float64,
	value float64,
	raw interface{},
) error {
	if step == nil {
		return nil
	}

	if *step == 1.0 && value != math.Floor(value) {
		return errors.NewValidationError(
			"value",
			"must be integer",
			raw,
		)
	}

	return nil
}
