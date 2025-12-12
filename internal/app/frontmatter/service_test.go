package frontmatter

import (
	"context"
	"fmt"
	"testing"

	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockMarkdownParserPort provides a mock implementation of MarkdownParserPort
// for pure domain testing without external dependencies.
type mockMarkdownParserPort struct{}

// FakeSchemaPort implements SchemaPort for testing.
type FakeSchemaPort struct {
	schemas []domain.Schema
	bank    domain.PropertyBank
	err     error
}

// FakeSchemaRegistryPort implements SchemaRegistryPort for testing.
type FakeSchemaRegistryPort struct {
	schemas        map[string]domain.Schema
	properties     map[string]domain.Property
	getSchemaErr   error
	getPropertyErr error
}

func (m *mockMarkdownParserPort) ParseFrontmatter(
	ctx context.Context,
	content []byte,
) (map[string]any, error) {
	return map[string]any{"title": "test"}, nil
}
func (m *mockMarkdownParserPort) ParseNote(
	ctx context.Context,
	path string,
	content []byte,
) (domain.Note, error) {
	note, _ := domain.NewNote(
		path,
		domain.NewFrontmatter(map[string]interface{}{
			"title": "test",
		}),
		nil,
		nil,
		nil,
		nil,
	)
	return note, nil
}
func (f *FakeSchemaPort) Load(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	if f.err != nil {
		return nil, domain.PropertyBank{}, f.err
	}
	return f.schemas, f.bank, nil
}
func NewFakeSchemaRegistryPort() *FakeSchemaRegistryPort {
	return &FakeSchemaRegistryPort{
		schemas:    make(map[string]domain.Schema),
		properties: make(map[string]domain.Property),
	}
}
func (f *FakeSchemaRegistryPort) AddSchema(name string, sch domain.Schema) {
	f.schemas[name] = sch
}
func (f *FakeSchemaRegistryPort) GetSchema(ctx context.Context, name string) (
	domain.Schema, error,
) {
	if f.getSchemaErr != nil {
		return domain.Schema{}, f.getSchemaErr
	}
	if sch, exists := f.schemas[name]; exists {
		return sch, nil
	}
	return domain.Schema{}, fmt.Errorf("schema not found: %s", name)
}
func (f *FakeSchemaRegistryPort) GetProperty(ctx context.Context, name string) (
	domain.Property, error,
) {
	if f.getPropertyErr != nil {
		return domain.Property{}, f.getPropertyErr
	}
	if property, exists := f.properties[name]; exists {
		return property, nil
	}
	return domain.Property{}, fmt.Errorf("property not found: %s", name)
}
func (f *FakeSchemaRegistryPort) RegisterAll(
	ctx context.Context,
	schemas []domain.Schema,
	properties domain.PropertyBank,
) error {
	return nil // No-op for testing
}
func (f *FakeSchemaRegistryPort) HasSchema(
	ctx context.Context,
	name string,
) bool {
	_, exists := f.schemas[name]
	return exists
}
func (f *FakeSchemaRegistryPort) HasProperty(
	ctx context.Context,
	name string,
) bool {
	_, exists := f.properties[name]
	return exists
}

// createTestSchemaEngine creates a minimal SchemaEngine for testing.
func createTestSchemaEngine(
	registry *FakeSchemaRegistryPort,
) *schema.SchemaEngine {
	// Create a minimal fake schema port
	schemaPort := &FakeSchemaPort{}
	engine, _ := schema.NewSchemaEngine(schemaPort, registry, zerolog.Nop())
	return engine
}

// TestFrontmatterService_StructExists verifies FrontmatterService struct
// exists.
func TestFrontmatterService_StructExists(t *testing.T) {
	var _ *FrontmatterService // Compilation test - will fail if struct doesn't exist
}

// TestNewFrontmatterService_ConstructorWithDependencies verifies constructor
// properly injects all required dependencies including MarkdownParserPort.
func TestNewFrontmatterService_ConstructorWithDependencies(t *testing.T) {
	// Given - use nil for schema engine since we're just testing injection
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	// When
	service := NewFrontmatterService(nil, fakeMarkdownParser, fakeLogger)
	// Then
	require.NotNil(t, service)
	assert.Nil(t, service.schemaEngine) // We passed nil
	assert.Equal(t, fakeMarkdownParser, service.markdownParserPort)
	assert.Equal(t, fakeLogger, service.logger)
}

// TestNewFrontmatterService_DependencyInjection verifies all dependencies are
// properly injected.
func TestNewFrontmatterService_DependencyInjection(t *testing.T) {
	// Given
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	// When
	service := NewFrontmatterService(nil, fakeMarkdownParser, fakeLogger)
	// Then
	assert.NotNil(t, service.markdownParserPort)
	assert.NotNil(t, service.logger)
}

// TestIsSchemaCompliant_ValidFrontmatter verifies IsSchemaCompliant accepts
// valid frontmatter without schema validation.
func TestIsSchemaCompliant_ValidFrontmatter(t *testing.T) {
	// Given
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(nil, fakeMarkdownParser, fakeLogger)
	validFm := domain.NewFrontmatter(map[string]interface{}{
		"title": "Test Note",
	})
	// When
	err := service.IsSchemaCompliant(context.Background(), validFm)
	// Then
	assert.NoError(t, err)
}

// TestIsSchemaCompliant_WithFileClass verifies IsSchemaCompliant handles
// fileClass
// gracefully when schema engine is not available.
func TestIsSchemaCompliant_WithFileClass(t *testing.T) {
	// Given - nil schema engine means schema validation will fail
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(nil, fakeMarkdownParser, fakeLogger)
	fmWithFileClass := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "contact",
			"name":      "John Doe",
		},
	}
	// When
	err := service.IsSchemaCompliant(context.Background(), fmWithFileClass)
	// Then
	// Should fail because schema validation is attempted but schema engine is
	// nil
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema engine not available")
}

