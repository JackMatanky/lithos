package domain

import (
	"bytes"
	"path/filepath"
	"strings"
	"text/template"
	"time"
)

// Template represents an executable template for note generation.
// It provides domain abstraction over template execution, enabling testability
// and proper hexagonal architecture separation.
type Template interface {
	// ID returns the template identifier for composition and lookup
	ID() TemplateID
	// Content returns the raw template content for parsing
	Content() string
	// Execute renders the template with the provided data
	Execute(data any) (string, error)
}

// templateData represents the raw template data loaded from storage.
// It contains the template content and implements the Template interface.
type templateData struct {
	// id is the template name used for identification and composition.
	// It enables Go's text/template {{template "name"}} directive
	// functionality.
	id TemplateID
	// content contains the raw template text with Go text/template syntax and
	// Lithos functions. This includes directives like {{prompt}}, {{now}}, and
	// {{template}} references.
	content string
	// parsed caches the parsed template to avoid re-parsing on each execution
	parsed *template.Template
}

// TemplateID represents a template name used for identification and
// composition.
// It wraps a string to provide type safety and domain-specific semantics.
// TemplateID is meaningful in the domain (unlike NoteID which is opaque)
// because Go's text/template requires names for {{template "name"}} references.
type TemplateID string

// ID returns the template identifier.
func (t *templateData) ID() TemplateID {
	return t.id
}

// Content returns the raw template content.
func (t *templateData) Content() string {
	return t.content
}

// Execute renders the template with the provided data.
func (t *templateData) Execute(data any) (string, error) {
	// Parse template if not already cached
	if t.parsed == nil {
		// Include basic function map for template execution (subset of
		// TemplateEngine functions)
		funcMap := template.FuncMap{
			"now": func(format string) string {
				return time.Now().Format(format)
			},
			"toLower": strings.ToLower,
			"toUpper": strings.ToUpper,
			"folder":  filepath.Dir,
			"basename": func(p string) string {
				return strings.TrimSuffix(
					filepath.Base(p),
					filepath.Ext(filepath.Base(p)),
				)
			},
			"extension": filepath.Ext,
			"join":      filepath.Join,
		}
		tmpl, err := template.New(string(t.id)).Funcs(funcMap).Parse(t.content)
		if err != nil {
			return "", err
		}
		t.parsed = tmpl
	}

	var buf bytes.Buffer
	if execErr := t.parsed.Execute(&buf, data); execErr != nil {
		return "", execErr
	}
	return buf.String(), nil
}

// NewTemplateID creates a new TemplateID from a string value.
// The value typically represents a template basename (e.g., "contact-header")
// derived from filesystem paths by removing directory and extension.
func NewTemplateID(value string) TemplateID {
	return TemplateID(value)
}

// String returns the string representation of the TemplateID.
// This implements the standard Go Stringer interface for logging and debugging.
func (id TemplateID) String() string {
	return string(id)
}

// NewTemplate creates a new Template with the given ID and content.
// The content should contain valid Go text/template syntax.
// This constructor creates an executable template that implements the Template
// interface.
func NewTemplate(id TemplateID, content string) Template {
	return &templateData{
		id:      id,
		content: content,
		parsed:  nil,
	}
}
