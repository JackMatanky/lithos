package frontmatter

import (
	"context"
	"fmt"
	"math"
	"slices"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/converters"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// FieldValidator defines the interface for polymorphic field validation.
type FieldValidator interface {
	// Validate validates a frontmatter field value against a PropertySpec.
	Validate(
		fieldName string,
		value any,
		spec domain.PropertySpec,
	) error
}

// Validator defines a generic interface for type-safe property validation.
type Validator[T any] interface {
	Validate(fieldName string, value any, spec T) error
}

// ValidatorRegistry provides a central registry for property validators.
type ValidatorRegistry struct {
	validators map[domain.PropertySpecType]FieldValidator
}

// StringValidator validates string fields.
type StringValidator struct{}

// NumberValidator validates numeric fields.
type NumberValidator struct{}

// DateValidator validates date fields.
type DateValidator struct{}

// BoolValidator validates boolean fields.
type BoolValidator struct{}

// NewValidatorRegistry creates a new empty registry.
func NewValidatorRegistry() *ValidatorRegistry {
	return &ValidatorRegistry{
		validators: make(map[domain.PropertySpecType]FieldValidator),
	}
}

// Register registers a validator for a specific property type.
func (r *ValidatorRegistry) Register(
	propType domain.PropertySpecType,
	v FieldValidator,
) {
	r.validators[propType] = v
}

// Get retrieves a validator for a specific property type.
func (r *ValidatorRegistry) Get(
	propType domain.PropertySpecType,
) (FieldValidator, bool) {
	v, ok := r.validators[propType]
	return v, ok
}

// DefaultValidatorRegistry creates a registry with all standard validators.
func DefaultValidatorRegistry() *ValidatorRegistry {
	r := NewValidatorRegistry()
	r.Register(domain.PropertyTypeString, &StringValidator{})
	r.Register(domain.PropertyTypeNumber, &NumberValidator{})
	r.Register(domain.PropertyTypeDate, &DateValidator{})
	r.Register(domain.PropertyTypeBool, &BoolValidator{})
	return r
}

// Validate validates string values against StringSpec constraints.
func (v *StringValidator) Validate(
	fieldName string,
	value any,
	spec domain.PropertySpec,
) error {
	stringValue, ok := converters.ToString(value)
	if !ok {
		return errors.NewFrontmatterError(
			"field value is not a string",
			fieldName,
			nil,
		)
	}

	stringSpec, ok2 := spec.(*domain.StringSpec)
	if !ok2 {
		return errors.NewFrontmatterError(
			"property spec is not StringSpec",
			fieldName,
			nil,
		)
	}

	if err := v.validateEnum(fieldName, stringValue, stringSpec); err != nil {
		return err
	}

	return v.validatePattern(fieldName, stringValue, stringSpec)
}

func (v *StringValidator) validateEnum(
	fieldName, value string,
	spec *domain.StringSpec,
) error {
	if len(spec.Enum) > 0 {
		if !slices.Contains(spec.Enum, value) {
			return errors.NewFrontmatterError(
				"value not in allowed enum",
				fieldName,
				nil,
			)
		}
	}
	return nil
}

func (v *StringValidator) validatePattern(
	fieldName, value string,
	spec *domain.StringSpec,
) error {
	if spec.Pattern != "" {
		if err := spec.Validate(context.Background()); err != nil {
			return errors.NewFrontmatterError(
				"invalid pattern regex in schema",
				fieldName,
				err,
			)
		}
		if !spec.Match(value) {
			return errors.NewFrontmatterError(
				fmt.Sprintf("value does not match pattern: %s", spec.Pattern),
				fieldName,
				nil,
			)
		}
	}
	return nil
}

// Validate validates numeric values against NumberSpec constraints.
func (v *NumberValidator) Validate(
	fieldName string,
	value any,
	spec domain.PropertySpec,
) error {
	numValue, ok := converters.ToFloat64(value)
	if !ok {
		return errors.NewFrontmatterError(
			"field value is not numeric",
			fieldName,
			nil,
		)
	}

	var numberSpec *domain.NumberSpec
	switch s := spec.(type) {
	case *domain.NumberSpec:
		numberSpec = s
	case domain.NumberSpec:
		numberSpec = &s
	default:
		return errors.NewFrontmatterError("property spec is not NumberSpec", fieldName, nil)
	}

	if numberSpec.Min != nil && numValue < *numberSpec.Min {
		return errors.NewFrontmatterError("value below minimum", fieldName, nil)
	}
	if numberSpec.Max != nil && numValue > *numberSpec.Max {
		return errors.NewFrontmatterError("value above maximum", fieldName, nil)
	}

	return v.validateStep(fieldName, numValue, numberSpec)
}

func (v *NumberValidator) validateStep(
	fieldName string,
	numValue float64,
	spec *domain.NumberSpec,
) error {
	if spec.Step != nil {
		step := *spec.Step
		base := 0.0
		if spec.Min != nil {
			base = *spec.Min
		}
		const epsilon = 1e-9
		remainder := math.Abs(math.Mod(numValue-base, step))
		if remainder > epsilon && math.Abs(remainder-step) > epsilon {
			return errors.NewFrontmatterError(
				fmt.Sprintf(
					"value must be a multiple of %g starting from %g",
					step,
					base,
				),
				fieldName,
				nil,
			)
		}
	}
	return nil
}

// Validate validates date values against DateSpec constraints.
func (v *DateValidator) Validate(
	fieldName string,
	value any,
	spec domain.PropertySpec,
) error {
	var dateSpec *domain.DateSpec
	switch s := spec.(type) {
	case *domain.DateSpec:
		dateSpec = s
	case domain.DateSpec:
		dateSpec = &s
	default:
		return errors.NewFrontmatterError("property spec is not DateSpec", fieldName, nil)
	}

	format := dateSpec.Format
	if format == "" {
		format = "2006-01-02T15:04:05Z07:00"
	}

	var dateValue string
	switch val := value.(type) {
	case string:
		dateValue = val
	case time.Time:
		dateValue = val.Format(format)
	default:
		return errors.NewFrontmatterError("field value is not a string", fieldName, nil)
	}

	if _, err := time.Parse(format, dateValue); err != nil {
		return errors.NewFrontmatterError("invalid date format", fieldName, err)
	}
	return nil
}

// Validate validates boolean values against BoolSpec constraints.
func (v *BoolValidator) Validate(
	fieldName string,
	value any,
	spec domain.PropertySpec,
) error {
	if _, ok := value.(bool); !ok {
		return errors.NewFrontmatterError(
			"field value is not a boolean",
			fieldName,
			nil,
		)
	}
	switch spec.(type) {
	case *domain.BoolSpec, domain.BoolSpec:
		return nil
	default:
		return errors.NewFrontmatterError("property spec is not BoolSpec", fieldName, nil)
	}
}
