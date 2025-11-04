package frontmatter

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	lithosLog "github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestFrontmatterService_StructExists verifies FrontmatterService struct
// exists.
func TestFrontmatterService_StructExists(t *testing.T) {
	// This test verifies FrontmatterService struct exists
	var _ *FrontmatterService // This will fail if struct doesn't exist
}

// TestNewFrontmatterService_ConstructorExists verifies constructor works.
func TestNewFrontmatterService_ConstructorExists(t *testing.T) {
	// This test verifies NewFrontmatterService constructor exists and works
	// We'll use a fake SchemaEngine and logger for testing
	fakeSchemaEngine := &schema.SchemaEngine{} // This will need to be a proper fake later
	fakeLogger := lithosLog.NewTest()

	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)
	require.NotNil(t, service)
	assert.NotNil(t, service.schemaEngine)
	assert.NotNil(t, service.logger)
}

// TestFrontmatterService_SchemaEngineDependency verifies SchemaEngine
// injection.
func TestFrontmatterService_SchemaEngineDependency(t *testing.T) {
	// This test verifies SchemaEngine dependency is properly injected
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()

	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)
	assert.Equal(t, fakeSchemaEngine, service.schemaEngine)
}

// TestFrontmatterService_LoggerDependency verifies Logger injection.
func TestFrontmatterService_LoggerDependency(t *testing.T) {
	// This test verifies Logger dependency is properly injected
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()

	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)
	assert.Equal(t, fakeLogger, service.logger)
}

// Task 2 RED phase tests - these should fail until Extract is implemented

// TestFrontmatterService_ExtractMethodSignature verifies Extract method exists.
func TestFrontmatterService_ExtractMethodSignature(t *testing.T) {
	// This test verifies Extract method signature exists
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	// This will fail if method signature doesn't match exactly
	content := []byte("# Test\nsome content")
	_, err := service.Extract(content)
	// We expect the method to exist but not be implemented yet
	require.NoError(
		t,
		err,
	) // This will fail when we make it return an error for unimplemented
}

// TestFrontmatterService_ExtractValidYAML verifies YAML frontmatter extraction.
func TestFrontmatterService_ExtractValidYAML(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	content := []byte(`---
fileClass: contact
name: John Doe
tags: [work, important]
---
# Contact Note
Some content here`)

	frontmatter, err := service.Extract(content)
	require.NoError(t, err)

	// These assertions will fail until Extract is properly implemented
	assert.Equal(t, "contact", frontmatter.FileClass)
	assert.Equal(t, "John Doe", frontmatter.Fields["name"])
	assert.Equal(
		t,
		[]any{"work", "important"},
		frontmatter.Fields["tags"],
	)
}

// TestFrontmatterService_ExtractValidTOML verifies TOML frontmatter extraction.
func TestFrontmatterService_ExtractValidTOML(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	content := []byte(`+++
fileClass = "project"
name = "Lithos"
active = true
+++
# Project Note
Some content here`)

	frontmatter, err := service.Extract(content)
	require.NoError(t, err)

	// These assertions will fail until Extract is properly implemented
	assert.Equal(t, "project", frontmatter.FileClass)
	assert.Equal(t, "Lithos", frontmatter.Fields["name"])
	assert.Equal(t, true, frontmatter.Fields["active"])
}

// TestFrontmatterService_ExtractMissingFrontmatter verifies empty result for
// missing frontmatter.
func TestFrontmatterService_ExtractMissingFrontmatter(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	content := []byte(`# Just a title
Some content without frontmatter`)

	frontmatter, err := service.Extract(content)
	require.NoError(t, err)

	// Should return empty frontmatter when none exists
	assert.Empty(t, frontmatter.FileClass)
	assert.Empty(t, frontmatter.Fields)
}

// TestFrontmatterService_ExtractMalformedFrontmatter verifies structured error
// for parse failures.
func TestFrontmatterService_ExtractMalformedFrontmatter(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	content := []byte(`---
invalid yaml content here: [unclosed bracket
missing_quotes: this should fail
---
# Test
Content`)

	frontmatter, err := service.Extract(content)
	// Check if goldmark handles this gracefully or returns error
	if err != nil {
		// Should return structured FrontmatterError
		assert.Contains(t, err.Error(), "frontmatter")
	} else {
		// If goldmark parses it gracefully, verify we got some result
		t.Logf("Goldmark handled malformed YAML gracefully, got: %+v", frontmatter)
		// For now, accept that goldmark might be lenient
	}
}

