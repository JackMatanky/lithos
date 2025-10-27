package schema

import (
	"context"
	stdErrors "errors"
	"fmt"
	"strings"
	"sync"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	sharederrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

const baseSchemaName = "base"

type mockSchemaLoaderPort struct {
	schemas             []domain.Schema
	propertyBank        *domain.PropertyBank
	loadSchemasErr      error
	loadPropertyBankErr error
}

func (m *mockSchemaLoaderPort) LoadSchemas(
	ctx context.Context,
) ([]domain.Schema, error) {
	if m.loadSchemasErr != nil {
		return nil, m.loadSchemasErr
	}

	if err := ctx.Err(); err != nil {
		return nil, err
	}

	return m.schemas, nil
}

func (m *mockSchemaLoaderPort) LoadPropertyBank(
	ctx context.Context,
) (*domain.PropertyBank, error) {
	if m.loadPropertyBankErr != nil {
		return nil, m.loadPropertyBankErr
	}

	if err := ctx.Err(); err != nil {
		return nil, err
	}

	return m.propertyBank, nil
}

func TestNewSchemaRegistryAdapter(t *testing.T) {
	loader := &mockSchemaLoaderPort{}
	cfg := newMockConfigPort("/vault")

	adapter := NewSchemaRegistryAdapter(loader, cfg)

	if adapter == nil {
		t.Fatal("expected adapter to be created")
	}

	if adapter.loader != loader {
		t.Error("loader dependency not set correctly")
	}

	if adapter.config != cfg {
		t.Error("config dependency not set correctly")
	}

	if adapter.store == nil {
		t.Error("registry store should be initialized")
	}
}

func TestInitialize_Success(t *testing.T) {
	baseSchema := domain.NewSchema(
		baseSchemaName,
		[]domain.Property{
			domain.NewProperty(
				"title",
				true,
				false,
				domain.StringPropertySpec{
					Enum:    []string{},
					Pattern: "",
				},
			),
			domain.NewProperty(
				"internal_id",
				false,
				false,
				domain.StringPropertySpec{},
			),
		},
	)

	childSchema := domain.NewSchemaWithExtends(
		"article",
		baseSchemaName,
		[]string{"internal_id"},
		[]domain.Property{
			domain.NewProperty(
				"summary",
				false,
				false,
				domain.StringPropertySpec{},
			),
		},
	)

	propertyBank := domain.NewPropertyBank("schemas/properties")

	loader := &mockSchemaLoaderPort{
		schemas:      []domain.Schema{baseSchema, childSchema},
		propertyBank: &propertyBank,
	}
	cfg := newMockConfigPort("/vault")

	adapter := NewSchemaRegistryAdapter(loader, cfg)

	result := adapter.Initialize(context.Background())
	if result.IsErr() {
		t.Fatalf("Initialize returned error: %v", result.Error())
	}

	base, exists := adapter.Get(baseSchemaName)
	if !exists {
		t.Fatal("expected base schema to exist")
	}

	if len(base.GetResolvedProperties()) != 2 {
		t.Errorf(
			"expected base to have 2 resolved properties, got %d",
			len(base.GetResolvedProperties()),
		)
	}

	child, exists := adapter.Get("article")
	if !exists {
		t.Fatal("expected article schema to exist")
	}

	props := child.GetResolvedProperties()
	if len(props) != 2 {
		t.Fatalf(
			"expected article to have 2 resolved properties, got %d",
			len(props),
		)
	}

	names := map[string]bool{}
	for _, prop := range props {
		names[prop.Name] = true
	}

	if !names["title"] {
		t.Error("expected inherited property 'title'")
	}
	if !names["summary"] {
		t.Error("expected child property 'summary'")
	}
	if names["internal_id"] {
		t.Error("expected 'internal_id' to be excluded")
	}
}

func TestInitialize_LoadSchemasError(t *testing.T) {
	loader := &mockSchemaLoaderPort{
		loadSchemasErr: stdErrors.New("failed to load"),
	}
	cfg := newMockConfigPort("/vault")

	adapter := NewSchemaRegistryAdapter(loader, cfg)

	result := adapter.Initialize(context.Background())
	if !result.IsErr() {
		t.Fatal("expected Initialize to fail")
	}
}

func TestInitialize_ContextCancelled(t *testing.T) {
	loader := &mockSchemaLoaderPort{}
	cfg := newMockConfigPort("/vault")
	adapter := NewSchemaRegistryAdapter(loader, cfg)

	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	result := adapter.Initialize(ctx)
	if !result.IsErr() {
		t.Fatal("expected Initialize to respect context cancellation")
	}

	if !stdErrors.Is(result.Error(), context.Canceled) {
		t.Errorf("expected context.Canceled, got %v", result.Error())
	}
}

func TestInitialize_DetectsInheritanceCycles(t *testing.T) {
	schemaA := domain.NewSchemaWithExtends(
		"a",
		"b",
		nil,
		nil,
	)
	schemaB := domain.NewSchemaWithExtends(
		"b",
		"a",
		nil,
		nil,
	)

	propertyBank := domain.NewPropertyBank("schemas/properties")

	loader := &mockSchemaLoaderPort{
		schemas:      []domain.Schema{schemaA, schemaB},
		propertyBank: &propertyBank,
	}
	cfg := newMockConfigPort("/vault")
	adapter := NewSchemaRegistryAdapter(loader, cfg)

	result := adapter.Initialize(context.Background())
	if !result.IsErr() {
		t.Fatal("expected Initialize to fail for cyclic inheritance")
	}

	var schemaErr sharederrors.SchemaError
	if !stdErrors.As(result.Error(), &schemaErr) {
		t.Fatalf("expected SchemaError, got %T", result.Error())
	}

	if !strings.Contains(schemaErr.Error(), "cyclic inheritance") {
		t.Errorf(
			"expected cyclic inheritance message, got %s",
			schemaErr.Error(),
		)
	}
}

func TestGet_ConcurrentAccess(t *testing.T) {
	baseSchema := domain.NewSchema(
		baseSchemaName,
		[]domain.Property{
			domain.NewProperty(
				"title",
				true,
				false,
				domain.StringPropertySpec{},
			),
		},
	)

	propertyBank := domain.NewPropertyBank("schemas/properties")

	loader := &mockSchemaLoaderPort{
		schemas:      []domain.Schema{baseSchema},
		propertyBank: &propertyBank,
	}

	cfg := newMockConfigPort("/vault")
	adapter := NewSchemaRegistryAdapter(loader, cfg)

	if result := adapter.Initialize(context.Background()); result.IsErr() {
		t.Fatalf("unexpected initialization error: %v", result.Error())
	}

	const workers = 8
	var wg sync.WaitGroup
	errCh := make(chan error, workers)

	for range workers {
		wg.Add(1)
		go func() {
			defer wg.Done()

			schema, ok := adapter.Get(baseSchemaName)
			if !ok {
				errCh <- fmt.Errorf("schema not found")
				return
			}

			if schema.Name != baseSchemaName {
				errCh <- fmt.Errorf("unexpected schema name: %s", schema.Name)
			}
		}()
	}

	wg.Wait()
	close(errCh)

	for err := range errCh {
		t.Error(err)
	}
}

func TestGet_NotFound(t *testing.T) {
	propertyBank := domain.NewPropertyBank("schemas/properties")

	loader := &mockSchemaLoaderPort{propertyBank: &propertyBank}
	cfg := newMockConfigPort("/vault")
	adapter := NewSchemaRegistryAdapter(loader, cfg)

	result := adapter.Initialize(context.Background())
	if result.IsErr() {
		t.Fatalf("unexpected initialization error: %v", result.Error())
	}

	_, exists := adapter.Get("missing")
	if exists {
		t.Error("expected missing schema to return false")
	}
}
