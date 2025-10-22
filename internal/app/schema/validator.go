// Package schema provides domain services for schema validation and processing.
// This package implements the application layer business logic for validating
// schema definitions, property banks, and orchestrating PropertySpec
// validation.
package schema

import (
	"context"
	"errors"
	"fmt"
	"math"
	"reflect"
	"regexp"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// SchemaValidator implements domain service for validating schema definitions,
// property banks, properties, and orchestrating PropertySpec validation. All
// validation
// methods extracted from domain models per architectural refactoring.
type SchemaValidator struct{}

// NewSchemaValidator creates a new SchemaValidator.
// Pure validation logic with no external dependencies.
func NewSchemaValidator() *SchemaValidator {
	return &SchemaValidator{}
}

// ValidateSchema validates a complete schema definition for structural
// integrity.
// Returns Result[ValidationResult] with detailed validation errors.
// Extracted from domain.Schema.Validate() method.
func (v *SchemaValidator) ValidateSchema(
	ctx context.Context,
	schema *domain.Schema,
) lithoserrors.Result[lithoserrors.ValidationResult] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[lithoserrors.ValidationResult](ctx.Err())
	default:
	}

	result := lithoserrors.NewValidationResult()

	// Validate schema name using extracted logic
	if err := v.validateSchemaName(schema.Name); err != nil {
		var validationErr lithoserrors.ValidationError
		if errors.As(err, &validationErr) {
			result.AddError(lithoserrors.NewFieldValidationError(
				validationErr.Property(),
				validationErr.Reason(),
				validationErr.Value(),
				err,
			))
		} else {
			result.AddError(v.wrapValidationError("name", err))
		}
	}

	// Validate extends relationship using extracted logic
	if err := v.validateSchemaExtends(schema.Name, schema.Extends); err != nil {
		var validationErr lithoserrors.ValidationError
		if errors.As(err, &validationErr) {
			result.AddError(lithoserrors.NewFieldValidationError(
				validationErr.Property(),
				validationErr.Reason(),
				validationErr.Value(),
				err,
			))
		} else {
			result.AddError(v.wrapValidationError("extends", err))
		}
	}

	// Validate excludes list using extracted logic
	if err := v.validateSchemaExcludes(schema.Excludes); err != nil {
		var validationErr lithoserrors.ValidationError
		if errors.As(err, &validationErr) {
			result.AddError(lithoserrors.NewFieldValidationError(
				validationErr.Property(),
				validationErr.Reason(),
				validationErr.Value(),
				err,
			))
		} else {
			result.AddError(v.wrapValidationError("excludes", err))
		}
	}

	// Validate properties using extracted logic
	if err := v.validateSchemaProperties(schema.Properties); err != nil {
		result.AddError(v.wrapValidationError("properties", err))
	}

	return lithoserrors.Ok[lithoserrors.ValidationResult](result)
}

// ValidatePropertyBank validates a property bank definition and all its
// properties.
// Returns Result[ValidationResult] with detailed validation errors.
// Extracted from domain.PropertyBank.Validate() method.
func (v *SchemaValidator) ValidatePropertyBank(
	ctx context.Context,
	propertyBank *domain.PropertyBank,
) lithoserrors.Result[lithoserrors.ValidationResult] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[lithoserrors.ValidationResult](ctx.Err())
	default:
	}

	result := lithoserrors.NewValidationResult()

	// Validate property bank location using extracted logic
	if err := v.validatePropertyBankLocation(propertyBank.Location); err != nil {
		var validationErr lithoserrors.ValidationError
		if errors.As(err, &validationErr) {
			result.AddError(lithoserrors.NewFieldValidationError(
				validationErr.Property(),
				validationErr.Reason(),
				validationErr.Value(),
				err,
			))
		} else {
			result.AddError(v.wrapValidationError("location", err))
		}
	}

	// Validate all registered properties using extracted logic
	if err := v.validatePropertyBankProperties(propertyBank.Properties); err != nil {
		result.AddError(v.wrapValidationError("properties", err))
	}

	return lithoserrors.Ok[lithoserrors.ValidationResult](result)
}

