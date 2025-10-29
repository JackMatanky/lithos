package domain

import (
	"strings"
	"testing"
)

const testPropertyName = "title"

// TestNewPropertyBank_Valid tests creating a PropertyBank with valid
// properties.
func TestNewPropertyBank_Valid(t *testing.T) {
	properties := map[string]Property{
		"title": {
			Name:     "title",
			Required: true,
			Array:    false,
			Spec: &StringSpec{
				Pattern: "^.{1,200}$",
			},
		},
		"tags": {
			Name:     "tags",
			Required: false,
			Array:    true,
			Spec:     &StringSpec{},
		},
	}

	pb, err := NewPropertyBank(properties)
	if err != nil {
		t.Fatalf("NewPropertyBank failed: %v", err)
	}
	if pb == nil {
		t.Fatal("NewPropertyBank returned nil")
	}
	if len(pb.Properties) != 2 {
		t.Errorf("Expected 2 properties, got %d", len(pb.Properties))
	}
}

// TestNewPropertyBank_EmptyPropertyID tests that empty property IDs are
// rejected.
func TestNewPropertyBank_EmptyPropertyID(t *testing.T) {
	properties := map[string]Property{
		"": {
			Name:     "title",
			Required: true,
			Spec:     &StringSpec{},
		},
	}

	_, err := NewPropertyBank(properties)
	if err == nil {
		t.Fatal("NewPropertyBank should fail with empty property ID")
	}
	if err.Error() != "property ID cannot be empty" {
		t.Errorf("Expected 'property ID cannot be empty', got %v", err)
	}
}

// TestNewPropertyBank_InvalidProperty tests that invalid properties are
// rejected.
func TestNewPropertyBank_InvalidProperty(t *testing.T) {
	properties := map[string]Property{
		"invalid": {
			Name:     "", // Empty name should fail validation
			Required: true,
			Spec:     &StringSpec{},
		},
	}

	_, err := NewPropertyBank(properties)
	if err == nil {
		t.Fatal("NewPropertyBank should fail with invalid property")
	}
	if !strings.Contains(
		err.Error(),
		"property invalid: property name cannot be empty",
	) {
		t.Errorf(
			"Expected error containing 'property name cannot be empty', got %v",
			err,
		)
	}
}

// TestPropertyBank_Lookup_Found tests successful property lookup.
func TestPropertyBank_Lookup_Found(t *testing.T) {
	prop := Property{
		Name:     "title",
		Required: true,
		Spec:     &StringSpec{},
	}
	properties := map[string]Property{
		"title": prop,
	}

	pb, err := NewPropertyBank(properties)
	if err != nil {
		t.Fatalf("NewPropertyBank failed: %v", err)
	}

	found, exists := pb.Lookup("title")
	if !exists {
		t.Fatal("Lookup should find existing property")
	}
	if found.Name != testPropertyName {
		t.Errorf("Expected name 'title', got %s", found.Name)
	}
}

// TestPropertyBank_Lookup_NotFound tests lookup of non-existent properties.
func TestPropertyBank_Lookup_NotFound(t *testing.T) {
	properties := map[string]Property{
		"title": {
			Name:     "title",
			Required: true,
			Spec:     &StringSpec{},
		},
	}

	pb, err := NewPropertyBank(properties)
	if err != nil {
		t.Fatalf("NewPropertyBank failed: %v", err)
	}

	_, exists := pb.Lookup("nonexistent")
	if exists {
		t.Fatal("Lookup should not find non-existent property")
	}
}

// TestPropertyBank_Lookup_ReturnsCopy tests that Lookup returns copies for
// immutability.
func TestPropertyBank_Lookup_ReturnsCopy(t *testing.T) {
	prop := Property{
		Name:     "title",
		Required: true,
		Spec:     &StringSpec{},
	}
	properties := map[string]Property{
		"title": prop,
	}

	pb, err := NewPropertyBank(properties)
	if err != nil {
		t.Fatalf("NewPropertyBank failed: %v", err)
	}

	found, exists := pb.Lookup("title")
	if !exists {
		t.Fatal("Lookup should find existing property")
	}

	// Modify the returned property
	found.Name = "modified" //nolint:govet // intentional for immutability test

	// Check that internal state wasn't affected
	original, _ := pb.Lookup("title")
	if original.Name != testPropertyName {
		t.Error("Lookup should return copy, internal state was modified")
	}
}

// TestPropertyBank_UnmarshalFromFixture tests PropertyBank creation with
// fixture data.
func TestPropertyBank_UnmarshalFromFixture(t *testing.T) {
	// Create PropertyBank with same data as fixture
	properties := map[string]Property{
		"standard_title": {
			Name:     "title",
			Required: true,
			Array:    false,
			Spec: &StringSpec{
				Pattern: "^.{1,200}$",
			},
		},
		"standard_tags": {
			Name:     "tags",
			Required: false,
			Array:    true,
			Spec:     &StringSpec{},
		},
		"iso_date": {
			Name:     "date",
			Required: true,
			Array:    false,
			Spec: &DateSpec{
				Format: "2006-01-02",
			},
		},
	}

	pb, err := NewPropertyBank(properties)
	if err != nil {
		t.Fatalf("NewPropertyBank from fixture data failed: %v", err)
	}

	if len(pb.Properties) != 3 {
		t.Errorf("Expected 3 properties, got %d", len(pb.Properties))
	}

	// Check specific properties
	title, exists := pb.Lookup("standard_title")
	if !exists {
		t.Fatal("standard_title not found")
	}
	if title.Name != "title" {
		t.Errorf("Expected name 'title', got %s", title.Name)
	}

	tags, exists := pb.Lookup("standard_tags")
	if !exists {
		t.Fatal("standard_tags not found")
	}
	if !tags.Array {
		t.Error("standard_tags should be array")
	}

	date, exists := pb.Lookup("iso_date")
	if !exists {
		t.Fatal("iso_date not found")
	}
	if date.Name != "date" {
		t.Errorf("Expected name 'date', got %s", date.Name)
	}
}
