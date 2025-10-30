package schema

import (
	"context"
	"errors"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

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
	registerAllErr error
}

// Load implements SchemaPort.Load for testing.
func (f *FakeSchemaPort) Load(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	if f.err != nil {
		return nil, domain.PropertyBank{}, f.err
	}
	return f.schemas, f.bank, nil
}

// GetSchema implements SchemaRegistryPort.GetSchema for testing.
func (f *FakeSchemaRegistryPort) GetSchema(
	ctx context.Context,
	name string,
) (domain.Schema, error) {
	if f.getSchemaErr != nil {
		return domain.Schema{}, f.getSchemaErr
	}
	if schema, exists := f.schemas[name]; exists {
		return schema, nil
	}
	return domain.Schema{}, errors.New("schema not found")
}

// GetProperty implements SchemaRegistryPort.GetProperty for testing.
func (f *FakeSchemaRegistryPort) GetProperty(
	ctx context.Context,
	name string,
) (domain.Property, error) {
	if f.getPropertyErr != nil {
		return domain.Property{}, f.getPropertyErr
	}
	if prop, exists := f.properties[name]; exists {
		return prop, nil
	}
	return domain.Property{}, errors.New("property not found")
}

// HasSchema checks if a schema exists in the fake registry.
func (f *FakeSchemaRegistryPort) HasSchema(
	ctx context.Context,
	name string,
) bool {
	_, exists := f.schemas[name]
	return exists
}

// HasProperty checks if a property exists in the fake registry.
// HasProperty checks if a property exists in the fake registry.
func (f *FakeSchemaRegistryPort) HasProperty(
	ctx context.Context,
	name string,
) bool {
	_, exists := f.properties[name]
	return exists
}

// RegisterAll registers all schemas and the property bank in the fake registry.
func (f *FakeSchemaRegistryPort) RegisterAll(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
) error {
	if f.registerAllErr != nil {
		return f.registerAllErr
	}
	f.schemas = make(map[string]domain.Schema)
	for _, schema := range schemas {
		f.schemas[schema.Name] = schema
	}
	f.properties = bank.Properties
	return nil
}

// TestSchemaEngine_NewSchemaEngine_ValidDependencies tests constructor with
// valid dependencies.
func TestSchemaEngine_NewSchemaEngine_ValidDependencies(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)
	require.NotNil(t, engine)
	assert.NotNil(t, engine.validator)
	assert.NotNil(t, engine.resolver)
}

// TestSchemaEngine_NewSchemaEngine_NilSchemaPort tests constructor with nil
// schema port.
func TestSchemaEngine_NewSchemaEngine_NilSchemaPort(t *testing.T) {
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(nil, registryPort, log)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schemaPort cannot be nil")
	assert.Nil(t, engine)
}

// TestSchemaEngine_NewSchemaEngine_NilRegistryPort tests constructor with nil
// registry port.
func TestSchemaEngine_NewSchemaEngine_NilRegistryPort(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, nil, log)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "registryPort cannot be nil")
	assert.Nil(t, engine)
}

// TestSchemaEngine_Load_Success tests successful Load() execution through all
// stages.
func TestSchemaEngine_Load_Success(t *testing.T) {
	schemaPort := &FakeSchemaPort{
		schemas: []domain.Schema{
			{Name: "test-schema", Properties: []domain.Property{}},
		},
		bank: domain.PropertyBank{Properties: map[string]domain.Property{}},
	}
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	err = engine.Load(context.Background())
	require.NoError(t, err)

	// Verify schemas were registered
	assert.True(t, registryPort.HasSchema(context.Background(), "test-schema"))
}

// TestSchemaEngine_Load_SchemaPortFailure tests Load() failure at
// SchemaPort.Load stage.
func TestSchemaEngine_Load_SchemaPortFailure(t *testing.T) {
	schemaPort := &FakeSchemaPort{
		err: errors.New("schema port error"),
	}
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	err = engine.Load(context.Background())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema loading failed")
	assert.Contains(t, err.Error(), "schema port error")
}

// TestSchemaEngine_Load_ValidationFailure tests Load() failure at
// SchemaValidator stage.
func TestSchemaEngine_Load_ValidationFailure(t *testing.T) {
	schemaPort := &FakeSchemaPort{
		schemas: []domain.Schema{
			{
				Name:       "",
				Properties: []domain.Property{},
			}, // Invalid schema (empty name)
		},
		bank: domain.PropertyBank{Properties: map[string]domain.Property{}},
	}
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	err = engine.Load(context.Background())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema validation failed")
}

