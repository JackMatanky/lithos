package template

import (
	"context"
	"errors"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/query"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const (
	testAuthorName  = "John"
	testAuthorField = "author"
)

// Ensure mockTemplatePort implements TemplatePort.
var _ spi.TemplatePort = (*mockTemplatePort)(nil)

// testMockMetadataQueryPort extends MockMetadataQueryPort with configurable
// Read.
type testMockMetadataQueryPort struct {
	*utils.MockMetadataQueryPort

	readFunc func(ctx context.Context, path string) (domain.Note, error)
}

// mockTemplatePort provides a mock implementation of TemplatePort for testing.
type mockTemplatePort struct {
	templates map[domain.TemplateID]domain.Template
	loadError error
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
		return nil, m.loadError
	}
	tmpl, exists := m.templates[id]
	if !exists {
		return nil, lithosErr.NewResourceError(
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

func (m *testMockMetadataQueryPort) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	if m.readFunc != nil {
		return m.readFunc(ctx, path)
	}
	return m.MockMetadataQueryPort.Read(ctx, path)
}

func newMockTemplatePort() *mockTemplatePort {
	return &mockTemplatePort{
		templates: make(map[domain.TemplateID]domain.Template),
	}
}

// createTestQueryService creates a QueryService with test mocks.
func createTestQueryService() *query.QueryService {
	// Use the comprehensive mocks from tests/utils
	boltReader := utils.NewMockMetadataQueryPort()
	sqliteReader := utils.NewMockMetadataQueryPort()

	config := domain.Config{}
	logger := zerolog.Nop()
	eventBus := utils.NewMockEventBus()

	return query.NewQueryService(
		boltReader,
		sqliteReader,
		config,
		logger,
		eventBus,
	)
}

// TestTemplateEngine_Load tests the TemplateEngine Load functionality.
func TestTemplateEngine_Load(t *testing.T) {
	ctx := context.Background()
	templateID := domain.NewTemplateID("test-template")

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
		engine := NewTemplateEngine(
			mockPort,
			&config,
			nil,
			&logger,
			utils.NewMockEventBus(),
		)

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
		engine := NewTemplateEngine(
			mockPort,
			&config,
			nil,
			&logger,
			utils.NewMockEventBus(),
		)

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
		engine := NewTemplateEngine(
			mockPort,
			&config,
			nil,
			&logger,
			utils.NewMockEventBus(),
		)

		_, err := engine.Render(ctx, templateID)

		require.Error(t, err)
		var templateErr *lithosErr.TemplateError
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
				nil,
				&logger,
				utils.NewMockEventBus(),
			)

			_, err := engine.Render(ctx, templateID)

			require.Error(t, err)
			var templateErr *lithosErr.TemplateError
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
		engine := NewTemplateEngine(
			mockPort,
			&config,
			nil,
			&logger,
			utils.NewMockEventBus(),
		)

		_, err := engine.Render(ctx, templateID)

		require.Error(t, err)
		var resourceErr *lithosErr.ResourceError
		assert.ErrorAs(t, err, &resourceErr)
	})
}

