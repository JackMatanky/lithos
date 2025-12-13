// Package schema provides domain services for schema validation and processing.
package schema

import (
	"context"
	"errors"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// assertSchemaResult checks result against error expectation.
func assertSchemaResult[T any](
	t *testing.T,
	result lithoserrors.Result[T],
	expectErr bool,
) {
	t.Helper()
	if expectErr {
		if result.IsOk() {
			t.Errorf("expected error, got success")
		}
		return
	}

	if result.IsErr() {
		t.Errorf("expected success, got error: %v", result.Error())
	}
}

// verifyPropertyBankLoaded checks that property bank was loaded successfully.
func verifyPropertyBankLoaded(
	t *testing.T,
	result lithoserrors.Result[*domain.PropertyBank],
	engine *SchemaEngine,
) {
	t.Helper()
	bank, _ := result.Unwrap()
	if bank == nil {
		t.Errorf("expected property bank, got nil")
	}
	if engine.propertyBankMap == nil {
		t.Errorf("property bank not stored in engine")
	}
}

// loadPropertyBankForTest loads property bank for testing.
func loadPropertyBankForTest(t *testing.T, engine *SchemaEngine) {
	t.Helper()
	loadResult := engine.LoadPropertyBank(context.Background())
	if loadResult.IsErr() {
		t.Fatalf("failed to load property bank: %v", loadResult.Error())
	}
}

// verifyPropertyName checks that retrieved property has expected name.
func verifyPropertyName(
	t *testing.T,
	result lithoserrors.Result[domain.Property],
	expectedName string,
) {
	t.Helper()
	property, _ := result.Unwrap()
	if property.Name != expectedName {
		t.Errorf(
			"expected property name %s, got %s",
			expectedName,
			property.Name,
		)
	}
}

// verifyPropertyExists checks that property existence matches expectation.
func verifyPropertyExists(
	t *testing.T,
	result lithoserrors.Result[bool],
	expected bool,
) {
	t.Helper()
	exists, _ := result.Unwrap()
	if exists != expected {
		t.Errorf("expected %v, got %v", expected, exists)
	}
}

// verifySchemaName checks that retrieved schema has expected name.
func verifySchemaName(
	t *testing.T,
	result lithoserrors.Result[domain.Schema],
	expectedName string,
) {
	t.Helper()
	schema, _ := result.Unwrap()
	if schema.Name != expectedName {
		t.Errorf("expected schema name %s, got %s", expectedName, schema.Name)
	}
}

// verifySchemaExists checks that schema existence matches expectation.
func verifySchemaExists(
	t *testing.T,
	result lithoserrors.Result[bool],
	expected bool,
) {
	t.Helper()
	exists, _ := result.Unwrap()
	if exists != expected {
		t.Errorf("expected %v, got %v", expected, exists)
	}
}

// mockSchemaLoaderPort implements SchemaLoaderPort for testing.
type mockSchemaLoaderPort struct {
	schemas      []domain.Schema
	propertyBank *domain.PropertyBank
	loadErr      error
}

func (m *mockSchemaLoaderPort) LoadSchemas(
	ctx context.Context,
) ([]domain.Schema, error) {
	if m.loadErr != nil {
		return nil, m.loadErr
	}
	return m.schemas, nil
}

func (m *mockSchemaLoaderPort) LoadPropertyBank(
	ctx context.Context,
) (*domain.PropertyBank, error) {
	if m.loadErr != nil {
		return nil, m.loadErr
	}
	return m.propertyBank, nil
}

// mockSchemaRegistryPort implements SchemaRegistryPort for testing.
type mockSchemaRegistryPort struct {
	schemas map[string]domain.Schema
}

func (m *mockSchemaRegistryPort) Get(name string) (domain.Schema, bool) {
	schema, exists := m.schemas[name]
	return schema, exists
}

func TestSchemaEngine_LoadSchema(t *testing.T) {
	tests := []struct {
		name          string
		schemas       []domain.Schema
		loaderErr     error
		validatorErr  error
		expectErr     bool
		expectedCount int
	}{
		{
			name: "successful schema loading and validation",
			schemas: []domain.Schema{
				{Name: "test1", Properties: []domain.Property{}},
				{Name: "test2", Properties: []domain.Property{}},
			},
			expectErr:     false,
			expectedCount: 2,
		},
		{
			name:      "loader error",
			loaderErr: errors.New("load failed"),
			expectErr: true,
		},
		{
			name: "validation error",
			schemas: []domain.Schema{
				{Name: "", Properties: []domain.Property{}}, // Invalid name
			},
			expectErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup mocks
			loader := &mockSchemaLoaderPort{
				schemas: tt.schemas,
				loadErr: tt.loaderErr,
			}
			registry := &mockSchemaRegistryPort{}
			validator := NewSchemaValidator()

			engine := NewSchemaEngine(loader, registry, validator)

			// Execute
			result := engine.LoadSchema(context.Background())

			// Assert
			assertSchemaResult(t, result, tt.expectErr)
			if !tt.expectErr {
				schemas, _ := result.Unwrap()
				if len(schemas) != tt.expectedCount {
					t.Errorf(
						"expected %d schemas, got %d",
						tt.expectedCount,
						len(schemas),
					)
				}
			}
		})
	}
}

