package frontmatter

import (
	"context"
	"fmt"
	"testing"

	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// FakeQueryService provides a mock implementation for testing FileSpec
// validation.
type FakeQueryService struct {
	pathQueryResults map[string][]domain.Note
	pathQueryError   error
}

// FakeSchemaRegistryPort implements SchemaRegistryPort for testing.
type FakeSchemaRegistryPort struct {
	schemas        map[string]domain.Schema
	properties     map[string]domain.Property
	getSchemaErr   error
	getPropertyErr error
}

// FakeSchemaPort implements SchemaPort for testing.
type FakeSchemaPort struct {
	schemas []domain.Schema
	bank    domain.PropertyBank
	err     error
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

func NewFakeQueryService() *FakeQueryService {
	return &FakeQueryService{
		pathQueryResults: make(map[string][]domain.Note),
	}
}

func (f *FakeQueryService) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	if f.pathQueryError != nil {
		return nil, f.pathQueryError
	}
	if results, exists := f.pathQueryResults[opts.Value]; exists {
		return results, nil
	}
	return []domain.Note{}, nil // Not found
}

func (f *FakeQueryService) SetPathQueryResult(
	path string,
	results []domain.Note,
) {
	f.pathQueryResults[path] = results
}

// createTestSchemaEngine creates a minimal SchemaEngine for testing.
func createTestSchemaEngine(
	registry *FakeSchemaRegistryPort,
) *schema.SchemaEngine {
	// Create a minimal fake schema port
	schemaPort := &FakeSchemaPort{}
	engine, _ := schema.NewSchemaEngine(
		schemaPort,
		registry,
		zerolog.Nop(),
		nil,
	)
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
	fakeLogger := zerolog.Nop()
	// When
	service := NewFrontmatterService(nil, fakeLogger, nil, nil)
	// Then
	require.NotNil(t, service)
	assert.Nil(t, service.schemaEngine) // We passed nil
	assert.Equal(t, fakeLogger, service.logger)
}

// TestNewFrontmatterService_DependencyInjection verifies all dependencies are
// properly injected.
func TestNewFrontmatterService_DependencyInjection(t *testing.T) {
	// Given
	fakeLogger := zerolog.Nop()
	// When
	service := NewFrontmatterService(nil, fakeLogger, nil, nil)
	// Then
	assert.NotNil(t, service.logger)
}

// TestIsSchemaCompliant_ValidFrontmatter verifies IsSchemaCompliant accepts
// valid frontmatter without schema validation.
func TestIsSchemaCompliant_ValidFrontmatter(t *testing.T) {
	// Given
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(nil, fakeLogger, nil, nil)
	validFm := domain.NewFrontmatter(map[string]interface{}{
		"title": "Test Note",
	})
	// When
	err := service.IsSchemaCompliant(
		context.Background(),
		"notes/test.md",
		validFm,
	)
	// Then
	assert.NoError(t, err)
}

// TestIsSchemaCompliant_WithFileClass verifies IsSchemaCompliant handles
// fileClass
// gracefully when schema engine is not available.
func TestIsSchemaCompliant_WithFileClass(t *testing.T) {
	// Given - nil schema engine means schema validation will fail
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(nil, fakeLogger, nil, nil)
	fmWithFileClass := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "contact",
			"name":      "John Doe",
		},
	}
	// When
	err := service.IsSchemaCompliant(
		context.Background(),
		"notes/test.md",
		fmWithFileClass,
	)
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
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(
		engine,

		fakeLogger,
		nil,
		nil,
	)
	invalidFm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			// missing both required fields
		},
	}

	// When
	err := service.IsSchemaCompliant(
		context.Background(),
		"notes/test.md",
		invalidFm,
	)
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
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(
		engine,

		fakeLogger,
		nil,
		nil,
	)
	invalidFm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			// missing both required fields
		},
	}
	// When
	err := service.IsSchemaCompliant(
		context.Background(),
		"notes/test.md",
		invalidFm,
	)
	// Then
	require.Error(t, err)
	assert.Contains(t, err.Error(), "required field missing")
}