// ValidateProperty validates a single property definition.
// Returns Result[ValidationResult] with detailed validation errors.
// Extracted from domain.Property.Validate() method.
func (v *SchemaValidator) ValidateProperty(
	ctx context.Context,
	property domain.Property,
) lithoserrors.Result[lithoserrors.ValidationResult] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[lithoserrors.ValidationResult](ctx.Err())
	default:
	}

	result := lithoserrors.NewValidationResult()

	// Validate property name using extracted logic
	if err := v.validatePropertyName(property.Name); err != nil {
		var validationErr lithoserrors.ValidationError
		if errors.As(err, &validationErr) {
			result.AddError(lithoserrors.NewFieldValidationError(
				validationErr.Property(),
				validationErr.Reason(),
				validationErr.Value(),
				err,
			))
		} else {
			result.AddError(v.wrapValidationError("property", err))
		}
	}

	// Validate spec using extracted logic
	if err := v.validatePropertySpec(property.Spec); err != nil {
		var validationErr lithoserrors.ValidationError
		if errors.As(err, &validationErr) {
			result.AddError(lithoserrors.NewFieldValidationError(
				validationErr.Property(),
				validationErr.Reason(),
				validationErr.Value(),
				err,
			))
		} else {
			result.AddError(v.wrapValidationError("spec", err))
		}
	}

	return lithoserrors.Ok[lithoserrors.ValidationResult](result)
}

// ValidatePropertyValue validates a property value against its specification.
// Returns Result[ValidationResult] with detailed validation errors.
// Extracted from domain.Property.ValidateValue() method.
func (v *SchemaValidator) ValidatePropertyValue(
	ctx context.Context,
	property domain.Property,
	value interface{},
) lithoserrors.Result[lithoserrors.ValidationResult] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[lithoserrors.ValidationResult](ctx.Err())
	default:
	}

	result := lithoserrors.NewValidationResult()

	if property.Spec == nil {
		result.AddError(
			v.wrapValidationError("spec", errors.New("cannot be nil")),
		)
		return lithoserrors.Ok[lithoserrors.ValidationResult](result)
	}

	if err := v.validateValue(ctx, property, value); err != nil {
		result.AddError(v.wrapValidationError("value", err))
	}

	return lithoserrors.Ok[lithoserrors.ValidationResult](result)
}

// validateValue validates a property value, handling arrays and scalars.
func (v *SchemaValidator) validateValue(
	ctx context.Context,
	property domain.Property,
	value interface{},
) error {
	if property.Array {
		return v.validateArrayValue(ctx, property, value)
	}
	return v.validatePropertySpecValue(ctx, property.Spec, value)
}

// wrapValidationError converts a standard error into a FieldValidationError.
func (v *SchemaValidator) wrapValidationError(
	field string,
	err error,
) lithoserrors.FieldValidationError {
	return lithoserrors.NewFieldValidationError(field, err.Error(), nil, err)
}

// Private validation methods extracted from domain models

func (v *SchemaValidator) validateSchemaName(name string) error {
	trimmed := strings.TrimSpace(name)
	if trimmed == "" {
		return lithoserrors.NewValidationError("name", "cannot be empty", name)
	}
	if !isValidIdentifier(trimmed) {
		return lithoserrors.NewValidationError(
			"name",
			"must be valid identifier (letters, numbers, dash, underscore only)",
			name,
		)
	}
	return nil
}

func (v *SchemaValidator) validateSchemaExtends(name, extends string) error {
	trimmed := strings.TrimSpace(extends)
	if trimmed == "" {
		return nil
	}
	if !isValidIdentifier(trimmed) {
		return lithoserrors.NewValidationError(
			"extends",
			"must be valid identifier",
			extends,
		)
	}
	if trimmed == name {
		return lithoserrors.NewValidationError(
			"extends",
			"cannot reference itself",
			extends,
		)
	}
	return nil
}

func (v *SchemaValidator) validateSchemaExcludes(excludes []string) error {
	seen := make(map[string]struct{}, len(excludes))
	for _, exclude := range excludes {
		trimmed := strings.TrimSpace(exclude)
		if trimmed == "" {
			return lithoserrors.NewValidationError(
				"excludes",
				"cannot be empty",
				exclude,
			)
		}
		if !isValidIdentifier(trimmed) {
			return lithoserrors.NewValidationError(
				"excludes",
				"must be valid identifier",
				exclude,
			)
		}
		if _, exists := seen[trimmed]; exists {
			return lithoserrors.NewValidationError(
				"excludes",
				fmt.Sprintf("duplicate exclude property: %s", trimmed),
				exclude,
			)
		}
		seen[trimmed] = struct{}{}
	}
	return nil
}

func (v *SchemaValidator) validateSchemaProperties(
	properties []domain.Property,
) error {
	encountered := make(map[string]struct{}, len(properties))
	for index, prop := range properties {
		if err := v.validateSingleProperty(index, prop, encountered); err != nil {
			return err
		}
	}
	return nil
}