func TestSchemaEngine_LoadPropertyBank(t *testing.T) {
	tests := []struct {
		name         string
		propertyBank *domain.PropertyBank
		loaderErr    error
		expectErr    bool
	}{
		{
			name: "successful property bank loading and validation",
			propertyBank: &domain.PropertyBank{
				Location:   "test",
				Properties: map[string]domain.Property{},
			},
			expectErr: false,
		},
		{
			name:      "loader error",
			loaderErr: errors.New("load failed"),
			expectErr: true,
		},
		{
			name: "validation error",
			propertyBank: &domain.PropertyBank{
				Location:   "",
				Properties: map[string]domain.Property{}, // Invalid location
			},
			expectErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup mocks
			loader := &mockSchemaLoaderPort{
				propertyBank: tt.propertyBank,
				loadErr:      tt.loaderErr,
			}
			registry := &mockSchemaRegistryPort{}
			validator := NewSchemaValidator()

			engine := NewSchemaEngine(loader, registry, validator)

			// Execute
			result := engine.LoadPropertyBank(context.Background())

			// Assert
			assertSchemaResult(t, result, tt.expectErr)
			if !tt.expectErr {
				verifyPropertyBankLoaded(t, result, engine)
			}
		})
	}
}

func TestSchemaEngine_GetSchema(t *testing.T) {
	testSchema := domain.Schema{Name: "test", Properties: []domain.Property{}}

	tests := []struct {
		name       string
		schemaName string
		schemas    map[string]domain.Schema
		expectErr  bool
	}{
		{
			name:       "schema found",
			schemaName: "test",
			schemas:    map[string]domain.Schema{"test": testSchema},
			expectErr:  false,
		},
		{
			name:       "schema not found",
			schemaName: "missing",
			schemas:    map[string]domain.Schema{"test": testSchema},
			expectErr:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup mocks
			loader := &mockSchemaLoaderPort{}
			registry := &mockSchemaRegistryPort{schemas: tt.schemas}
			validator := NewSchemaValidator()

			engine := NewSchemaEngine(loader, registry, validator)

			// Execute
			result := engine.GetSchema(context.Background(), tt.schemaName)

			// Assert
			assertSchemaResult(t, result, tt.expectErr)
			if !tt.expectErr {
				verifySchemaName(t, result, tt.schemaName)
			}
		})
	}
}

func TestSchemaEngine_HasSchema(t *testing.T) {
	testSchema := domain.Schema{Name: "test", Properties: []domain.Property{}}

	tests := []struct {
		name       string
		schemaName string
		schemas    map[string]domain.Schema
		expected   bool
	}{
		{
			name:       "schema exists",
			schemaName: "test",
			schemas:    map[string]domain.Schema{"test": testSchema},
			expected:   true,
		},
		{
			name:       "schema does not exist",
			schemaName: "missing",
			schemas:    map[string]domain.Schema{"test": testSchema},
			expected:   false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup mocks
			loader := &mockSchemaLoaderPort{}
			registry := &mockSchemaRegistryPort{schemas: tt.schemas}
			validator := NewSchemaValidator()

			engine := NewSchemaEngine(loader, registry, validator)

			// Execute
			result := engine.HasSchema(context.Background(), tt.schemaName)

			// Assert
			assertSchemaResult(t, result, false)
			if result.IsOk() {
				verifySchemaExists(t, result, tt.expected)
			}
		})
	}
}

