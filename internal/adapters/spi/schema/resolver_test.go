package schema

import (
	"context"
	"fmt"
	"strings"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
)

// Test data helpers.
func createTestProperty(name string, required bool) domain.Property {
	return domain.NewProperty(
		name,
		required,
		false,
		domain.StringPropertySpec{},
	)
}

func createTestSchema(
	name, extends string,
	excludes []string,
	properties []domain.Property,
) domain.Schema {
	if extends == "" {
		return domain.NewSchema(name, properties)
	}
	return domain.NewSchemaWithExtends(name, extends, excludes, properties)
}

// Test inheritance resolver construction.
func TestNewInheritanceResolver_Success(t *testing.T) {
	schemas := []domain.Schema{
		createTestSchema("base", "", nil, []domain.Property{
			createTestProperty("title", true),
		}),
		createTestSchema("article", "base", nil, []domain.Property{
			createTestProperty("summary", false),
		}),
	}

	resolver, err := NewInheritanceResolver(schemas)
	if err != nil {
		t.Fatalf("expected successful resolver creation, got error: %v", err)
	}

	if resolver == nil {
		t.Fatal("expected non-nil resolver")
	}

	if len(resolver.lookup) != 2 {
		t.Errorf("expected 2 schemas in lookup, got %d", len(resolver.lookup))
	}

	if len(resolver.cache) != 0 {
		t.Errorf(
			"expected empty cache initially, got %d entries",
			len(resolver.cache),
		)
	}
}

func TestNewInheritanceResolver_InvalidSchema(t *testing.T) {
	schemas := []domain.Schema{
		{
			Name:       "", // Invalid: empty name
			Properties: []domain.Property{},
		},
	}

	resolver, err := NewInheritanceResolver(schemas)
	if err == nil {
		t.Fatal("expected error for invalid schema")
	}

	if resolver != nil {
		t.Error("expected nil resolver for invalid input")
	}

	if !strings.Contains(err.Error(), "schema validation failed") {
		t.Errorf("expected schema validation error, got: %v", err)
	}
}

func TestNewInheritanceResolver_DuplicateSchemaNames(t *testing.T) {
	schemas := []domain.Schema{
		createTestSchema("duplicate", "", nil, []domain.Property{
			createTestProperty("prop1", true),
		}),
		createTestSchema("duplicate", "", nil, []domain.Property{
			createTestProperty("prop2", true),
		}),
	}

	resolver, err := NewInheritanceResolver(schemas)
	if err == nil {
		t.Fatal("expected error for duplicate schema names")
	}

	if resolver != nil {
		t.Error("expected nil resolver for duplicate names")
	}

	if !strings.Contains(err.Error(), "duplicate schema name") {
		t.Errorf("expected duplicate schema name error, got: %v", err)
	}
}

// Test single-level inheritance.
func TestResolveAll_SingleLevelInheritance(t *testing.T) {
	base := createTestSchema("base", "", nil, []domain.Property{
		createTestProperty("title", true),
		createTestProperty("created", false),
	})

	child := createTestSchema("article", "base", nil, []domain.Property{
		createTestProperty("summary", false),
	})

	resolver, err := NewInheritanceResolver([]domain.Schema{base, child})
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err != nil {
		t.Fatalf("failed to resolve schemas: %v", err)
	}

	if len(resolved) != 2 {
		t.Fatalf("expected 2 resolved schemas, got %d", len(resolved))
	}

	// Check base schema (no inheritance)
	baseResolved := resolved["base"]
	baseProps := baseResolved.GetResolvedProperties()
	if len(baseProps) != 2 {
		t.Errorf("expected base to have 2 properties, got %d", len(baseProps))
	}

	// Check child schema (inherits from base)
	childResolved := resolved["article"]
	childProps := childResolved.GetResolvedProperties()
	if len(childProps) != 3 {
		t.Errorf(
			"expected article to have 3 properties, got %d",
			len(childProps),
		)
	}

	// Verify inherited properties exist
	propNames := make(map[string]bool)
	for _, prop := range childProps {
		propNames[prop.Name] = true
	}

	expectedProps := []string{"title", "created", "summary"}
	for _, expected := range expectedProps {
		if !propNames[expected] {
			t.Errorf("expected property %s in article schema", expected)
		}
	}
}

