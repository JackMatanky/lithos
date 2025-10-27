// Package frontmatter provides tests for frontmatter validation services.
package frontmatter

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// assertValidation checks validation result against expectation.
func assertValidation[T any](
	t *testing.T,
	result errors.Result[T],
	wantValid bool,
) {
	t.Helper()
	if wantValid {
		if result.IsErr() {
			t.Errorf("expected valid result, got error: %v", result.Error())
		}
		return
	}

	if result.IsOk() {
		t.Errorf("expected error result, got valid")
	}
}

// mockSchemaEngine implements the interface needed for testing.
type mockSchemaEngine struct {
	schema domain.Schema
	err    error
}

func (m *mockSchemaEngine) GetSchema(
	ctx context.Context,
	name string,
) errors.Result[domain.Schema] {
	if m.err != nil {
		return errors.Err[domain.Schema](m.err)
	}
	return errors.Ok[domain.Schema](m.schema)
}

func TestFrontmatterValidator_Validate(t *testing.T) {
	tests := []struct {
		name         string
		schemaName   string
		frontmatter  domain.Frontmatter
		mockSchema   domain.Schema
		mockError    error
		wantValid    bool
		wantErrCount int
	}{
		{
			name:       "valid frontmatter with all required fields",
			schemaName: "test",
			frontmatter: domain.Frontmatter{
				Fields: map[string]interface{}{
					"title": "Test Note",
					"tags":  []string{"test", "validation"},
				},
			},
			mockSchema: domain.Schema{
				Name: "test",
				Properties: []domain.Property{
					{
						Name:     "title",
						Required: true,
						Array:    false,
						Spec:     &domain.StringPropertySpec{},
					},
					{
						Name:     "tags",
						Required: false,
						Array:    true,
						Spec:     &domain.StringPropertySpec{},
					},
				},
				ResolvedProperties: []domain.Property{
					{
						Name:     "title",
						Required: true,
						Array:    false,
						Spec:     &domain.StringPropertySpec{},
					},
					{
						Name:     "tags",
						Required: false,
						Array:    true,
						Spec:     &domain.StringPropertySpec{},
					},
				},
			},
			wantValid:    true,
			wantErrCount: 0,
		},
		{
			name:       "missing required field",
			schemaName: "test",
			frontmatter: domain.Frontmatter{
				Fields: map[string]interface{}{},
			},
			mockSchema: domain.Schema{
				Name: "test",
				Properties: []domain.Property{
					{
						Name:     "title",
						Required: true,
						Array:    false,
						Spec:     &domain.StringPropertySpec{},
					},
				},
				ResolvedProperties: []domain.Property{
					{
						Name:     "title",
						Required: true,
						Array:    false,
						Spec:     &domain.StringPropertySpec{},
					},
				},
			},
			wantValid:    false,
			wantErrCount: 1,
		},
		{
			name:       "schema not found",
			schemaName: "nonexistent",
			frontmatter: domain.Frontmatter{
				Fields: map[string]interface{}{},
			},
			mockError: errors.NewValidationError(
				"schema",
				"not found",
				"nonexistent",
			),
			wantValid:    false,
			wantErrCount: 0, // Different error type
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockEngine := &mockSchemaEngine{
				schema: tt.mockSchema,
				err:    tt.mockError,
			}

			validator := NewFrontmatterValidator(mockEngine)
			result := validator.Validate(
				context.Background(),
				tt.schemaName,
				tt.frontmatter,
			)

			assertValidation(t, result, tt.wantValid)
		})
	}
}