func (v *SchemaValidator) validateSingleProperty(
	index int,
	prop domain.Property,
	encountered map[string]struct{},
) error {
	// Validate the property itself
	if err := v.ValidateProperty(context.Background(), prop); err.IsErr() {
		return fmt.Errorf("property %d (%s): %w", index, prop.Name, err.Error())
	}

	// Check for duplicates
	if _, exists := encountered[prop.Name]; exists {
		return lithoserrors.NewValidationError(
			"properties",
			fmt.Sprintf("duplicate property name: %s", prop.Name),
			prop.Name,
		)
	}
	encountered[prop.Name] = struct{}{}
	return nil
}

func (v *SchemaValidator) validateArrayValue(
	ctx context.Context,
	property domain.Property,
	value interface{},
) error {
	if value == nil {
		return nil
	}

	rv := reflect.ValueOf(value)
	if rv.Kind() != reflect.Slice && rv.Kind() != reflect.Array {
		return lithoserrors.NewValidationError(
			"value",
			"must be array or slice",
			value,
		)
	}

	for i := range rv.Len() {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		elem := rv.Index(i).Interface()
		if err := v.validatePropertySpecValue(ctx, property.Spec, elem); err != nil {
			var validationErr lithoserrors.ValidationError
			if errors.As(err, &validationErr) {
				field := fmt.Sprintf(
					"value[%d].%s",
					i,
					validationErr.Property(),
				)
				return lithoserrors.NewValidationError(
					field,
					validationErr.Reason(),
					validationErr.Value(),
				)
			}
			return err
		}
	}
	return nil
}

func (v *SchemaValidator) validatePropertySpecValue(
	ctx context.Context,
	spec domain.PropertySpec,
	value interface{},
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	// Extract validation logic from PropertySpec implementations
	switch typed := spec.(type) {
	case domain.StringPropertySpec:
		return v.validateStringPropertySpecValue(ctx, typed, value)
	case domain.NumberPropertySpec:
		return v.validateNumberPropertySpecValue(ctx, typed, value)
	case domain.DatePropertySpec:
		return v.validateDatePropertySpecValue(ctx, typed, value)
	case domain.FilePropertySpec:
		return v.validateFilePropertySpecValue(ctx, typed, value)
	case domain.BoolPropertySpec:
		return v.validateBoolPropertySpecValue(ctx, typed, value)
	default:
		return lithoserrors.NewValidationError(
			"spec",
			fmt.Sprintf("unknown property spec type: %T", spec),
			spec,
		)
	}
}

// PropertySpec normalization and type checking (extracted from domain)

func (v *SchemaValidator) normalizeSpec(
	spec domain.PropertySpec,
) (domain.PropertySpec, error) {
	if spec == nil {
		return nil, fmt.Errorf("property spec cannot be nil")
	}

	if deref, handled, err := v.dereferencedSpec(spec); handled {
		return deref, err
	}

	return spec, nil
}

// dereferenceSpecIfPointer dereferences a pointer spec if it's a pointer type,
// returning the dereferenced value, whether it was a pointer, and any error.
func dereferenceSpecIfPointer(
	spec domain.PropertySpec,
) (domain.PropertySpec, bool, error) {
	switch typed := spec.(type) {
	case *domain.StringPropertySpec:
		return dereferenceStringSpec(typed)
	case *domain.NumberPropertySpec:
		return dereferenceNumberSpec(typed)
	case *domain.DatePropertySpec:
		return dereferenceDateSpec(typed)
	case *domain.FilePropertySpec:
		return dereferenceFileSpec(typed)
	case *domain.BoolPropertySpec:
		return dereferenceBoolSpec(typed)
	default:
		return spec, false, nil // Not a pointer, return as-is
	}
}

// dereferenceStringSpec dereferences a StringPropertySpec pointer.
func dereferenceStringSpec(
	typed *domain.StringPropertySpec,
) (domain.PropertySpec, bool, error) {
	if typed == nil {
		return nil, true, fmt.Errorf("string property spec cannot be nil")
	}
	return *typed, true, nil
}

// dereferenceNumberSpec dereferences a NumberPropertySpec pointer.
func dereferenceNumberSpec(
	typed *domain.NumberPropertySpec,
) (domain.PropertySpec, bool, error) {
	if typed == nil {
		return nil, true, fmt.Errorf("number property spec cannot be nil")
	}
	return *typed, true, nil
}

