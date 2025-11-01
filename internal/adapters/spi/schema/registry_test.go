package schema

import (
	"context"
	"sync"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestSchemaRegistryAdapter_RegisterAll tests successful registration of
// schemas and properties.
func TestSchemaRegistryAdapter_RegisterAll(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	schemas := []domain.Schema{
		{Name: "test-schema", Properties: []domain.Property{
			{Name: "field1", Spec: &domain.StringSpec{}},
		}},
	}
	bank := domain.PropertyBank{
		Properties: map[string]domain.Property{
			"prop1": {Name: "prop1"},
		},
	}

	err := registry.RegisterAll(context.Background(), schemas, bank)
	require.NoError(t, err)

	// Verify schemas registered
	assert.True(t, registry.HasSchema(context.Background(), "test-schema"))
	schema, err := registry.GetSchema(context.Background(), "test-schema")
	require.NoError(t, err)
	assert.Equal(t, "test-schema", schema.Name)

	// Verify properties registered
	assert.True(t, registry.HasProperty(context.Background(), "prop1"))
	property, err := registry.GetProperty(context.Background(), "prop1")
	require.NoError(t, err)
	assert.Equal(t, "prop1", property.Name)
}

// TestSchemaRegistryAdapter_Idempotency tests RegisterAll clears existing data.
func TestSchemaRegistryAdapter_Idempotency(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	// First registration
	schemas1 := []domain.Schema{{Name: "schema1"}}
	bank1 := domain.PropertyBank{
		Properties: map[string]domain.Property{"prop1": {Name: "prop1"}},
	}
	err := registry.RegisterAll(context.Background(), schemas1, bank1)
	require.NoError(t, err)

	assert.True(t, registry.HasSchema(context.Background(), "schema1"))
	assert.True(t, registry.HasProperty(context.Background(), "prop1"))

	// Second registration with different data
	schemas2 := []domain.Schema{{Name: "schema2"}}
	bank2 := domain.PropertyBank{
		Properties: map[string]domain.Property{"prop2": {Name: "prop2"}},
	}
	err = registry.RegisterAll(context.Background(), schemas2, bank2)
	require.NoError(t, err)

	// Only schema2 and prop2 should exist (schema1 and prop1 cleared)
	assert.False(t, registry.HasSchema(context.Background(), "schema1"))
	assert.False(t, registry.HasProperty(context.Background(), "prop1"))
	assert.True(t, registry.HasSchema(context.Background(), "schema2"))
	assert.True(t, registry.HasProperty(context.Background(), "prop2"))
}

// TestSchemaRegistryAdapter_GetSchema_Success tests successful schema
// retrieval.
func TestSchemaRegistryAdapter_GetSchema_Success(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	schema := domain.Schema{
		Name: "test",
		Properties: []domain.Property{
			{Name: "field", Spec: &domain.StringSpec{}},
		},
	}
	err := registry.RegisterAll(
		context.Background(),
		[]domain.Schema{schema},
		domain.PropertyBank{},
	)
	require.NoError(t, err)

	result, err := registry.GetSchema(context.Background(), "test")
	require.NoError(t, err)
	assert.Equal(t, "test", result.Name)
	assert.Len(t, result.Properties, 1)
	assert.Equal(t, "field", result.Properties[0].Name)
}

// TestSchemaRegistryAdapter_GetSchema_NotFound tests error handling when schema
// is not found.
func TestSchemaRegistryAdapter_GetSchema_NotFound(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	_, err := registry.GetSchema(context.Background(), "nonexistent")
	require.Error(t, err)
	require.ErrorIs(t, err, lithoserrors.ErrNotFound)

	var schemaErr *lithoserrors.SchemaError
	require.ErrorAs(t, err, &schemaErr)
	assert.Equal(t, "nonexistent", schemaErr.SchemaName)
}

// TestSchemaRegistryAdapter_GetProperty_Success tests successful property
// retrieval.
func TestSchemaRegistryAdapter_GetProperty_Success(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	property := domain.Property{Name: "test-prop"}
	bank := domain.PropertyBank{
		Properties: map[string]domain.Property{"test-prop": property},
	}
	err := registry.RegisterAll(context.Background(), []domain.Schema{}, bank)
	require.NoError(t, err)

	result, err := registry.GetProperty(context.Background(), "test-prop")
	require.NoError(t, err)
	assert.Equal(t, "test-prop", result.Name)
}

// TestSchemaRegistryAdapter_GetProperty_NotFound tests error handling when
// property is not found.
func TestSchemaRegistryAdapter_GetProperty_NotFound(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	_, err := registry.GetProperty(context.Background(), "nonexistent")
	require.Error(t, err)
	require.ErrorIs(t, err, lithoserrors.ErrNotFound)

	var schemaErr *lithoserrors.SchemaError
	require.ErrorAs(t, err, &schemaErr)
	assert.Equal(t, "nonexistent", schemaErr.SchemaName)
}

// TestSchemaRegistryAdapter_HasSchema tests HasSchema method behavior.
func TestSchemaRegistryAdapter_HasSchema(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	// Initially false
	assert.False(t, registry.HasSchema(context.Background(), "test"))

	// After registration, true
	schema := domain.Schema{Name: "test"}
	err := registry.RegisterAll(
		context.Background(),
		[]domain.Schema{schema},
		domain.PropertyBank{},
	)
	require.NoError(t, err)
	assert.True(t, registry.HasSchema(context.Background(), "test"))

	// After re-registration without schema, false
	err = registry.RegisterAll(
		context.Background(),
		[]domain.Schema{},
		domain.PropertyBank{},
	)
	require.NoError(t, err)
	assert.False(t, registry.HasSchema(context.Background(), "test"))
}

// TestSchemaRegistryAdapter_HasProperty tests HasProperty method behavior.
func TestSchemaRegistryAdapter_HasProperty(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	// Initially false
	assert.False(t, registry.HasProperty(context.Background(), "test"))

	// After registration, true
	property := domain.Property{Name: "test"}
	bank := domain.PropertyBank{
		Properties: map[string]domain.Property{"test": property},
	}
	err := registry.RegisterAll(context.Background(), []domain.Schema{}, bank)
	require.NoError(t, err)
	assert.True(t, registry.HasProperty(context.Background(), "test"))

	// After re-registration without property, false
	err = registry.RegisterAll(
		context.Background(),
		[]domain.Schema{},
		domain.PropertyBank{},
	)
	require.NoError(t, err)
	assert.False(t, registry.HasProperty(context.Background(), "test"))
}

// TestSchemaRegistryAdapter_ConcurrentReads tests thread safety with concurrent
// read operations.
func TestSchemaRegistryAdapter_ConcurrentReads(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	// Register test data
	schema := domain.Schema{Name: "test-schema"}
	property := domain.Property{Name: "test-prop"}
	bank := domain.PropertyBank{
		Properties: map[string]domain.Property{"test-prop": property},
	}
	err := registry.RegisterAll(
		context.Background(),
		[]domain.Schema{schema},
		bank,
	)
	require.NoError(t, err)

	// Concurrent reads
	var wg sync.WaitGroup
	for range 100 {
		wg.Add(1)
		go func() {
			defer wg.Done()
			// Test GetSchema
			s, err2 := registry.GetSchema(context.Background(), "test-schema")
			assert.NoError(t, err2)
			assert.Equal(t, "test-schema", s.Name)

			// Test GetProperty
			p, err2 := registry.GetProperty(context.Background(), "test-prop")
			assert.NoError(t, err2)
			assert.Equal(t, "test-prop", p.Name)

			// Test Has methods
			assert.True(
				t,
				registry.HasSchema(context.Background(), "test-schema"),
			)
			assert.True(
				t,
				registry.HasProperty(context.Background(), "test-prop"),
			)
		}()
	}
	wg.Wait()
}

// TestSchemaRegistryAdapter_DefensiveCopying tests returned values cannot
// mutate internal state.
func TestSchemaRegistryAdapter_DefensiveCopying(t *testing.T) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	// Register original schema
	originalSchema := domain.Schema{
		Name: "test",
		Properties: []domain.Property{
			{Name: "field1", Spec: &domain.StringSpec{}},
		},
	}
	originalProperty := domain.Property{Name: "prop1"}
	bank := domain.PropertyBank{
		Properties: map[string]domain.Property{"prop1": originalProperty},
	}

	err := registry.RegisterAll(
		context.Background(),
		[]domain.Schema{originalSchema},
		bank,
	)
	require.NoError(t, err)

	// Get schema and mutate returned copy
	retrievedSchema, err := registry.GetSchema(context.Background(), "test")
	require.NoError(t, err)
	retrievedSchema.Name = "mutated"
	// Mutate property directly since it's now a concrete type
	retrievedSchema.Properties[0].Name = "mutated-field"
	_ = retrievedSchema // intentionally mutated to test defensive copying

	// Get property and mutate returned copy
	retrievedProperty, err := registry.GetProperty(
		context.Background(),
		"prop1",
	)
	require.NoError(t, err)
	retrievedProperty.Name = "mutated-prop"
	_ = retrievedProperty // intentionally mutated to test defensive copying

	// Verify internal state unchanged
	internalSchema, err := registry.GetSchema(context.Background(), "test")
	require.NoError(t, err)
	assert.Equal(t, "test", internalSchema.Name)
	assert.Equal(t, "field1", internalSchema.Properties[0].Name)

	internalProperty, err := registry.GetProperty(context.Background(), "prop1")
	require.NoError(t, err)
	assert.Equal(t, "prop1", internalProperty.Name)
}

