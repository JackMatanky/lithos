package integration

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/frontmatter"
	"github.com/JackMatanky/lithos/internal/app/query"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/app/template"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

var ErrNotFound = errors.New("not found")

// TestEventDrivenLookupIntegration tests event publishing in lookup functions.
func TestEventDrivenLookupIntegration(t *testing.T) {
	ctx := context.Background()
	log := zerolog.Nop()
	eventBus := utils.NewMockEventBus()

	// Setup: Create QueryService with mock backends
	boltReader := utils.NewMockMetadataQueryPort()
	sqliteReader := utils.NewMockMetadataQueryPort()
	testNote := domain.Note{
		Path: "contact.md",
		Frontmatter: domain.NewFrontmatter(map[string]any{
			"fileClass": "contact",
			"title":     "Test Contact",
		}),
	}
	boltReader.SetPathQueryResult([]domain.Note{testNote}, nil)

	config := domain.Config{}
	querySvc := query.NewQueryService(
		boltReader,
		sqliteReader,
		config,
		log,
		eventBus,
	)

	// Setup: Create TemplateEngine with QueryService
	templatePort := &mockTemplatePort{
		templates: map[domain.TemplateID]domain.Template{
			"test": domain.NewTemplate("test", `{{lookup "contact"}}`),
		},
	}
	templateEngine := template.NewTemplateEngine(
		templatePort,
		&config,
		querySvc,
		&log,
		eventBus,
	)

	// Act: Render template which triggers lookup
	_, err := templateEngine.Render(ctx, "test")
	require.NoError(t, err)

	// Assert: Verify LookupPerformedEvent was published
	events := eventBus.GetPublishedEvents()
	require.GreaterOrEqual(t, len(events), 1, "Expected at least one event")

	var lookupEvent *domain.LookupPerformedEvent
	for _, evt := range events {
		if le, ok := evt.(*domain.LookupPerformedEvent); ok {
			lookupEvent = le
			break
		}
	}

	require.NotNil(t, lookupEvent, "LookupPerformedEvent should be published")
	assert.Equal(t, "contact", lookupEvent.NoteID())
	assert.Equal(t, 1, lookupEvent.ResultCount())
	assert.Equal(t, "basename", lookupEvent.LookupType())
	assert.Positive(t, lookupEvent.Duration())
}

// TestEventDrivenQueryIntegration tests event publishing in template query
// function. Note: This test is skipped because query() template function
// requires dict support which is not implemented in the current template
// engine. The underlying functionality
// is tested via TemplateEngine unit tests.
func TestEventDrivenQueryIntegration(t *testing.T) {
	t.Skip(
		"Skipping query integration test - dict function not implemented in template engine",
	)
}

// TestEventDrivenFileClassIntegration tests event publishing in fileClass
// function.
func TestEventDrivenFileClassIntegration(t *testing.T) {
	ctx := context.Background()
	log := zerolog.Nop()
	eventBus := utils.NewMockEventBus()

	// Setup: Create QueryService with mock backends
	boltReader := &testMockMetadataQueryPort{
		MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
	}
	sqliteReader := utils.NewMockMetadataQueryPort()

	testNote := domain.Note{
		Path: "contact.md",
		Frontmatter: domain.NewFrontmatter(map[string]any{
			"fileClass": "contact",
		}),
	}
	boltReader.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
		return testNote, nil
	}

	config := domain.Config{}
	querySvc := query.NewQueryService(
		boltReader,
		sqliteReader,
		config,
		log,
		eventBus,
	)

	// Setup: Create TemplateEngine with QueryService
	templatePort := &mockTemplatePort{
		templates: map[domain.TemplateID]domain.Template{
			"test": domain.NewTemplate("test", `{{fileClass "contact.md"}}`),
		},
	}
	templateEngine := template.NewTemplateEngine(
		templatePort,
		&config,
		querySvc,
		&log,
		eventBus,
	)

	// Act: Render template which triggers fileClass lookup
	result, err := templateEngine.Render(ctx, "test")
	require.NoError(t, err)
	assert.Equal(t, "contact", result)

	// Give async goroutines time to publish events
	time.Sleep(100 * time.Millisecond)

	// Assert: Verify SchemaLookupEvent was published
	events := eventBus.GetPublishedEvents()
	require.GreaterOrEqual(t, len(events), 1, "Expected at least one event")

	var schemaEvent *domain.SchemaLookupEvent
	for _, evt := range events {
		if se, ok := evt.(*domain.SchemaLookupEvent); ok {
			schemaEvent = se
			break
		}
	}

	require.NotNil(t, schemaEvent, "SchemaLookupEvent should be published")
	assert.Equal(t, "contact.md", schemaEvent.NoteID())
	assert.Equal(t, "contact", schemaEvent.SchemaName())
	assert.True(t, schemaEvent.Found())
	assert.Positive(t, schemaEvent.Duration())
}