// TestIsSchemaCompliant_SchemaValidationInvalidFieldType verifies schema
// validation coerces scalar values to strings when appropriate.
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
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(
		engine,

		fakeLogger,
		nil,
		nil,
	)

	// Test coercion of number to string (common YAML shorthand)
	numericTitleFm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"title":     123, // Coerced to "123" by StringValidator
		},
	}
	err := service.IsSchemaCompliant(
		context.Background(),
		"notes/test.md",
		numericTitleFm,
	)
	require.NoError(t, err, "numeric values should coerce to strings")

	// Test actual invalid type (slice/map cannot coerce)
	invalidFm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"title": []string{
				"invalid",
				"array",
			}, // Cannot coerce array to string
		},
	}
	err = service.IsSchemaCompliant(
		context.Background(),
		"notes/test.md",
		invalidFm,
	)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "field value must not be an array")
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
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(
		engine,

		fakeLogger,
		nil,
		nil,
	)
	invalidFm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			// missing both required fields
		},
	}
	// When
	err := service.IsSchemaCompliant(
		context.Background(),
		"notes/test.md",
		invalidFm,
	)
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
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(
		engine,

		fakeLogger,
		nil,
		nil,
	)
	fmWithUnknownSchema := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "unknown-schema",
			"title":     "Test",
		},
	}
	// When
	err := service.IsSchemaCompliant(
		context.Background(),
		"notes/test.md",
		fmWithUnknownSchema,
	)
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
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(
		engine,

		fakeLogger,
		nil,
		nil,
	)
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
			err := service.IsSchemaCompliant(
				context.Background(),
				"notes/test.md",
				fm,
			)
			if tt.shouldPass {
				assert.NoError(
					t,
					err,
					"expected validation to pass for %v",
					tt.value,
				)
			} else {
				require.Error(t, err, "expected validation to fail for %v", tt.value)
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
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(
		engine,

		fakeLogger,
		nil,
		nil,
	)
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
			err := service.IsSchemaCompliant(
				context.Background(),
				"notes/test.md",
				fm,
			)
			if tt.shouldPass {
				assert.NoError(
					t,
					err,
					"expected validation to pass for %v",
					tt.value,
				)
			} else {
				require.Error(t, err, "expected validation to fail for %v", tt.value)
			}
		})
	}
}

// TestValidate_MethodExists verifies Validate method exists and has correct
// signature.
func TestValidate_MethodExists(t *testing.T) {
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(nil, fakeLogger, nil, nil)
	var _ = service.Validate
}

// TestValidate_CallsIsSchemaCompliant verifies Validate calls
// IsSchemaCompliant.
func TestValidate_CallsIsSchemaCompliant(t *testing.T) {
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(nil, fakeLogger, nil, nil)
	fm := domain.NewFrontmatter(map[string]interface{}{
		"title": "Test",
	})
	err := service.Validate(context.Background(), "test.md", fm)
	// Should not error since no schema validation is triggered
	assert.NoError(t, err)
}

// TestValidate_FileSpecPropertyDetection_Single verifies FileSpec property
// detection for single values.
func TestValidate_FileSpecPropertyDetection_Single(t *testing.T) {
	// Given - schema with FileSpec property
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	fakeQuery.SetPathQueryResult(
		"contacts/john.md",
		[]domain.Note{{Path: "contacts/john.md"}},
	)
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "contacts/john.md",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	assert.NoError(t, err)
}

// TestValidate_FileSpecPropertyDetection_Array verifies FileSpec property
// detection for arrays.
func TestValidate_FileSpecPropertyDetection_Array(t *testing.T) {
	// Given - schema with FileSpec array property
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"references",
		true,
		true,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	fakeQuery.SetPathQueryResult(
		"contacts/john.md",
		[]domain.Note{{Path: "contacts/john.md"}},
	)
	// contacts/jane.md not set, so returns empty = not found
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass":  "note",
			"references": []string{"contacts/john.md", "contacts/jane.md"},
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then - should fail because second file doesn't exist
	require.Error(t, err)
	assert.Contains(t, err.Error(), "file not found")
}

// TestValidate_ValidFilePathValidation verifies valid file paths pass
// validation.
func TestValidate_ValidFilePathValidation(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	fakeQuery.SetPathQueryResult(
		"contacts/john.md",
		[]domain.Note{{Path: "contacts/john.md"}},
	)
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "contacts/john.md",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	assert.NoError(t, err)
}

// TestValidate_FileNotFoundValidationError verifies invalid file paths return
// ValidationError.
func TestValidate_FileNotFoundValidationError(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	// missing.md not set, so returns empty = not found
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "missing.md",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	require.Error(t, err)
	assert.Contains(t, err.Error(), "file not found")
}