// TestSchemaRegistryAdapter_DefensiveCopyingResolvedProperties tests that
// ResolvedProperties are defensively copied to prevent external mutation.
func TestSchemaRegistryAdapter_DefensiveCopyingResolvedProperties(
	t *testing.T,
) {
	registry := NewSchemaRegistryAdapter(logger.NewTest())

	// Create schema with ResolvedProperties
	originalSchema := domain.Schema{
		Name: "test-resolved",
		Properties: []domain.Property{
			{Name: "field1", Spec: &domain.StringSpec{}},
		},
		ResolvedProperties: []domain.Property{
			{Name: "resolved1", Spec: &domain.StringSpec{}},
			{Name: "resolved2", Spec: &domain.StringSpec{}},
		},
	}

	bank := domain.PropertyBank{
		Properties: map[string]domain.Property{},
	}

	err := registry.RegisterAll(
		context.Background(),
		[]domain.Schema{originalSchema},
		bank,
	)
	require.NoError(t, err)

	// Get schema and mutate returned copy's ResolvedProperties
	retrievedSchema, err := registry.GetSchema(
		context.Background(),
		"test-resolved",
	)
	require.NoError(t, err)

	// Mutate the ResolvedProperties slice
	retrievedSchema.ResolvedProperties[0].Name = "mutated-resolved"
	retrievedSchema.ResolvedProperties = append(
		retrievedSchema.ResolvedProperties,
		domain.Property{Name: "added"},
	)
	_ = retrievedSchema // intentionally mutated to test defensive copying

	// Verify internal state unchanged
	internalSchema, err := registry.GetSchema(
		context.Background(),
		"test-resolved",
	)
	require.NoError(t, err)
	assert.Equal(t, "resolved1", internalSchema.ResolvedProperties[0].Name)
	assert.Len(
		t,
		internalSchema.ResolvedProperties,
		2,
	) // Should still be 2, not 3
}
