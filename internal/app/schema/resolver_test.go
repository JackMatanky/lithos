package schema

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const meetingNoteSchemaName = "meeting-note"

// TestSchemaResolver_Resolve_SingleRootSchema tests resolution of a single
// schema with no inheritance.
func TestSchemaResolver_Resolve_SingleRootSchema(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{
			Name: "note",
			Properties: []domain.Property{
				{Name: "title", Required: true, Spec: domain.StringSpec{}},
				{Name: "tags", Required: false, Spec: domain.StringSpec{}},
			},
		},
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	resolved, err := resolver.Resolve(context.Background(), schemas, bank)
	require.NoError(t, err)
	require.Len(t, resolved, 1)

	schema := resolved[0]
	assert.Equal(t, "note", schema.Name)
	assert.Len(t, schema.ResolvedProperties, 2)
	assert.True(t, hasResolvedProperty(schema, "title"))
	assert.True(t, hasResolvedProperty(schema, "tags"))

	// Verify property details
	titleProp := getResolvedProperty(schema, "title")
	assert.True(t, titleProp.Required)
}

// TestSchemaResolver_Resolve_TwoLevelInheritance tests simple parent-child
// inheritance.
func TestSchemaResolver_Resolve_TwoLevelInheritance(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{
			Name: "base",
			Properties: []domain.Property{
				{Name: "title", Required: true, Spec: domain.StringSpec{}},
				{Name: "tags", Required: false, Spec: domain.StringSpec{}},
			},
		},
		{
			Name:    meetingNoteSchemaName,
			Extends: "base",
			Properties: []domain.Property{
				{Name: "attendees", Required: true, Spec: domain.StringSpec{}},
			},
		},
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	resolved, err := resolver.Resolve(context.Background(), schemas, bank)
	require.NoError(t, err)

	// Find resolved child
	var child domain.Schema
	for _, s := range resolved {
		if s.Name == meetingNoteSchemaName {
			child = s
			break
		}
	}

	// Verify inheritance
	assert.Len(
		t,
		child.ResolvedProperties,
		3,
	) // title, tags, attendees
	assert.True(t, hasResolvedProperty(child, "title"))     // from parent
	assert.True(t, hasResolvedProperty(child, "tags"))      // from parent
	assert.True(t, hasResolvedProperty(child, "attendees")) // from child
}

// TestSchemaResolver_Resolve_MultiLevelInheritance tests three-level
// inheritance chain.
func TestSchemaResolver_Resolve_MultiLevelInheritance(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{
			Name: "note",
			Properties: []domain.Property{
				{Name: "title", Required: true, Spec: domain.StringSpec{}},
				{Name: "tags", Required: false, Spec: domain.StringSpec{}},
			},
		},
		{
			Name:    "base-note",
			Extends: "note",
			Properties: []domain.Property{
				{Name: "created", Required: true, Spec: domain.StringSpec{}},
			},
		},
		{
			Name:    meetingNoteSchemaName,
			Extends: "base-note",
			Properties: []domain.Property{
				{Name: "attendees", Required: true, Spec: domain.StringSpec{}},
			},
		},
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	resolved, err := resolver.Resolve(context.Background(), schemas, bank)
	require.NoError(t, err)

	// Find resolved grandchild
	var grandchild domain.Schema
	for _, s := range resolved {
		if s.Name == meetingNoteSchemaName {
			grandchild = s
			break
		}
	}

	// Verify multi-level inheritance
	assert.Len(
		t,
		grandchild.ResolvedProperties,
		4,
	) // title, tags, created, attendees
	assert.True(
		t,
		hasResolvedProperty(grandchild, "title"),
	) // from grandparent
	assert.True(
		t,
		hasResolvedProperty(grandchild, "tags"),
	) // from grandparent
	assert.True(t, hasResolvedProperty(grandchild, "created"))   // from parent
	assert.True(t, hasResolvedProperty(grandchild, "attendees")) // from child
}

// TestSchemaResolver_Resolve_ExcludesRemovesProperties tests that Excludes
// properly removes parent properties.
func TestSchemaResolver_Resolve_ExcludesRemovesProperties(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{
			Name: "base",
			Properties: []domain.Property{
				{Name: "title", Required: true, Spec: domain.StringSpec{}},
				{Name: "tags", Required: false, Spec: domain.StringSpec{}},
				{
					Name:     "internal_id",
					Required: false,
					Spec:     domain.StringSpec{},
				},
			},
		},
		{
			Name:     "public-note",
			Extends:  "base",
			Excludes: []string{"internal_id"},
			Properties: []domain.Property{
				{Name: "published", Required: true, Spec: domain.StringSpec{}},
			},
		},
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	resolved, err := resolver.Resolve(context.Background(), schemas, bank)
	require.NoError(t, err)

	// Find resolved child
	var child domain.Schema
	for _, s := range resolved {
		if s.Name == "public-note" {
			child = s
			break
		}
	}

	// Verify exclusion
	assert.Len(
		t,
		child.ResolvedProperties,
		3,
	) // title, tags, published (internal_id excluded)
	assert.True(t, hasResolvedProperty(child, "title"))
	assert.True(t, hasResolvedProperty(child, "tags"))
	assert.True(t, hasResolvedProperty(child, "published"))
	assert.False(t, hasResolvedProperty(child, "internal_id")) // excluded
}