// TestValidate_PathNormalization_DotSlash verifies path normalization handles
// ./ and ../.
func TestValidate_PathNormalization_DotSlash(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	fakeQuery.SetPathQueryResult(
		"./contacts/john.md",
		[]domain.Note{{Path: "contacts/john.md"}},
	)
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "./contacts/john.md",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	assert.NoError(t, err) // Path normalization should work
}

// TestValidate_WikilinkResolutionSuccess verifies wikilink format [[basename]]
// resolves correctly.
func TestValidate_WikilinkResolutionSuccess(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	fakeQuery.SetPathQueryResult(
		"john",
		[]domain.Note{{Path: "contacts/john.md"}},
	)
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "[[john]]",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	assert.NoError(t, err)
}

// TestValidate_WikilinkAmbiguousError verifies ambiguous wikilinks return
// error.
func TestValidate_WikilinkAmbiguousError(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	fakeQuery.SetPathQueryResult("john", []domain.Note{
		{Path: "contacts/john-1.md"},
		{Path: "contacts/john-2.md"},
	}) // Multiple results = ambiguous
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "[[john]]",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	require.Error(t, err)
	assert.Contains(t, err.Error(), "ambiguous reference")
}

// TestValidate_ValidationErrorFieldName verifies ValidationError includes field
// name.
func TestValidate_ValidationErrorFieldName(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	// missing.md not set = not found
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "missing.md",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	require.Error(t, err)
	// Error should include field name "reference"
	assert.Contains(t, err.Error(), "reference")
}

// TestValidate_ValidationErrorReasonAndValue verifies ValidationError includes
// reason and value.
func TestValidate_ValidationErrorReasonAndValue(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	// missing.md not set = not found
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "missing.md",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	require.Error(t, err)
	assert.Contains(t, err.Error(), "file not found")
	assert.Contains(t, err.Error(), "missing.md")
}

// TestValidate_ValidationErrorRemediationHints verifies ValidationError
// includes remediation hints.
func TestValidate_ValidationErrorRemediationHints(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	// missing.md not set = not found
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "missing.md",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	require.Error(t, err)
	// Should include some remediation hint
	assert.Contains(t, err.Error(), "Check if")
}

// TestValidate_ErrorAggregationForArrays verifies multiple FileSpec errors are
// aggregated.
func TestValidate_ErrorAggregationForArrays(t *testing.T) {
	// Given - schema with FileSpec array
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"references",
		true,
		true,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	// Neither missing1.md nor missing2.md set = both not found
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass":  "note",
			"references": []string{"missing1.md", "missing2.md"},
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	require.Error(t, err)
	// Should contain multiple errors
	assert.Contains(t, err.Error(), "missing1.md")
	assert.Contains(t, err.Error(), "missing2.md")
}

// TestValidate_AbsolutePathSupport verifies absolute paths are supported.
func TestValidate_AbsolutePathSupport(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	fakeQuery.SetPathQueryResult(
		"/absolute/contacts/john.md",
		[]domain.Note{{Path: "/absolute/contacts/john.md"}},
	)
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "/absolute/contacts/john.md",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	assert.NoError(t, err)
}

// TestValidate_VaultRelativePathSupport verifies vault-relative paths work.
func TestValidate_VaultRelativePathSupport(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	fakeQuery.SetPathQueryResult(
		"contacts/john.md",
		[]domain.Note{{Path: "contacts/john.md"}},
	)
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "contacts/john.md",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	assert.NoError(t, err)
}

// TestValidate_TrailingSlashNormalization verifies trailing slashes are
// removed.
func TestValidate_TrailingSlashNormalization(t *testing.T) {
	// Given
	registry := NewFakeSchemaRegistryPort()
	fileProp, _ := domain.NewProperty(
		"reference",
		true,
		false,
		&domain.FileSpec{},
	)
	sch := domain.Schema{
		Name:       "note",
		Properties: []domain.Property{*fileProp},
	}
	registry.AddSchema("note", sch)
	engine := createTestSchemaEngine(registry)
	fakeLogger := zerolog.Nop()
	fakeQuery := NewFakeQueryService()
	fakeQuery.SetPathQueryResult("contacts", []domain.Note{{Path: "contacts/"}})
	service := NewFrontmatterService(engine, fakeLogger, nil, fakeQuery)

	fm := domain.Frontmatter{
		Fields: map[string]interface{}{
			"fileClass": "note",
			"reference": "contacts/",
		},
	}

	// When
	err := service.Validate(context.Background(), "test.md", fm)

	// Then
	assert.NoError(t, err)
}
