// Package template provides the TemplateEngine domain service for template
// rendering. It orchestrates template loading, parsing, and execution with
// custom functions for the lithos new and find commands.
package template

import (
	"context"
	"fmt"
	"hash/fnv"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"text/template"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/query"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// TemplateEngine provides template rendering capabilities with custom
// functions.
// It loads templates from the TemplatePort, parses them with Go text/template,
// and executes them with domain-specific functions for file path control and
// timestamp formatting.
//
// TemplateEngine follows hexagonal architecture principles:
// - Depends on TemplatePort (SPI) for template loading
// - Accepts Config for vault path access
// - Uses QueryService for schema-aware lookups
// - Uses EventBus for publishing lookup events
// - Uses zerolog for structured logging
// - Returns domain errors (ResourceError, TemplateError) for clean error
// handling.
type TemplateEngine struct {
	templatePort spi.TemplatePort
	config       *domain.Config
	queryService *query.QueryService
	eventBus     events.EventBus
	log          *zerolog.Logger
	mu           sync.RWMutex
	funcMap      template.FuncMap
	compiled     map[domain.TemplateID]cachedTemplate
}

type cachedTemplate struct {
	tpl      *template.Template
	checksum uint64
}

// NewTemplateEngine creates a new TemplateEngine with injected dependencies.
// The TemplateEngine is ready to load and render templates immediately after
// construction. Dependencies are injected following dependency inversion
// principles.
//
// Parameters:
//   - templatePort: SPI adapter for loading templates from storage
//   - config: Application configuration containing vault path and settings
//   - queryService: Query service for schema-aware lookups (optional)
//   - log: Structured logger for operation tracing and debugging
//   - eventBus: Event bus for publishing lookup events
//
// Returns a pointer to the initialized TemplateEngine.
func NewTemplateEngine(
	templatePort spi.TemplatePort,
	config *domain.Config,
	queryService *query.QueryService,
	log *zerolog.Logger,
	eventBus events.EventBus,
) *TemplateEngine {
	return &TemplateEngine{
		templatePort: templatePort,
		config:       config,
		queryService: queryService,
		eventBus:     eventBus,
		log:          log,
		mu:           sync.RWMutex{},
		funcMap:      nil,
		compiled:     make(map[domain.TemplateID]cachedTemplate),
	}
}

// Render loads, parses, and executes a template with custom functions.
// This is the main public method for template rendering in Lithos.
//
// The method follows a 6-step workflow:
// 1. Load template via Load(ctx, templateID)
// 2. Create text/template instance with template.ID as name
// 3. Register function map via buildFuncMap()
// 4. Parse template.Content using template.Parse()
// 5. Execute template with empty data context (static rendering for Epic 1)
// 6. Return rendered string
//
// Parameters:
//   - ctx: Context for cancellation and timeout control
//   - templateID: The identifier of the template to render
//
// Returns:
//   - string: The rendered template content
//
// - error: ResourceError if template not found, TemplateError for parse/execute
// issues.
func (e *TemplateEngine) Render(
	ctx context.Context,
	templateID domain.TemplateID,
) (string, error) {
	// Step 1: Load template
	tmpl, err := e.Load(ctx, templateID)
	if err != nil {
		return "", err // ResourceError from Load()
	}

	// Step 2-3: Create text/template with function map
	t, err := e.getCompiledTemplate(ctx, tmpl)
	if err != nil {
		return "", err
	}

	// Step 5-6: Execute with empty data context and return
	var buf strings.Builder
	if executeErr := t.Execute(&buf, nil); executeErr != nil {
		return "", errors.NewTemplateError(
			fmt.Sprintf("execute error in template '%s'", tmpl.ID()),
			string(tmpl.ID()),
			executeErr,
		)
	}

	e.log.Info().
		Str("templateID", string(templateID)).
		Msg("template rendered successfully")
	return buf.String(), nil
}

// Load retrieves a template by its ID from the TemplatePort.
// This method delegates to the injected TemplatePort adapter, providing
// a clean domain service interface while maintaining hexagonal architecture
// separation of concerns.
//
// The method logs the loading operation at debug level for observability
// and returns the template with its content ready for rendering.
//
// Parameters:
//   - ctx: Context for cancellation and timeout control
//   - templateID: The identifier of the template to load
//
// Returns:
//   - Template: The loaded template with ID and content
//   - error: ResourceError if template not found or loading fails
func (e *TemplateEngine) Load(
	ctx context.Context,
	templateID domain.TemplateID,
) (domain.Template, error) {
	e.log.Debug().Str("templateID", string(templateID)).Msg("loading template")
	return e.templatePort.Load(ctx, templateID)
}

// buildFuncMap creates and returns a template.FuncMap containing all custom
// template functions for Lithos. This includes basic string manipulation
// functions and file path control functions.
//
// The function map is registered with Go's text/template engine to enable
// domain-specific functionality in templates. Functions are organized into
// logical categories for maintainability.
//
// Parameters:
//   - ctx: Context for query operations, enabling timeout and cancellation
//
// Returns a template.FuncMap ready for use with template.Funcs().
func (e *TemplateEngine) buildFuncMap(ctx context.Context) template.FuncMap {
	e.mu.RLock()
	if e.funcMap != nil {
		defer e.mu.RUnlock()
		return e.funcMap
	}
	e.mu.RUnlock()

	e.mu.Lock()
	defer e.mu.Unlock()
	if e.funcMap != nil {
		return e.funcMap
	}

	funcMap := template.FuncMap{
		// String manipulation functions
		"title":   strings.Title, //nolint:staticcheck // TODO: migrate to golang.org/x/text/cases
		"toLower": strings.ToLower,
		"toUpper": strings.ToUpper,
		"dict":    buildDict,

		// Date/time functions
		"now": func(format string) string {
			return time.Now().Format(format)
		},

		// File path control functions
		"path":   func() string { return "" }, // Empty for Epic 1
		"folder": filepath.Dir,
		"basename": func(p string) string {
			base := filepath.Base(p)
			return strings.TrimSuffix(base, filepath.Ext(base))
		},
		"extension": filepath.Ext,
		"join":      filepath.Join,
		"vaultPath": func() string { return e.config.VaultPath },

		// Schema-aware lookup functions (requires queryService)
		"lookup":      e.makeLookupFunc(ctx),
		"query":       e.makeQueryFunc(ctx),
		"fileClass":   e.makeFileClassFunc(ctx),
		"sortByTitle": sortNotesByTitle,
	}

	e.funcMap = funcMap
	return e.funcMap
}

// makeLookupFunc returns a closure over QueryService for looking up notes by
// basename.
// The closure delegates to QueryService.PathQuery with basename scope.
// Returns a single note or error for not found/ambiguous cases.
// Publishes LookupPerformedEvent for observability.
//
// Error cases:
//   - "not found" when basename matches no notes
//
// - "ambiguous basename X: found N matches" when basename matches multiple
// notes
//
// Thread-safe: Closure captures QueryService pointer which is thread-safe for
// reads.
func (e *TemplateEngine) makeLookupFunc(
	ctx context.Context,
) func(string) (domain.Note, error) {
	return func(basename string) (domain.Note, error) {
		start := time.Now()

		if e.queryService == nil {
			return domain.Note{}, fmt.Errorf(
				"lookup failed: query service not available",
			)
		}

		// Query by basename
		notes, err := e.queryService.PathQuery(ctx, spi.PathQueryOptions{
			Scope: spi.PathQueryScopeBasename,
			Value: basename,
		})
		if err != nil {
			// Publish event for failed lookup
			duration := time.Since(start)
			event := domain.MustNewLookupPerformedEvent(
				basename,
				0,
				duration,
				"basename",
				time.Now(),
			)
			go func() {
				if pubErr := e.eventBus.Publish(ctx, event); pubErr != nil {
					e.log.Error().
						Err(pubErr).
						Msg("failed to publish LookupPerformedEvent")
				}
			}()
			return domain.Note{}, fmt.Errorf("lookup failed: %w", err)
		}

		// Publish event for successful lookup
		duration := time.Since(start)
		event := domain.MustNewLookupPerformedEvent(
			basename,
			len(notes),
			duration,
			"basename",
			time.Now(),
		)
		go func() {
			if pubErr := e.eventBus.Publish(ctx, event); pubErr != nil {
				e.log.Error().
					Err(pubErr).
					Msg("failed to publish LookupPerformedEvent")
			}
		}()

		// Handle result cases
		if len(notes) == 0 {
			return domain.Note{}, fmt.Errorf("note not found: %s", basename)
		}
		if len(notes) > 1 {
			return domain.Note{}, fmt.Errorf(
				"ambiguous basename %s: found %d matches",
				basename,
				len(notes),
			)
		}

		// Return defensive copy of the single matching note
		return notes[0].Clone(), nil
	}
}

// makeQueryFunc returns a closure over QueryService for querying notes by
// frontmatter fields. The closure delegates to QueryService.FrontmatterQuery
// with field/value pairs.
// Returns a slice of notes (empty slice if no matches).
// Publishes QueryPerformedEvent for observability.
// Supports type-agnostic comparison (int 2 == float 2.0) and delegates to
// MetadataQueryPort for indexed lookups.
//
// Query Semantics:
// - Returns empty slice (not error) for non-matching frontmatter (collection
// lookup)
// - Type normalization: int/float conversion for numeric comparison
// - Supports multiple filters with AND logic (all filters must match)
// - Logs debug message with field and value for troubleshooting
// - Routes directly to SQLite for complex frontmatter queries (deep path only)
//
// Example:
//
//	notes := queryService.FrontmatterQuery("author", "John Doe")
//	notes := queryService.FrontmatterQuery("tags", "project-x")
//	notes := queryService.FrontmatterQuery("status", "draft")
//	notes := queryService.FrontmatterQuery("priority", 2) // matches float 2.0
func (e *TemplateEngine) makeQueryFunc(
	ctx context.Context,
) func(map[string]any) ([]domain.Note, error) {
	return func(filter map[string]any) ([]domain.Note, error) {
		start := time.Now()

		if e.queryService == nil {
			e.publishQueryPerformedEvent(ctx, filter, 0, time.Since(start))
			return nil, fmt.Errorf("query failed: query service not available")
		}

		// Process all filters in the map (AND logic - all filters must match)
		if len(filter) == 0 {
			e.publishQueryPerformedEvent(ctx, filter, 0, time.Since(start))
			return []domain.Note{}, nil
		}

		// Execute query with filters
		result, err := e.executeFilteredQuery(ctx, filter)
		if err != nil {
			e.publishQueryPerformedEvent(
				ctx,
				filter,
				len(result),
				time.Since(start),
			)
			return nil, err
		}

		// Publish event for successful query
		e.publishQueryPerformedEvent(
			ctx,
			filter,
			len(result),
			time.Since(start),
		)

		// Return defensive copies
		clonedNotes := make([]domain.Note, len(result))
		for i := range result {
			clonedNotes[i] = result[i].Clone()
		}
		return clonedNotes, nil
	}
}

// executeFilteredQuery executes frontmatter queries with AND logic for multiple
// filters.
// Extracted from makeQueryFunc to reduce cognitive complexity.
func (e *TemplateEngine) executeFilteredQuery(
	ctx context.Context,
	filter map[string]any,
) ([]domain.Note, error) {
	var result []domain.Note
	first := true

	for field, value := range filter {
		notes, err := e.queryService.FrontmatterQuery(ctx, field, value)
		if err != nil {
			return nil, fmt.Errorf("query failed: %w", err)
		}

		if first {
			result = notes
			first = false
		} else {
			// Intersect results (AND logic)
			result = intersectNotes(result, notes)
		}
	}

	return result, nil
}

// intersectNotes returns notes that appear in both slices (intersection).
func intersectNotes(a, b []domain.Note) []domain.Note {
	noteMap := make(map[string]domain.Note, len(a))
	for i := range a {
		noteMap[a[i].Path] = a[i]
	}

	result := make(
		[]domain.Note,
		0,
		len(a),
	) // Pre-allocate with reasonable capacity
	for i := range b {
		if existing, found := noteMap[b[i].Path]; found {
			result = append(result, existing)
		}
	}

	return result
}

func sortNotesByTitle(notes []domain.Note) []domain.Note {
	sorted := make([]domain.Note, len(notes))
	copy(sorted, notes)
	sort.SliceStable(sorted, func(i, j int) bool {
		titleI := strings.ToLower(sorted[i].Title())
		titleJ := strings.ToLower(sorted[j].Title())
		if titleI == titleJ {
			return sorted[i].Path < sorted[j].Path
		}
		return titleI < titleJ
	})
	return sorted
}

func buildDict(values ...any) (map[string]any, error) {
	if len(values)%2 != 0 {
		return nil, fmt.Errorf("dict requires an even number of arguments")
	}
	dict := make(map[string]any, len(values)/2)
	for i := 0; i < len(values); i += 2 {
		key, ok := values[i].(string)
		if !ok {
			return nil, fmt.Errorf("dict keys must be strings (arg %d)", i)
		}
		dict[key] = values[i+1]
	}
	return dict, nil
}

// makeFileClassFunc returns a closure over QueryService for extracting
// fileClass from a note.
// The closure delegates to QueryService.IDQuery then extracts fileClass field.
// Returns fileClass string or empty string for missing note/field.
// Publishes SchemaLookupEvent for observability.
// Handles errors gracefully without crashing templates.
//
// Thread-safe: Closure captures QueryService pointer which is thread-safe for
// reads.
func (e *TemplateEngine) makeFileClassFunc(
	ctx context.Context,
) func(string) string {
	return func(noteID string) string {
		start := time.Now()

		if e.queryService == nil {
			e.publishSchemaLookupEvent(
				ctx,
				noteID,
				"",
				false,
				time.Since(start),
			)
			e.log.Error().
				Msg("fileClass lookup failed: query service not available")
			return ""
		}

		// Query note by ID (path)
		note, err := e.queryService.IDQuery(ctx, noteID)
		if err != nil {
			e.publishSchemaLookupEvent(
				ctx,
				noteID,
				"",
				false,
				time.Since(start),
			)
			e.log.Debug().
				Str("noteID", noteID).
				Err(err).
				Msg("fileClass lookup failed: note not found")
			return ""
		}

		// Extract fileClass from frontmatter
		fileClass := note.FileClass()
		found := fileClass != ""

		// Publish event for lookup result
		e.publishSchemaLookupEvent(
			ctx,
			noteID,
			fileClass,
			found,
			time.Since(start),
		)

		if !found {
			e.log.Debug().
				Str("noteID", noteID).
				Msg("fileClass field missing from note")
			return ""
		}
		return fileClass
	}
}

func (e *TemplateEngine) getFuncMap(ctx context.Context) template.FuncMap {
	return e.buildFuncMap(ctx)
}

func (e *TemplateEngine) getCompiledTemplate(
	ctx context.Context,
	tmpl domain.Template,
) (*template.Template, error) {
	checksum := checksumString(tmpl.Content())

	e.mu.RLock()
	if cached, ok := e.compiled[tmpl.ID()]; ok && cached.checksum == checksum {
		defer e.mu.RUnlock()
		return cached.tpl, nil
	}
	e.mu.RUnlock()

	parsed, err := template.New(string(tmpl.ID())).
		Funcs(e.getFuncMap(ctx)).
		Parse(tmpl.Content())
	if err != nil {
		return nil, errors.NewTemplateError(
			fmt.Sprintf("parse error in template '%s'", tmpl.ID()),
			string(tmpl.ID()),
			err,
		)
	}

	e.mu.Lock()
	e.compiled[tmpl.ID()] = cachedTemplate{
		tpl:      parsed,
		checksum: checksum,
	}
	e.mu.Unlock()

	return parsed, nil
}

func checksumString(s string) uint64 {
	hasher := fnv.New64a()
	_, err := hasher.Write([]byte(s))
	if err != nil {
		// This should never happen with fnv hash, but we handle it to satisfy
		// linter
		panic(fmt.Sprintf("unexpected error writing to fnv hasher: %v", err))
	}
	return hasher.Sum64()
}

// publishSchemaLookupEvent is a helper to publish SchemaLookupEvent
// asynchronously. Reduces code duplication in makeFileClassFunc.
func (e *TemplateEngine) publishSchemaLookupEvent(
	ctx context.Context,
	noteID string,
	fileClass string,
	found bool,
	duration time.Duration,
) {
	if e.eventBus == nil {
		return
	}
	event := domain.MustNewSchemaLookupEvent(
		noteID,
		fileClass,
		found,
		duration,
		time.Now(),
	)
	go func() {
		if publishErr := e.eventBus.Publish(ctx, event); publishErr != nil {
			e.log.Error().
				Err(publishErr).
				Msg("failed to publish SchemaLookupEvent")
		}
	}()
}

// publishQueryPerformedEvent is a helper to publish QueryPerformedEvent
// asynchronously. Reduces code duplication in makeQueryFunc.
func (e *TemplateEngine) publishQueryPerformedEvent(
	ctx context.Context,
	filter map[string]any,
	resultCount int,
	duration time.Duration,
) {
	if e.eventBus == nil {
		return
	}
	event := domain.MustNewQueryPerformedEvent(
		filter,
		resultCount,
		duration,
		"frontmatter",
		time.Now(),
	)
	go func() {
		if publishErr := e.eventBus.Publish(ctx, event); publishErr != nil {
			e.log.Error().
				Err(publishErr).
				Msg("failed to publish QueryPerformedEvent")
		}
	}()
}
