package schema

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const testMeetingNoteName = "meeting_note"

// TestSchemaExtender_ExtendSchemas_SingleRootSchema tests resolution of a
// single
// schema with no inheritance.
func TestSchemaExtender_ExtendSchemas_SingleRootSchema(t *testing.T) {
	extender := NewSchemaExtender()

	schemas := []domain.Schema{
		{
			Name: "note",
			Properties: []domain.Property{
				{
					Name:     "title",
					Required: true,
					Spec:     domain.StringSpec{},
				},
				{
					Name:     "tags",
					Required: false,
					Spec:     domain.StringSpec{},
				},
			},
		},
	}

	resolved, err := extender.ExtendSchemas(context.Background(), schemas)
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

// TestSchemaExtender_ExtendSchemas_TwoLevelInheritance tests simple
// parent-child
// inheritance.
func TestSchemaExtender_ExtendSchemas_TwoLevelInheritance(t *testing.T) {
	extender := NewSchemaExtender()

	schemas := []domain.Schema{
		{
			Name: "base",
			Properties: []domain.Property{
				{
					Name:     "title",
					Required: true,
					Spec:     domain.StringSpec{},
				},
				{
					Name:     "tags",
					Required: false,
					Spec:     domain.StringSpec{},
				},
			},
		},
		{
			Name:    testMeetingNoteName,
			Extends: "base",
			Properties: []domain.Property{
				{
					Name:     "attendees",
					Required: true,
					Spec:     domain.StringSpec{},
				},
			},
		},
	}

	resolved, err := extender.ExtendSchemas(context.Background(), schemas)
	require.NoError(t, err)

	// Find resolved child
	var child domain.Schema
	for _, s := range resolved {
		if s.Name == testMeetingNoteName {
			child = s
			break
		}
	}

	// Should have all 3 properties: title, tags (from parent), attendees (from
	// child)
	assert.Len(t, child.ResolvedProperties, 3)
	assert.True(t, hasResolvedProperty(child, "title"))
	assert.True(t, hasResolvedProperty(child, "tags"))
	assert.True(t, hasResolvedProperty(child, "attendees"))
}

// TestSchemaExtender_ExtendSchemas_MultiLevelInheritance tests grandparent →
// parent → child.
func TestSchemaExtender_ExtendSchemas_MultiLevelInheritance(t *testing.T) {
	extender := NewSchemaExtender()

	// Grandparent
	grandparent := domain.Schema{
		Name: "note",
		Properties: []domain.Property{
			{Name: "title", Required: true, Spec: domain.StringSpec{}},
			{Name: "tags", Required: false, Spec: domain.StringSpec{}},
		},
	}

	// Parent extends grandparent, adds property
	parent := domain.Schema{
		Name:    "base-note",
		Extends: "note",
		Properties: []domain.Property{
			{Name: "created", Required: true, Spec: domain.StringSpec{}},
		},
	}

	// Child extends parent, overrides title, excludes tags
	child := domain.Schema{
		Name:     "meeting_note",
		Extends:  "base-note",
		Excludes: []string{"tags"},
		Properties: []domain.Property{
			{
				Name:     "title",
				Required: false,
				Spec:     domain.StringSpec{},
			}, // Override
			{Name: "attendees", Required: true, Spec: domain.StringSpec{}},
		},
	}

	schemas := []domain.Schema{grandparent, parent, child}
	resolved, err := extender.ExtendSchemas(context.Background(), schemas)

	require.NoError(t, err)

	// Find resolved child
	var resolvedChild domain.Schema
	for _, s := range resolved {
		if s.Name == testMeetingNoteName {
			resolvedChild = s
			break
		}
	}

	// Verify resolved properties
	assert.Len(
		t,
		resolvedChild.ResolvedProperties,
		3,
	) // title, created, attendees
	// tags excluded
	assert.True(t, hasResolvedProperty(resolvedChild, "title"))
	assert.True(t, hasResolvedProperty(resolvedChild, "created"))
	assert.True(t, hasResolvedProperty(resolvedChild, "attendees"))
	assert.False(t, hasResolvedProperty(resolvedChild, "tags"))

	// Verify title is overridden (Required: false from child)
	titleProp := getResolvedProperty(resolvedChild, "title")
	assert.False(t, titleProp.Required)
}

// TestSchemaExtender_ExtendSchemas_CircularInheritance tests circular
// dependency detection.
func TestSchemaExtender_ExtendSchemas_CircularInheritance(t *testing.T) {
	extender := NewSchemaExtender()

	schemas := []domain.Schema{
		{
			Name:    "a",
			Extends: "b",
			Properties: []domain.Property{
				{Name: "prop_a", Required: true, Spec: domain.StringSpec{}},
			},
		},
		{
			Name:    "b",
			Extends: "c",
			Properties: []domain.Property{
				{Name: "prop_b", Required: true, Spec: domain.StringSpec{}},
			},
		},
		{
			Name:    "c",
			Extends: "a", // Creates cycle: a → b → c → a
			Properties: []domain.Property{
				{Name: "prop_c", Required: true, Spec: domain.StringSpec{}},
			},
		},
	}

	_, err := extender.ExtendSchemas(context.Background(), schemas)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "circular inheritance")
	assert.Contains(t, err.Error(), "→") // Should show cycle path
}