// Test multi-level inheritance chains (AC 2.6.3).
func TestResolveAll_MultiLevelInheritance(t *testing.T) {
	// A -> B -> C inheritance chain
	schemaA := createTestSchema("a", "", nil, []domain.Property{
		createTestProperty("prop_a", true),
	})

	schemaB := createTestSchema("b", "a", nil, []domain.Property{
		createTestProperty("prop_b", false),
	})

	schemaC := createTestSchema("c", "b", nil, []domain.Property{
		createTestProperty("prop_c", false),
	})

	resolver, err := NewInheritanceResolver(
		[]domain.Schema{schemaA, schemaB, schemaC},
	)
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err != nil {
		t.Fatalf("failed to resolve schemas: %v", err)
	}

	// Check that C has all properties from A, B, and C
	schemaC_resolved := resolved["c"]
	props := schemaC_resolved.GetResolvedProperties()

	if len(props) != 3 {
		t.Errorf("expected C to have 3 properties, got %d", len(props))
	}

	propNames := make(map[string]bool)
	for _, prop := range props {
		propNames[prop.Name] = true
	}

	expectedProps := []string{"prop_a", "prop_b", "prop_c"}
	for _, expected := range expectedProps {
		if !propNames[expected] {
			t.Errorf("expected property %s in schema c", expected)
		}
	}
}

// Test property override semantics (AC 2.6.3).
func TestResolveAll_PropertyOverrides(t *testing.T) {
	base := createTestSchema("base", "", nil, []domain.Property{
		createTestProperty("title", false), // Not required in base
		createTestProperty("tags", false),
	})

	child := createTestSchema("article", "base", nil, []domain.Property{
		createTestProperty("title", true), // Override: now required
		createTestProperty("summary", false),
	})

	resolver, err := NewInheritanceResolver([]domain.Schema{base, child})
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err != nil {
		t.Fatalf("failed to resolve schemas: %v", err)
	}

	childResolved := resolved["article"]
	props := childResolved.GetResolvedProperties()

	// Find the title property to check override
	var titleProp *domain.Property
	for _, prop := range props {
		if prop.Name == "title" {
			titleProp = &prop
			break
		}
	}

	if titleProp == nil {
		t.Fatal("expected title property in resolved article schema")
	}

	if !titleProp.Required {
		t.Error("expected title to be required (child override should win)")
	}

	// Verify all expected properties are present
	if len(props) != 3 {
		t.Errorf(
			"expected 3 properties (title, tags, summary), got %d",
			len(props),
		)
	}
}

// Test excludes functionality (AC 2.6.2).
func TestResolveAll_ExcludesProperties(t *testing.T) {
	base := createTestSchema("note", "", nil, []domain.Property{
		createTestProperty("title", true),
		createTestProperty("tags", false),
		createTestProperty("created", false),
	})

	child := createTestSchema(
		"meeting-note",
		"note",
		[]string{"created"},
		[]domain.Property{
			createTestProperty("agenda", false),
			createTestProperty("tags", true), // Override tags requirement
		},
	)

	resolver, err := NewInheritanceResolver([]domain.Schema{base, child})
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err != nil {
		t.Fatalf("failed to resolve schemas: %v", err)
	}

	childResolved := resolved["meeting-note"]
	props := childResolved.GetResolvedProperties()

	// Verify excluded property is not present
	propNames := make(map[string]bool)
	for _, prop := range props {
		propNames[prop.Name] = true
	}

	if propNames["created"] {
		t.Error("expected 'created' property to be excluded")
	}

	// Verify expected properties are present
	expectedProps := []string{"title", "tags", "agenda"}
	for _, expected := range expectedProps {
		if !propNames[expected] {
			t.Errorf("expected property %s in meeting-note schema", expected)
		}
	}

	// Verify tags override is applied
	var tagsProp *domain.Property
	for _, prop := range props {
		if prop.Name == "tags" {
			tagsProp = &prop
			break
		}
	}

	if tagsProp == nil {
		t.Fatal("expected tags property in resolved meeting-note schema")
	}

	if !tagsProp.Required {
		t.Error("expected tags to be required (child override should win)")
	}
}