// TestSchemaResolver_Resolve_ChildOverridesParent tests that child properties
// completely override parent properties with same name.
func TestSchemaResolver_Resolve_ChildOverridesParent(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{
			Name: "base",
			Properties: []domain.Property{
				{
					Name:     "title",
					Required: true,
					Spec:     domain.StringSpec{},
				}, // required in parent
				{Name: "tags", Required: false, Spec: domain.StringSpec{}},
			},
		},
		{
			Name:    "flexible-note",
			Extends: "base",
			Properties: []domain.Property{
				{
					Name:     "title",
					Required: false,
					Spec:     domain.StringSpec{},
				}, // optional in child
			},
		},
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	resolved, err := resolver.Resolve(context.Background(), schemas, bank)
	require.NoError(t, err)

	// Find resolved child
	var child domain.Schema
	for _, s := range resolved {
		if s.Name == "flexible-note" {
			child = s
			break
		}
	}

	// Verify override
	assert.Len(
		t,
		child.ResolvedProperties,
		2,
	) // title (overridden), tags (inherited)

	titleProp := getResolvedProperty(child, "title")
	assert.False(t, titleProp.Required) // Child's version (optional)

	tagsProp := getResolvedProperty(child, "tags")
	assert.False(t, tagsProp.Required) // Parent's version unchanged
}

// TestSchemaResolver_Resolve_CircularDependency tests that circular
// inheritance is detected and returns informative error.
func TestSchemaResolver_Resolve_CircularDependency(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{Name: "a", Extends: "b"},
		{Name: "b", Extends: "c"},
		{Name: "c", Extends: "a"}, // Creates cycle: a → b → c → a
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	_, err := resolver.Resolve(context.Background(), schemas, bank)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "circular inheritance")
	assert.Contains(t, err.Error(), "a")
	assert.Contains(t, err.Error(), "b")
	assert.Contains(t, err.Error(), "c")
}

// TestSchemaResolver_Resolve_MissingRefError tests that missing $ref targets
// return informative errors.
func TestSchemaResolver_Resolve_MissingRefError(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{
			Name: "note",
			Properties: []domain.Property{
				{Name: "title", Ref: "standard_title"}, // Missing in bank
			},
		},
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	_, err := resolver.Resolve(context.Background(), schemas, bank)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "standard_title")
	assert.Contains(t, err.Error(), "$ref")
	assert.Contains(t, err.Error(), "not found")
}

// TestSchemaResolver_Resolve_RefSubstitutionSuccess tests successful $ref
// substitution with PropertyBank definitions.
func TestSchemaResolver_Resolve_RefSubstitutionSuccess(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{
			Name: "note",
			Properties: []domain.Property{
				{Name: "title", Required: true, Ref: "standard_title"},
				{Name: "tags", Required: false, Spec: domain.StringSpec{}},
			},
		},
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{
		"standard_title": {Name: "title", Spec: domain.StringSpec{}},
	}}

	resolved, err := resolver.Resolve(context.Background(), schemas, bank)
	require.NoError(t, err)
	require.Len(t, resolved, 1)

	schema := resolved[0]
	assert.Len(t, schema.ResolvedProperties, 2)

	// Verify $ref substitution
	titleProp := getResolvedProperty(schema, "title")
	assert.True(
		t,
		titleProp.Required,
	) // From original property
	assert.Empty(
		t,
		titleProp.Ref,
	) // Ref cleared after substitution
	assert.IsType(t, domain.StringSpec{}, titleProp.Spec) // Spec from bank
}

// TestSchemaResolver_Resolve_OriginalsSchemasUnchanged tests that original
// schemas are not mutated during resolution.
func TestSchemaResolver_Resolve_OriginalSchemasUnchanged(t *testing.T) {
	resolver := NewSchemaResolver()

	original := []domain.Schema{
		{
			Name:    "child",
			Extends: "parent",
			Properties: []domain.Property{
				{Name: "child_prop", Required: true, Spec: domain.StringSpec{}},
			},
		},
		{
			Name: "parent",
			Properties: []domain.Property{
				{
					Name:     "parent_prop",
					Required: false,
					Spec:     domain.StringSpec{},
				},
			},
		},
	}

	// Make copies to compare
	originalCopy := make([]domain.Schema, len(original))
	copy(originalCopy, original)

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	_, err := resolver.Resolve(context.Background(), original, bank)
	require.NoError(t, err)

	// Verify originals unchanged
	assert.Equal(t, originalCopy, original)

	// Verify ResolvedProperties were empty in originals
	for _, schema := range original {
		assert.Empty(t, schema.ResolvedProperties)
	}
}

