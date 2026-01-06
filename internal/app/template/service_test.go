package template

import (
	"context"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
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
	testAuthorName   = "John"
	testAuthorField  = "author"
	testContactClass = "contact"
)

// Ensure mockTemplatePort implements TemplatePort.
var _ spi.TemplatePort = (*mockTemplatePort)(nil)

type compositeBackend struct {
	spi.CacheReaderPort
	spi.MetadataQueryPort
}

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

func (m *testMockMetadataQueryPort) List(
	ctx context.Context,
) ([]domain.Note, error) {
	return nil, nil
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

func newMockTemplatePort() *mockTemplatePort {
	return &mockTemplatePort{
		templates: make(map[domain.TemplateID]domain.Template),
	}
}

// createTestQueryService creates a QueryService with test mocks.
func createTestQueryService() *query.QueryService {
	// Use the comprehensive mocks from tests/utils
	boltPort := utils.NewMockMetadataQueryPort()
	sqlitePort := utils.NewMockMetadataQueryPort()

	bolt := &testMockMetadataQueryPort{MockMetadataQueryPort: boltPort}
	sqlite := &testMockMetadataQueryPort{MockMetadataQueryPort: sqlitePort}

	config := domain.DefaultConfig()
	logger := zerolog.Nop()
	eventBus := utils.NewMockEventBus()

	router := query.NewStorageRouter(
		compositeBackend{utils.NewMockCacheReaderPort(), bolt},
		compositeBackend{utils.NewMockCacheReaderPort(), sqlite},
	)

	return query.NewQueryService(
		router,
		config,
		logger,
		eventBus,
	)
}

func newTestQueryServiceFromPorts(
	boltPort, sqlitePort spi.MetadataQueryPort,
	config domain.Config,
	log zerolog.Logger,
	eventBus events.EventBus,
) *query.QueryService {
	var boltBackend, sqliteBackend query.QueryBackend
	if boltPort != nil {
		if reader, ok := boltPort.(spi.CacheReaderPort); ok {
			boltBackend = compositeBackend{reader, boltPort}
		} else {
			boltBackend = compositeBackend{utils.NewMockCacheReaderPort(), boltPort}
		}
	}
	if sqlitePort != nil {
		if reader, ok := sqlitePort.(spi.CacheReaderPort); ok {
			sqliteBackend = compositeBackend{reader, sqlitePort}
		} else {
			sqliteBackend = compositeBackend{utils.NewMockCacheReaderPort(), sqlitePort}
		}
	}

	router := query.NewStorageRouter(boltBackend, sqliteBackend)
	return query.NewQueryService(router, config, log, eventBus)
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
	config := domain.DefaultConfig()
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

		cfg := domain.DefaultConfig()
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := newTestQueryServiceFromPorts(
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

		cfg := domain.DefaultConfig()
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := newTestQueryServiceFromPorts(
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
	config := domain.DefaultConfig()
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

			_, err := queryFunc(map[string]any{"file_class": "contact"})
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
				"file_class": "contact",
			}),
		}
		mockSqlite.SetFrontmatterQueryResult([]domain.Note{testNote}, nil)

		cfg := domain.DefaultConfig()
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := newTestQueryServiceFromPorts(
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

		result, err := queryFunc(map[string]any{"file_class": "contact"})
		require.NoError(t, err)
		assert.Len(t, result, 1)
		assert.Equal(t, "contact.md", result[0].Path)
	})

	t.Run("query returns empty slice for no matches", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		mockSqlite.SetFrontmatterQueryResult([]domain.Note{}, nil)

		cfg := domain.DefaultConfig()
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := newTestQueryServiceFromPorts(
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

		result, err := queryFunc(map[string]any{"file_class": "nonexistent"})
		require.NoError(t, err)
		assert.Empty(t, result)
	})
}