func TestFrontmatterValidator_validateStringPropertySpec(t *testing.T) {
	validator := NewFrontmatterValidator(nil)

	tests := []struct {
		name      string
		fieldName string
		value     interface{}
		spec      *domain.StringPropertySpec
		wantValid bool
	}{
		{
			name:      "valid string",
			fieldName: "title",
			value:     "Test Title",
			spec:      &domain.StringPropertySpec{},
			wantValid: true,
		},
		{
			name:      "invalid type",
			fieldName: "title",
			value:     123,
			spec:      &domain.StringPropertySpec{},
			wantValid: false,
		},
		{
			name:      "enum validation - valid",
			fieldName: "status",
			value:     "draft",
			spec: &domain.StringPropertySpec{
				Enum: []string{"draft", "published"},
			},
			wantValid: true,
		},
		{
			name:      "enum validation - invalid",
			fieldName: "status",
			value:     "invalid",
			spec: &domain.StringPropertySpec{
				Enum: []string{"draft", "published"},
			},
			wantValid: false,
		},
		{
			name:      "pattern validation - valid",
			fieldName: "email",
			value:     "test@example.com",
			spec: &domain.StringPropertySpec{
				Pattern: `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`,
			},
			wantValid: true,
		},
		{
			name:      "pattern validation - invalid",
			fieldName: "email",
			value:     "invalid-email",
			spec: &domain.StringPropertySpec{
				Pattern: `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`,
			},
			wantValid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := validator.validateStringPropertySpec(
				tt.fieldName,
				tt.value,
				tt.spec,
			)

			assertValidation(t, result, tt.wantValid)
		})
	}
}

func TestFrontmatterValidator_validateNumberPropertySpec(t *testing.T) {
	validator := NewFrontmatterValidator(nil)

	minVal := 0.0
	maxVal := 100.0
	stepVal := 1.0

	tests := []struct {
		name      string
		fieldName string
		value     interface{}
		spec      *domain.NumberPropertySpec
		wantValid bool
	}{
		{
			name:      "valid integer",
			fieldName: "count",
			value:     42,
			spec:      &domain.NumberPropertySpec{},
			wantValid: true,
		},
		{
			name:      "valid float",
			fieldName: "price",
			value:     19.99,
			spec:      &domain.NumberPropertySpec{},
			wantValid: true,
		},
		{
			name:      "invalid type",
			fieldName: "count",
			value:     "not-a-number",
			spec:      &domain.NumberPropertySpec{},
			wantValid: false,
		},
		{
			name:      "minimum constraint - valid",
			fieldName: "age",
			value:     25,
			spec:      &domain.NumberPropertySpec{Min: &minVal},
			wantValid: true,
		},
		{
			name:      "minimum constraint - invalid",
			fieldName: "age",
			value:     -5,
			spec:      &domain.NumberPropertySpec{Min: &minVal},
			wantValid: false,
		},
		{
			name:      "maximum constraint - valid",
			fieldName: "percentage",
			value:     85,
			spec:      &domain.NumberPropertySpec{Max: &maxVal},
			wantValid: true,
		},
		{
			name:      "maximum constraint - invalid",
			fieldName: "percentage",
			value:     150,
			spec:      &domain.NumberPropertySpec{Max: &maxVal},
			wantValid: false,
		},
		{
			name:      "step constraint - valid",
			fieldName: "quantity",
			value:     10,
			spec:      &domain.NumberPropertySpec{Step: &stepVal},
			wantValid: true,
		},
		{
			name:      "step constraint - invalid",
			fieldName: "quantity",
			value:     7.5,
			spec:      &domain.NumberPropertySpec{Step: &stepVal},
			wantValid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := validator.validateNumberPropertySpec(
				tt.fieldName,
				tt.value,
				tt.spec,
			)

			assertValidation(t, result, tt.wantValid)
		})
	}
}

func TestFrontmatterValidator_validateDatePropertySpec(t *testing.T) {
	validator := NewFrontmatterValidator(nil)

	tests := []struct {
		name      string
		fieldName string
		value     interface{}
		spec      *domain.DatePropertySpec
		wantValid bool
	}{
		{
			name:      "valid RFC3339 date",
			fieldName: "created",
			value:     "2023-10-22T10:15:00Z",
			spec:      &domain.DatePropertySpec{},
			wantValid: true,
		},
		{
			name:      "invalid type",
			fieldName: "created",
			value:     123456789,
			spec:      &domain.DatePropertySpec{},
			wantValid: false,
		},
		{
			name:      "custom format - valid",
			fieldName: "date",
			value:     "2023-10-22",
			spec:      &domain.DatePropertySpec{Format: "2006-01-02"},
			wantValid: true,
		},
		{
			name:      "custom format - invalid",
			fieldName: "date",
			value:     "10/22/2023",
			spec:      &domain.DatePropertySpec{Format: "2006-01-02"},
			wantValid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := validator.validateDatePropertySpec(
				tt.fieldName,
				tt.value,
				tt.spec,
			)

			assertValidation(t, result, tt.wantValid)
		})
	}
}

