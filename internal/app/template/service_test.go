package template

import (
	"context"
	"errors"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Ensure mockTemplatePort implements TemplatePort.
var _ spi.TemplatePort = (*mockTemplatePort)(nil)

// mockTemplatePort provides a mock implementation of TemplatePort for testing.
type mockTemplatePort struct {
	templates map[domain.TemplateID]domain.Template
	loadError error
}

func newMockTemplatePort() *mockTemplatePort {
	return &mockTemplatePort{
		templates: make(map[domain.TemplateID]domain.Template),
	}
}

// List returns a list of available template IDs.
func (m *mockTemplatePort) List(
	ctx context.Context,
) ([]domain.TemplateID, error) {
	var ids []domain.TemplateID
	for id := range m.templates {
		ids = append(ids, id)
	}
	return ids, nil
}

// Load retrieves a template by ID.
func (m *mockTemplatePort) Load(
	ctx context.Context,
	id domain.TemplateID,
) (domain.Template, error) {
	if m.loadError != nil {
		return domain.Template{}, m.loadError
	}
	tmpl, exists := m.templates[id]
	if !exists {
		return domain.Template{}, domainerrors.NewResourceError(
			"template",
			"load",
			string(id),
			nil,
		)
	}
	return tmpl, nil
}

func (m *mockTemplatePort) setTemplates(
	templates map[domain.TemplateID]domain.Template,
) {
	m.templates = templates
}

func (m *mockTemplatePort) setLoadError(err error) {
	m.loadError = err
}

// TestTemplateEngine_Load tests the TemplateEngine Load functionality.
func TestTemplateEngine_Load(t *testing.T) {
	ctx := context.Background()
	templateID := domain.NewTemplateID("test-template")
	template := domain.NewTemplate(templateID, "test content")

	t.Run("delegates to TemplatePort correctly", func(t *testing.T) {
		mockPort := newMockTemplatePort()
		mockPort.setTemplates(map[domain.TemplateID]domain.Template{
			templateID: template,
		})

		config := domain.Config{}
		logger := zerolog.Nop()
		engine := NewTemplateEngine(mockPort, &config, &logger)

		result, err := engine.Load(ctx, templateID)

		require.NoError(t, err)
		assert.Equal(t, template, result)
	})

	t.Run("propagates errors from port", func(t *testing.T) {
		expectedErr := errors.New("port error")
		mockPort := newMockTemplatePort()
		mockPort.setLoadError(expectedErr)

		config := domain.Config{}
		logger := zerolog.Nop()
		engine := NewTemplateEngine(mockPort, &config, &logger)

		_, err := engine.Load(ctx, templateID)

		assert.Equal(t, expectedErr, err)
	})

	t.Run("uses path control functions correctly", func(t *testing.T) {
		testTemplate := domain.NewTemplate(
			templateID,
			`Path: {{path}}, Vault: {{vaultPath}}`,
		)
		mockPort := newMockTemplatePort()
		mockPort.setTemplates(map[domain.TemplateID]domain.Template{
			templateID: testTemplate,
		})

		config := domain.Config{VaultPath: "/test/vault"}
		logger := zerolog.Nop()
		engine := NewTemplateEngine(mockPort, &config, &logger)

		result, err := engine.Render(ctx, templateID)

		require.NoError(t, err)
		assert.Equal(t, "Path: , Vault: /test/vault", result)
	})

	t.Run("uses now function correctly", func(t *testing.T) {
		tmpl := domain.NewTemplate(templateID, `Date: {{now "2006-01-02"}}`)
		mockPort := newMockTemplatePort()
		mockPort.setTemplates(map[domain.TemplateID]domain.Template{
			templateID: tmpl,
		})

		config := domain.Config{}
		logger := zerolog.Nop()
		engine := NewTemplateEngine(mockPort, &config, &logger)

		result, err := engine.Render(ctx, templateID)

		require.NoError(t, err)
		// Should be today's date in YYYY-MM-DD format
		assert.Regexp(t, `^Date: \d{4}-\d{2}-\d{2}$`, result)
	})

	t.Run("parse error returns TemplateError with details", func(t *testing.T) {
		// Invalid template syntax
		invalidTemplate := domain.NewTemplate(templateID, `{{invalid syntax}}`)
		mockPort := newMockTemplatePort()
		mockPort.setTemplates(map[domain.TemplateID]domain.Template{
			templateID: invalidTemplate,
		})

		config := domain.Config{}
		logger := zerolog.Nop()
		engine := NewTemplateEngine(mockPort, &config, &logger)

		_, err := engine.Render(ctx, templateID)

		require.Error(t, err)
		var templateErr *domainerrors.TemplateError
		require.ErrorAs(t, err, &templateErr)
		assert.Equal(t, "test-template", templateErr.TemplateID())
		assert.Contains(t, err.Error(), "parse error")
	})

	t.Run(
		"execute error returns TemplateError with context",
		func(t *testing.T) {
			// Template that references a non-existent template
			errorTemplate := domain.NewTemplate(
				templateID,
				`{{template "nonexistent"}}`,
			)
			mockPort := newMockTemplatePort()
			mockPort.setTemplates(map[domain.TemplateID]domain.Template{
				templateID: errorTemplate,
			})

			config := domain.Config{}
			logger := zerolog.Nop()
			engine := NewTemplateEngine(
				mockPort,
				&config,
				&logger,
			)

			_, err := engine.Render(ctx, templateID)

			require.Error(t, err)
			var templateErr *domainerrors.TemplateError
			require.ErrorAs(t, err, &templateErr)
			assert.Equal(t, "test-template", templateErr.TemplateID())
			assert.Contains(t, err.Error(), "execute error")
		},
	)

	t.Run("template not found propagates ResourceError", func(t *testing.T) {
		mockPort := newMockTemplatePort()
		// No templates set, so Load will fail

		config := domain.Config{}
		logger := zerolog.Nop()
		engine := NewTemplateEngine(mockPort, &config, &logger)

		_, err := engine.Render(ctx, templateID)

		require.Error(t, err)
		var resourceErr *domainerrors.ResourceError
		assert.ErrorAs(t, err, &resourceErr)
	})
}

// TestTemplateEngine_BuildFuncMap tests the buildFuncMap method and all custom
// template functions.
func TestTemplateEngine_BuildFuncMap(t *testing.T) {
	config := domain.Config{VaultPath: "/test/vault"}
	logger := zerolog.Nop()
	engine := NewTemplateEngine(nil, &config, &logger)
	funcMap := engine.buildFuncMap()

	t.Run("now function returns formatted timestamp", func(t *testing.T) {
		nowFunc := funcMap["now"].(func(string) string)
		result := nowFunc("2006-01-02")
		// Should be today's date in YYYY-MM-DD format
		assert.Regexp(t, `^\d{4}-\d{2}-\d{2}$`, result)
	})

	t.Run("toLower converts to lowercase", func(t *testing.T) {
		toLowerFunc := funcMap["toLower"].(func(string) string)
		assert.Equal(t, "hello", toLowerFunc("HELLO"))
		assert.Equal(t, "world", toLowerFunc("World"))
	})

	t.Run("toUpper converts to uppercase", func(t *testing.T) {
		toUpperFunc := funcMap["toUpper"].(func(string) string)
		assert.Equal(t, "HELLO", toUpperFunc("hello"))
		assert.Equal(t, "WORLD", toUpperFunc("World"))
	})

	t.Run("folder returns parent directory", func(t *testing.T) {
		folderFunc := funcMap["folder"].(func(string) string)
		assert.Equal(t, "/path/to", folderFunc("/path/to/file.txt"))
		assert.Equal(t, ".", folderFunc("file.txt"))
	})

	t.Run("basename strips path and extension", func(t *testing.T) {
		basenameFunc := funcMap["basename"].(func(string) string)
		assert.Equal(t, "file", basenameFunc("/path/to/file.txt"))
		assert.Equal(t, "document", basenameFunc("document.md"))
		assert.Equal(t, "test", basenameFunc("test"))
	})

	t.Run("extension returns extension with dot", func(t *testing.T) {
		extensionFunc := funcMap["extension"].(func(string) string)
		assert.Equal(t, ".txt", extensionFunc("/path/to/file.txt"))
		assert.Equal(t, ".md", extensionFunc("document.md"))
		assert.Empty(t, extensionFunc("test"))
	})

	t.Run("join uses OS-appropriate path separator", func(t *testing.T) {
		joinFunc := funcMap["join"].(func(...string) string)
		result := joinFunc("path", "to", "file")
		// Should contain path separator appropriate for the OS
		assert.Contains(t, result, string(filepath.Separator))
	})

	t.Run("vaultPath returns config value", func(t *testing.T) {
		vaultPathFunc := funcMap["vaultPath"].(func() string)
		assert.Equal(t, "/test/vault", vaultPathFunc())
	})
}
