//go:build ignore

package frontmatter

import (
	"context"
	"fmt"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/dto"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
)

// Mock implementations for testing

type MockVaultReader struct {
	readFunc func(ctx context.Context, path string) (dto.VaultFile, error)
}

type MockSchemaPort struct{}

type MockSchemaRegistry struct {
	getSchemaFunc func(ctx context.Context, name string) (domain.Schema, error)
}

type MockQueryService struct {
	byPathFunc func(ctx context.Context, path string) (bool, error)
}

type MockFieldValidator struct {
	fieldType string
}

func (m *MockVaultReader) Read(
	ctx context.Context,
	path string,
) (dto.VaultFile, error) {
	if m.readFunc != nil {
		return m.readFunc(ctx, path)
	}
	return dto.VaultFile{}, nil
}

func (m *MockSchemaPort) Load(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	return []domain.Schema{}, domain.PropertyBank{}, nil
}

func (m *MockSchemaRegistry) GetSchema(
	ctx context.Context,
	name string,
) (domain.Schema, error) {
	if m.getSchemaFunc != nil {
		return m.getSchemaFunc(ctx, name)
	}
	return domain.Schema{}, nil
}

func (m *MockSchemaRegistry) GetProperty(
	ctx context.Context,
	name string,
) (domain.Property, error) {
	return domain.Property{}, nil // Not used in tests
}

func (m *MockSchemaRegistry) HasSchema(ctx context.Context, name string) bool {
	return true // Not used in tests
}

func (m *MockSchemaRegistry) HasProperty(
	ctx context.Context,
	name string,
) bool {
	return true // Not used in tests
}

func (m *MockSchemaRegistry) RegisterAll(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
) error {
	return nil // Not used in tests
}

func (m *MockQueryService) ByPath(
	ctx context.Context,
	path string,
) (bool, error) {
	if m.byPathFunc != nil {
		return m.byPathFunc(ctx, path)
	}
	return true, nil
}

func (m *MockFieldValidator) FieldType() string {
	return m.fieldType
}

func (m *MockFieldValidator) Validate(
	fieldName string,
	fieldValue interface{},
	prop domain.Property,
) error {
	// Mock implementation - validate basic types
	switch m.fieldType {
	case "string":
		if _, ok := fieldValue.(string); !ok {
			return errors.NewFrontmatterError(
				fmt.Sprintf(
					"field '%s' must be string, got %T",
					fieldName,
					fieldValue,
				),
				fieldName,
				nil,
			)
		}
	case "number":
		if _, ok := fieldValue.(float64); !ok {
			if _, ok := fieldValue.(int); !ok {
				return errors.NewFrontmatterError(
					fmt.Sprintf(
						"field '%s' must be number, got %T",
						fieldName,
						fieldValue,
					),
					fieldName,
					nil,
				)
			}
		}
	}
	return nil
}

func TestNewFrontmatterService(t *testing.T) {
	tests := []struct {
		name           string
		vaultReader    spi.VaultReaderPort
		schemaRegistry spi.SchemaRegistryPort
		validators     []FieldValidator
		log            zerolog.Logger
		expectError    bool
	}{
		{
			name:           "valid construction",
			vaultReader:    &MockVaultReader{},
			schemaRegistry: &MockSchemaRegistry{},
			validators: []FieldValidator{
				&MockFieldValidator{fieldType: "string"},
			},
			log:         zerolog.New(nil),
			expectError: false,
		},
		{
			name:           "nil vaultReader",
			vaultReader:    nil,
			schemaRegistry: &MockSchemaRegistry{},
			validators: []FieldValidator{
				&MockFieldValidator{fieldType: "string"},
			},
			log:         zerolog.New(nil),
			expectError: true,
		},
		{
			name:           "nil schemaRegistry",
			vaultReader:    &MockVaultReader{},
			schemaRegistry: nil,
			validators: []FieldValidator{
				&MockFieldValidator{fieldType: "string"},
			},
			log:         zerolog.New(nil),
			expectError: true,
		},
		{
			name:           "empty validators",
			vaultReader:    &MockVaultReader{},
			schemaRegistry: &MockSchemaRegistry{},
			validators:     []FieldValidator{},
			log:            zerolog.New(nil),
			expectError:    false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			service, err := NewFrontmatterService(
				tt.vaultReader,
				tt.schemaRegistry,
				tt.validators,
				tt.log,
			)

			if tt.expectError {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.NotNil(t, service)
			}
		})
	}
}

