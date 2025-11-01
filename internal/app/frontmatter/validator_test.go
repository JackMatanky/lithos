package frontmatter

import (
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/require"
)

// TestFieldValidator_InterfaceExists verifies FieldValidator interface exists.
func TestFieldValidator_InterfaceExists(t *testing.T) {
	// This test verifies FieldValidator interface exists
	var _ FieldValidator // This will fail if interface doesn't exist
}

// TestStringValidator_ImplementsFieldValidator verifies StringValidator
// implements FieldValidator.
func TestStringValidator_ImplementsFieldValidator(t *testing.T) {
	// This test verifies StringValidator implements FieldValidator
	validator := &StringValidator{}
	var _ FieldValidator = validator // This will fail if StringValidator doesn't implement interface
}

// TestNumberValidator_ImplementsFieldValidator verifies NumberValidator
// implements FieldValidator.
func TestNumberValidator_ImplementsFieldValidator(t *testing.T) {
	// This test verifies NumberValidator implements FieldValidator
	validator := &NumberValidator{}
	var _ FieldValidator = validator // This will fail if NumberValidator doesn't implement interface
}

// TestDateValidator_ImplementsFieldValidator verifies DateValidator implements
// FieldValidator.
func TestDateValidator_ImplementsFieldValidator(t *testing.T) {
	// This test verifies DateValidator implements FieldValidator
	validator := &DateValidator{}
	var _ FieldValidator = validator // This will fail if DateValidator doesn't implement interface
}

// TestBoolValidator_ImplementsFieldValidator verifies BoolValidator implements
// FieldValidator.
func TestBoolValidator_ImplementsFieldValidator(t *testing.T) {
	// This test verifies BoolValidator implements FieldValidator
	validator := &BoolValidator{}
	var _ FieldValidator = validator // This will fail if BoolValidator doesn't implement interface
}

// TestStringValidator_ValidateMethod verifies StringValidator validation logic.
func TestStringValidator_ValidateMethod(t *testing.T) {
	validator := &StringValidator{}
	stringSpec := &domain.StringSpec{
		Enum: []string{"allowed", "valid"},
	}

	// This will fail until validation logic is implemented
	err := validator.Validate("testField", "allowed", stringSpec)
	require.NoError(t, err, "Valid enum value should pass validation")

	err = validator.Validate("testField", "invalid", stringSpec)
	require.Error(t, err, "Invalid enum value should fail validation")

	// Test non-string value
	err = validator.Validate("testField", 123, stringSpec)
	require.Error(t, err, "Non-string value should fail validation")

	// Test wrong spec type
	wrongSpec := &domain.NumberSpec{}
	err = validator.Validate("testField", "valid", wrongSpec)
	require.Error(t, err, "Wrong spec type should fail validation")

	// Test string without enum (should pass)
	noEnumSpec := &domain.StringSpec{}
	err = validator.Validate("testField", "any string", noEnumSpec)
	require.NoError(t, err, "String without enum constraints should pass")
}

// TestNumberValidator_ValidateMethod verifies NumberValidator validation logic.
func TestNumberValidator_ValidateMethod(t *testing.T) {
	validator := &NumberValidator{}
	minValue := 0.0
	maxValue := 100.0
	numberSpec := &domain.NumberSpec{
		Min: &minValue,
		Max: &maxValue,
	}

	// This will fail until validation logic is implemented
	err := validator.Validate("testField", 50.0, numberSpec)
	require.NoError(t, err, "Value within range should pass validation")

	err = validator.Validate("testField", 150.0, numberSpec)
	require.Error(t, err, "Value above max should fail validation")

	// Test different numeric types
	err = validator.Validate("testField", int(25), numberSpec)
	require.NoError(t, err, "Int value should pass validation")

	err = validator.Validate("testField", int64(25), numberSpec)
	require.NoError(t, err, "Int64 value should pass validation")

	err = validator.Validate("testField", float32(25.5), numberSpec)
	require.NoError(t, err, "Float32 value should pass validation")

	// Test non-numeric value
	err = validator.Validate("testField", "not a number", numberSpec)
	require.Error(t, err, "Non-numeric value should fail validation")

	// Test wrong spec type
	wrongSpec := &domain.StringSpec{}
	err = validator.Validate("testField", 25.0, wrongSpec)
	require.Error(t, err, "Wrong spec type should fail validation")
}

// TestBoolValidator_ValidateMethod verifies BoolValidator validation logic.
func TestBoolValidator_ValidateMethod(t *testing.T) {
	validator := &BoolValidator{}
	boolSpec := &domain.BoolSpec{}

	// This will fail until validation logic is implemented
	err := validator.Validate("testField", true, boolSpec)
	require.NoError(t, err, "Boolean value should pass validation")

	err = validator.Validate("testField", "not a bool", boolSpec)
	require.Error(t, err, "Non-boolean value should fail validation")

	// Test false value
	err = validator.Validate("testField", false, boolSpec)
	require.NoError(t, err, "False boolean should pass validation")

	// Test wrong spec type
	wrongSpec := &domain.StringSpec{}
	err = validator.Validate("testField", true, wrongSpec)
	require.Error(t, err, "Wrong spec type should fail validation")
}

// TestDateValidator_ValidateMethod verifies DateValidator validation logic.
func TestDateValidator_ValidateMethod(t *testing.T) {
	validator := &DateValidator{}
	dateSpec := &domain.DateSpec{
		Format: "2006-01-02", // YYYY-MM-DD format
	}

	// Test valid date
	err := validator.Validate("testField", "2023-12-01", dateSpec)
	require.NoError(t, err, "Valid date should pass validation")

	// Test invalid date format
	err = validator.Validate("testField", "invalid-date", dateSpec)
	require.Error(t, err, "Invalid date format should fail validation")

	// Test non-string value
	err = validator.Validate("testField", 123, dateSpec)
	require.Error(t, err, "Non-string value should fail validation")

	// Test with default RFC3339 format
	defaultDateSpec := &domain.DateSpec{}
	err = validator.Validate(
		"testField",
		"2023-12-01T10:15:30Z",
		defaultDateSpec,
	)
	require.NoError(t, err, "Valid RFC3339 date should pass validation")

	// Test wrong spec type
	wrongSpec := &domain.StringSpec{}
	err = validator.Validate("testField", "2023-12-01", wrongSpec)
	require.Error(t, err, "Wrong spec type should fail validation")
}