// Test cycle detection (AC 2.6.4).
func TestResolveAll_DetectsCycleDirectA2B2A(t *testing.T) {
	schemaA := createTestSchema("a", "b", nil, []domain.Property{
		createTestProperty("prop_a", true),
	})

	schemaB := createTestSchema("b", "a", nil, []domain.Property{
		createTestProperty("prop_b", false),
	})

	resolver, err := NewInheritanceResolver([]domain.Schema{schemaA, schemaB})
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err == nil {
		t.Fatal("expected error for cyclic inheritance")
	}

	if resolved != nil {
		t.Error("expected nil result for cyclic inheritance")
	}

	if !strings.Contains(err.Error(), "cyclic inheritance") {
		t.Errorf("expected cyclic inheritance error, got: %v", err)
	}

	// Check that the error message contains the cycle path
	if !strings.Contains(err.Error(), "a -> b -> a") &&
		!strings.Contains(err.Error(), "b -> a -> b") {
		t.Errorf("expected cycle path in error message, got: %v", err)
	}
}

func TestResolveAll_DetectsCycleIndirectA2B2C2A(t *testing.T) {
	schemaA := createTestSchema("a", "c", nil, []domain.Property{
		createTestProperty("prop_a", true),
	})

	schemaB := createTestSchema("b", "a", nil, []domain.Property{
		createTestProperty("prop_b", false),
	})

	schemaC := createTestSchema("c", "b", nil, []domain.Property{
		createTestProperty("prop_c", false),
	})

	resolver, err := NewInheritanceResolver(
		[]domain.Schema{schemaA, schemaB, schemaC},
	)
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err == nil {
		t.Fatal("expected error for cyclic inheritance")
	}

	if resolved != nil {
		t.Error("expected nil result for cyclic inheritance")
	}

	if !strings.Contains(err.Error(), "cyclic inheritance") {
		t.Errorf("expected cyclic inheritance error, got: %v", err)
	}

	// Should contain some permutation of the cycle
	errMsg := err.Error()
	hasCyclePath := strings.Contains(errMsg, "a -> c -> b -> a") ||
		strings.Contains(errMsg, "b -> a -> c -> b") ||
		strings.Contains(errMsg, "c -> b -> a -> c")

	if !hasCyclePath {
		t.Errorf("expected cycle path in error message, got: %v", err)
	}
}

// Test self-reference cycle detection.
func TestResolveAll_DetectsSelfReference(t *testing.T) {
	// The schema validation catches self-reference during construction
	selfRefSchema := createTestSchema("self", "self", nil, []domain.Property{
		createTestProperty("prop", true),
	})

	resolver, err := NewInheritanceResolver([]domain.Schema{selfRefSchema})
	if err == nil {
		t.Fatal("expected error during resolver creation for self-reference")
	}

	if resolver != nil {
		t.Error("expected nil resolver for self-reference")
	}

	if !strings.Contains(err.Error(), "cannot reference itself") {
		t.Errorf("expected self-reference validation error, got: %v", err)
	}
}

// Test missing parent schema.
func TestResolveAll_MissingParent(t *testing.T) {
	child := createTestSchema("child", "missing-parent", nil, []domain.Property{
		createTestProperty("prop", true),
	})

	resolver, err := NewInheritanceResolver([]domain.Schema{child})
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err == nil {
		t.Fatal("expected error for missing parent schema")
	}

	if resolved != nil {
		t.Error("expected nil result for missing parent")
	}

	if !strings.Contains(err.Error(), "schema not found") {
		t.Errorf("expected schema not found error, got: %v", err)
	}
}

// Test context cancellation.
func TestResolveAll_ContextCancellation(t *testing.T) {
	schemas := []domain.Schema{
		createTestSchema("base", "", nil, []domain.Property{
			createTestProperty("title", true),
		}),
	}

	resolver, err := NewInheritanceResolver(schemas)
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	ctx, cancel := context.WithCancel(context.Background())
	cancel() // Cancel immediately

	resolved, err := resolver.ResolveAll(ctx)
	if err == nil {
		t.Fatal("expected error for canceled context")
	}

	if resolved != nil {
		t.Error("expected nil result for canceled context")
	}
}

// Test empty schema list.
func TestResolveAll_EmptySchemas(t *testing.T) {
	resolver, err := NewInheritanceResolver([]domain.Schema{})
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err != nil {
		t.Fatalf("unexpected error for empty schemas: %v", err)
	}

	if len(resolved) != 0 {
		t.Errorf(
			"expected empty result for empty schemas, got %d",
			len(resolved),
		)
	}
}