// TestTemplateEngine_FileClassFunction tests the fileClass template function.
func TestTemplateEngine_FileClassFunction(t *testing.T) {
	config := domain.DefaultConfig()
	logger := zerolog.Nop()

	t.Run(
		"file_class function exists and has correct signature",
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
			fileClassFunc, ok := funcMap["file_class"]

			require.True(t, ok, "file_class function should be registered")
			fn, ok := fileClassFunc.(func(string) string)
			require.True(
				t,
				ok,
				"file_class should have signature func(string) string",
			)
			assert.NotNil(t, fn, "file_class function should not be nil")
		},
	)

	t.Run(
		"file_class returns empty string when QueryService unavailable",
		func(t *testing.T) {
			engine := NewTemplateEngine(
				nil,
				&config,
				nil,
				&logger,
				utils.NewMockEventBus(),
			)

			funcMap := engine.buildFuncMap(context.Background())
			fileClassFunc := funcMap["file_class"].(func(string) string)

			result := fileClassFunc("test.md")
			assert.Empty(t, result)
		},
	)

	t.Run(
		"file_class returns file_class for successful lookup",
		func(t *testing.T) {
			mockBolt := &testMockMetadataQueryPort{
				MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
			}
			mockSqlite := utils.NewMockMetadataQueryPort()

			cfg := domain.DefaultConfig()
			domain.SetInstanceForTesting(&cfg)
			defer domain.ResetConfigForTesting()

			testNote := domain.Note{
				Path: "contact.md",
				Frontmatter: domain.NewFrontmatter(map[string]any{
					"file_class": "contact",
				}),
			}
			mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
				return testNote, nil
			}

			log := zerolog.Nop()
			eventBus := utils.NewMockEventBus()

			querySvc := newTestQueryServiceFromPorts(
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
				eventBus,
			)

			funcMap := engine.buildFuncMap(context.Background())
			fileClassFunc := funcMap["file_class"].(func(string) string)

			result := fileClassFunc("contact.md")
			assert.Equal(t, "contact", result)
		},
	)

	t.Run("file_class handles various file_class values", func(t *testing.T) {
		mockBolt := &testMockMetadataQueryPort{
			MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
		}
		mockSqlite := utils.NewMockMetadataQueryPort()

		cfg := domain.DefaultConfig()
		domain.SetInstanceForTesting(&cfg)
		defer domain.ResetConfigForTesting()

		testNote := domain.Note{
			Path: "test.md",
			Frontmatter: domain.NewFrontmatter(map[string]any{
				"file_class": "custom_schema",
			}),
		}
		mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
			return testNote, nil
		}

		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := newTestQueryServiceFromPorts(
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
			eventBus,
		)

		funcMap := engine.buildFuncMap(context.Background())
		fileClassFunc := funcMap["file_class"].(func(string) string)

		result := fileClassFunc("test.md")
		assert.Equal(t, "custom_schema", result)
	})

	t.Run("file_class returns empty for note not found", func(t *testing.T) {
		mockBolt := &testMockMetadataQueryPort{
			MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
		}
		mockSqlite := utils.NewMockMetadataQueryPort()

		mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
			return domain.Note{}, lithosErr.ErrNotFound
		}

		cfg := domain.DefaultConfig()
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		querySvc := newTestQueryServiceFromPorts(
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
			eventBus,
		)

		funcMap := engine.buildFuncMap(context.Background())
		fileClassFunc := funcMap["file_class"].(func(string) string)

		result := fileClassFunc("nonexistent.md")
		assert.Empty(t, result)
	})
}

