package domain

import (
	"text/template"
)

// Template represents a template file used by the TemplateEngine for rendering.
// It contains both the raw template content and optional cached parsed AST.
type Template struct {
	FilePath string             // Absolute path to template file
	Name     string             // Human-readable display name
	Content  string             // Raw template text with Go template syntax
	Parsed   *template.Template // Optional cached AST
}
