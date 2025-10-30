package schema

import (
	"context"
	"fmt"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
)

// SchemaEngine orchestrates the complete schema processing pipeline from
// loading through validation, resolution, and registration.
//
// SchemaEngine coordinates the schema loading process by executing stages in
// the documented order, ensuring proper dependency handling and fail-fast
// behavior. It provides centralized access to resolved schemas and properties
// through generic accessor methods.
//
// Architecture Reference: docs/architecture/components.md#schemaengine
// Requirements: FR5 (Schema Loading), FR7 (Schema Registry), NFR3 (Indexing
// Observability) from docs/prd/requirements.md
//
// Processing Pipeline:
//  1. SchemaPort.Load() - Load raw schemas and property bank from storage
//  2. SchemaValidator.ValidateAll() - Validate structural integrity and
//     cross-schema references
//  3. SchemaResolver.Resolve() - Resolve inheritance chains and $ref
//     substitutions
//  4. SchemaRegistryPort.RegisterAll() - Register resolved schemas for fast
//     lookups
//
// Each stage is logged with duration for observability (NFR3 requirement).
// Fail-fast behavior ensures any stage failure stops the pipeline immediately.
//
// Generic Accessors:
// SchemaEngine provides type-safe generic methods for schema and property
// retrieval:
//   - Get[Schema](ctx, "schema-name") retrieves a resolved schema
//   - Get[Property](ctx, "property-name") retrieves a property from the bank
//   - Has[Schema](ctx, "schema-name") checks schema existence
//   - Has[Property](ctx, "property-name") checks property existence
//
// Dependencies:
//   - SchemaPort: Loads raw schemas from storage (injected)
//   - SchemaRegistryPort: Provides fast in-memory schema access (injected)
//   - Logger: Provides observability for each pipeline stage (injected)
//   - SchemaValidator: Validates schemas (internally instantiated)
//   - SchemaResolver: Resolves inheritance (internally instantiated).
type SchemaEngine struct {
	// Injected dependencies
	schemaPort   spi.SchemaPort
	registryPort spi.SchemaRegistryPort
	log          zerolog.Logger

	// Internal services (not injected)
	validator *SchemaValidator
	resolver  *SchemaResolver
}

// NewSchemaEngine creates a new SchemaEngine with the specified dependencies.
//
// The constructor validates that all injected dependencies are non-nil and
// internally instantiates the pure domain services (SchemaValidator and
// SchemaResolver) that have no external dependencies.
//
// Dependencies:
//   - schemaPort: Interface for loading schemas from storage
//   - registryPort: Interface for fast in-memory schema access
//   - log: Logger for pipeline stage observability
//
// Returns error if any dependency is nil.
func NewSchemaEngine(
	schemaPort spi.SchemaPort,
	registryPort spi.SchemaRegistryPort,
	log zerolog.Logger,
) (*SchemaEngine, error) {
	// Validate injected dependencies
	if schemaPort == nil {
		return nil, fmt.Errorf("schemaPort cannot be nil")
	}
	if registryPort == nil {
		return nil, fmt.Errorf("registryPort cannot be nil")
	}

	// Create engine with dependencies and internal services
	return &SchemaEngine{
		schemaPort:   schemaPort,
		registryPort: registryPort,
		log:          log,
		validator:    NewSchemaValidator(),
		resolver:     NewSchemaResolver(),
	}, nil
}

// Load executes the complete schema processing pipeline in documented order.
//
// Pipeline Stages:
//  1. Load raw schemas and property bank from storage
//  2. Validate structural integrity and cross-schema references
//  3. Resolve inheritance chains and $ref substitutions
//  4. Register resolved schemas for fast lookups
//
// Each stage is logged with duration for observability (NFR3 requirement).
// Fail-fast behavior: any stage failure stops the pipeline and returns error.
//
// Context is used for cancellation and deadline propagation across all stages.
//
// Returns error if any stage fails. On success, schemas are loaded and
// registered for use by accessor methods.
func (e *SchemaEngine) Load(ctx context.Context) error {
	startTime := time.Now()

	schemas, bank, loadErr := e.loadSchemas(ctx)
	if loadErr != nil {
		return loadErr
	}

	if validateErr := e.validateSchemas(ctx, schemas, bank); validateErr != nil {
		return validateErr
	}

	resolvedSchemas, resolveErr := e.resolveSchemas(ctx, schemas, bank)
	if resolveErr != nil {
		return resolveErr
	}

	if registerErr := e.registerSchemas(ctx, resolvedSchemas, bank, startTime); registerErr != nil {
		return registerErr
	}

	return nil
}

