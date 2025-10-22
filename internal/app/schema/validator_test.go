package schema

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
)

func TestSchemaValidator_ValidateSchema(t *testing.T) {
	tests := []struct {
		name         string
		schema       domain.Schema
		expectValid  bool
		expectErrors int
	}{
		{
			name: "valid schema passes validation",
			schema: domain.NewSchema("note", []domain.Property{
				domain.NewProperty(
					"title",
					true,
					false,
					domain.StringPropertySpec{},
				),
				domain.NewProperty(
					"author",
					false,
					false,
					domain.StringPropertySpec{},
				),
			}),
			expectValid:  true,
			expectErrors: 0,
		},
		{
			name: "empty schema name fails validation",
			schema: domain.NewSchema("", []domain.Property{
				domain.NewProperty(
					"title",
					true,
					false,
					domain.StringPropertySpec{},
				),
			}),
			expectValid:  false,
			expectErrors: 1,
		},
		{
			name: "invalid schema name fails validation",
			schema: domain.NewSchema("invalid-name!", []domain.Property{
				domain.NewProperty(
					"title",
					true,
					false,
					domain.StringPropertySpec{},
				),
			}),
			expectValid:  false,
			expectErrors: 1,
		},
		{
			name: "self-referencing extends fails validation",
			schema: domain.Schema{
				Name:    "note",
				Extends: "note",
				Properties: []domain.Property{
					domain.NewProperty(
						"title",
						true,
						false,
						domain.StringPropertySpec{},
					),
				},
			},
			expectValid:  false,
			expectErrors: 1,
		},
		{
			name: "duplicate properties fail validation",
			schema: domain.NewSchema("note", []domain.Property{
				domain.NewProperty(
					"title",
					true,
					false,
					domain.StringPropertySpec{},
				),
				domain.NewProperty(
					"title",
					false,
					false,
					domain.StringPropertySpec{},
				),
			}),
			expectValid:  false,
			expectErrors: 1,
		},
		{
			name: "duplicate excludes fail validation",
			schema: domain.Schema{
				Name:     "note",
				Extends:  "base",
				Excludes: []string{"field1", "field1"},
				Properties: []domain.Property{
					domain.NewProperty(
						"title",
						true,
						false,
						domain.StringPropertySpec{},
					),
				},
			},
			expectValid:  false,
			expectErrors: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			validator := NewSchemaValidator()
			ctx := context.Background()

			// Execute
			result := validator.ValidateSchema(ctx, &tt.schema)

			// Verify
			if result.IsErr() {
				t.Fatalf("unexpected error: %v", result.Error())
			}

			validationResult := result.Value()
			if validationResult.IsValid() != tt.expectValid {
				t.Errorf(
					"expected valid=%v, got valid=%v",
					tt.expectValid,
					validationResult.IsValid(),
				)
			}

			if len(validationResult.Errors) != tt.expectErrors {
				t.Errorf(
					"expected %d errors, got %d",
					tt.expectErrors,
					len(validationResult.Errors),
				)
			}
		})
	}
}

func TestSchemaValidator_ValidatePropertyBank(t *testing.T) {
	tests := []struct {
		name         string
		propertyBank *domain.PropertyBank
		expectValid  bool
		expectErrors int
	}{
		{
			name: "valid property bank passes validation",
			propertyBank: func() *domain.PropertyBank {
				pb := domain.NewPropertyBank("schemas/properties/")
				_ = pb.RegisterProperty(
					"title",
					domain.NewProperty(
						"title",
						true,
						false,
						domain.StringPropertySpec{},
					),
				)
				_ = pb.RegisterProperty(
					"author",
					domain.NewProperty(
						"author",
						false,
						false,
						domain.StringPropertySpec{},
					),
				)
				return &pb
			}(),
			expectValid:  true,
			expectErrors: 0,
		},
		{
			name: "empty location passes validation",
			propertyBank: func() *domain.PropertyBank {
				pb := domain.NewPropertyBank("")
				_ = pb.RegisterProperty(
					"title",
					domain.NewProperty(
						"title",
						true,
						false,
						domain.StringPropertySpec{},
					),
				)
				return &pb
			}(),
			expectValid:  true,
			expectErrors: 0,
		},
		{
			name: "property bank with invalid property passes validation",
			propertyBank: func() *domain.PropertyBank {
				pb := domain.NewPropertyBank("schemas/properties/")
				invalidProp := domain.NewProperty(
					"",
					true,
					false,
					domain.StringPropertySpec{},
				) // empty name
				_ = pb.RegisterProperty("invalid", invalidProp)
				return &pb
			}(),
			expectValid:  true,
			expectErrors: 0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			validator := NewSchemaValidator()
			ctx := context.Background()

			// Execute
			result := validator.ValidatePropertyBank(ctx, tt.propertyBank)

			// Verify
			if result.IsErr() {
				t.Fatalf("unexpected error: %v", result.Error())
			}

			validationResult := result.Value()
			if validationResult.IsValid() != tt.expectValid {
				t.Errorf(
					"expected valid=%v, got valid=%v",
					tt.expectValid,
					validationResult.IsValid(),
				)
			}

			if len(validationResult.Errors) != tt.expectErrors {
				t.Errorf(
					"expected %d errors, got %d",
					tt.expectErrors,
					len(validationResult.Errors),
				)
			}
		})
	}
}