// TestSchemaResolver_Resolve_EmptySchemas tests resolver handles empty schema
// slice gracefully.
func TestSchemaResolver_Resolve_EmptySchemas(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{}
	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	resolved, err := resolver.Resolve(context.Background(), schemas, bank)
	require.NoError(t, err)
	assert.Empty(t, resolved)
}

// TestSchemaResolver_Resolve_PreservesOriginalFields tests that resolved
// schemas preserve original Extends/Excludes/Properties fields.
func TestSchemaResolver_Resolve_PreservesOriginalFields(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{
			Name:     "child",
			Extends:  "parent",
			Excludes: []string{"unwanted"},
			Properties: []domain.Property{
				{Name: "child_prop", Required: true, Spec: domain.StringSpec{}},
			},
		},
		{
			Name: "parent",
			Properties: []domain.Property{
				{
					Name:     "parent_prop",
					Required: false,
					Spec:     domain.StringSpec{},
				},
				{Name: "unwanted", Required: false, Spec: domain.StringSpec{}},
			},
		},
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{}}

	resolved, err := resolver.Resolve(context.Background(), schemas, bank)
	require.NoError(t, err)

	// Find resolved child
	var child domain.Schema
	for _, s := range resolved {
		if s.Name == "child" {
			child = s
			break
		}
	}

	// Verify original fields preserved
	assert.Equal(t, "parent", child.Extends)
	assert.Equal(t, []string{"unwanted"}, child.Excludes)
	assert.Len(t, child.Properties, 1) // Original properties count
	assert.Equal(t, "child_prop", child.Properties[0].Name)

	// Verify resolved properties are populated
	assert.NotEmpty(t, child.ResolvedProperties)
	assert.Len(
		t,
		child.ResolvedProperties,
		2,
	) // parent_prop, child_prop (unwanted excluded)
}

// TestSchemaResolver_Resolve_ComplexInheritanceWithRefs tests complex scenario
// with inheritance, excludes, overrides, and $ref substitution.
func TestSchemaResolver_Resolve_ComplexInheritanceWithRefs(t *testing.T) {
	resolver := NewSchemaResolver()

	schemas := []domain.Schema{
		{
			Name: "base",
			Properties: []domain.Property{
				{Name: "title", Required: true, Spec: domain.StringSpec{}},
				{Name: "tags", Required: false, Spec: domain.StringSpec{}},
				{Name: "internal", Required: false, Spec: domain.StringSpec{}},
			},
		},
		{
			Name:     "note",
			Extends:  "base",
			Excludes: []string{"internal"},
			Properties: []domain.Property{
				{
					Name:     "title",
					Required: false,
					Ref:      "standard_title",
				}, // Override + ref
				{Name: "created", Required: true, Spec: domain.StringSpec{}},
			},
		},
	}

	bank := domain.PropertyBank{Properties: map[string]domain.Property{
		"standard_title": {Name: "title", Spec: domain.StringSpec{}},
	}}

	resolved, err := resolver.Resolve(context.Background(), schemas, bank)
	require.NoError(t, err)

	// Find resolved note
	var note domain.Schema
	for _, s := range resolved {
		if s.Name == "note" {
			note = s
			break
		}
	}

	// Verify complex resolution
	assert.Len(
		t,
		note.ResolvedProperties,
		3,
	) // title (overridden+ref), tags (inherited), created (new)

	// Verify title override with ref substitution
	titleProp := getResolvedProperty(note, "title")
	assert.False(t, titleProp.Required) // Child's requirement (override)
	assert.Empty(t, titleProp.Ref)      // Ref cleared after substitution

	// Verify inherited property
	assert.True(t, hasResolvedProperty(note, "tags"))

	// Verify excluded property
	assert.False(t, hasResolvedProperty(note, "internal"))

	// Verify new property
	assert.True(t, hasResolvedProperty(note, "created"))
}

// Helper functions for test assertions

func hasResolvedProperty(schema domain.Schema, name string) bool {
	for _, prop := range schema.ResolvedProperties {
		if prop.Name == name {
			return true
		}
	}
	return false
}

func getResolvedProperty(schema domain.Schema, name string) domain.Property {
	for _, prop := range schema.ResolvedProperties {
		if prop.Name == name {
			return prop
		}
	}
	return domain.Property{} // Not found
}
