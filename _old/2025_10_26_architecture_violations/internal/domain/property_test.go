package domain

import (
	"fmt"
	"sync"
	"testing"
)

const testPropertyName = "test_prop"

func TestNewProperty(t *testing.T) {
	spec := StringPropertySpec{}
	prop := NewProperty(testPropertyName, true, false, spec)

	if prop.Name != testPropertyName {
		t.Errorf("Name = %q, want %q", prop.Name, testPropertyName)
	}
	if !prop.Required {
		t.Errorf("Required = %v, want %v", prop.Required, true)
	}
	if prop.Array {
		t.Errorf("Array = %v, want %v", prop.Array, false)
	}
	if prop.Spec == nil {
		t.Error("Spec should not be nil")
	}

	typeName, err := prop.TypeName()
	if err != nil {
		t.Fatalf("TypeName() error = %v", err)
	}
	if typeName != propertyTypeString {
		t.Errorf("TypeName() = %q, want %q", typeName, propertyTypeString)
	}
}

func TestPropertyTypeNamePointerSpec(t *testing.T) {
	spec := &NumberPropertySpec{}
	prop := NewProperty("count", false, false, spec)

	typeName, err := prop.TypeName()
	if err != nil {
		t.Fatalf("TypeName() error = %v", err)
	}
	if typeName != propertyTypeNumber {
		t.Errorf("TypeName() = %q, want %q", typeName, propertyTypeNumber)
	}
}

func TestNewPropertyBank(t *testing.T) {
	tests := []struct {
		name         string
		location     string
		wantLocation string
	}{
		{
			name:         "empty location uses default",
			location:     "",
			wantLocation: "schemas/properties/",
		},
		{
			name:         "custom location",
			location:     "custom/path/",
			wantLocation: "custom/path/",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := NewPropertyBank(tt.location)
			if got.Location != tt.wantLocation {
				t.Errorf(
					"NewPropertyBank() location = %v, want %v",
					got.Location,
					tt.wantLocation,
				)
			}
			if len(got.Properties) != 0 {
				t.Errorf(
					"NewPropertyBank() properties should be empty, got %v",
					got.Properties,
				)
			}
		})
	}
}

func TestPropertyBank_RegisterProperty(t *testing.T) {
	pb := NewPropertyBank("")

	stringProp := NewProperty("title", true, false, StringPropertySpec{
		Enum: []string{"Mr", "Mrs", "Dr"},
	})

	tests := []struct {
		name      string
		regName   string
		property  Property
		wantError bool
	}{
		{
			name:      "valid registration",
			regName:   "another_title",
			property:  stringProp,
			wantError: false,
		},
		{
			name:      "empty name",
			regName:   "",
			property:  stringProp,
			wantError: true,
		},
		{
			name:      "whitespace name",
			regName:   "  test  ",
			property:  stringProp,
			wantError: true,
		},
		{
			name:      "duplicate name",
			regName:   "standard_title",
			property:  stringProp,
			wantError: true,
		},
	}

	// Register first property
	err := pb.RegisterProperty("standard_title", stringProp)
	if err != nil {
		t.Fatalf("Failed to register initial property: %v", err)
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			regErr := pb.RegisterProperty(tt.regName, tt.property)
			if (regErr != nil) != tt.wantError {
				t.Errorf(
					"RegisterProperty() error = %v, wantError %v",
					regErr,
					tt.wantError,
				)
			}
		})
	}
}

func TestPropertyBank_RegisterPropertyConcurrent(t *testing.T) {
	pb := NewPropertyBank("")
	baseProp := NewProperty("title", false, false, StringPropertySpec{})

	const total = 20
	var wg sync.WaitGroup

	for i := range total {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			name := fmt.Sprintf("prop_%d", i)
			if err := pb.RegisterProperty(name, baseProp); err != nil {
				t.Errorf("RegisterProperty(%s) error = %v", name, err)
			}
		}(i)
	}

	wg.Wait()

	if got := len(pb.Properties); got != total {
		t.Errorf("len(Properties) = %d, want %d", got, total)
	}
}
