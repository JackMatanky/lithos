// Package schema provides SPI adapter implementations for schema registry
// operations.
//
// It implements the hexagonal architecture pattern by providing concrete
// implementations of the SchemaRegistryPort interface defined in the ports
// layer.
// The SchemaRegistryAdapter specifically provides thread-safe in-memory storage
// and retrieval of loaded and resolved schemas and properties.
package schema

import (
	"context"
	"sync"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// Ensure SchemaRegistryAdapter implements SchemaRegistryPort at compile time.
var _ spi.SchemaRegistryPort = (*SchemaRegistryAdapter)(nil)

// SchemaRegistryAdapter implements the SchemaRegistryPort interface using
// thread-safe in-memory maps for schema and property storage.
//
// It provides fast lookups for schemas and properties with defensive copying
// to prevent external mutation of internal state. The adapter uses RWMutex
// for concurrent read access while ensuring exclusive write access during
// registration operations.
//
// Architecture Reference: docs/architecture/components.md#schemaregistryport
// Implementation: In-memory registry with thread-safe access patterns.
type SchemaRegistryAdapter struct {
	// schemas stores schema definitions keyed by name
	schemas map[string]domain.Schema
	// properties stores property definitions keyed by name
	properties map[string]domain.Property
	// mu provides thread-safe access with read/write locking
	mu sync.RWMutex
	// log provides structured logging for operations
	log zerolog.Logger
}

// NewSchemaRegistryAdapter creates a new SchemaRegistryAdapter with dependency
// injection.
//
// The adapter is initialized with empty maps and ready for RegisterAll() calls.
// Logger is injected for structured logging of registry operations.
//
// Returns a pointer to the initialized adapter.
// gocritic hugeParam warning suppressed as this is the correct zerolog usage
// pattern.
func NewSchemaRegistryAdapter(log zerolog.Logger) *SchemaRegistryAdapter {
	return &SchemaRegistryAdapter{
		schemas:    make(map[string]domain.Schema),
		properties: make(map[string]domain.Property),
		mu:         sync.RWMutex{},
		log:        log,
	}
}

// RegisterAll implements SchemaRegistryPort.RegisterAll.
// Registers all schemas and properties into the registry with thread-safe write
// access.
//
// Clears existing entries before registration (idempotent behavior).
// Stores defensive copies to prevent external mutation of registry state.
// Enables re-registration without stale data (aligns with FR9).
//
// Context is used for cancellation during potentially long-running operations.
func (a *SchemaRegistryAdapter) RegisterAll(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
) error {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	a.mu.Lock()
	defer a.mu.Unlock()

	// Clear existing entries (idempotent)
	a.schemas = make(map[string]domain.Schema)
	a.properties = make(map[string]domain.Property)

	// Register schemas (defensive copy - deep copy Properties slice)
	for _, schema := range schemas {
		// Deep copy the Properties slice to prevent external mutation
		propertiesCopy := make([]domain.IProperty, len(schema.Properties))
		copy(propertiesCopy, schema.Properties)
		schemaCopy := schema
		schemaCopy.Properties = propertiesCopy
		a.schemas[schema.Name] = schemaCopy
	}

	// Register properties (defensive copy - Go value semantics)
	for id, prop := range bank.Properties {
		a.properties[id] = prop // Go copies by value
	}

	// Log registration
	a.log.Info().
		Int("schemas", len(schemas)).
		Int("properties", len(bank.Properties)).
		Msg("registered schemas and properties")

	return nil
}

// GetSchema implements SchemaRegistryPort.GetSchema.
// Retrieves a schema by name from the registry with thread-safe read access.
//
// Returns SchemaError with ErrNotFound classification when schema doesn't
// exist.
// Returns defensive copy to prevent external mutation of registry state.
//
// Context is used for cancellation during potentially long-running operations.
func (a *SchemaRegistryAdapter) GetSchema(
	ctx context.Context,
	name string,
) (domain.Schema, error) {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return domain.Schema{}, ctx.Err()
	default:
	}

	a.mu.RLock()
	defer a.mu.RUnlock()

	schema, exists := a.schemas[name]
	if !exists {
		return domain.Schema{}, lithoserrors.NewSchemaError(
			"schema not found",
			name,
			lithoserrors.ErrNotFound,
		)
	}

	// Return defensive copy to prevent external mutation
	propertiesCopy := make([]domain.IProperty, len(schema.Properties))
	copy(propertiesCopy, schema.Properties)
	schemaCopy := schema
	schemaCopy.Properties = propertiesCopy

	return schemaCopy, nil
}

// GetProperty implements SchemaRegistryPort.GetProperty.
// Retrieves a property from the property bank by name with thread-safe read
// access.
//
// Returns SchemaError with ErrNotFound classification when property doesn't
// exist.
// Returns defensive copy to prevent external mutation of registry state.
//
// Context is used for cancellation during potentially long-running operations.
func (a *SchemaRegistryAdapter) GetProperty(
	ctx context.Context,
	name string,
) (domain.Property, error) {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return domain.Property{}, ctx.Err()
	default:
	}

	a.mu.RLock()
	defer a.mu.RUnlock()

	property, exists := a.properties[name]
	if !exists {
		return domain.Property{}, lithoserrors.NewSchemaError(
			"property not found",
			name,
			lithoserrors.ErrNotFound,
		)
	}

	return property, nil // Returns copy (Go value semantics)
}

// HasSchema implements SchemaRegistryPort.HasSchema.
// Checks if a schema exists in the registry with thread-safe read access.
//
// Never errors, returns bool only for existence check.
// Thread-safe for concurrent access.
//
// Context is used for cancellation during potentially long-running operations.
func (a *SchemaRegistryAdapter) HasSchema(
	ctx context.Context,
	name string,
) bool {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return false
	default:
	}

	a.mu.RLock()
	defer a.mu.RUnlock()

	_, exists := a.schemas[name]
	return exists
}

// HasProperty implements SchemaRegistryPort.HasProperty.
// Checks if a property exists in the property bank with thread-safe read
// access.
//
// Never errors, returns bool only for existence check.
// Thread-safe for concurrent access.
//
// Context is used for cancellation during potentially long-running operations.
func (a *SchemaRegistryAdapter) HasProperty(
	ctx context.Context,
	name string,
) bool {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return false
	default:
	}

	a.mu.RLock()
	defer a.mu.RUnlock()

	_, exists := a.properties[name]
	return exists
}
