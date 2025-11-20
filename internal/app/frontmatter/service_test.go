package frontmatter

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockMarkdownParserPort provides a mock implementation of MarkdownParserPort
// for pure domain testing without external dependencies.
type mockMarkdownParserPort struct{}

func (m *mockMarkdownParserPort) ParseFrontmatter(
	ctx context.Context,
	content []byte,
) (map[string]any, error) {
	return map[string]any{"title": "test"}, nil
}

// TestFrontmatterService_StructExists verifies FrontmatterService struct
// exists.
func TestFrontmatterService_StructExists(t *testing.T) {
	var _ *FrontmatterService // Compilation test - will fail if struct doesn't exist
}

// TestNewFrontmatterService_ConstructorWithDependencies verifies constructor
// properly injects all required dependencies including MarkdownParserPort.
func TestNewFrontmatterService_ConstructorWithDependencies(t *testing.T) {
	// Given - use nil for schema engine since we're just testing injection
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()

	// When
	service := NewFrontmatterService(nil, fakeMarkdownParser, fakeLogger)

	// Then
	require.NotNil(t, service)
	assert.Nil(t, service.schemaEngine) // We passed nil
	assert.Equal(t, fakeMarkdownParser, service.markdownParserPort)
	assert.Equal(t, fakeLogger, service.logger)
}

// TestNewFrontmatterService_DependencyInjection verifies all dependencies are
// properly injected.
func TestNewFrontmatterService_DependencyInjection(t *testing.T) {
	// Given
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()

	// When
	service := NewFrontmatterService(nil, fakeMarkdownParser, fakeLogger)

	// Then
	assert.NotNil(t, service.markdownParserPort)
	assert.NotNil(t, service.logger)
}

// TestIsSchemaCompliant_ValidFrontmatter verifies IsSchemaCompliant accepts
// valid frontmatter without schema validation.
func TestIsSchemaCompliant_ValidFrontmatter(t *testing.T) {
	// Given
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(nil, fakeMarkdownParser, fakeLogger)

	validFm := domain.NewFrontmatter(map[string]interface{}{
		"title": "Test Note",
	})

	// When
	err := service.IsSchemaCompliant(context.Background(), validFm)

	// Then
	assert.NoError(t, err)
}

// TestIsSchemaCompliant_WithFileClass verifies IsSchemaCompliant handles
// fileClass
// gracefully when schema engine is not available.
func TestIsSchemaCompliant_WithFileClass(t *testing.T) {
	// Given - nil schema engine means schema validation will fail
	fakeMarkdownParser := &mockMarkdownParserPort{}
	fakeLogger := zerolog.Nop()
	service := NewFrontmatterService(nil, fakeMarkdownParser, fakeLogger)

	fmWithFileClass := domain.Frontmatter{
		FileClass: "contact",
		Fields: map[string]interface{}{
			"name": "John Doe",
		},
	}

	// When
	err := service.IsSchemaCompliant(context.Background(), fmWithFileClass)

	// Then
	// Should fail because schema validation is attempted but schema engine is
	// nil
	require.Error(t, err)
	assert.Contains(t, err.Error(), "schema engine not available")
}
