// Package schema provides domain services for schema validation and processing.
// This package implements the application layer business logic for coordinating
// schema loading, validation, and access operations.
package schema

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// SchemaEngine implements domain service for coordinating schema loading,
// validation, and business logic operations. SchemaEngine serves as the ONLY
// gateway for all schema access operations in the application layer.
//
// All validation methods extracted from domain models per architectural
// refactoring.
type SchemaEngine struct {
	loader             spi.SchemaLoaderPort
	registry           spi.SchemaRegistryPort
	validator          *SchemaValidator
	propertyBankMap    map[string]domain.Property // Stores loaded property bank for property access
	propertyBankLoaded bool                       // Tracks if property bank has been loaded
}

// NewSchemaEngine creates a new SchemaEngine with dependency injection.
// SchemaEngine coordinates schema operations through its dependencies:
// - SchemaLoaderPort for loading schemas and property banks from storage
// - SchemaRegistryPort for accessing resolved schemas
// - SchemaValidator for validating schema definitions and property banks.
func NewSchemaEngine(
	loader spi.SchemaLoaderPort,
	registry spi.SchemaRegistryPort,
	validator *SchemaValidator,
) *SchemaEngine {
	return &SchemaEngine{
		loader:             loader,
		registry:           registry,
		validator:          validator,
		propertyBankMap:    make(map[string]domain.Property),
		propertyBankLoaded: false,
	}
}

// LoadSchema loads and validates all schema definitions.
// Returns Result[[]Schema] with loaded and validated schemas.
// This method coordinates SchemaLoaderPort loading with SchemaValidator
// validation.
func (e *SchemaEngine) LoadSchema(
	ctx context.Context,
) lithoserrors.Result[[]domain.Schema] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[[]domain.Schema](ctx.Err())
	default:
	}

	// Load schemas through SchemaLoaderPort
	schemas, err := e.loader.LoadSchemas(ctx)
	if err != nil {
		return lithoserrors.Err[[]domain.Schema](err)
	}

	// Validate each schema using SchemaValidator
	validatedSchemas := make([]domain.Schema, 0, len(schemas))
	for _, schema := range schemas {
		// Validate schema definition
		validationResult := e.validator.ValidateSchema(ctx, &schema)
		if validationResult.IsErr() {
			return lithoserrors.Err[[]domain.Schema](
				lithoserrors.NewSchemaError(
					schema.Name,
					"schema validation failed",
					validationResult.Error(),
				),
			)
		}

		// Check if validation found errors
		if result, _ := validationResult.Unwrap(); !result.IsValid() {
			return lithoserrors.Err[[]domain.Schema](
				lithoserrors.NewSchemaError(
					schema.Name,
					"schema validation failed",
					nil,
				),
			)
		}

		validatedSchemas = append(validatedSchemas, schema)
	}

	return lithoserrors.Ok[[]domain.Schema](validatedSchemas)
}

// LoadPropertyBank loads and validates the property bank.
// Returns Result[*PropertyBank] with loaded and validated property bank.
// This method coordinates SchemaLoaderPort loading with SchemaValidator
// validation.
// Stores the loaded property bank for subsequent property access operations.
func (e *SchemaEngine) LoadPropertyBank(
	ctx context.Context,
) lithoserrors.Result[*domain.PropertyBank] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[*domain.PropertyBank](ctx.Err())
	default:
	}

	// Load property bank through SchemaLoaderPort
	propertyBank, err := e.loader.LoadPropertyBank(ctx)
	if err != nil {
		return lithoserrors.Err[*domain.PropertyBank](err)
	}

	// Validate property bank using SchemaValidator
	validationResult := e.validator.ValidatePropertyBank(ctx, propertyBank)
	if validationResult.IsErr() {
		return lithoserrors.Err[*domain.PropertyBank](
			lithoserrors.NewSchemaError(
				"property_bank",
				"property bank validation failed",
				validationResult.Error(),
			),
		)
	}

	// Check if validation found errors
	if result, _ := validationResult.Unwrap(); !result.IsValid() {
		return lithoserrors.Err[*domain.PropertyBank](
			lithoserrors.NewSchemaError(
				"property_bank",
				"property bank validation failed",
				nil,
			),
		)
	}

	// Store the validated property bank map for property access operations
	e.propertyBankMap = make(map[string]domain.Property)
	for name, prop := range propertyBank.Properties {
		e.propertyBankMap[name] = prop
	}
	e.propertyBankLoaded = true

	return lithoserrors.Ok[*domain.PropertyBank](propertyBank)
}

// GetSchema retrieves a validated schema by name.
// Returns Result[Schema] with the resolved schema.
// Schema must be pre-validated by SchemaEngine before retrieval.
func (e *SchemaEngine) GetSchema(
	ctx context.Context,
	name string,
) lithoserrors.Result[domain.Schema] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[domain.Schema](ctx.Err())
	default:
	}

	// Retrieve schema from registry
	schema, exists := e.registry.Get(name)
	if !exists {
		return lithoserrors.Err[domain.Schema](
			lithoserrors.NewSchemaNotFoundError(name),
		)
	}

	return lithoserrors.Ok[domain.Schema](schema)
}

// HasSchema checks if a schema exists by name.
// Returns Result[bool] indicating schema existence.
func (e *SchemaEngine) HasSchema(
	ctx context.Context,
	name string,
) lithoserrors.Result[bool] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[bool](ctx.Err())
	default:
	}

	// Check schema existence in registry
	_, exists := e.registry.Get(name)
	return lithoserrors.Ok[bool](exists)
}

// GetProperty retrieves a property from the property bank by name.
// Returns Result[Property] with the property definition.
// Property bank must be pre-loaded and validated by SchemaEngine.
func (e *SchemaEngine) GetProperty(
	ctx context.Context,
	name string,
) lithoserrors.Result[domain.Property] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[domain.Property](ctx.Err())
	default:
	}

	// Check if property bank has been loaded
	if !e.propertyBankLoaded {
		return lithoserrors.Err[domain.Property](
			lithoserrors.NewSchemaError(
				"property_bank",
				"property bank not loaded - call LoadPropertyBank first",
				nil,
			),
		)
	}

	// Retrieve property from property bank map
	property, exists := e.propertyBankMap[name]
	if !exists {
		return lithoserrors.Err[domain.Property](
			lithoserrors.NewSchemaError(
				name,
				"property not found in property bank",
				nil,
			),
		)
	}

	return lithoserrors.Ok[domain.Property](property)
}

// HasProperty checks if a property exists in the property bank by name.
// Returns Result[bool] indicating property existence.
// Property bank must be pre-loaded by SchemaEngine.
func (e *SchemaEngine) HasProperty(
	ctx context.Context,
	name string,
) lithoserrors.Result[bool] {
	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return lithoserrors.Err[bool](ctx.Err())
	default:
	}

	// Check if property bank has been loaded
	if !e.propertyBankLoaded {
		return lithoserrors.Err[bool](
			lithoserrors.NewSchemaError(
				"property_bank",
				"property bank not loaded - call LoadPropertyBank first",
				nil,
			),
		)
	}

	// Check property existence in property bank map
	_, exists := e.propertyBankMap[name]
	return lithoserrors.Ok[bool](exists)
}