// Test immutability concept - properties are resolved once and stored (AC
// 2.6.5).
func TestResolveAll_PropertiesResolvedOnce(t *testing.T) {
	base := createTestSchema("base", "", nil, []domain.Property{
		createTestProperty("title", true),
	})

	child := createTestSchema("child", "base", nil, []domain.Property{
		createTestProperty("summary", false),
	})

	resolver, err := NewInheritanceResolver([]domain.Schema{base, child})
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err != nil {
		t.Fatalf("failed to resolve schemas: %v", err)
	}

	childResolved := resolved["child"]

	// Verify that resolved properties are populated (not nil)
	if childResolved.ResolvedProperties == nil {
		t.Error("expected ResolvedProperties to be populated after resolution")
	}

	// Verify correct number of properties were resolved
	props := childResolved.GetResolvedProperties()
	if len(props) != 2 {
		t.Errorf("expected 2 resolved properties, got %d", len(props))
	}

	// Verify inheritance worked correctly
	propNames := make(map[string]bool)
	for _, prop := range props {
		propNames[prop.Name] = true
	}

	if !propNames["title"] || !propNames["summary"] {
		t.Error("expected both inherited and child properties to be resolved")
	}
}

// Test topological ordering implicitly through complex inheritance.
func TestResolveAll_TopologicalOrdering(t *testing.T) {
	// Create a complex dependency graph:
	// root -> intermediate1, intermediate2
	// leaf1 -> intermediate1
	// leaf2 -> intermediate2
	// complex -> leaf1, leaf2

	root := createTestSchema("root", "", nil, []domain.Property{
		createTestProperty("root_prop", true),
	})

	intermediate1 := createTestSchema(
		"intermediate1",
		"root",
		nil,
		[]domain.Property{
			createTestProperty("int1_prop", false),
		},
	)

	intermediate2 := createTestSchema(
		"intermediate2",
		"root",
		nil,
		[]domain.Property{
			createTestProperty("int2_prop", false),
		},
	)

	leaf1 := createTestSchema("leaf1", "intermediate1", nil, []domain.Property{
		createTestProperty("leaf1_prop", false),
	})

	leaf2 := createTestSchema("leaf2", "intermediate2", nil, []domain.Property{
		createTestProperty("leaf2_prop", false),
	})

	schemas := []domain.Schema{root, intermediate1, intermediate2, leaf1, leaf2}

	resolver, err := NewInheritanceResolver(schemas)
	if err != nil {
		t.Fatalf("failed to create resolver: %v", err)
	}

	resolved, err := resolver.ResolveAll(context.Background())
	if err != nil {
		t.Fatalf("failed to resolve complex inheritance: %v", err)
	}

	// Verify all schemas are resolved
	if len(resolved) != 5 {
		t.Errorf("expected 5 resolved schemas, got %d", len(resolved))
	}

	// Verify leaf1 has properties from root and intermediate1
	leaf1Resolved := resolved["leaf1"]
	leaf1Props := leaf1Resolved.GetResolvedProperties()

	expectedLeaf1Props := []string{"root_prop", "int1_prop", "leaf1_prop"}
	if len(leaf1Props) != 3 {
		t.Errorf("expected leaf1 to have 3 properties, got %d", len(leaf1Props))
	}

	propNames := make(map[string]bool)
	for _, prop := range leaf1Props {
		propNames[prop.Name] = true
	}

	for _, expected := range expectedLeaf1Props {
		if !propNames[expected] {
			t.Errorf("expected property %s in leaf1", expected)
		}
	}
}

// Benchmark tests for performance verification.
func BenchmarkResolveAll_SingleLevel(b *testing.B) {
	schemas := make([]domain.Schema, 100)
	for i := range 100 {
		if i == 0 {
			schemas[i] = createTestSchema("base", "", nil, []domain.Property{
				createTestProperty("prop1", true),
				createTestProperty("prop2", false),
			})
		} else {
			schemas[i] = createTestSchema(
				fmt.Sprintf("child%d", i),
				"base",
				nil,
				[]domain.Property{
					createTestProperty(fmt.Sprintf("child_prop%d", i), false),
				},
			)
		}
	}

	b.ResetTimer()
	for range b.N {
		resolver, _ := NewInheritanceResolver(schemas)
		_, _ = resolver.ResolveAll(context.Background())
	}
}