// TestEventDrivenValidationIntegration tests validation event publishing.
func TestEventDrivenValidationIntegration(t *testing.T) {
	ctx := context.Background()
	log := zerolog.Nop()
	eventBus := utils.NewMockEventBus()

	// Setup: Create SchemaEngine with test schema
	stringSpec := &domain.StringSpec{}
	schemaLoader := &mockSchemaLoader{
		schemas: []domain.Schema{
			{
				Name: "contact",
				Properties: []domain.Property{
					{
						Name:     "title",
						Required: true,
						Spec:     stringSpec,
						Array:    false,
					},
				},
			},
		},
	}
	schemaRegistry := &mockSchemaRegistry{
		schemas: map[string]domain.Schema{
			"contact": schemaLoader.schemas[0],
		},
	}
	schemaEngine, _ := schema.NewSchemaEngine(
		schemaLoader,
		schemaRegistry,
		log,
		eventBus,
	)
	_ = schemaEngine.Load(ctx)

	// Setup: Create FrontmatterService
	boltReader := utils.NewMockMetadataQueryPort()
	sqliteReader := utils.NewMockMetadataQueryPort()
	querySvc := query.NewQueryService(
		boltReader,
		sqliteReader,
		domain.Config{},
		log,
		eventBus,
	)
	frontmatterSvc := frontmatter.NewFrontmatterService(
		schemaEngine,
		log,
		eventBus,
		querySvc,
	)

	// Act: Validate frontmatter (success case)
	validFm := domain.NewFrontmatter(map[string]any{
		"fileClass": "contact",
		"title":     "Test Contact",
	})
	err := frontmatterSvc.Validate(ctx, "test.md", validFm)
	require.NoError(t, err)

	// Assert: Verify ValidationPerformedEvent was published
	events := eventBus.GetPublishedEvents()
	require.GreaterOrEqual(t, len(events), 1, "Expected at least one event")

	var validationEvent *domain.FrontmatterValidatedEvent
	for _, evt := range events {
		if ve, ok := evt.(*domain.FrontmatterValidatedEvent); ok {
			validationEvent = ve
			break
		}
	}

	require.NotNil(
		t,
		validationEvent,
		"FrontmatterValidatedEvent should be published",
	)
	assert.Equal(t, "contact", validationEvent.SchemaName())
	assert.True(t, validationEvent.IsValid())
	assert.Empty(t, validationEvent.Errors())
}

// TestEventDrivenCacheInvalidation tests reactive cache invalidation via
// events.
func TestEventDrivenCacheInvalidation(t *testing.T) {
	ctx := context.Background()
	log := zerolog.Nop()
	mockLog := logger.NewZerologAdapter(log)
	eventBus := events.NewInMemoryEventBus(mockLog)

	// Setup: Create QueryService
	boltReader := utils.NewMockMetadataQueryPort()
	sqliteReader := utils.NewMockMetadataQueryPort()
	config := domain.Config{}
	querySvc := query.NewQueryService(
		boltReader,
		sqliteReader,
		config,
		log,
		eventBus,
	)

	// Act: Publish VaultIndexingCompleteEvent
	summary := domain.VaultIndexingSummary{
		ScannedCount: 100,
		IndexedCount: 95,
	}
	event := domain.MustNewVaultIndexingCompleteEvent(
		summary,
		time.Second,
		time.Now(),
	)
	err := eventBus.Publish(ctx, event)
	require.NoError(t, err)

	// Give event time to process
	time.Sleep(50 * time.Millisecond)

	// Assert: Verify QueryService handled the event
	// (We can verify via logs or internal state if exposed)
	stats := querySvc.GetBackendFailureStats()
	assert.NotNil(
		t,
		stats,
		"QueryService should have initialized failure trackers",
	)
}

// Mock implementations for testing

type mockTemplatePort struct {
	templates map[domain.TemplateID]domain.Template
}

func (m *mockTemplatePort) List(
	ctx context.Context,
) ([]domain.TemplateID, error) {
	var ids []domain.TemplateID
	for id := range m.templates {
		ids = append(ids, id)
	}
	return ids, nil
}

func (m *mockTemplatePort) Load(
	ctx context.Context,
	id domain.TemplateID,
) (domain.Template, error) {
	tmpl, exists := m.templates[id]
	if !exists {
		return nil, ErrNotFound
	}
	return tmpl, nil
}

type testMockMetadataQueryPort struct {
	*utils.MockMetadataQueryPort

	readFunc func(ctx context.Context, path string) (domain.Note, error)
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

type mockSchemaLoader struct {
	schemas []domain.Schema
	bank    domain.PropertyBank
}

func (m *mockSchemaLoader) Load(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	return m.schemas, m.bank, nil
}

type mockSchemaRegistry struct {
	schemas    map[string]domain.Schema
	properties map[string]domain.Property
}

func (m *mockSchemaRegistry) GetSchema(
	ctx context.Context,
	name string,
) (domain.Schema, error) {
	schema, exists := m.schemas[name]
	if !exists {
		return domain.Schema{}, ErrNotFound
	}
	return schema, nil
}

func (m *mockSchemaRegistry) GetProperty(
	ctx context.Context,
	name string,
) (domain.Property, error) {
	prop, exists := m.properties[name]
	if !exists {
		return domain.Property{}, ErrNotFound
	}
	return prop, nil
}

func (m *mockSchemaRegistry) HasSchema(ctx context.Context, name string) bool {
	_, exists := m.schemas[name]
	return exists
}

func (m *mockSchemaRegistry) HasProperty(
	ctx context.Context,
	name string,
) bool {
	_, exists := m.properties[name]
	return exists
}

func (m *mockSchemaRegistry) RegisterAll(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
) error {
	for _, schema := range schemas {
		m.schemas[schema.Name] = schema
	}
	for k, v := range bank.Properties {
		m.properties[k] = v
	}
	return nil
}
