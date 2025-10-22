package schema

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// mockSchemaRegistry implements spi.SchemaRegistryPort for testing.
type mockSchemaRegistry struct {
	schemas map[string]domain.Schema
}

// testCase represents a test case for validation testing.
type testCase struct {
	name           string
	schemaName     string
	frontmatter    domain.Frontmatter
	setupRegistry  func(*mockSchemaRegistry)
	expectValid    bool
	expectErrors   int
	expectedFields []string
}

func newMockSchemaRegistry() *mockSchemaRegistry {
	return &mockSchemaRegistry{
		schemas: make(map[string]domain.Schema),
	}
}

func (m *mockSchemaRegistry) Get(name string) (domain.Schema, bool) {
	schema, exists := m.schemas[name]
	return schema, exists
}

func (m *mockSchemaRegistry) addSchema(schema *domain.Schema) {
	m.schemas[schema.Name] = *schema
}

// verifyValidationResult checks the result of a validation against expected
// outcomes.
func verifyValidationResult(
	t *testing.T,
	result lithoserrors.Result[lithoserrors.ValidationResult],
	tt *testCase,
) {
	if result.IsErr() {
		var schemaNotFoundError lithoserrors.SchemaNotFoundError
		if !errors.As(result.Error(), &schemaNotFoundError) {
			t.Fatalf("unexpected error: %v", result.Error())
		}
		if tt.expectErrors != 0 {
			t.Errorf(
				"expected validation errors, but got schema not found error",
			)
		}
		return
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

	// Check expected fields in errors
	if len(tt.expectedFields) > 0 {
		errorFields := make(map[string]bool)
		for _, err := range validationResult.Errors {
			errorFields[err.Field()] = true
		}

		for _, expectedField := range tt.expectedFields {
			if !errorFields[expectedField] {
				t.Errorf(
					"expected error for field %s, but not found",
					expectedField,
				)
			}
		}
	}
}

func TestSchemaValidator_Validate(t *testing.T) {
	tests := []testCase{
		{
			name:       "valid frontmatter passes validation",
			schemaName: "note",
			frontmatter: domain.Frontmatter{
				FileClass: "note",
				Fields: map[string]interface{}{
					"title":  "Test Note",
					"author": "Test Author",
				},
			},
			setupRegistry: func(m *mockSchemaRegistry) {
				schema := domain.NewSchema("note", []domain.Property{
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
				})
				m.addSchema(&schema)
			},
			expectValid:    true,
			expectErrors:   0,
			expectedFields: nil,
		},
		{
			name:       "missing required field fails validation",
			schemaName: "note",
			frontmatter: domain.Frontmatter{
				FileClass: "note",
				Fields:    map[string]interface{}{},
			},
			setupRegistry: func(m *mockSchemaRegistry) {
				schema := domain.NewSchema("note", []domain.Property{
					domain.NewProperty(
						"title",
						true,
						false,
						domain.StringPropertySpec{},
					),
				})
				m.addSchema(&schema)
			},
			expectValid:    false,
			expectErrors:   1,
			expectedFields: []string{"title"},
		},
		{
			name:       "array constraint violation fails validation",
			schemaName: "note",
			frontmatter: domain.Frontmatter{
				FileClass: "note",
				Fields: map[string]interface{}{
					"tags": "single-tag", // should be array
				},
			},
			setupRegistry: func(m *mockSchemaRegistry) {
				schema := domain.NewSchema("note", []domain.Property{
					domain.NewProperty(
						"tags",
						false,
						true,
						domain.StringPropertySpec{},
					),
				})
				m.addSchema(&schema)
			},
			expectValid:    false,
			expectErrors:   1,
			expectedFields: []string{"tags"},
		},
		{
			name:       "schema not found fails validation",
			schemaName: "nonexistent",
			frontmatter: domain.Frontmatter{
				FileClass: "nonexistent",
				Fields:    map[string]interface{}{},
			},
			setupRegistry:  func(*mockSchemaRegistry) {},
			expectValid:    false,
			expectErrors:   0, // SchemaNotFoundError is returned as error, not in ValidationResult
			expectedFields: nil,
		},
		{
			name:       "fileClass mismatch fails validation",
			schemaName: "note",
			frontmatter: domain.Frontmatter{
				FileClass: "different",
				Fields:    map[string]interface{}{},
			},
			setupRegistry: func(m *mockSchemaRegistry) {
				schema := domain.NewSchema("note", []domain.Property{
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
				})
				m.addSchema(&schema)
			},
			expectValid:    false,
			expectErrors:   1,
			expectedFields: []string{"fileClass"},
		},
		{
			name:       "PropertySpec validation error",
			schemaName: "note",
			frontmatter: domain.Frontmatter{
				FileClass: "note",
				Fields: map[string]interface{}{
					"priority": "invalid", // enum should be "high", "medium", "low"
				},
			},
			setupRegistry: func(m *mockSchemaRegistry) {
				schema := domain.NewSchema("note", []domain.Property{
					domain.NewProperty(
						"priority",
						false,
						false,
						domain.StringPropertySpec{
							Enum: []string{"high", "medium", "low"},
						},
					),
				})
				m.addSchema(&schema)
			},
			expectValid:    false,
			expectErrors:   1,
			expectedFields: []string{"priority"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			registry := newMockSchemaRegistry()
			tt.setupRegistry(registry)
			validator := NewSchemaValidator(registry)
			ctx := context.Background()

			// Execute
			result := validator.Validate(ctx, tt.schemaName, tt.frontmatter)

			// Verify
			verifyValidationResult(t, result, &tt)
		})
	}
}

func TestSchemaValidator_PropertySpecTypes(t *testing.T) {
	tests := []struct {
		name        string
		property    domain.Property
		fieldValue  interface{}
		expectValid bool
	}{
		{
			name: "StringPropertySpec with enum - valid",
			property: domain.NewProperty(
				"status",
				false,
				false,
				domain.StringPropertySpec{
					Enum: []string{"draft", "published"},
				},
			),
			fieldValue:  "draft",
			expectValid: true,
		},
		{
			name: "StringPropertySpec with enum - invalid",
			property: domain.NewProperty(
				"status",
				false,
				false,
				domain.StringPropertySpec{
					Enum: []string{"draft", "published"},
				},
			),
			fieldValue:  "invalid",
			expectValid: false,
		},
		{
			name: "NumberPropertySpec - valid integer",
			property: domain.NewProperty(
				"count",
				false,
				false,
				domain.NumberPropertySpec{
					Min:  &[]float64{0}[0],
					Max:  &[]float64{100}[0],
					Step: &[]float64{1}[0], // integer
				},
			),
			fieldValue:  42.0,
			expectValid: true,
		},
		{
			name: "NumberPropertySpec - invalid range",
			property: domain.NewProperty(
				"count",
				false,
				false,
				domain.NumberPropertySpec{
					Min: &[]float64{0}[0],
					Max: &[]float64{100}[0],
				},
			),
			fieldValue:  150.0,
			expectValid: false,
		},
		{
			name: "DatePropertySpec - valid RFC3339",
			property: domain.NewProperty(
				"created",
				false,
				false,
				domain.DatePropertySpec{
					Format: time.RFC3339,
				},
			),
			fieldValue:  "2023-01-01T12:00:00Z",
			expectValid: true,
		},
		{
			name: "DatePropertySpec - invalid format",
			property: domain.NewProperty(
				"created",
				false,
				false,
				domain.DatePropertySpec{
					Format: time.RFC3339,
				},
			),
			fieldValue:  "invalid-date",
			expectValid: false,
		},
		{
			name: "BoolPropertySpec - valid",
			property: domain.NewProperty(
				"published",
				false,
				false,
				domain.BoolPropertySpec{},
			),
			fieldValue:  true,
			expectValid: true,
		},
		{
			name: "BoolPropertySpec - invalid",
			property: domain.NewProperty(
				"published",
				false,
				false,
				domain.BoolPropertySpec{},
			),
			fieldValue:  "true",
			expectValid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			registry := newMockSchemaRegistry()
			schema := domain.NewSchema("test", []domain.Property{tt.property})
			registry.addSchema(&schema)

			validator := NewSchemaValidator(registry)
			frontmatter := domain.Frontmatter{
				FileClass: "test",
				Fields: map[string]interface{}{
					tt.property.Name: tt.fieldValue,
				},
			}

			ctx := context.Background()

			// Execute
			result := validator.Validate(ctx, "test", frontmatter)

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
				for _, err := range validationResult.Errors {
					t.Logf("Validation error: %v", err)
				}
			}
		})
	}
}