// dereferenceDateSpec dereferences a DatePropertySpec pointer.
func dereferenceDateSpec(
	typed *domain.DatePropertySpec,
) (domain.PropertySpec, bool, error) {
	if typed == nil {
		return nil, true, fmt.Errorf("date property spec cannot be nil")
	}
	return *typed, true, nil
}

// dereferenceFileSpec dereferences a FilePropertySpec pointer.
func dereferenceFileSpec(
	typed *domain.FilePropertySpec,
) (domain.PropertySpec, bool, error) {
	if typed == nil {
		return nil, true, fmt.Errorf("file property spec cannot be nil")
	}
	return *typed, true, nil
}

// dereferenceBoolSpec dereferences a BoolPropertySpec pointer.
func dereferenceBoolSpec(
	typed *domain.BoolPropertySpec,
) (domain.PropertySpec, bool, error) {
	if typed == nil {
		return nil, true, fmt.Errorf("bool property spec cannot be nil")
	}
	return *typed, true, nil
}

func (v *SchemaValidator) dereferencedSpec(
	spec domain.PropertySpec,
) (domain.PropertySpec, bool, error) {
	return dereferenceSpecIfPointer(spec)
}

func (v *SchemaValidator) propertyTypeName(
	spec domain.PropertySpec,
) (string, error) {
	switch spec.(type) {
	case domain.StringPropertySpec:
		return "string", nil
	case domain.NumberPropertySpec:
		return "number", nil
	case domain.DatePropertySpec:
		return "date", nil
	case domain.FilePropertySpec:
		return "file", nil
	case domain.BoolPropertySpec:
		return "bool", nil
	default:
		return "", fmt.Errorf("unknown property spec type: %T", spec)
	}
}

// Validation helper functions extracted from domain

var propertyNamePattern = regexp.MustCompile(`^[a-zA-Z][a-zA-Z0-9_-]*$`)

var identifierRegexp = regexp.MustCompile(`^[a-zA-Z][a-zA-Z0-9_-]*$`)

func isValidIdentifier(value string) bool {
	return identifierRegexp.MatchString(value)
}

// PropertySpec validation implementations extracted from domain models

func (v *SchemaValidator) validateStringPropertySpecValue(
	ctx context.Context,
	spec domain.StringPropertySpec,
	value interface{},
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	str, err := v.validateStringType(value)
	if err != nil {
		return err
	}

	if enumErr := v.validateStringEnum(str, spec.Enum, value); enumErr != nil {
		return enumErr
	}

	return v.validateStringPattern(str, spec.Pattern, value)
}

func (v *SchemaValidator) validateNumberPropertySpecValue(
	ctx context.Context,
	spec domain.NumberPropertySpec,
	value interface{},
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	num, ok := value.(float64)
	if !ok {
		return lithoserrors.NewValidationError("value", "must be number", value)
	}

	if err := v.validateNumberBounds(spec.Min, spec.Max, num, value); err != nil {
		return err
	}

	return v.validateStepConstraint(spec.Step, num, value)
}

func (v *SchemaValidator) validateDatePropertySpecValue(
	ctx context.Context,
	spec domain.DatePropertySpec,
	value interface{},
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	str, err := v.validateDateType(value)
	if err != nil {
		return err
	}

	return v.validateDateFormat(str, spec.Format, value)
}

func (v *SchemaValidator) validateFilePropertySpecValue(
	ctx context.Context,
	spec domain.FilePropertySpec,
	value interface{},
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	str, err := v.validateFileType(value)
	if err != nil {
		return err
	}

	// FileClass validation deferred to runtime file system integration
	// Use spec to satisfy linter - validates parameter is present
	_ = spec.FileClass

	return v.validateFilePath(str, value)
}

func (v *SchemaValidator) validateBoolPropertySpecValue(
	ctx context.Context,
	spec domain.BoolPropertySpec,
	value interface{},
) error {
	// Check for context cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	_, ok := value.(bool)
	if !ok {
		return lithoserrors.NewValidationError(
			"value",
			"must be boolean",
			value,
		)
	}
	return nil
}

// Helper functions extracted from PropertySpec implementations

func (v *SchemaValidator) validateStringType(
	value interface{},
) (string, error) {
	str, ok := value.(string)
	if !ok {
		return "", lithoserrors.NewValidationError(
			"value",
			"must be string",
			value,
		)
	}
	return str, nil
}