func TestFrontmatterValidator_validateFilePropertySpec(t *testing.T) {
	validator := NewFrontmatterValidator(nil)

	tests := []struct {
		name      string
		fieldName string
		value     interface{}
		spec      *domain.FilePropertySpec
		wantValid bool
	}{
		{
			name:      "valid file path",
			fieldName: "attachment",
			value:     "docs/file.md",
			spec:      &domain.FilePropertySpec{},
			wantValid: true,
		},
		{
			name:      "invalid type",
			fieldName: "attachment",
			value:     123,
			spec:      &domain.FilePropertySpec{},
			wantValid: false,
		},
		// File class validation is not implemented in this validator
		// (would require filesystem integration)
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := validator.validateFilePropertySpec(
				tt.fieldName,
				tt.value,
				tt.spec,
			)

			assertValidation(t, result, tt.wantValid)
		})
	}
}

func TestFrontmatterValidator_validateBoolPropertySpec(t *testing.T) {
	validator := NewFrontmatterValidator(nil)

	tests := []struct {
		name      string
		fieldName string
		value     interface{}
		spec      *domain.BoolPropertySpec
		wantValid bool
	}{
		{
			name:      "valid true",
			fieldName: "published",
			value:     true,
			spec:      &domain.BoolPropertySpec{},
			wantValid: true,
		},
		{
			name:      "valid false",
			fieldName: "draft",
			value:     false,
			spec:      &domain.BoolPropertySpec{},
			wantValid: true,
		},
		{
			name:      "invalid type",
			fieldName: "published",
			value:     "yes",
			spec:      &domain.BoolPropertySpec{},
			wantValid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := validator.validateBoolPropertySpec(
				tt.fieldName,
				tt.value,
				tt.spec,
			)

			assertValidation(t, result, tt.wantValid)
		})
	}
}

func TestFrontmatterValidator_validateField(t *testing.T) {
	validator := NewFrontmatterValidator(nil)

	tests := []struct {
		name        string
		frontmatter domain.Frontmatter
		property    domain.Property
		wantValid   bool
	}{
		{
			name: "required field present",
			frontmatter: domain.Frontmatter{
				Fields: map[string]interface{}{"title": "Test"},
			},
			property: domain.Property{
				Name:     "title",
				Required: true,
				Array:    false,
				Spec:     &domain.StringPropertySpec{},
			},
			wantValid: true,
		},
		{
			name: "required field missing",
			frontmatter: domain.Frontmatter{
				Fields: map[string]interface{}{},
			},
			property: domain.Property{
				Name:     "title",
				Required: true,
				Array:    false,
				Spec:     &domain.StringPropertySpec{},
			},
			wantValid: false,
		},
		{
			name: "optional field missing",
			frontmatter: domain.Frontmatter{
				Fields: map[string]interface{}{},
			},
			property: domain.Property{
				Name:     "tags",
				Required: false,
				Array:    true,
				Spec:     &domain.StringPropertySpec{},
			},
			wantValid: true,
		},
		{
			name: "array constraint - valid",
			frontmatter: domain.Frontmatter{
				Fields: map[string]interface{}{"tags": []string{"test"}},
			},
			property: domain.Property{
				Name:     "tags",
				Required: true,
				Array:    true,
				Spec:     &domain.StringPropertySpec{},
			},
			wantValid: true,
		},
		{
			name: "array constraint - invalid",
			frontmatter: domain.Frontmatter{
				Fields: map[string]interface{}{"tags": "single-tag"},
			},
			property: domain.Property{
				Name:     "tags",
				Required: true,
				Array:    true,
				Spec:     &domain.StringPropertySpec{},
			},
			wantValid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := validator.validateField(tt.frontmatter, tt.property)

			assertValidation(t, result, tt.wantValid)
		})
	}
}