// TestSchemaExtender_ExtendSchemas_PropertyOverride tests child property
// overriding parent.
func TestSchemaExtender_ExtendSchemas_PropertyOverride(t *testing.T) {
	extender := NewSchemaExtender()

	schemas := []domain.Schema{
		{
			Name: "base",
			Properties: []domain.Property{
				{Name: "title", Required: true, Spec: domain.StringSpec{}},
				{Name: "status", Required: false, Spec: domain.StringSpec{}},
			},
		},
		{
			Name:    "derived",
			Extends: "base",
			Properties: []domain.Property{
				{
					Name:     "title",
					Required: false,
					Spec:     domain.StringSpec{},
				}, // Override
				{Name: "extra", Required: true, Spec: domain.StringSpec{}},
			},
		},
	}

	resolved, err := extender.ExtendSchemas(context.Background(), schemas)
	require.NoError(t, err)

	// Find derived schema
	var derived domain.Schema
	for _, s := range resolved {
		if s.Name == "derived" {
			derived = s
			break
		}
	}

	assert.Len(
		t,
		derived.ResolvedProperties,
		3,
	) // title (overridden), status, extra

	// Verify title was overridden (Required: false from child)
	titleProp := getResolvedProperty(derived, "title")
	assert.False(t, titleProp.Required)

	// Verify status inherited unchanged
	statusProp := getResolvedProperty(derived, "status")
	assert.False(t, statusProp.Required) // Original from parent

	// Verify extra property from child
	extraProp := getResolvedProperty(derived, "extra")
	assert.True(t, extraProp.Required)
}

// TestSchemaExtender_ExtendSchemas_ExcludesProperty tests excludes
// functionality.
func TestSchemaExtender_ExtendSchemas_ExcludesProperty(t *testing.T) {
	extender := NewSchemaExtender()

	schemas := []domain.Schema{
		{
			Name: "base",
			Properties: []domain.Property{
				{Name: "title", Required: true, Spec: domain.StringSpec{}},
				{Name: "tags", Required: false, Spec: domain.StringSpec{}},
				{
					Name:     "internal_id",
					Required: true,
					Spec:     domain.StringSpec{},
				},
			},
		},
		{
			Name:     "public_note",
			Extends:  "base",
			Excludes: []string{"internal_id"}, // Remove internal_id from parent
			Properties: []domain.Property{
				{Name: "author", Required: true, Spec: domain.StringSpec{}},
			},
		},
	}

	resolved, err := extender.ExtendSchemas(context.Background(), schemas)
	require.NoError(t, err)

	// Find derived schema
	var publicNote domain.Schema
	for _, s := range resolved {
		if s.Name == "public_note" {
			publicNote = s
			break
		}
	}

	assert.Len(
		t,
		publicNote.ResolvedProperties,
		3,
	) // title, tags, author (internal_id excluded)
	assert.True(t, hasResolvedProperty(publicNote, "title"))
	assert.True(t, hasResolvedProperty(publicNote, "tags"))
	assert.True(t, hasResolvedProperty(publicNote, "author"))
	assert.False(t, hasResolvedProperty(publicNote, "internal_id")) // Excluded
}

// TestSchemaExtender_ExtendSchemas_ContextCancellation tests context
// cancellation handling.
func TestSchemaExtender_ExtendSchemas_ContextCancellation(t *testing.T) {
	extender := NewSchemaExtender()

	// Create canceled context
	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	schemas := []domain.Schema{
		{
			Name: "test",
			Properties: []domain.Property{
				{Name: "title", Required: true, Spec: domain.StringSpec{}},
			},
		},
	}

	_, err := extender.ExtendSchemas(ctx, schemas)

	require.Error(t, err)
	assert.Equal(t, context.Canceled, err)
}

// Helper functions

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
