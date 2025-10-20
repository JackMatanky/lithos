package domain

import "testing"

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

func TestPropertyValidate(t *testing.T) {
	tests := []struct {
		name    string
		prop    Property
		wantErr bool
	}{
		{
			name: "valid property",
			prop: Property{
				Name:     "valid_name",
				Required: true,
				Array:    false,
				Spec:     StringPropertySpec{},
			},
			wantErr: false,
		},
		{
			name: "empty name",
			prop: Property{
				Name:     "",
				Required: true,
				Array:    false,
				Spec:     StringPropertySpec{},
			},
			wantErr: true,
		},
		{
			name: "invalid name characters",
			prop: Property{
				Name:     "invalid name",
				Required: true,
				Array:    false,
				Spec:     StringPropertySpec{},
			},
			wantErr: true,
		},
		{
			name: "nil spec",
			prop: Property{
				Name:     "test",
				Required: true,
				Array:    false,
				Spec:     nil,
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.prop.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"Property.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
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

// unknownSpec is used for testing unknown spec types.
type unknownSpec struct{}

func (unknownSpec) Validate(interface{}) error { return nil }

func TestPropertyTypeNameUnknownSpec(t *testing.T) {
	prop := NewProperty("custom", false, false, unknownSpec{})
	if err := prop.Validate(); err == nil {
		t.Error("Validate() expected error for unknown spec type")
	}
}

func TestPropertyValidateValue(t *testing.T) {
	stringProp := NewProperty("tags", false, true, StringPropertySpec{})
	if err := stringProp.ValidateValue([]interface{}{"a", "b"}); err != nil {
		t.Fatalf("ValidateValue() error = %v, want nil", err)
	}

	// Nil value is allowed for array properties (handled by presence checks
	// elsewhere)
	if err := stringProp.ValidateValue(nil); err != nil {
		t.Fatalf("ValidateValue(nil) error = %v, want nil", err)
	}

	if err := stringProp.ValidateValue("not-an-array"); err == nil {
		t.Fatal("ValidateValue() expected error for non-array value")
	}

	if err := stringProp.ValidateValue([]interface{}{"a", 10}); err == nil {
		t.Fatal("ValidateValue() expected error for mixed element types")
	}

	scalarProp := NewProperty("title", true, false, StringPropertySpec{})
	if err := scalarProp.ValidateValue("hello"); err != nil {
		t.Fatalf("ValidateValue() error = %v, want nil", err)
	}

	if err := scalarProp.ValidateValue(42); err == nil {
		t.Fatal("ValidateValue() expected error for invalid scalar type")
	}
}