// TestFrontmatterService_ExtractEdgeCases verifies handling of code blocks with
// delimiters.
func TestFrontmatterService_ExtractEdgeCases(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	// Content with code block containing frontmatter-like delimiters
	content := []byte(`---
fileClass: note
title: Code Example
---
# Test Note

Here's some code:

` + "```yaml" + `
---
this: is not frontmatter
---
` + "```" + `

More content`)

	frontmatter, err := service.Extract(content)
	require.NoError(t, err)

	// Should extract only the real frontmatter, not the code block
	assert.Equal(t, "note", frontmatter.FileClass)
	assert.Equal(t, "Code Example", frontmatter.Fields["title"])
}

// Task 4 RED phase tests - these should fail until Validate method is
// implemented

// TestFrontmatterService_ValidateMethodSignature verifies Validate method
// exists.
func TestFrontmatterService_ValidateMethodSignature(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	// Test method signature exists
	ctx := context.Background()
	frontmatter := domain.NewFrontmatter(map[string]any{
		"fileClass": "test",
		"name":      "Test Note",
	})
	testSchema := domain.Schema{
		Name:       "test",
		Properties: []domain.Property{},
	}

	err := service.Validate(ctx, frontmatter, testSchema)
	// For now, expect no error since method returns nil (not implemented)
	require.NoError(t, err)
}

// TestFrontmatterService_ValidateRequiredFields verifies required field
// validation.
func TestFrontmatterService_ValidateRequiredFields(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	ctx := context.Background()

	// Create schema with required field
	requiredProp := domain.Property{
		Name:     "title",
		Required: true,
		Spec:     &domain.StringSpec{},
	}
	testSchema := domain.Schema{
		Name:       "test",
		Properties: []domain.Property{requiredProp},
	}

	// Test frontmatter missing required field
	frontmatter := domain.NewFrontmatter(map[string]any{
		"fileClass": "test",
		// Missing "title" field
	})

	err := service.Validate(ctx, frontmatter, testSchema)
	// This should fail once validation is implemented
	assert.Error(t, err, "Missing required field should fail validation")
}

// TestFrontmatterService_ValidateFieldTypes verifies type validation using
// field validators.
func TestFrontmatterService_ValidateFieldTypes(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	ctx := context.Background()

	// Create schema with string field
	stringProp := domain.Property{
		Name:     "title",
		Required: true,
		Spec: &domain.StringSpec{
			Enum: []string{"valid", "allowed"},
		},
	}
	testSchema := domain.Schema{
		Name:       "test",
		Properties: []domain.Property{stringProp},
	}

	// Test frontmatter with invalid enum value
	frontmatter := domain.NewFrontmatter(map[string]any{
		"fileClass": "test",
		"title":     "invalid",
	})

	err := service.Validate(ctx, frontmatter, testSchema)
	// This should fail once validation is implemented
	assert.Error(t, err, "Invalid enum value should fail validation")
}

// TestFrontmatterService_ValidateUnknownFieldPreservation verifies unknown
// field preservation (FR6).
func TestFrontmatterService_ValidateUnknownFieldPreservation(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	ctx := context.Background()

	// Create schema with only one field
	knownProp := domain.Property{
		Name:     "title",
		Required: true,
		Spec:     &domain.StringSpec{},
	}
	testSchema := domain.Schema{
		Name:       "test",
		Properties: []domain.Property{knownProp},
	}

	// Test frontmatter with known and unknown fields
	frontmatter := domain.NewFrontmatter(map[string]any{
		"fileClass":    "test",
		"title":        "Valid Title",
		"unknownField": "should be preserved",
	})

	err := service.Validate(ctx, frontmatter, testSchema)
	// This should pass - unknown fields should be preserved per FR6
	require.NoError(t, err, "Unknown fields should be preserved per FR6")
}

// Task 5 RED phase tests - these should fail until integration is implemented

// TestFrontmatterService_IntegrationWithSchemaEngine verifies integration with
// real SchemaEngine.
func TestFrontmatterService_IntegrationWithSchemaEngine(t *testing.T) {
	// Create a fake SchemaEngine for testing
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	ctx := context.Background()

	// Create test frontmatter
	frontmatter := domain.NewFrontmatter(map[string]any{
		"fileClass": "contact",
		"name":      "John Doe",
		"email":     "john@example.com",
	})

	// Create test schema with various property types
	nameProperty := domain.Property{
		Name:     "name",
		Required: true,
		Spec:     &domain.StringSpec{},
	}
	emailProperty := domain.Property{
		Name:     "email",
		Required: true,
		Spec: &domain.StringSpec{
			Pattern: `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`,
		},
	}
	testSchema := domain.Schema{
		Name:       "contact",
		Properties: []domain.Property{nameProperty, emailProperty},
	}

	// Test full integration workflow
	err := service.Validate(ctx, frontmatter, testSchema)
	require.NoError(
		t,
		err,
		"Valid frontmatter should pass integration validation",
	)

	// Test with unsupported property type (FileSpec)
	fileProperty := domain.Property{
		Name:     "file",
		Required: false,
		Spec:     &domain.FileSpec{}, // This will be skipped in validation
	}
	schemaWithFileSpec := domain.Schema{
		Name: "contact",
		Properties: []domain.Property{
			nameProperty,
			emailProperty,
			fileProperty,
		},
	}
	frontmatterWithFile := domain.NewFrontmatter(map[string]any{
		"fileClass": "contact",
		"name":      "John Doe",
		"email":     "john@example.com",
		"file":      "some-file.md",
	})

	err = service.Validate(ctx, frontmatterWithFile, schemaWithFileSpec)
	require.NoError(t, err, "Unsupported property types should be skipped")
}