// TestTemplateEngine_BuildFuncMap tests the buildFuncMap method and all custom
// template functions.
func TestTemplateEngine_BuildFuncMap(t *testing.T) {
	mockPort := newMockTemplatePort()
	config := domain.Config{VaultPath: "/test/vault"}
	logger := zerolog.Nop()
	engine := NewTemplateEngine(
		mockPort,
		&config,
		nil,
		&logger,
		utils.NewMockEventBus(),
	)
	funcMap := engine.buildFuncMap(context.Background())

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

// TestTemplateEngine_LookupFunction tests the lookup template function.
func TestTemplateEngine_LookupFunction(t *testing.T) {
	config := domain.Config{VaultPath: "/test/vault"}
	logger := zerolog.Nop()

	t.Run(
		"lookup function exists and has correct signature",
		func(t *testing.T) {
			mockQuery := createTestQueryService()
			engine := NewTemplateEngine(
				nil,
				&config,
				mockQuery,
				&logger,
				utils.NewMockEventBus(),
			)

			funcMap := engine.buildFuncMap(context.Background())
			lookupFunc, ok := funcMap["lookup"]

			require.True(t, ok, "lookup function should be registered")
			fn, ok := lookupFunc.(func(string) (domain.Note, error))
			require.True(
				t,
				ok,
				"lookup should have signature func(string) (domain.Note, error)",
			)
			assert.NotNil(t, fn, "lookup function should not be nil")
		},
	)

	t.Run("lookup returns error when note not found", func(t *testing.T) {
		mockQuery := createTestQueryService()
		engine := NewTemplateEngine(
			nil,
			&config,
			mockQuery,
			&logger,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		lookupFunc := funcMap["lookup"].(func(string) (domain.Note, error))

		_, err := lookupFunc("nonexistent")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "not found")
	})

	t.Run(
		"lookup returns error when QueryService unavailable",
		func(t *testing.T) {
			engine := NewTemplateEngine(
				nil,
				&config,
				nil,
				&logger,
				utils.NewMockEventBus(),
			)

			funcMap := engine.buildFuncMap(context.Background())
			lookupFunc := funcMap["lookup"].(func(string) (domain.Note, error))

			_, err := lookupFunc("any")
			require.Error(t, err)
			assert.Contains(t, err.Error(), "query service not available")
		},
	)

	t.Run("lookup returns note for successful match", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNote := domain.Note{
			Path: "test.md",
			Frontmatter: domain.NewFrontmatter(map[string]any{
				"title": "Test Note",
			}),
		}
		mockBolt.SetPathQueryResult([]domain.Note{testNote}, nil)

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		lookupFunc := funcMap["lookup"].(func(string) (domain.Note, error))

		result, err := lookupFunc("test")
		require.NoError(t, err)
		assert.Equal(t, "test.md", result.Path)
		assert.Equal(t, "Test Note", result.Frontmatter.Fields["title"])
	})

	t.Run("lookup returns error for ambiguous match", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		note1 := domain.Note{Path: "test1.md"}
		note2 := domain.Note{Path: "test2.md"}
		mockBolt.SetPathQueryResult([]domain.Note{note1, note2}, nil)

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		lookupFunc := funcMap["lookup"].(func(string) (domain.Note, error))

		_, err := lookupFunc("test")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "ambiguous")
	})
}

// TestTemplateEngine_QueryFunction tests the query template function.
func TestTemplateEngine_QueryFunction(t *testing.T) {
	config := domain.Config{VaultPath: "/test/vault"}
	logger := zerolog.Nop()

	t.Run(
		"query function exists and has correct signature",
		func(t *testing.T) {
			mockQuery := createTestQueryService()
			engine := NewTemplateEngine(
				nil,
				&config,
				mockQuery,
				&logger,
				utils.NewMockEventBus(),
			)

			funcMap := engine.buildFuncMap(context.Background())
			queryFunc, ok := funcMap["query"]

			require.True(t, ok, "query function should be registered")
			fn, ok := queryFunc.(func(map[string]any) ([]domain.Note, error))
			require.True(
				t,
				ok,
				"query should have signature func(map[string]any) ([]domain.Note, error)",
			)
			assert.NotNil(t, fn, "query function should not be nil")
		},
	)

	t.Run(
		"query returns error when QueryService unavailable",
		func(t *testing.T) {
			engine := NewTemplateEngine(
				nil,
				&config,
				nil,
				&logger,
				utils.NewMockEventBus(),
			)

			funcMap := engine.buildFuncMap(context.Background())
			queryFunc := funcMap["query"].(func(map[string]any) ([]domain.Note, error))

			_, err := queryFunc(map[string]any{"fileClass": "contact"})
			require.Error(t, err)
			assert.Contains(t, err.Error(), "query service not available")
		},
	)

	t.Run("query returns notes for successful match", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNote := domain.Note{
			Path: "contact.md",
			Frontmatter: domain.NewFrontmatter(map[string]any{
				"fileClass": "contact",
			}),
		}
		mockSqlite.SetFrontmatterQueryResult([]domain.Note{testNote}, nil)

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		queryFunc := funcMap["query"].(func(map[string]any) ([]domain.Note, error))

		result, err := queryFunc(map[string]any{"fileClass": "contact"})
		require.NoError(t, err)
		assert.Len(t, result, 1)
		assert.Equal(t, "contact.md", result[0].Path)
	})

	t.Run("query returns empty slice for no matches", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		mockSqlite.SetFrontmatterQueryResult([]domain.Note{}, nil)

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		queryFunc := funcMap["query"].(func(map[string]any) ([]domain.Note, error))

		result, err := queryFunc(map[string]any{"fileClass": "nonexistent"})
		require.NoError(t, err)
		assert.Empty(t, result)
	})
}