func TestSchemaValidator_ValidateProperty(t *testing.T) {
	tests := []struct {
		name         string
		property     domain.Property
		expectValid  bool
		expectErrors int
	}{
		{
			name: "valid property passes validation",
			property: domain.NewProperty(
				"title",
				true,
				false,
				domain.StringPropertySpec{},
			),
			expectValid:  true,
			expectErrors: 0,
		},
		{
			name: "empty name fails validation",
			property: domain.NewProperty(
				"",
				true,
				false,
				domain.StringPropertySpec{},
			),
			expectValid:  false,
			expectErrors: 1,
		},
		{
			name: "invalid name fails validation",
			property: domain.NewProperty(
				"invalid-name!",
				true,
				false,
				domain.StringPropertySpec{},
			),
			expectValid:  false,
			expectErrors: 1,
		},
		{
			name: "nil spec fails validation",
			property: domain.Property{
				Name:     "title",
				Required: true,
				Array:    false,
				Spec:     nil,
			},
			expectValid:  false,
			expectErrors: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			validator := NewSchemaValidator()
			ctx := context.Background()

			// Execute
			result := validator.ValidateProperty(ctx, tt.property)

			// Verify
			if result.IsErr() {
				t.Fatalf("unexpected error: %v", result.Error())
			}

			validationResult := result.Value()
			if validationResult.IsValid() != tt.expectValid {
				t.Errorf(
					"expected valid=%v, got valid=%v",
					tt.expectValid,
					validationResult.IsValid(),
				)
			}

			if len(validationResult.Errors) != tt.expectErrors {
				t.Errorf(
					"expected %d errors, got %d",
					tt.expectErrors,
					len(validationResult.Errors),
				)
			}
		})
	}
}

func TestSchemaValidator_ValidatePropertyValue(t *testing.T) {
	tests := []struct {
		name         string
		property     domain.Property
		value        interface{}
		expectValid  bool
		expectErrors int
	}{
		{
			name: "valid string value passes validation",
			property: domain.NewProperty(
				"title",
				true,
				false,
				domain.StringPropertySpec{},
			),
			value:        "Test Title",
			expectValid:  true,
			expectErrors: 0,
		},
		{
			name: "nil spec fails validation",
			property: domain.Property{
				Name:     "title",
				Required: true,
				Array:    false,
				Spec:     nil,
			},
			value:        "test",
			expectValid:  false,
			expectErrors: 1,
		},
		{
			name: "array property with array value passes validation",
			property: domain.NewProperty(
				"tags",
				false,
				true,
				domain.StringPropertySpec{},
			),
			value:        []interface{}{"tag1", "tag2"},
			expectValid:  true,
			expectErrors: 0,
		},
		{
			name: "array property with scalar value fails validation",
			property: domain.NewProperty(
				"tags",
				false,
				true,
				domain.StringPropertySpec{},
			),
			value:        "single-tag",
			expectValid:  false,
			expectErrors: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			validator := NewSchemaValidator()
			ctx := context.Background()

			// Execute
			result := validator.ValidatePropertyValue(
				ctx,
				tt.property,
				tt.value,
			)

			// Verify
			if result.IsErr() {
				t.Fatalf("unexpected error: %v", result.Error())
			}

			validationResult := result.Value()
			if validationResult.IsValid() != tt.expectValid {
				t.Errorf(
					"expected valid=%v, got valid=%v",
					tt.expectValid,
					validationResult.IsValid(),
				)
			}

			if len(validationResult.Errors) != tt.expectErrors {
				t.Errorf(
					"expected %d errors, got %d",
					tt.expectErrors,
					len(validationResult.Errors),
				)
			}
		})
	}
}