func TestSchemaValidator_ContextCancellation(t *testing.T) {
	// Setup
	registry := newMockSchemaRegistry()
	schema := domain.NewSchema("test", []domain.Property{
		domain.NewProperty("field1", false, false, domain.StringPropertySpec{}),
		domain.NewProperty("field2", false, false, domain.StringPropertySpec{}),
		domain.NewProperty("field3", false, false, domain.StringPropertySpec{}),
	})
	registry.addSchema(&schema)

	validator := NewSchemaValidator(registry)
	frontmatter := domain.Frontmatter{
		FileClass: "test",
		Fields: map[string]interface{}{
			"field1": "value1",
			"field2": "value2",
			"field3": "value3",
		},
	}

	// Test with already canceled context
	cancelledCtx, cancel := context.WithCancel(context.Background())
	cancel() // Cancel immediately

	result := validator.Validate(cancelledCtx, "test", frontmatter)

	if !result.IsErr() {
		t.Error("expected error due to canceled context, but got success")
	}

	if !errors.Is(result.Error(), context.Canceled) {
		t.Errorf("expected context.Canceled, got %v", result.Error())
	}
}

func TestSchemaValidator_InheritanceValidation(t *testing.T) {
	// Setup schema with resolved properties (simulating inheritance)
	registry := newMockSchemaRegistry()

	// Simulate resolved properties after inheritance
	childSchema := domain.NewSchema("child", []domain.Property{
		domain.NewProperty(
			"title",
			true,
			false,
			domain.StringPropertySpec{},
		), // inherited
		domain.NewProperty("category", false, false, domain.StringPropertySpec{
			Enum: []string{"tech", "business"},
		}), // child-specific
	})

	// Set resolved properties to simulate inheritance resolution
	childSchema.SetResolvedProperties([]domain.Property{
		domain.NewProperty("title", true, false, domain.StringPropertySpec{}),
		domain.NewProperty("category", false, false, domain.StringPropertySpec{
			Enum: []string{"tech", "business"},
		}),
	})

	registry.addSchema(&childSchema)

	validator := NewSchemaValidator(registry)

	tests := []struct {
		name        string
		frontmatter domain.Frontmatter
		expectValid bool
	}{
		{
			name: "valid inherited and child properties",
			frontmatter: domain.Frontmatter{
				FileClass: "child",
				Fields: map[string]interface{}{
					"title":    "Test Title",
					"category": "tech",
				},
			},
			expectValid: true,
		},
		{
			name: "missing inherited required field",
			frontmatter: domain.Frontmatter{
				FileClass: "child",
				Fields: map[string]interface{}{
					"category": "tech",
				},
			},
			expectValid: false,
		},
		{
			name: "invalid child property enum",
			frontmatter: domain.Frontmatter{
				FileClass: "child",
				Fields: map[string]interface{}{
					"title":    "Test Title",
					"category": "invalid",
				},
			},
			expectValid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			result := validator.Validate(ctx, "child", tt.frontmatter)

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
		})
	}
}