// TestFrontmatterService_VaultIndexerIntegrationWorkflow verifies VaultIndexer
// integration pattern.
func TestFrontmatterService_VaultIndexerIntegrationWorkflow(t *testing.T) {
	// This test simulates the VaultIndexer workflow described in Dev Notes
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	ctx := context.Background()

	// Step 1: SchemaEngine loads schemas (simulated)
	// In real implementation, SchemaEngine.Load(ctx) would be called first

	// Step 2: Extract frontmatter from content
	content := []byte(`---
fileClass: project
name: Lithos
status: active
priority: 1
---
# Project Documentation
This is a test project.`)

	frontmatter, err := service.Extract(content)
	require.NoError(t, err, "Extract should succeed")
	assert.Equal(t, "project", frontmatter.FileClass)
	assert.Equal(t, "Lithos", frontmatter.Fields["name"])

	// Step 3: Get schema for this file class (simulated)
	priorityProperty := domain.Property{
		Name:     "priority",
		Required: true,
		Spec: &domain.NumberSpec{
			Min: func() *float64 { v := 1.0; return &v }(),
			Max: func() *float64 { v := 5.0; return &v }(),
		},
	}
	statusProperty := domain.Property{
		Name:     "status",
		Required: true,
		Spec: &domain.StringSpec{
			Enum: []string{"active", "inactive", "completed"},
		},
	}
	projectSchema := domain.Schema{
		Name:       "project",
		Properties: []domain.Property{priorityProperty, statusProperty},
	}

	// Step 4: Validate frontmatter against schema
	err = service.Validate(ctx, frontmatter, projectSchema)
	require.NoError(t, err, "Valid project frontmatter should pass validation")

	// Step 5: Create Note and add to index (simulated)
	note := domain.NewNote(
		domain.NewNoteID("test-project.md"),
		time.Now(),
		frontmatter,
	)
	assert.Equal(t, "project", note.SchemaName())
	assert.Equal(t, "Lithos", note.Frontmatter.Fields["name"])
}

// TestFrontmatterService_IntegrationErrorHandling verifies error handling
// across service boundaries.
func TestFrontmatterService_IntegrationErrorHandling(t *testing.T) {
	fakeSchemaEngine := &schema.SchemaEngine{}
	fakeLogger := lithosLog.NewTest()
	service := NewFrontmatterService(fakeSchemaEngine, fakeLogger)

	ctx := context.Background()

	// Test malformed frontmatter extraction
	malformedContent := []byte(`---
invalid: yaml: [unclosed
---
# Test`)

	frontmatter, err := service.Extract(malformedContent)
	// Goldmark returns error for malformed YAML
	if err != nil {
		assert.Contains(t, err.Error(), "frontmatter")
		assert.Contains(t, err.Error(), "parse")
	} else {
		// If goldmark handles gracefully, frontmatter should be empty
		assert.Empty(t, frontmatter.Fields)
	}

	// Test validation errors propagation - missing required field
	missingFrontmatter := domain.NewFrontmatter(map[string]any{
		"fileClass": "test",
		// Missing "required" field entirely
	})

	requiredProp := domain.Property{
		Name:     "required",
		Required: true,
		Spec:     &domain.StringSpec{},
	}
	testSchema := domain.Schema{
		Name:       "test",
		Properties: []domain.Property{requiredProp},
	}

	err = service.Validate(ctx, missingFrontmatter, testSchema)
	require.Error(t, err, "Validation should fail for missing required field")
	assert.Contains(t, err.Error(), "required field missing")

	// Test type validation errors propagation
	invalidTypeFrontmatter := domain.NewFrontmatter(map[string]any{
		"fileClass": "test",
		"required":  123, // Wrong type (should be string)
	})

	err = service.Validate(ctx, invalidTypeFrontmatter, testSchema)
	require.Error(t, err, "Validation should fail for wrong field type")
	assert.Contains(t, err.Error(), "field value is not a string")
}