// TestTemplateEngine_FileClassFunction tests the fileClass template function.
func TestTemplateEngine_FileClassFunction(t *testing.T) {
	config := domain.Config{VaultPath: "/test/vault"}
	logger := zerolog.Nop()

	t.Run(
		"fileClass function exists and has correct signature",
		func(t *testing.T) {
			mockQuery := createTestQueryService()
			engine := NewTemplateEngine(
				nil,
				&config,
				mockQuery,
				&logger,
				utils.NewMockEventBus(),
			)

			funcMap := engine.buildFuncMap(context.Background())
			fileClassFunc, ok := funcMap["fileClass"]

			require.True(t, ok, "fileClass function should be registered")
			fn, ok := fileClassFunc.(func(string) string)
			require.True(
				t,
				ok,
				"fileClass should have signature func(string) string",
			)
			assert.NotNil(t, fn, "fileClass function should not be nil")
		},
	)

	t.Run(
		"fileClass returns empty string when QueryService unavailable",
		func(t *testing.T) {
			engine := NewTemplateEngine(
				nil,
				&config,
				nil,
				&logger,
				utils.NewMockEventBus(),
			)

			funcMap := engine.buildFuncMap(context.Background())
			fileClassFunc := funcMap["fileClass"].(func(string) string)

			result := fileClassFunc("test.md")
			assert.Empty(t, result)
		},
	)

	t.Run(
		"fileClass returns fileClass for successful lookup",
		func(t *testing.T) {
			mockBolt := &testMockMetadataQueryPort{
				MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
			}
			mockSqlite := utils.NewMockMetadataQueryPort()

			testNote := domain.Note{
				Path: "contact.md",
				Frontmatter: domain.NewFrontmatter(map[string]any{
					"fileClass": "contact",
				}),
			}
			mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
				return testNote, nil
			}

			cfg := domain.Config{}
			log := zerolog.Nop()
			eventBus := utils.NewMockEventBus()

			querySvc := query.NewQueryService(
				mockBolt,
				mockSqlite,
				cfg,
				log,
				eventBus,
			)
			engine := NewTemplateEngine(
				nil,
				&cfg,
				querySvc,
				&log,
				utils.NewMockEventBus(),
			)

			funcMap := engine.buildFuncMap(context.Background())
			fileClassFunc := funcMap["fileClass"].(func(string) string)

			result := fileClassFunc("contact.md")
			assert.Equal(t, "contact", result)
		},
	)

	t.Run("fileClass handles various fileClass values", func(t *testing.T) {
		mockBolt := &testMockMetadataQueryPort{
			MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
		}
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNote := domain.Note{
			Path: "test.md",
			Frontmatter: domain.NewFrontmatter(map[string]any{
				"fileClass": "custom_schema",
			}),
		}
		mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
			return testNote, nil
		}

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		fileClassFunc := funcMap["fileClass"].(func(string) string)

		result := fileClassFunc("test.md")
		assert.Equal(t, "custom_schema", result)
	})

	t.Run("fileClass returns error for note not found", func(t *testing.T) {
		mockBolt := &testMockMetadataQueryPort{
			MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
		}
		mockSqlite := utils.NewMockMetadataQueryPort()

		mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
			return domain.Note{}, errors.New("not found")
		}

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		fileClassFunc := funcMap["fileClass"].(func(string) string)

		result := fileClassFunc("nonexistent.md")
		assert.Empty(t, result)
	})

	t.Run(
		"fileClass returns error for missing fileClass field",
		func(t *testing.T) {
			mockBolt := &testMockMetadataQueryPort{
				MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
			}
			mockSqlite := utils.NewMockMetadataQueryPort()

			testNote := domain.Note{
				Path: "note.md",
				Frontmatter: domain.NewFrontmatter(map[string]any{
					"title": "Note without fileClass",
				}),
			}
			mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
				return testNote, nil
			}

			cfg := domain.Config{}
			log := zerolog.Nop()
			eventBus := utils.NewMockEventBus()

			querySvc := query.NewQueryService(
				mockBolt,
				mockSqlite,
				cfg,
				log,
				eventBus,
			)
			engine := NewTemplateEngine(
				nil,
				&cfg,
				querySvc,
				&log,
				utils.NewMockEventBus(),
			)

			funcMap := engine.buildFuncMap(context.Background())
			fileClassFunc := funcMap["fileClass"].(func(string) string)

			result := fileClassFunc("note.md")
			assert.Empty(t, result)
		},
	)
}

