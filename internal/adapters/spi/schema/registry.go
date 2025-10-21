// Package schema provides filesystem-based implementations of schema loading
// ports for hexagonal architecture.
//
// This file contains the SchemaRegistryAdapter which coordinates schema
// loading, inheritance resolution, and registry storage behind the SPI port.
package schema

import (
	"context"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	sharederrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/JackMatanky/lithos/internal/shared/registry"
)

// SchemaRegistryAdapter implements SchemaRegistryPort using the shared
// registry package for thread-safe schema storage.
//
// The adapter coordinates schema loading via SchemaLoaderPort, resolves
// inheritance chains, and exposes read-only access to resolved schemas.
type SchemaRegistryAdapter struct {
	loader spi.SchemaLoaderPort
	config spi.ConfigPort
	store  registry.Registry[domain.Schema]
}

/* ---------------------------------------------------------- */
/*                         Constructor                        */
/* ---------------------------------------------------------- */

// NewSchemaRegistryAdapter creates a new SchemaRegistryAdapter with dependency
// injection for loader and config ports.
func NewSchemaRegistryAdapter(
	loader spi.SchemaLoaderPort,
	config spi.ConfigPort,
) *SchemaRegistryAdapter {
	return &SchemaRegistryAdapter{
		loader: loader,
		config: config,
		store:  registry.New[domain.Schema](),
	}
}

/* ---------------------------------------------------------- */
/*                    Initialization Process                  */
/* ---------------------------------------------------------- */

// Initialize loads schemas and property bank data, resolves inheritance, and
// populates the internal registry. Uses Result[T] pattern for error handling.
func (s *SchemaRegistryAdapter) Initialize(
	ctx context.Context,
) sharederrors.Result[struct{}] {
	if err := s.ensureContextActive(ctx); err != nil {
		return sharederrors.Err[struct{}](err)
	}

	s.logInitialization()

	schemas, schemaErr := s.loadSchemas(ctx)
	if schemaErr != nil {
		return sharederrors.Err[struct{}](schemaErr)
	}

	propertyBank, bankErr := s.loadPropertyBank(ctx)
	if bankErr != nil {
		return sharederrors.Err[struct{}](bankErr)
	}

	if err := s.ensurePropertyBankValid(propertyBank); err != nil {
		return sharederrors.Err[struct{}](err)
	}

	resolved, resolveErr := s.resolveSchemas(ctx, schemas)
	if resolveErr != nil {
		return sharederrors.Err[struct{}](resolveErr)
	}

	s.refreshRegistry(resolved)
	s.logCompletion(len(resolved))

	return sharederrors.Ok(struct{}{})
}

func (s *SchemaRegistryAdapter) ensureContextActive(ctx context.Context) error {
	return ctx.Err()
}

func (s *SchemaRegistryAdapter) loadSchemas(
	ctx context.Context,
) ([]domain.Schema, error) {
	return s.loader.LoadSchemas(ctx)
}

func (s *SchemaRegistryAdapter) loadPropertyBank(
	ctx context.Context,
) (*domain.PropertyBank, error) {
	bank, err := s.loader.LoadPropertyBank(ctx)
	if err != nil {
		return nil, err
	}

	if bank == nil {
		return nil, sharederrors.NewSchemaError(
			"property_bank",
			"loader returned nil property bank",
		)
	}

	return bank, nil
}

func (s *SchemaRegistryAdapter) ensurePropertyBankValid(
	propertyBank *domain.PropertyBank,
) error {
	return propertyBank.Validate()
}

func (s *SchemaRegistryAdapter) resolveSchemas(
	ctx context.Context,
	schemas []domain.Schema,
) (map[string]domain.Schema, error) {
	resolver, err := NewInheritanceResolver(schemas)
	if err != nil {
		return nil, err
	}

	return resolver.ResolveAll(ctx)
}

func (s *SchemaRegistryAdapter) refreshRegistry(
	resolved map[string]domain.Schema,
) {
	s.store.Clear()
	for name, schema := range resolved {
		s.store.Register(name, schema)
	}
}

func (s *SchemaRegistryAdapter) logInitialization() {
	cfg := s.config.Config()
	entry := newRegistryLogger()
	entry.Info().
		Str("operation", "initialize").
		Str("schemas_dir", cfg.SchemasDir).
		Msg("initializing schema registry")
}

func (s *SchemaRegistryAdapter) logCompletion(count int) {
	entry := newRegistryLogger()
	entry.Info().
		Str("operation", "initialize").
		Int("schema_count", count).
		Msg("schema registry initialized successfully")
}

func newRegistryLogger() logger.Logger {
	return logger.WithComponent("spi.schema.registry")
}

/* ---------------------------------------------------------- */
/*                    SchemaRegistryPort API                  */
/* ---------------------------------------------------------- */

// Get implements SchemaRegistryPort.Get.
func (s *SchemaRegistryAdapter) Get(name string) (domain.Schema, bool) {
	if strings.TrimSpace(name) == "" {
		return domain.Schema{}, false
	}

	if !s.store.Exists(name) {
		return domain.Schema{}, false
	}

	return s.store.Get(name), true
}