func TestSchemaEngine_GetProperty(t *testing.T) {
	testProperty := domain.Property{
		Name: "testProp",
		Spec: domain.StringPropertySpec{},
	}

	propertyBank := &domain.PropertyBank{
		Location:   "test",
		Properties: map[string]domain.Property{"testProp": testProperty},
	}

	tests := []struct {
		name         string
		propertyName string
		loadBank     bool
		expectErr    bool
	}{
		{
			name:         "property found",
			propertyName: "testProp",
			loadBank:     true,
			expectErr:    false,
		},
		{
			name:         "property not found",
			propertyName: "missing",
			loadBank:     true,
			expectErr:    true,
		},
		{
			name:         "property bank not loaded",
			propertyName: "testProp",
			loadBank:     false,
			expectErr:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup mocks
			loader := &mockSchemaLoaderPort{propertyBank: propertyBank}
			registry := &mockSchemaRegistryPort{}
			validator := NewSchemaValidator()

			engine := NewSchemaEngine(loader, registry, validator)

			// Load property bank if required
			if tt.loadBank {
				loadPropertyBankForTest(t, engine)
			}

			// Execute
			result := engine.GetProperty(context.Background(), tt.propertyName)

			// Assert
			assertSchemaResult(t, result, tt.expectErr)
			if !tt.expectErr {
				verifyPropertyName(t, result, tt.propertyName)
			}
		})
	}
}

func TestSchemaEngine_HasProperty(t *testing.T) {
	testProperty := domain.Property{
		Name: "testProp",
		Spec: domain.StringPropertySpec{},
	}

	propertyBank := &domain.PropertyBank{
		Location:   "test",
		Properties: map[string]domain.Property{"testProp": testProperty},
	}

	tests := []struct {
		name         string
		propertyName string
		loadBank     bool
		expected     bool
		expectErr    bool
	}{
		{
			name:         "property exists",
			propertyName: "testProp",
			loadBank:     true,
			expected:     true,
			expectErr:    false,
		},
		{
			name:         "property does not exist",
			propertyName: "missing",
			loadBank:     true,
			expected:     false,
			expectErr:    false,
		},
		{
			name:         "property bank not loaded",
			propertyName: "testProp",
			loadBank:     false,
			expectErr:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup mocks
			loader := &mockSchemaLoaderPort{propertyBank: propertyBank}
			registry := &mockSchemaRegistryPort{}
			validator := NewSchemaValidator()

			engine := NewSchemaEngine(loader, registry, validator)

			// Load property bank if required
			if tt.loadBank {
				loadPropertyBankForTest(t, engine)
			}

			// Execute
			result := engine.HasProperty(context.Background(), tt.propertyName)

			// Assert
			assertSchemaResult(t, result, tt.expectErr)
			if !tt.expectErr {
				verifyPropertyExists(t, result, tt.expected)
			}
		})
	}
}

func TestSchemaEngine_ContextCancellation(t *testing.T) {
	// Setup mocks
	loader := &mockSchemaLoaderPort{}
	registry := &mockSchemaRegistryPort{}
	validator := NewSchemaValidator()

	engine := NewSchemaEngine(loader, registry, validator)

	// Create canceled context
	ctx, cancel := context.WithCancel(context.Background())
	cancel() // Cancel immediately

	// Test all methods with canceled context
	testCases := []struct {
		name string
		test func() error
	}{
		{"LoadSchema", func() error {
			result := engine.LoadSchema(ctx)
			if result.IsOk() {
				return nil
			}
			return result.Error()
		}},
		{"LoadPropertyBank", func() error {
			result := engine.LoadPropertyBank(ctx)
			if result.IsOk() {
				return nil
			}
			return result.Error()
		}},
		{"GetSchema", func() error {
			result := engine.GetSchema(ctx, "test")
			if result.IsOk() {
				return nil
			}
			return result.Error()
		}},
		{"HasSchema", func() error {
			result := engine.HasSchema(ctx, "test")
			if result.IsOk() {
				return nil
			}
			return result.Error()
		}},
		{"GetProperty", func() error {
			result := engine.GetProperty(ctx, "test")
			if result.IsOk() {
				return nil
			}
			return result.Error()
		}},
		{"HasProperty", func() error {
			result := engine.HasProperty(ctx, "test")
			if result.IsOk() {
				return nil
			}
			return result.Error()
		}},
	}

	for _, tc := range testCases {
		t.Run(tc.name+"_canceled_context", func(t *testing.T) {
			err := tc.test()
			if err == nil {
				t.Errorf("expected context cancellation error, got success")
			} else if !errors.Is(err, context.Canceled) {
				t.Errorf("expected context.Canceled error, got %v", err)
			}
		})
	}
}