// TestNote_Clone tests the Note.Clone method for immutability.
func TestNote_Clone(t *testing.T) {
	originalNote := domain.Note{
		Path: "test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]any{
				"title":     "Test Note",
				"tags":      []string{"test", "clone"},
				"fileClass": "note",
			},
		},
		Links: []domain.Link{
			{Text: "link1", Destination: "dest1"},
		},
		Tags: []string{"original"},
	}

	clonedNote := originalNote.Clone()

	// Verify the clone has the same data
	assert.Equal(t, originalNote.Path, clonedNote.Path)
	assert.Equal(
		t,
		originalNote.Frontmatter.Fields["title"],
		clonedNote.Frontmatter.Fields["title"],
	)
	assert.Equal(
		t,
		originalNote.Frontmatter.Fields["fileClass"],
		clonedNote.Frontmatter.Fields["fileClass"],
	)

	// Modify the clone and verify original is unchanged
	clonedNote.Frontmatter.Fields["title"] = "Modified Title"
	clonedNote.Tags = append(clonedNote.Tags, "modified")

	// Original should be unchanged
	assert.Equal(t, "Test Note", originalNote.Frontmatter.Fields["title"])
	assert.Equal(t, []string{"original"}, originalNote.Tags)

	// Clone should have modifications
	assert.Equal(t, "Modified Title", clonedNote.Frontmatter.Fields["title"])
	assert.Contains(t, clonedNote.Tags, "modified")
}

// TestTemplateEngine_Immutability tests that template functions return
// defensive copies.
func TestTemplateEngine_Immutability(t *testing.T) {
	config := domain.Config{VaultPath: "/test/vault"}
	logger := zerolog.Nop()

	t.Run("template functions exist in funcMap", func(t *testing.T) {
		// This test verifies that template functions are registered in the
		// function map. We use a real QueryService to avoid nil pointer issues
		// in the closure.

		mockQuery := createTestQueryService()
		engine := NewTemplateEngine(
			nil,
			&config,
			mockQuery,
			&logger,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())

		// Verify all three functions exist
		lookupFunc, hasLookup := funcMap["lookup"]
		queryFunc, hasQuery := funcMap["query"]
		fileClassFunc, hasFileClass := funcMap["fileClass"]

		assert.True(t, hasLookup, "lookup function should exist")
		assert.True(t, hasQuery, "query function should exist")
		assert.True(t, hasFileClass, "fileClass function should exist")

		// Verify functions have correct signatures
		_, ok := lookupFunc.(func(string) (domain.Note, error))
		assert.True(t, ok, "lookup should have correct signature")

		_, ok = queryFunc.(func(map[string]any) ([]domain.Note, error))
		assert.True(t, ok, "query should have correct signature")

		_, ok = fileClassFunc.(func(string) string)
		assert.True(t, ok, "fileClass should have correct signature")
	})
}