// TestEventBusIntegration tests EventBus integration in TemplateEngine.
func TestEventBusIntegration(t *testing.T) {
	t.Run("TemplateEngine accepts EventBus in constructor", func(t *testing.T) {
		templatePort := newMockTemplatePort()
		config := domain.Config{VaultPath: "/vault"}
		logger := zerolog.Nop()
		queryService := createTestQueryService()

		engine := NewTemplateEngine(
			templatePort,
			&config,
			queryService,
			&logger,
			utils.NewMockEventBus(),
		)

		require.NotNil(t, engine)
	})

	t.Run("lookup function publishes LookupPerformedEvent", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNote := domain.Note{
			Path: "contact.md",
			Frontmatter: domain.NewFrontmatter(map[string]any{
				"file_class": "contact",
			}),
		}
		mockBolt.SetPathQueryResult([]domain.Note{testNote}, nil)

		cfg := domain.DefaultConfig()
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		// Use a separate event bus for query service to avoid count mismatch
		querySvc := newTestQueryServiceFromPorts(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			utils.NewMockEventBus(),
		)
		engine := NewTemplateEngine(nil, &cfg, querySvc, &log, eventBus)

		funcMap := engine.buildFuncMap(context.Background())
		lookupFunc := funcMap["lookup"].(func(string) (domain.Note, error))

		result, err := lookupFunc("contact")
		require.NoError(t, err)
		assert.Equal(t, "contact.md", result.Path)

		// Give async goroutines time to publish events
		time.Sleep(50 * time.Millisecond)

		// Verify event was published
		publishedEvents := eventBus.GetPublishedEvents()
		require.Len(t, publishedEvents, 1)
		lookupEvent, ok := publishedEvents[0].(*events.LookupPerformedEvent)
		require.True(t, ok)
		assert.Equal(t, "contact", lookupEvent.NoteID())
		assert.Equal(t, 1, lookupEvent.ResultCount())
		assert.Equal(t, "basename", lookupEvent.LookupType())
	})

	t.Run(
		"file_class function publishes SchemaLookupEvent",
		func(t *testing.T) {
			mockBolt := &testMockMetadataQueryPort{
				MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
			}
			mockSqlite := utils.NewMockMetadataQueryPort()

			cfg := domain.DefaultConfig()
			domain.SetInstanceForTesting(&cfg)
			defer domain.ResetConfigForTesting()

			testNote := domain.Note{
				Path: "contact.md",
				Frontmatter: domain.NewFrontmatter(map[string]any{
					"file_class": "contact",
				}),
			}
			mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
				return testNote, nil
			}

			log := zerolog.Nop()
			eventBus := utils.NewMockEventBus()

			// Use a separate event bus for query service
			querySvc := newTestQueryServiceFromPorts(
				mockBolt,
				mockSqlite,
				cfg,
				log,
				utils.NewMockEventBus(),
			)
			engine := NewTemplateEngine(nil, &cfg, querySvc, &log, eventBus)

			funcMap := engine.buildFuncMap(context.Background())
			fileClassFunc := funcMap["file_class"].(func(string) string)

			result := fileClassFunc("contact.md")
			assert.Equal(t, "contact", result)

			// Give async goroutines time to publish events
			time.Sleep(50 * time.Millisecond)

			publishedEvents := eventBus.GetPublishedEvents()
			require.Len(t, publishedEvents, 1)
			schemaEvent, ok := publishedEvents[0].(*events.SchemaLookupEvent)
			require.True(t, ok)
			assert.Equal(t, "contact.md", schemaEvent.NoteID())
			assert.Equal(t, "contact", schemaEvent.SchemaName())
			assert.True(t, schemaEvent.Found())
		},
	)

	t.Run("query function publishes QueryPerformedEvent", func(t *testing.T) {
		mockBolt := utils.NewMockMetadataQueryPort()
		mockSqlite := utils.NewMockMetadataQueryPort()

		testNotes := []domain.Note{
			{
				Path: "note1.md",
				Frontmatter: domain.NewFrontmatter(map[string]any{
					testAuthorField: testAuthorName,
					"file_class":    "contact",
				}),
			},
		}
		mockSqlite.SetFrontmatterQueryResult(testNotes, nil)

		cfg := domain.DefaultConfig()
		log := zerolog.Nop()
		eventBus := utils.NewMockEventBus()

		// Use separate event bus
		querySvc := newTestQueryServiceFromPorts(
			mockBolt,
			mockSqlite,
			cfg,
			log,
			utils.NewMockEventBus(),
		)
		engine := NewTemplateEngine(nil, &cfg, querySvc, &log, eventBus)

		funcMap := engine.buildFuncMap(context.Background())
		queryFunc := funcMap["query"].(func(map[string]any) ([]domain.Note, error))

		result, err := queryFunc(
			map[string]any{testAuthorField: testAuthorName},
		)
		require.NoError(t, err)
		assert.Len(t, result, 1)

		// Give async goroutines time to publish events
		time.Sleep(50 * time.Millisecond)

		publishedEvents := eventBus.GetPublishedEvents()
		require.Len(t, publishedEvents, 1)
		queryEvent, ok := publishedEvents[0].(*events.QueryPerformedEvent)
		require.True(t, ok)
		assert.Equal(t, 1, queryEvent.ResultCount())
		assert.Equal(t, "frontmatter", queryEvent.QueryType())
	})
}
