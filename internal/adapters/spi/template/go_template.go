package template

import (
	"bytes"
	"path/filepath"
	"strings"
	"text/template"

	"github.com/JackMatanky/lithos/internal/domain"
)

// GoTemplateAdapter creates and manages Go template instances with shared
// FuncMap.
type GoTemplateAdapter struct {
	parsedSet *template.Template
	funcMap   template.FuncMap
}

// GoTemplate wraps *template.Template to implement domain.Template.
type GoTemplate struct {
	id      domain.TemplateID
	tmpl    *template.Template
	content string
}

// TemplateNotFoundError indicates a template was not found.
type TemplateNotFoundError struct {
	ID domain.TemplateID
}

// NewGoTemplateAdapter creates a new adapter with shared function map.
func NewGoTemplateAdapter(
	templateDir string,
	funcMap template.FuncMap,
) (*GoTemplateAdapter, error) {
	adapter := &GoTemplateAdapter{
		parsedSet: nil,
		funcMap:   funcMap,
	}

	// Parse all templates with shared function map
	baseTmpl := template.New("base").Funcs(funcMap)
	pattern := filepath.Join(templateDir, "*.tmpl")
	tmpl, err := baseTmpl.ParseGlob(pattern)
	if err != nil {
		// If no files match, that's OK - just return empty adapter
		if strings.Contains(err.Error(), "matches no files") {
			return adapter, nil
		}
		return nil, err
	}

	// Store the parsed template set
	adapter.parsedSet = tmpl

	return adapter, nil
}

// GetTemplate returns a GoTemplate wrapper for the given ID.
func (g *GoTemplateAdapter) GetTemplate(
	id domain.TemplateID,
) (domain.Template, error) {
	if g.parsedSet == nil {
		return nil, &TemplateNotFoundError{ID: id}
	}

	// Check if the template exists in the set
	tmpl := g.parsedSet.Lookup(string(id) + ".tmpl")
	if tmpl == nil {
		return nil, &TemplateNotFoundError{ID: id}
	}

	return &GoTemplate{
		id:      id,
		tmpl:    g.parsedSet, // Use the full set for template references
		content: "",          // Content not stored in this adapter
	}, nil
}

// ID returns the template identifier.
func (g *GoTemplate) ID() domain.TemplateID {
	return g.id
}

// Content returns the raw template content.
func (g *GoTemplate) Content() string {
	return g.content
}

// Execute renders the template with the provided data.
func (g *GoTemplate) Execute(data any) (string, error) {
	var buf bytes.Buffer
	// Execute the specific template by name
	if err := g.tmpl.ExecuteTemplate(&buf, string(g.id)+".tmpl", data); err != nil {
		return "", err
	}
	return buf.String(), nil
}

// Error returns the error message.
func (e *TemplateNotFoundError) Error() string {
	return "template not found: " + string(e.ID)
}