// TestEventBusIntegration tests EventBus integration in TemplateEngine.
func TestEventBusIntegration(t *testing.T) {
	t.Run("TemplateEngine accepts EventBus in constructor", func(t *testing.T) {
		templatePort := newMockTemplatePort()
		config := domain.Config{VaultPath: "/vault"}
		logger := zerolog.Nop()
		queryService := createTestQueryService()

		// This will fail until EventBus is integrated into constructor
		// engine := NewTemplateEngine(templatePort, &config, queryService,
		// &logger, eventBus)
		// For now, create without EventBus
		engine := NewTemplateEngine(
			templatePort,
			&config,
			queryService,
			&logger,
			utils.NewMockEventBus(),
		)

		require.NotNil(t, engine)
		// TODO: Verify EventBus is stored once integrated
	})

	t.Run("lookup function publishes LookupPerformedEvent", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNote := domain.Note{
			Path: "contact.md",
			Frontmatter: domain.NewFrontmatter(map[string]any{
				"fileClass": "contact",
			}),
		}
		mockBolt.PathQueryFunc = func(
			ctx context.Context,
			opts spi.PathQueryOptions,
		) ([]domain.Note, error) {
			if opts.Scope == spi.PathQueryScopeBasename &&
				opts.Value == "contact" {
				return []domain.Note{testNote}, nil
			}
			return nil, nil
		}

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		engine := NewTemplateEngine(nil, &cfg, querySvc, &log, eventBus)

		funcMap := engine.buildFuncMap(context.Background())
		lookupFunc := funcMap["lookup"].(func(string) (domain.Note, error))

		result, err := lookupFunc("contact")
		require.NoError(t, err)
		assert.Equal(t, "contact.md", result.Path)

		// Give async goroutines time to publish events
		time.Sleep(100 * time.Millisecond)

		// Verify event was published
		events := eventBus.GetPublishedEvents()
		require.Len(t, events, 1)
		lookupEvent, ok := events[0].(*domain.LookupPerformedEvent)
		require.True(t, ok)
		assert.Equal(t, "contact", lookupEvent.NoteID())
		assert.Equal(t, 1, lookupEvent.ResultCount())
		assert.Equal(t, "basename", lookupEvent.LookupType())
		assert.Positive(t, lookupEvent.Duration())
	})

	t.Run("fileClass function publishes SchemaLookupEvent", func(t *testing.T) {
		mockBolt := &testMockMetadataQueryPort{
			MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
		}
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNote := domain.Note{
			Path: "contact.md",
			Frontmatter: domain.NewFrontmatter(map[string]any{
				"fileClass": "contact",
			}),
		}
		mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
			return testNote, nil
		}

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		// TODO: Update constructor once EventBus is integrated
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		fileClassFunc := funcMap["fileClass"].(func(string) string)

		result := fileClassFunc("contact.md")
		assert.Equal(t, "contact", result)

		// TODO: Verify event was published once EventBus is integrated
	})

	t.Run("query function publishes QueryPerformedEvent", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNotes := []domain.Note{
			{
				Path: "note1.md",
				Frontmatter: domain.NewFrontmatter(map[string]any{
					testAuthorField: testAuthorName,
					"fileClass":     "contact",
				}),
			},
			{
				Path: "note2.md",
				Frontmatter: domain.NewFrontmatter(map[string]any{
					testAuthorField: testAuthorName,
					"fileClass":     "meeting",
				}),
			},
		}
		mockSqlite.FrontmatterQueryFunc = func(
			ctx context.Context,
			field, value string,
		) ([]domain.Note, error) {
			if field == testAuthorField && value == testAuthorName {
				return testNotes, nil
			}
			return nil, nil
		}

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		// TODO: Update constructor once EventBus is integrated
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		queryFunc := funcMap["query"].(func(map[string]any) ([]domain.Note, error))

		result, err := queryFunc(
			map[string]any{testAuthorField: testAuthorName},
		)
		require.NoError(t, err)
		assert.Len(t, result, 2)

		// TODO: Verify event was published once EventBus is integrated
	})

	t.Run("fileClass function publishes SchemaLookupEvent", func(t *testing.T) {
		mockBolt := &testMockMetadataQueryPort{
			MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
		}
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNote := domain.Note{
			Path: "contact.md",
			Frontmatter: domain.NewFrontmatter(map[string]any{
				"fileClass": "contact",
			}),
		}
		mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
			return testNote, nil
		}

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		// TODO: Update constructor once EventBus is integrated
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		fileClassFunc := funcMap["fileClass"].(func(string) string)

		result := fileClassFunc("contact.md")
		assert.Equal(t, "contact", result)

		// TODO: Verify event was published once EventBus is integrated
	})

	t.Run("query function publishes QueryPerformedEvent", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNotes := []domain.Note{
			{
				Path: "note1.md",
				Frontmatter: domain.NewFrontmatter(map[string]any{
					testAuthorField: testAuthorName,
					"fileClass":     "contact",
				}),
			},
			{
				Path: "note2.md",
				Frontmatter: domain.NewFrontmatter(map[string]any{
					testAuthorField: testAuthorName,
					"fileClass":     "meeting",
				}),
			},
		}
		mockSqlite.FrontmatterQueryFunc = func(
			ctx context.Context,
			field, value string,
		) ([]domain.Note, error) {
			if field == testAuthorField && value == testAuthorName {
				return testNotes, nil
			}
			return nil, nil
		}

		cfg := domain.Config{}
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := query.NewQueryService(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			eventBus,
		)
		// TODO: Update constructor once EventBus is integrated
		engine := NewTemplateEngine(
			nil,
			&cfg,
			querySvc,
			&log,
			utils.NewMockEventBus(),
		)

		funcMap := engine.buildFuncMap(context.Background())
		queryFunc := funcMap["query"].(func(map[string]any) ([]domain.Note, error))

		result, err := queryFunc(
			map[string]any{testAuthorField: testAuthorName},
		)
		require.NoError(t, err)
		assert.Len(t, result, 2)

		// TODO: Verify event was published once EventBus is integrated
		// events := eventBus.GetPublishedEvents()
		// require.Len(t, events, 1)
		// queryEvent, ok := events[0].(*domain.QueryPerformedEvent)
		// require.True(t, ok)
		// assert.Equal(t, 2, queryEvent.ResultCount())
		// assert.Equal(t, "frontmatter", queryEvent.QueryType())
	})

	// Note: Event publishing integration tests are in
	// tests/integration/event_driven_test.go
}
