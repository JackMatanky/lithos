package schema

import (
	"errors"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	sharederrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

func TestPropertySpecConversion_String(t *testing.T) {
	// Test string property conversion
	specMap := map[string]interface{}{
		"pattern": "^test.*",
		"enum":    []interface{}{"value1", "value2"},
	}

	specInterface, err := buildSpecFromMap("string", specMap)
	if err != nil {
		t.Fatalf("buildSpecFromMap failed: %v", err)
	}

	spec, ok := specInterface.(domain.StringPropertySpec)
	if !ok {
		t.Fatalf("Expected StringPropertySpec, got %T", specInterface)
	}

	if spec.Pattern != "^test.*" {
		t.Errorf("Expected pattern '^test.*', got '%s'", spec.Pattern)
	}
	if len(spec.Enum) != 2 {
		t.Errorf("Expected 2 enum values, got %d", len(spec.Enum))
	}
	if spec.Enum[0] != "value1" || spec.Enum[1] != "value2" {
		t.Errorf("Expected enum ['value1', 'value2'], got %v", spec.Enum)
	}
}

func TestPropertySpecConversion_Number(t *testing.T) {
	// Test number property conversion
	specMap := map[string]interface{}{
		"min":  0.0,
		"max":  100.0,
		"step": 1.0,
	}

	specInterface, err := buildSpecFromMap("number", specMap)
	if err != nil {
		t.Fatalf("buildSpecFromMap failed: %v", err)
	}

	spec, ok := specInterface.(domain.NumberPropertySpec)
	if !ok {
		t.Fatalf("Expected NumberPropertySpec, got %T", specInterface)
	}

	if spec.Min == nil || *spec.Min != 0.0 {
		t.Errorf("Expected min 0.0, got %v", spec.Min)
	}
	if spec.Max == nil || *spec.Max != 100.0 {
		t.Errorf("Expected max 100.0, got %v", spec.Max)
	}
	if spec.Step == nil || *spec.Step != 1.0 {
		t.Errorf("Expected step 1.0, got %v", spec.Step)
	}
}

func TestPropertySpecConversion_Date(t *testing.T) {
	// Test date property conversion
	specMap := map[string]interface{}{
		"format": "RFC3339",
	}

	specInterface, err := buildSpecFromMap("date", specMap)
	if err != nil {
		t.Fatalf("buildSpecFromMap failed: %v", err)
	}

	spec, ok := specInterface.(domain.DatePropertySpec)
	if !ok {
		t.Fatalf("Expected DatePropertySpec, got %T", specInterface)
	}

	if spec.Format != "RFC3339" {
		t.Errorf("Expected format 'RFC3339', got '%s'", spec.Format)
	}
}

func TestPropertySpecConversion_File(t *testing.T) {
	// Test file property conversion
	specMap := map[string]interface{}{
		"fileClass": "document",
		"directory": "/uploads",
	}

	specInterface, err := buildSpecFromMap("file", specMap)
	if err != nil {
		t.Fatalf("buildSpecFromMap failed: %v", err)
	}

	spec, ok := specInterface.(domain.FilePropertySpec)
	if !ok {
		t.Fatalf("Expected FilePropertySpec, got %T", specInterface)
	}

	if spec.FileClass != "document" {
		t.Errorf("Expected fileClass 'document', got '%s'", spec.FileClass)
	}
	if spec.Directory != "/uploads" {
		t.Errorf("Expected directory '/uploads', got '%s'", spec.Directory)
	}
}

func TestPropertySpecConversion_Bool(t *testing.T) {
	// Test bool property conversion
	specMap := map[string]interface{}{}

	specInterface, err := buildSpecFromMap("bool", specMap)
	if err != nil {
		t.Fatalf("buildSpecFromMap failed: %v", err)
	}

	_, ok := specInterface.(domain.BoolPropertySpec)
	if !ok {
		t.Fatalf("Expected BoolPropertySpec, got %T", specInterface)
	}
}

func TestPropertySpecConversion_UnknownType(t *testing.T) {
	adapter, _, _ := createTestAdapter()

	// Test unknown property type
	propDTO := propertyDTO{
		Type: "unknown",
		Spec: map[string]interface{}{},
	}

	_, err := adapter.convertPropertySpecToConcreteType(propDTO)
	if err == nil {
		t.Fatal("Expected error for unknown property type")
	}

	var validationErr sharederrors.ValidationError
	if !errors.As(err, &validationErr) {
		t.Errorf("Expected ValidationError, got %T", err)
	}
}

func TestRefResolution_SimpleReference(t *testing.T) {
	adapter, _, _ := createTestAdapter()

	// Test simple $ref extraction
	ref := "#/properties/common-email"
	propertyName := adapter.extractRefPropertyName(ref)

	if propertyName != "common-email" {
		t.Errorf("Expected 'common-email', got '%s'", propertyName)
	}
}

func TestRefResolution_CrossFileReference(t *testing.T) {
	adapter, _, _ := createTestAdapter()

	// Test cross-file $ref extraction
	ref := "common.json#/properties/timestamp"
	propertyName := adapter.extractRefPropertyName(ref)

	if propertyName != "timestamp" {
		t.Errorf("Expected 'timestamp', got '%s'", propertyName)
	}
}

func TestRefResolution_InvalidReference(t *testing.T) {
	adapter, _, _ := createTestAdapter()

	// Test invalid $ref format
	ref := "invalid-reference"
	propertyName := adapter.extractRefPropertyName(ref)

	if propertyName != "" {
		t.Errorf(
			"Expected empty string for invalid ref, got '%s'",
			propertyName,
		)
	}
}
