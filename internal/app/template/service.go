// Package template provides template rendering and function registration.
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
	"github.com/JackMatanky/lithos/internal/shared/converters"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"golang.org/x/text/cases"
	"golang.org/x/text/language"
)

// TemplateEngine handles template compilation and execution with a central
// function registry.
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
func (e *TemplateEngine) Render(
	ctx context.Context,
	templateID domain.TemplateID,
) (string, error) {
	// Step 1: Load template
	tmpl, err := e.templatePort.Load(ctx, templateID)
	if err != nil {
		return "", err
	}

	// Step 2-3: Create text/template with function map
	t, err := e.getCompiledTemplate(ctx, tmpl)
	if err != nil {
		return "", err
	}

	// Step 5-6: Execute with empty data context
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

// buildFuncMap registers all core template functions.
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
		"title":   cases.Title(language.Und).String,
		"toLower": strings.ToLower,
		"toUpper": strings.ToUpper,
		"dict":    buildDict,

		// Date/time functions
		"now": func(format string) string {
			return time.Now().Format(format)
		},

		// File path control functions
		"path":   func() string { return "" },
		"folder": filepath.Dir,
		"basename": func(p string) string {
			base := filepath.Base(p)
			return strings.TrimSuffix(base, filepath.Ext(base))
		},
		"extension": filepath.Ext,
		"join":      filepath.Join,
		"vaultPath": func() string { return e.config.VaultPath },

		// Schema-aware lookup functions
		"lookup":      e.makeLookupFunc(ctx),
		"query":       e.makeQueryFunc(ctx),
		"fileClass":   e.makeFileClassFunc(ctx),
		"file_class":  e.makeFileClassFunc(ctx),
		"sortByTitle": sortNotesByTitle,
	}

	e.funcMap = funcMap
	return e.funcMap
}

func (e *TemplateEngine) getFuncMap(ctx context.Context) template.FuncMap {
	return e.buildFuncMap(ctx)
}

func (e *TemplateEngine) getCompiledTemplate(
	ctx context.Context,
	tmpl domain.Template,
) (*template.Template, error) {
	checksum := calculateChecksum(tmpl.Content())

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

// makeLookupFunc returns a closure over QueryService for looking up single
// notes.
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
			publishLookup(
				ctx,
				e.eventBus,
				*e.log,
				basename,
				0,
				duration,
				"basename",
			)
			return domain.Note{}, fmt.Errorf("lookup failed: %w", err)
		}

		// Publish event for successful lookup
		duration := time.Since(start)
		publishLookup(
			ctx,
			e.eventBus,
			*e.log,
			basename,
			len(notes),
			duration,
			"basename",
		)

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
// frontmatter fields.
func (e *TemplateEngine) makeQueryFunc(
	ctx context.Context,
) func(map[string]any) ([]domain.Note, error) {
	return func(filter map[string]any) ([]domain.Note, error) {
		start := time.Now()

		if e.queryService == nil {
			publishQuery(ctx, e.eventBus, *e.log, filter, 0, time.Since(start))
			return nil, fmt.Errorf("query failed: query service not available")
		}

		// Process all filters in the map (AND logic - all filters must match)
		if len(filter) == 0 {
			publishQuery(ctx, e.eventBus, *e.log, filter, 0, time.Since(start))
			return []domain.Note{}, nil
		}

		// Execute query with filters
		result, err := e.executeFilteredQuery(ctx, filter)
		if err != nil {
			publishQuery(
				ctx,
				e.eventBus,
				*e.log,
				filter,
				len(result),
				time.Since(start),
			)
			return nil, err
		}

		publishQuery(
			ctx,
			e.eventBus,
			*e.log,
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
func (e *TemplateEngine) executeFilteredQuery(
	ctx context.Context,
	filter map[string]any,
) ([]domain.Note, error) {
	var result []domain.Note
	first := true

	for field, value := range filter {
		canonicalValue, ok := converters.Canonicalize(value)
		if !ok {
			return nil, fmt.Errorf(
				"query failed: cannot canonicalize value for field %s",
				field,
			)
		}

		notes, err := e.queryService.FrontmatterQuery(
			ctx,
			field,
			fmt.Sprintf("%v", canonicalValue),
		)
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

// makeFileClassFunc returns a closure over FrontmatterService for extracting
// fileClass from a note.
func (e *TemplateEngine) makeFileClassFunc(
	ctx context.Context,
) func(string) string {
	return func(noteID string) string {
		start := time.Now()

		if e.queryService == nil {
			publishSchemaLookup(
				ctx,
				e.eventBus,
				*e.log,
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
			publishSchemaLookup(
				ctx,
				e.eventBus,
				*e.log,
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
		publishSchemaLookup(
			ctx,
			e.eventBus,
			*e.log,
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
	)
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

// calculateChecksum produces a 64-bit FNV-1a hash of the content string.
func calculateChecksum(content string) uint64 {
	hasher := fnv.New64a()
	_, err := hasher.Write([]byte(content))
	if err != nil {
		panic(fmt.Sprintf("unexpected error writing to fnv hasher: %v", err))
	}
	return hasher.Sum64()
}
