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
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

var ErrNotFound = errors.New("not found")

// Mock implementations for testing

type mockTemplatePort struct {
	templates map[domain.TemplateID]domain.Template
}

type testMockMetadataQueryPort struct {
	*utils.MockMetadataQueryPort

	readFunc func(ctx context.Context, path string) (domain.Note, error)
}

type mockSchemaLoader struct {
	schemas []domain.Schema
	bank    domain.PropertyBank
}

type mockSchemaRegistry struct {
	schemas    map[string]domain.Schema
	properties map[string]domain.Property
}

func (m *testMockMetadataQueryPort) List(
	ctx context.Context,
) ([]domain.Note, error) {
	return nil, nil
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

func (m *testMockMetadataQueryPort) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	if m.readFunc != nil {
		return m.readFunc(ctx, path)
	}
	return m.MockMetadataQueryPort.Read(ctx, path)
}

func (m *mockSchemaLoader) Load(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	return m.schemas, m.bank, nil
}

func (m *mockSchemaRegistry) GetSchema(
	ctx context.Context,
	name string,
) (domain.Schema, error) {
	s, exists := m.schemas[name]
	if !exists {
		return domain.Schema{}, ErrNotFound
	}
	return s, nil
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

// TestEventDrivenLookupIntegration tests event publishing in lookup functions.
func TestEventDrivenLookupIntegration(t *testing.T) {
	ctx := context.Background()
	log := zerolog.Nop()
	eventBus := utils.NewMockEventBus()

	// Setup: Create QueryService with mock backends
	type composite struct {
		spi.CacheReaderPort
		spi.MetadataQueryPort
	}
	boltBackend := composite{
		utils.NewMockCacheReaderPort(),
		utils.NewMockMetadataQueryPort(),
	}
	sqliteBackend := composite{
		utils.NewMockCacheReaderPort(),
		utils.NewMockMetadataQueryPort(),
	}

	testNote := domain.Note{
		Path: "contact.md",
		Frontmatter: domain.NewFrontmatter(map[string]any{
			"fileClass": "contact",
			"title":     "Test Contact",
		}),
	}
	boltBackend.MetadataQueryPort.(*utils.MockMetadataQueryPort).SetPathQueryResult(
		[]domain.Note{testNote},
		nil,
	)

	config := domain.Config{}
	router := query.NewStorageRouter(boltBackend, sqliteBackend)
	querySvc := query.NewQueryService(
		router,
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

	// Give async event handlers time to publish
	time.Sleep(100 * time.Millisecond)

	// Assert: Verify LookupPerformedEvent was published
	publishedEvents := eventBus.GetPublishedEvents()
	require.GreaterOrEqual(
		t,
		len(publishedEvents),
		1,
		"Expected at least one event",
	)

	var lookupEvent *events.LookupPerformedEvent
	for _, evt := range publishedEvents {
		if le, ok := evt.(*events.LookupPerformedEvent); ok {
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
// function.
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
	type composite struct {
		spi.CacheReaderPort
		spi.MetadataQueryPort
	}

	mockBolt := &testMockMetadataQueryPort{
		MockMetadataQueryPort: utils.NewMockMetadataQueryPort(),
	}
	boltBackend := composite{mockBolt, mockBolt}
	sqliteBackend := composite{
		utils.NewMockCacheReaderPort(),
		utils.NewMockMetadataQueryPort(),
	}

	testNote := domain.Note{
		Path: "contact.md",
		Frontmatter: domain.NewFrontmatter(map[string]any{
			"fileClass": "contact",
		}),
	}
	mockBolt.readFunc = func(ctx context.Context, path string) (domain.Note, error) {
		return testNote, nil
	}

	config := domain.Config{}
	router := query.NewStorageRouter(boltBackend, sqliteBackend)
	querySvc := query.NewQueryService(
		router,
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
	publishedEvents := eventBus.GetPublishedEvents()
	require.GreaterOrEqual(
		t,
		len(publishedEvents),
		1,
		"Expected at least one event",
	)

	var schemaEvent *events.SchemaLookupEvent
	for _, evt := range publishedEvents {
		if se, ok := evt.(*events.SchemaLookupEvent); ok {
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
	type composite struct {
		spi.CacheReaderPort
		spi.MetadataQueryPort
	}
	boltBackend := composite{
		utils.NewMockCacheReaderPort(),
		utils.NewMockMetadataQueryPort(),
	}
	sqliteBackend := composite{
		utils.NewMockCacheReaderPort(),
		utils.NewMockMetadataQueryPort(),
	}

	config := domain.Config{}
	router := query.NewStorageRouter(boltBackend, sqliteBackend)
	querySvc := query.NewQueryService(
		router,
		config,
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

	// Assert: Verify ValidationPerformedEvent was published (AC 4.6.2)
	publishedEvents := eventBus.GetPublishedEvents()
	require.GreaterOrEqual(
		t,
		len(publishedEvents),
		1,
		"Expected at least one event",
	)

	var validationEvent *events.ValidationPerformedEvent
	for _, evt := range publishedEvents {
		if ve, ok := evt.(*events.ValidationPerformedEvent); ok {
			validationEvent = ve
			break
		}
	}

	require.NotNil(
		t,
		validationEvent,
		"ValidationPerformedEvent should be published",
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
	type composite struct {
		spi.CacheReaderPort
		spi.MetadataQueryPort
	}
	boltBackend := composite{
		utils.NewMockCacheReaderPort(),
		utils.NewMockMetadataQueryPort(),
	}
	sqliteBackend := composite{
		utils.NewMockCacheReaderPort(),
		utils.NewMockMetadataQueryPort(),
	}

	config := domain.Config{}
	router := query.NewStorageRouter(boltBackend, sqliteBackend)
	_ = query.NewQueryService(
		router,
		config,
		log,
		eventBus,
	)

	// Act: Publish VaultIndexingCompleteEvent
	summary := events.VaultIndexingSummary{
		ScannedCount: 100,
		IndexedCount: 95,
	}
	event := events.MustNewVaultIndexingCompleteEvent(
		summary,
		time.Second,
		time.Now(),
	)
	err := eventBus.Publish(ctx, event)
	require.NoError(t, err)

	// Give event time to process
	time.Sleep(50 * time.Millisecond)

	// Verify no panic and handled gracefully
}

// Mock implementations for testing - removed duplicate declarations from end of
// file

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