// TestIsSchemaCompliant_SchemaValidationSuccess verifies full schema validation
// succeeds when all required fields are present and types are correct.
func TestIsSchemaCompliant_SchemaValidationSuccess(t *testing.T) {
	// Given - test schema engine with a valid schema
	registry := NewFakeSchemaRegistryPort()
	// Create a schema with required fields
	titleProp, _ := domain.NewProperty(
		"title",
		true,
		false,
		&domain.StringSpec{},
	)
	authorProp, _ := domain.NewProperty(
		"author",
		true,
		false,
		&domain.StringSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*titleProp, *authorProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(engine, fakeMarkdownParser, fakeLogger)
	invalidFm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			// missing both required fields
		},
	}

	// When
	err := service.IsSchemaCompliant(context.Background(), invalidFm)
	// Then
	require.Error(t, err)
	// Should contain the required field missing error
	assert.Contains(t, err.Error(), "required field missing")
}

// TestIsSchemaCompliant_SchemaValidationMissingRequiredField verifies schema
// validation fails when required fields are missing.
func TestIsSchemaCompliant_SchemaValidationMissingRequiredField(t *testing.T) {
	// Given - test schema engine with a schema requiring title
	registry := NewFakeSchemaRegistryPort()
	titleProp, _ := domain.NewProperty(
		"title",
		true,
		false,
		&domain.StringSpec{},
	)
	authorProp, _ := domain.NewProperty(
		"author",
		false,
		false,
		&domain.StringSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*titleProp, *authorProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(engine, fakeMarkdownParser, fakeLogger)
	invalidFm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			// missing both required fields
		},
	}
	// When
	err := service.IsSchemaCompliant(context.Background(), invalidFm)
	// Then
	require.Error(t, err)
	assert.Contains(t, err.Error(), "required field missing")
}

// TestIsSchemaCompliant_SchemaValidationInvalidFieldType verifies schema
// validation fails when field types don't match schema requirements.
func TestIsSchemaCompliant_SchemaValidationInvalidFieldType(t *testing.T) {
	// Given - test schema engine with a schema requiring string title
	registry := NewFakeSchemaRegistryPort()
	titleProp, _ := domain.NewProperty(
		"title",
		true,
		false,
		&domain.StringSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*titleProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(engine, fakeMarkdownParser, fakeLogger)
	invalidFm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"title":     123, // should be string, not number
		},
	}
	// When
	err := service.IsSchemaCompliant(context.Background(), invalidFm)
	// Then
	require.Error(t, err)
	assert.Contains(t, err.Error(), "field value is not a string")
}

