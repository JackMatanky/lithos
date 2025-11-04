package integration

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"

	templateAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/template"
	templateService "github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestTemplateEngine_RenderStaticTemplate tests end-to-end template rendering
// with static template.
func TestTemplateEngine_RenderStaticTemplate(t *testing.T) {
	// Setup
	ctx := context.Background()
	testdataDir := filepath.Join("..", "..", "testdata", "templates")
	config := domain.Config{
		VaultPath:    "/test/vault",
		TemplatesDir: testdataDir,
	}
	logger := zerolog.Nop()

	// Create template loader with testdata directory
	loader := templateAdapter.NewTemplateLoaderAdapter(&config, &logger)

	// List templates to populate metadata cache
	_, err := loader.List(ctx)
	require.NoError(t, err)

	// Create template engine
	engine := templateService.NewTemplateEngine(loader, &config, &logger)

	// Load and render static template
	templateID := domain.NewTemplateID("static_template")
	result, err := engine.Render(ctx, templateID)
	require.NoError(t, err)

	// Load expected output
	expectedPath := filepath.Join(
		"..",
		"..",
		"testdata",
		"golden",
		"static_template_expected.md",
	)
	expectedBytes, err := os.ReadFile(expectedPath)
	require.NoError(t, err)
	expected := string(expectedBytes)

	// For the date field, we need to handle the dynamic date
	// The expected file has "Created: 2025-10-19" but the actual will have
	// today's date
	// We'll check that the format is correct and the static parts match

	lines := strings.Split(result, "\n")
	expectedLines := strings.Split(expected, "\n")

	// Check static parts
	assert.Equal(t, expectedLines[0], lines[0]) // # Static Template Example
	assert.Equal(t, expectedLines[1], lines[1]) // empty line
	assert.Equal(t, expectedLines[3], lines[3]) // Uppercase: HELLO WORLD
	assert.Equal(t, expectedLines[4], lines[4]) // Lowercase: hello world
	assert.Equal(t, expectedLines[5], lines[5]) // empty line
	assert.Equal(t, expectedLines[6], lines[6]) // This is a static template...
	assert.Equal(t, expectedLines[7], lines[7]) // empty line

	// Check date format (should be YYYY-MM-DD)
	dateLine := lines[2]
	assert.Contains(t, dateLine, "Created: ")
	datePart := strings.TrimPrefix(dateLine, "Created: ")
	assert.Regexp(t, `^\d{4}-\d{2}-\d{2}$`, datePart)
}
