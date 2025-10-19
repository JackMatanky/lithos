// Package template provides template parsing and execution services.
//
// This package implements template functionality using Go's text/template
// with custom functions for enhanced template capabilities.
package template

import (
	"strings"
	"text/template"
	"time"
)

// now returns the current time formatted according to the provided layout.
// If layout is empty, it uses "2006-01-02 15:04:05" format.
// Uses Go's reference time format: "2006-01-02 15:04:05" (Mon Jan 2 15:04:05
// -0700 MST 2006).
func now(layout string) string {
	if layout == "" {
		layout = "2006-01-02 15:04:05"
	}
	return time.Now().Format(layout)
}

// toLower converts the input string to lowercase.
// Returns empty string if input is empty.
func toLower(s string) string {
	return strings.ToLower(s)
}

// toUpper converts the input string to uppercase.
// Returns empty string if input is empty.
func toUpper(s string) string {
	return strings.ToUpper(s)
}

// NewFuncMap creates and returns a template.FuncMap containing all available
// template functions. This function map can be used with template.New().Funcs()
// to register functions for template execution.
//
// The function map includes:
//   - now: Format current time with optional layout
//   - toLower: Convert string to lowercase
//   - toUpper: Convert string to uppercase
//
// This design allows for easy extension by adding new functions to this map.
func NewFuncMap() template.FuncMap {
	return template.FuncMap{
		"now":     now,
		"toLower": toLower,
		"toUpper": toUpper,
	}
}
