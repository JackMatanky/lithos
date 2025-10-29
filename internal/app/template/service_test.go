package template

import (
	"context"
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