// loadSchemas executes the schema loading stage.
func (e *SchemaEngine) loadSchemas(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	e.log.Info().Msg("loading schemas...")
	stageStart := time.Now()

	schemas, bank, err := e.schemaPort.Load(ctx)
	if err != nil {
		e.log.Error().Err(err).Msg("failed to load schemas")
		return nil, domain.PropertyBank{}, fmt.Errorf(
			"schema loading failed: %w",
			err,
		)
	}

	stageDuration := time.Since(stageStart)
	e.log.Info().
		Int("schemas", len(schemas)).
		Int("properties", len(bank.Properties)).
		Dur("duration_ms", stageDuration).
		Msgf("loaded %d schemas and %d properties in %v",
			len(schemas), len(bank.Properties), stageDuration)

	return schemas, bank, nil
}

// validateSchemas executes the schema validation stage.
func (e *SchemaEngine) validateSchemas(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
) error {
	e.log.Info().Msg("validating schemas...")
	stageStart := time.Now()

	if err := e.validator.ValidateAll(ctx, schemas, bank); err != nil {
		e.log.Error().Err(err).Msg("schema validation failed")
		return fmt.Errorf("schema validation failed: %w", err)
	}

	stageDuration := time.Since(stageStart)
	e.log.Info().
		Dur("duration_ms", stageDuration).
		Msgf("validation complete in %v", stageDuration)

	return nil
}

// resolveSchemas executes the schema resolution stage.
func (e *SchemaEngine) resolveSchemas(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
) ([]domain.Schema, error) {
	e.log.Info().Msg("resolving inheritance...")
	stageStart := time.Now()

	resolvedSchemas, err := e.resolver.Resolve(ctx, schemas, bank)
	if err != nil {
		e.log.Error().Err(err).Msg("schema resolution failed")
		return nil, fmt.Errorf("schema resolution failed: %w", err)
	}

	stageDuration := time.Since(stageStart)
	e.log.Info().
		Dur("duration_ms", stageDuration).
		Msgf("resolution complete in %v", stageDuration)

	return resolvedSchemas, nil
}

// registerSchemas executes the schema registration stage.
func (e *SchemaEngine) registerSchemas(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
	startTime time.Time,
) error {
	e.log.Info().Msg("registering schemas...")
	stageStart := time.Now()

	if err := e.registryPort.RegisterAll(ctx, schemas, bank); err != nil {
		e.log.Error().Err(err).Msg("schema registration failed")
		return fmt.Errorf("schema registration failed: %w", err)
	}

	stageDuration := time.Since(stageStart)
	totalDuration := time.Since(startTime)
	e.log.Info().
		Int("schemas", len(schemas)).
		Dur("stage_duration_ms", stageDuration).
		Dur("total_duration_ms", totalDuration).
		Msgf("schema engine ready: %d schemas registered in %v total",
			len(schemas), totalDuration)

	return nil
}

// Get retrieves a schema or property by name using Go generics.
//
// Type Parameter T must be either domain.Schema or domain.Property.
// The function delegates to the appropriate SchemaRegistryPort method based
// on the type parameter.
//
// Usage Examples:
//
//	schema, err := Get[domain.Schema](engine, ctx, "meeting-note")
//	property, err := Get[domain.Property](engine, ctx, "standard_title")
//
// Returns SchemaError with ErrNotFound classification when the requested
// schema or property doesn't exist in the registry.
//
// Context is used for cancellation during registry access.
func Get[T domain.Schema | domain.Property](
	e *SchemaEngine,
	ctx context.Context,
	name string,
) (T, error) {
	var zero T

	// Use type switch to determine which registry method to call
	switch any(zero).(type) {
	case domain.Schema:
		schema, err := e.registryPort.GetSchema(ctx, name)
		if err != nil {
			return zero, err
		}
		return any(schema).(T), nil

	case domain.Property:
		property, err := e.registryPort.GetProperty(ctx, name)
		if err != nil {
			return zero, err
		}
		return any(property).(T), nil

	default:
		return zero, fmt.Errorf("unsupported type: must be Schema or Property")
	}
}

// Has checks if a schema or property exists by name using Go generics.
//
// Type Parameter T must be either domain.Schema or domain.Property.
// The function delegates to the appropriate SchemaRegistryPort method based
// on the type parameter.
//
// Usage Examples:
//
//	exists := Has[domain.Schema](engine, ctx, "meeting-note")
//	exists := Has[domain.Property](engine, ctx, "standard_title")
//
// Never returns an error - only boolean existence check.
// Returns false if the schema or property doesn't exist.
//
// Context is used for cancellation during registry access.
func Has[T domain.Schema | domain.Property](
	e *SchemaEngine,
	ctx context.Context,
	name string,
) bool {
	var zero T

	// Use type switch to determine which registry method to call
	switch any(zero).(type) {
	case domain.Schema:
		return e.registryPort.HasSchema(ctx, name)

	case domain.Property:
		return e.registryPort.HasProperty(ctx, name)

	default:
		return false
	}
}