func (v *SchemaValidator) validateStringEnum(
	str string,
	enum []string,
	raw interface{},
) error {
	if len(enum) == 0 {
		return nil // no enum constraint
	}

	for _, allowed := range enum {
		if str == allowed {
			return nil
		}
	}

	return lithoserrors.NewValidationError(
		"value",
		fmt.Sprintf("must be one of: %v", enum),
		raw,
	)
}

// Property validation methods extracted from domain.Property

func (v *SchemaValidator) validatePropertyName(name string) error {
	trimmed := strings.TrimSpace(name)
	if trimmed == "" {
		return lithoserrors.NewValidationError(
			"name",
			"cannot be empty",
			name,
		)
	}

	if !propertyNamePattern.MatchString(trimmed) {
		return lithoserrors.NewValidationError(
			"name",
			"must be valid YAML key (letters, numbers, dash, underscore only)",
			name,
		)
	}

	return nil
}

func (v *SchemaValidator) validatePropertySpec(spec domain.PropertySpec) error {
	if spec == nil {
		return lithoserrors.NewValidationError("spec", "cannot be nil", nil)
	}

	normalized, err := v.normalizeSpec(spec)
	if err != nil {
		return lithoserrors.NewValidationError("spec", err.Error(), nil)
	}

	_, typeErr := v.propertyTypeName(normalized)
	return typeErr
}

func (v *SchemaValidator) validatePropertyBankLocation(location string) error {
	if strings.TrimSpace(location) == "" {
		return lithoserrors.NewValidationError(
			"location",
			"cannot be empty",
			location,
		)
	}
	return nil
}

func (v *SchemaValidator) validatePropertyBankProperties(
	properties map[string]domain.Property,
) error {
	for name, property := range properties {
		result := v.ValidateProperty(context.Background(), property)
		if result.IsErr() {
			return fmt.Errorf(
				"property '%s' is invalid: %w",
				name,
				result.Error(),
			)
		}
	}
	return nil
}

func (v *SchemaValidator) validateStringPattern(
	str, pattern string,
	raw interface{},
) error {
	if pattern == "" {
		return nil // no pattern constraint
	}

	// Compile regex once for better performance
	compiled, err := regexp.Compile(pattern)
	if err != nil {
		return lithoserrors.NewValidationError(
			"value",
			fmt.Sprintf("invalid pattern: %s", err.Error()),
			raw,
		)
	}

	if !compiled.MatchString(str) {
		return lithoserrors.NewValidationError(
			"value",
			fmt.Sprintf("must match pattern: %s", pattern),
			raw,
		)
	}

	return nil
}

func (v *SchemaValidator) validateNumberBounds(
	minValue *float64,
	maxValue *float64,
	value float64,
	raw interface{},
) error {
	if minValue != nil && value < *minValue {
		return lithoserrors.NewValidationError(
			"value",
			fmt.Sprintf("must be >= %v", *minValue),
			raw,
		)
	}

	if maxValue != nil && value > *maxValue {
		return lithoserrors.NewValidationError(
			"value",
			fmt.Sprintf("must be <= %v", *maxValue),
			raw,
		)
	}

	return nil
}

func (v *SchemaValidator) validateStepConstraint(
	step *float64,
	value float64,
	raw interface{},
) error {
	if step == nil {
		return nil
	}

	if *step == 1.0 && value != math.Floor(value) {
		return lithoserrors.NewValidationError(
			"value",
			"must be integer",
			raw,
		)
	}

	return nil
}

func (v *SchemaValidator) validateFileType(value interface{}) (string, error) {
	str, ok := value.(string)
	if !ok {
		return "", lithoserrors.NewValidationError(
			"value",
			"must be string",
			value,
		)
	}
	return str, nil
}

func (v *SchemaValidator) validateFilePath(str string, raw interface{}) error {
	if str == "" {
		return lithoserrors.NewValidationError("value", "cannot be empty", raw)
	}
	return nil
}

func (v *SchemaValidator) validateDateType(value interface{}) (string, error) {
	str, ok := value.(string)
	if !ok {
		return "", lithoserrors.NewValidationError(
			"value",
			"must be string",
			value,
		)
	}
	return str, nil
}

func (v *SchemaValidator) validateDateFormat(
	str, format string,
	raw interface{},
) error {
	if format == "" {
		format = time.RFC3339
	}

	_, err := time.Parse(format, str)
	if err != nil {
		return lithoserrors.NewValidationError(
			"value",
			fmt.Sprintf("must be valid date in format: %s", format),
			raw,
		)
	}

	return nil
}