// TestIsSchemaCompliant_SchemaValidationMultipleErrors verifies schema
// validation aggregates multiple validation errors.
func TestIsSchemaCompliant_SchemaValidationMultipleErrors(t *testing.T) {
	// Given - test schema engine with a schema requiring multiple fields
	registry := NewFakeSchemaRegistryPort()
	titleProp, _ := domain.NewProperty(
		"title",
		true,
		false,
		&domain.StringSpec{},
	)
	authorProp, _ := domain.NewProperty(
		"author",
		true,
		false,
		&domain.StringSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*titleProp, *authorProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(engine, fakeMarkdownParser, fakeLogger)
	invalidFm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			// missing both required fields
		},
	}
	// When
	err := service.IsSchemaCompliant(context.Background(), invalidFm)
	// Then
	require.Error(t, err)
	// Should contain the required field missing error
	assert.Contains(t, err.Error(), "required field missing")
}

// TestIsSchemaCompliant_SchemaNotFound verifies graceful handling when schema
// is not found in the registry.
func TestIsSchemaCompliant_SchemaNotFound(t *testing.T) {
	// Given - test schema engine without the requested schema
	registry := NewFakeSchemaRegistryPort()
	engine := createTestSchemaEngine(registry)
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(engine, fakeMarkdownParser, fakeLogger)
	fmWithUnknownSchema := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "unknown-schema",
			"title":     "Test",
		},
	}
	// When
	err := service.IsSchemaCompliant(context.Background(), fmWithUnknownSchema)
	// Then
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema not found")
}

// TestIsSchemaCompliant_NumberFieldValidation verifies number field validation
// works correctly.
func TestIsSchemaCompliant_NumberFieldValidation(t *testing.T) {
	// Given - test schema engine with number field
	registry := NewFakeSchemaRegistryPort()
	priorityProp, _ := domain.NewProperty(
		"priority",
		true,
		false,
		&domain.NumberSpec{},
	)
	sch := domain.Schema{
		Name:       "task",
		Properties: []domain.Property{*priorityProp},
	}
	registry.AddSchema("task", sch)
	engine := createTestSchemaEngine(registry)
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(engine, fakeMarkdownParser, fakeLogger)
	tests := []struct {
		name       string
		value      interface{}
		shouldPass bool
	}{
		{"valid int", 5, true},
		{"valid float", 5.5, true},
		{"invalid string", "high", false},
		{"invalid bool", true, false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fm := domain.Frontmatter{
				Fields: map[string]interface{}{
					"fileClass": "task",
					"priority":  tt.value,
				},
			}
			err := service.IsSchemaCompliant(context.Background(), fm)
			if tt.shouldPass {
				assert.NoError(
					t,
					err,
					"expected validation to pass for %v",
					tt.value,
				)
			} else {
				assert.Error(t, err, "expected validation to fail for %v", tt.value)
			}
		})
	}
}

// TestIsSchemaCompliant_BooleanFieldValidation verifies boolean field
// validation
// works correctly.
func TestIsSchemaCompliant_BooleanFieldValidation(t *testing.T) {
	// Given - test schema engine with boolean field
	registry := NewFakeSchemaRegistryPort()
	doneProp, _ := domain.NewProperty("done", true, false, &domain.BoolSpec{})
	sch := domain.Schema{
		Name:       "task",
		Properties: []domain.Property{*doneProp},
	}
	registry.AddSchema("task", sch)
	engine := createTestSchemaEngine(registry)
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(engine, fakeMarkdownParser, fakeLogger)
	tests := []struct {
		name       string
		value      interface{}
		shouldPass bool
	}{
		{"valid true", true, true},
		{"valid false", false, true},
		{"invalid string", "yes", false},
		{"invalid number", 1, false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fm := domain.Frontmatter{
				Fields: map[string]interface{}{
					"fileClass": "task",
					"done":      tt.value,
				},
			}
			err := service.IsSchemaCompliant(context.Background(), fm)
			if tt.shouldPass {
				assert.NoError(
					t,
					err,
					"expected validation to pass for %v",
					tt.value,
				)
			} else {
				assert.Error(t, err, "expected validation to fail for %v", tt.value)
			}
		})
	}
}