func TestSchemaValidator_ArrayConstraints(t *testing.T) {
	tests := []struct {
		name        string
		property    domain.Property
		fieldValue  interface{}
		expectValid bool
	}{
		{
			name: "array property with array value - valid",
			property: domain.NewProperty(
				"tags",
				false,
				true,
				domain.StringPropertySpec{},
			),
			fieldValue:  []interface{}{"tag1", "tag2"},
			expectValid: true,
		},
		{
			name: "array property with scalar value - invalid",
			property: domain.NewProperty(
				"tags",
				false,
				true,
				domain.StringPropertySpec{},
			),
			fieldValue:  "single-tag",
			expectValid: false,
		},
		{
			name: "scalar property with scalar value - valid",
			property: domain.NewProperty(
				"title",
				false,
				false,
				domain.StringPropertySpec{},
			),
			fieldValue:  "Test Title",
			expectValid: true,
		},
		{
			name: "scalar property with array value - invalid",
			property: domain.NewProperty(
				"title",
				false,
				false,
				domain.StringPropertySpec{},
			),
			fieldValue:  []interface{}{"title1", "title2"},
			expectValid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			registry := newMockSchemaRegistry()
			schema := domain.NewSchema("test", []domain.Property{tt.property})
			registry.addSchema(&schema)

			validator := NewSchemaValidator(registry)
			frontmatter := domain.Frontmatter{
				FileClass: "test",
				Fields: map[string]interface{}{
					tt.property.Name: tt.fieldValue,
				},
			}

			ctx := context.Background()

			// Execute
			result := validator.Validate(ctx, "test", frontmatter)

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
		})
	}
}