// TestSchemaEngine_Load_ResolutionFailure tests Load() failure at
// SchemaResolver stage.
func TestSchemaEngine_Load_ResolutionFailure(t *testing.T) {
	schemaPort := &FakeSchemaPort{
		schemas: []domain.Schema{
			{Name: "a", Extends: "b"},
			{Name: "b", Extends: "a"}, // Circular dependency
		},
		bank: domain.PropertyBank{Properties: map[string]domain.Property{}},
	}
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	err = engine.Load(context.Background())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema resolution failed")
	assert.Contains(t, err.Error(), "circular inheritance")
}

// TestSchemaEngine_Load_RegistrationFailure tests Load() failure at
// SchemaRegistryPort stage.
func TestSchemaEngine_Load_RegistrationFailure(t *testing.T) {
	schemaPort := &FakeSchemaPort{
		schemas: []domain.Schema{
			{Name: "test-schema", Properties: []domain.Property{}},
		},
		bank: domain.PropertyBank{Properties: map[string]domain.Property{}},
	}
	registryPort := &FakeSchemaRegistryPort{
		registerAllErr: errors.New("registration error"),
	}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	err = engine.Load(context.Background())
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema registration failed")
	assert.Contains(t, err.Error(), "registration error")
}

// TestSchemaEngine_Get_SchemaSuccess tests Get[Schema]() success case.
func TestSchemaEngine_Get_SchemaSuccess(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	registryPort := &FakeSchemaRegistryPort{
		schemas: map[string]domain.Schema{
			"test-schema": {Name: "test-schema"},
		},
	}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	schema, err := Get[domain.Schema](
		engine,
		context.Background(),
		"test-schema",
	)
	require.NoError(t, err)
	assert.Equal(t, "test-schema", schema.Name)
}

// TestSchemaEngine_Get_PropertySuccess tests Get[Property]() success case.
func TestSchemaEngine_Get_PropertySuccess(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	registryPort := &FakeSchemaRegistryPort{
		properties: map[string]domain.Property{
			"test-prop": {Name: "test-prop"},
		},
	}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	property, err := Get[domain.Property](
		engine,
		context.Background(),
		"test-prop",
	)
	require.NoError(t, err)
	assert.Equal(t, "test-prop", property.Name)
}

// TestSchemaEngine_Get_SchemaNotFound tests Get[Schema]() error case.
func TestSchemaEngine_Get_SchemaNotFound(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	_, err = Get[domain.Schema](engine, context.Background(), "non-existent")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema not found")
}

// TestSchemaEngine_Get_PropertyNotFound tests Get[Property]() error case.
func TestSchemaEngine_Get_PropertyNotFound(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	_, err = Get[domain.Property](engine, context.Background(), "non-existent")
	require.Error(t, err)
	assert.Contains(t, err.Error(), "property not found")
}

// TestSchemaEngine_Has_SchemaTrue tests Has[Schema]() true case.
func TestSchemaEngine_Has_SchemaTrue(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	registryPort := &FakeSchemaRegistryPort{
		schemas: map[string]domain.Schema{
			"test-schema": {Name: "test-schema"},
		},
	}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	assert.True(
		t,
		Has[domain.Schema](engine, context.Background(), "test-schema"),
	)
}

// TestSchemaEngine_Has_SchemaFalse tests Has[Schema]() false case.
func TestSchemaEngine_Has_SchemaFalse(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	assert.False(
		t,
		Has[domain.Schema](engine, context.Background(), "non-existent"),
	)
}

// TestSchemaEngine_Has_PropertyTrue tests Has[Property]() true case.
func TestSchemaEngine_Has_PropertyTrue(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	registryPort := &FakeSchemaRegistryPort{
		properties: map[string]domain.Property{
			"test-prop": {Name: "test-prop"},
		},
	}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	assert.True(
		t,
		Has[domain.Property](engine, context.Background(), "test-prop"),
	)
}

// TestSchemaEngine_Has_PropertyFalse tests Has[Property]() false case.
func TestSchemaEngine_Has_PropertyFalse(t *testing.T) {
	schemaPort := &FakeSchemaPort{}
	registryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	engine, err := NewSchemaEngine(schemaPort, registryPort, log)
	require.NoError(t, err)

	assert.False(
		t,
		Has[domain.Property](engine, context.Background(), "non-existent"),
	)
}