func TestFrontmatterService_ValidateNote(t *testing.T) {
	// Setup mocks
	mockVaultReader := &MockVaultReader{}
	mockSchemaRegistry := &MockSchemaRegistry{}
	mockValidators := []FieldValidator{
		&MockFieldValidator{fieldType: "string"},
	}

	service, err := NewFrontmatterService(
		mockVaultReader,
		mockSchemaRegistry,
		mockValidators,
		zerolog.New(nil),
	)
	assert.NoError(t, err)

	tests := []struct {
		name        string
		vf          dto.VaultFile
		expectError bool
		expectedFM  domain.Frontmatter
	}{
		{
			name: "valid markdown with frontmatter",
			vf: dto.VaultFile{
				FileMetadata: dto.FileMetadata{
					Path:     "test/note.md",
					MimeType: "text/markdown",
				},
				Content: []byte(`---
fileClass: meeting_note
title: "Test Meeting"
date: "2023-10-01"
---
# Meeting Notes
Content here`),
			},
			expectError: false,
			expectedFM: domain.Frontmatter{
				FileClass: "meeting_note",
				Fields: map[string]interface{}{
					"title": "Test Meeting",
					"date":  "2023-10-01",
				},
			},
		},
		{
			name: "non-markdown mimetype",
			vf: dto.VaultFile{
				FileMetadata: dto.FileMetadata{
					Path:     "test/note.txt",
					MimeType: "text/plain",
				},
				Content: []byte("some content"),
			},
			expectError: true,
		},
		{
			name: "no frontmatter",
			vf: dto.VaultFile{
				FileMetadata: dto.FileMetadata{
					Path:     "test/note.md",
					MimeType: "text/markdown",
				},
				Content: []byte("# Just content\nNo frontmatter"),
			},
			expectError: false,
			expectedFM: domain.Frontmatter{
				FileClass: "",
				Fields:    map[string]interface{}{},
			},
		},
		{
			name: "invalid YAML",
			vf: dto.VaultFile{
				FileMetadata: dto.FileMetadata{
					Path:     "test/note.md",
					MimeType: "text/markdown",
				},
				Content: []byte(`---
invalid: yaml: content: [
---
Content`),
			},
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fm, err := service.Extract(tt.vf)

			if tt.expectError {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.expectedFM.FileClass, fm.FileClass)
				assert.Equal(t, tt.expectedFM.Fields, fm.Fields)
			}
		})
	}
}

func TestFrontmatterService_Validate(t *testing.T) {
	// Setup mocks
	mockVaultReader := &MockVaultReader{}
	mockSchemaRegistry := &MockSchemaRegistry{
		getSchemaFunc: func(ctx context.Context, name string) (domain.Schema, error) {
			if name == "meeting_note" {
				return domain.Schema{
					Name: "meeting_note",
					ResolvedProperties: []domain.Property{
						{
							Name:     "title",
							Required: true,
							Array:    false,
							Spec:     &domain.StringSpec{},
						},
						{
							Name:     "date",
							Required: true,
							Array:    false,
							Spec:     &domain.StringSpec{},
						},
					},
				}, nil
			}
			return domain.Schema{}, fmt.Errorf("schema not found")
		},
	}
	mockValidators := []FieldValidator{
		&MockFieldValidator{fieldType: "string"},
	}

	service, err := NewFrontmatterService(
		mockVaultReader,
		mockSchemaRegistry,
		mockValidators,
		zerolog.New(nil),
	)
	assert.NoError(t, err)

	tests := []struct {
		name        string
		fm          domain.Frontmatter
		expectError bool
	}{
		{
			name: "valid frontmatter",
			fm: domain.Frontmatter{
				FileClass: "meeting_note",
				Fields: map[string]interface{}{
					"title": "Test Meeting",
					"date":  "2023-10-01",
				},
			},
			expectError: false,
		},
		{
			name: "missing required field",
			fm: domain.Frontmatter{
				FileClass: "meeting_note",
				Fields: map[string]interface{}{
					"title": "Test Meeting",
					// missing date
				},
			},
			expectError: true,
		},
		{
			name: "invalid field type",
			fm: domain.Frontmatter{
				FileClass: "meeting_note",
				Fields: map[string]interface{}{
					"title": 123, // should be string
					"date":  "2023-10-01",
				},
			},
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := service.Validate(context.Background(), tt.fm)

			if tt.expectError {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestFrontmatterService_ValidateNote_LoadError(t *testing.T) {
	// Setup mocks
	mockVaultReader := &MockVaultReader{
		readFunc: func(ctx context.Context, path string) (dto.VaultFile, error) {
			return dto.VaultFile{}, assert.AnError
		},
	}
	mockSchemaRegistry := &MockSchemaRegistry{}
	mockValidators := []FieldValidator{&MockFieldValidator{fieldType: "string"}}

	service, err := NewFrontmatterService(
		mockVaultReader,
		mockSchemaRegistry,
		mockValidators,
		zerolog.New(nil),
	)
	assert.NoError(t, err)

	// Test
	result, err := service.ValidateNote(
		context.Background(),
		"nonexistent.md",
		"test_schema",
	)

	// Assertions
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to load note")
	assert.Equal(t, ValidationResult{}, result)
}
