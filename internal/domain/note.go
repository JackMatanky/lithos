// Package domain provides core domain types and business logic for Lithos.
package domain

import (
	"fmt"
	"strings"
)

// NoteValidationError represents validation errors for Note construction.
type NoteValidationError struct {
	Field   string
	Message string
}

// NoteFieldError represents errors accessing frontmatter fields.
type NoteFieldError struct {
	Field   string
	Message string
}

// Note represents a core business entity for a markdown note.
// It is a rich domain aggregate root combining identity and parsed metadata.
// Infrastructure concerns (file paths, modification times) are kept in DTOs.
type Note struct {
	// Path is the vault-relative path as the note's identifier.
	Path string
	// Frontmatter contains content metadata from YAML frontmatter.
	Frontmatter Frontmatter
	// Links contains all links found in the markdown content.
	Links []Link
	// Headings contains the document structure hierarchy.
	Headings []Heading
	// Tags contains hashtags extracted from content.
	Tags []string
	// Tasks contains task list items with completion status.
	Tasks []TaskItem
	// Backlinks contains computed references to this note (populated after
	// construction).
	Backlinks []Link
}

// Link represents a link found in markdown content.
type Link struct {
	// Text is the display text for the link.
	Text string
	// Destination is the link target (note path, URL, etc.).
	Destination string
	// IsWikilink indicates if this is a wikilink format [[...]] vs markdown
	// [...](...).
	IsWikilink bool
}

// Heading represents a markdown heading with level and text.
type Heading struct {
	// Level is the heading level (1-6) corresponding to # count.
	Level int
	// Text is the heading text without # markers, trimmed of whitespace.
	Text string
}

// TaskItem represents a task/checkbox item from markdown.
type TaskItem struct {
	// Text is the task description without checkbox markers.
	Text string
	// IsChecked indicates completion status ([x] = true, [ ] = false).
	IsChecked bool
	// Line is the line number in the source markdown.
	Line int
}

// Frontmatter represents note content metadata extracted from YAML frontmatter.
// It is a rich domain entity with type-safe accessors and delegation methods.
type Frontmatter struct {
	// Fields contains the complete parsed YAML frontmatter as a flexible map.
	// Preserves all user-defined fields without filtering.
	Fields map[string]any
}

// Error returns the error message for NoteValidationError.
func (e NoteValidationError) Error() string {
	return fmt.Sprintf(
		"note validation failed for field '%s': %s",
		e.Field,
		e.Message,
	)
}

// Error returns the error message for NoteFieldError.
func (e NoteFieldError) Error() string {
	return fmt.Sprintf(
		"note field access failed for '%s': %s",
		e.Field,
		e.Message,
	)
}

// NewFrontmatter creates a new Frontmatter from parsed YAML fields.
// It creates a defensive copy of the fields map to ensure immutability.
// Frontmatter instances are immutable after construction - helper methods never
// mutate Fields. This guarantees that adapter-parsed data remains unchanged
// during domain operations. See docs/architecture/data-models.md#frontmatter
// for entity enrichment pattern.
func NewFrontmatter(fields map[string]any) Frontmatter {
	fieldsCopy := make(map[string]any)
	for k, v := range fields {
		fieldsCopy[k] = v
	}
	return Frontmatter{
		Fields: fieldsCopy,
	}
}

// FileClass returns the fileClass field from the frontmatter.
// Uses the configured key (default "fileClass") to extract schema reference
// and gracefully falls back to the standard key for backward compatibility.
func (f Frontmatter) FileClass() string {
	primaryKey := Instance().FileClassKey
	candidateKeys := []string{primaryKey}

	// Preserve compatibility with historical vaults that use "fileClass" or
	// "file_class" regardless of the configured key.
	switch primaryKey {
	case "fileClass":
		candidateKeys = append(candidateKeys, "file_class")
	case "file_class":
		candidateKeys = append(candidateKeys, "fileClass")
	default:
		candidateKeys = append(candidateKeys, "fileClass", "file_class")
	}

	for _, key := range candidateKeys {
		if val, ok := f.Fields[key].(string); ok && val != "" {
			return val
		}
	}

	return ""
}

// SchemaName returns the schema name (FileClass) for this frontmatter.
// This method provides a consistent interface for schema resolution.
// Delegates to FileClass() method.
func (f Frontmatter) SchemaName() string {
	return f.FileClass()
}

// NewNote creates a new Note from parsed metadata.
// This is the aggregate root constructor for the Note entity.
// Validates business rules and ensures defensive copies of slices.
// Backlinks start empty and are populated during enrichment phase.
func NewNote(
	path string,
	frontmatter Frontmatter,
	links []Link,
	headings []Heading,
	tags []string,
	tasks []TaskItem,
) (Note, error) {
	note := Note{
		Path:        path,
		Frontmatter: frontmatter,
		Links:       make([]Link, len(links)),
		Headings:    make([]Heading, len(headings)),
		Tags:        make([]string, len(tags)),
		Tasks:       make([]TaskItem, len(tasks)),
		Backlinks:   []Link{}, // Empty initially, populated during enrichment
	}

	// Defensive copy slices to prevent external mutation
	copy(note.Links, links)
	copy(note.Headings, headings)
	copy(note.Tags, tags)
	copy(note.Tasks, tasks)

	// Validate the note
	if err := note.Validate(); err != nil {
		return Note{}, err
	}

	return note, nil
}

// Validate enforces business rules for Note construction and mutation.
// Ensures path is non-empty, frontmatter is present, and slices are
// initialized.
//
// Validation Layer: Domain Layer (Semantic)
// - Validates business rules for Note entities
// - Does NOT perform parsing or structural validation
// See: docs/architecture/coding-standards.md#validation-layer-separation.
func (n Note) Validate() error {
	if strings.TrimSpace(n.Path) == "" {
		return NoteValidationError{
			Field:   "Path",
			Message: "path cannot be empty",
		}
	}
	if n.Frontmatter.Fields == nil {
		return NoteValidationError{
			Field:   "Frontmatter",
			Message: "frontmatter fields cannot be nil",
		}
	}
	// Links, Headings, Tags, Tasks can be empty but not nil (ensured by
	// factory)
	// Backlinks can be empty during construction, populated during enrichment
	return nil
}

// WithBacklinks creates a new Note with updated backlinks.
// Used during the enrichment phase to populate computed references.
func (n Note) WithBacklinks(backlinks []Link) Note {
	n.Backlinks = make([]Link, len(backlinks))
	copy(n.Backlinks, backlinks)
	return n
}

// SchemaName returns the schema name for this note.
// Delegates to the Frontmatter's SchemaName method.
func (n Note) SchemaName() string {
	return n.Frontmatter.SchemaName()
}

// FileClass returns the fileClass field from frontmatter.
// Delegates to Frontmatter.FileClass() method.
func (n Note) FileClass() string {
	return n.Frontmatter.FileClass()
}

// Title returns the title field from frontmatter.
// Delegates to Frontmatter.Title() method.
func (n Note) Title() string {
	return n.Frontmatter.Title()
}

// Aliases returns the aliases field from frontmatter.
// Delegates to Frontmatter.Aliases() method.
func (n Note) Aliases() []string {
	return n.Frontmatter.Aliases()
}

// HasFrontmatterField checks if a field exists in frontmatter.
// Delegates to Frontmatter.Has() method.
func (n Note) HasFrontmatterField(key string) bool {
	return n.Frontmatter.Has(key)
}

// GetFrontmatterString retrieves a string field from frontmatter.
// Returns the value and true if it exists and is a string, empty string and
// false otherwise.
func (n Note) GetFrontmatterString(key string) (string, bool) {
	val, ok := n.Frontmatter.Get(key)
	if !ok {
		return "", false
	}
	str, ok := val.(string)
	return str, ok
}

// Get retrieves a field value from the frontmatter.
// Returns the value and true if the field exists, nil and false otherwise.
// Part of generic field access helpers in
// docs/architecture/data-models.md#frontmatter.
func (f Frontmatter) Get(key string) (any, bool) {
	val, ok := f.Fields[key]
	return val, ok
}

// Has checks if a field exists in the frontmatter.
// Part of generic field access helpers in
// docs/architecture/data-models.md#frontmatter.
func (f Frontmatter) Has(key string) bool {
	_, ok := f.Fields[key]
	return ok
}

// Is checks if the field exists and is of the specified type T.
// This is a generic type checker that replaces individual IsString, IsBool,
// etc.
// Enables safe type checking before casting. Part of type inspector helpers
// in docs/architecture/data-models.md#frontmatter.
//
// Usage:
//
//	if f.Is[string]("title") { ... }
//	if f.Is[bool]("published") { ... }
//	if f.Is[map[string]any]("metadata") { ... }
func Is[T any](f Frontmatter, key string) bool {
	val, ok := f.Fields[key]
	if !ok {
		return false
	}
	_, ok = val.(T)
	return ok
}

// IsString checks if the field exists and is of string type.
// Enables safe type checking before casting. Part of type inspector helpers
// in docs/architecture/data-models.md#frontmatter.
//
// Deprecated: Use Is[string](f, key) instead.
func (f Frontmatter) IsString(key string) bool {
	return Is[string](f, key)
}

// IsArray checks if the field exists and is of array/slice type.
// Enables safe type checking before casting. Part of type inspector helpers
// in docs/architecture/data-models.md#frontmatter.
func (f Frontmatter) IsArray(key string) bool {
	val, ok := f.Fields[key]
	if !ok {
		return false
	}
	switch val.(type) {
	case []any, []string:
		return true
	default:
		return false
	}
}

// IsInt checks if the field exists and is of numeric type (int, int64,
// float64). Handles YAML's flexible number parsing. Enables safe type checking
// before casting. Part of type inspector helpers in
// docs/architecture/data-models.md#frontmatter.
func (f Frontmatter) IsInt(key string) bool {
	val, ok := f.Fields[key]
	if !ok {
		return false
	}
	switch val.(type) {
	case int, int64, float64:
		return true
	default:
		return false
	}
}

// IsBool checks if the field exists and is of boolean type.
// Enables safe type checking before casting. Part of type inspector helpers
// in docs/architecture/data-models.md#frontmatter.
//
// Deprecated: Use Is[bool](f, key) instead.
func (f Frontmatter) IsBool(key string) bool {
	return Is[bool](f, key)
}

// IsMap checks if the field exists and is of map type.
// Enables safe type checking before casting. Part of type inspector helpers
// in docs/architecture/data-models.md#frontmatter.
//
// Deprecated: Use Is[map[string]any](f, key) instead.
func (f Frontmatter) IsMap(key string) bool {
	return Is[map[string]any](f, key)
}

// GetFileClass retrieves the fileClass field using the configured key.
// Returns the fileClass value if it exists and is a string, empty string
// otherwise. Part of domain-level delegation helpers in
// docs/architecture/data-models.md#frontmatter.
//
// Deprecated: Use FileClass() method instead.
func (f Frontmatter) GetFileClass() string {
	return f.FileClass()
}

// Title retrieves the title field from the frontmatter.
// Returns the title value if it exists and is a string, empty string otherwise.
// Part of domain-level delegation helpers in
// docs/architecture/data-models.md#frontmatter.
func (f Frontmatter) Title() string {
	if val, ok := f.Fields["title"].(string); ok {
		return val
	}
	return ""
}

// stringSliceFrom converts various YAML representations of string lists to
// []string.
// Handles []string, []any (with string elements), and single strings.
// Used internally by Aliases() method.
func stringSliceFrom(val any) []string {
	if val == nil {
		return []string{}
	}

	// Handle []string directly
	if strSlice, ok := val.([]string); ok {
		return strSlice
	}

	// Handle []any with string elements
	if anySlice, ok := val.([]any); ok {
		result := make([]string, 0, len(anySlice))
		for _, item := range anySlice {
			if str, isString := item.(string); isString {
				result = append(result, str)
			}
		}
		return result
	}

	// Handle single string -> slice
	if str, ok := val.(string); ok {
		return []string{str}
	}

	return []string{}
}

// Aliases retrieves the aliases field from the frontmatter.
// Handles various array formats and normalizes to []string.
// Returns an empty slice if the field doesn't exist or cannot be converted.
// Delegates to stringSliceFrom for type conversion logic.
func (f Frontmatter) Aliases() []string {
	val, ok := f.Fields["aliases"]
	if !ok {
		return []string{}
	}
	return stringSliceFrom(val)
}

// Clone creates a deep copy of the Note to ensure immutability.
// Template functions return notes from QueryService's in-memory index.
// If templates could mutate these notes, they would corrupt the index.
// All template helpers must return defensive copies via this method.
//
// Returns a new Note with deep-copied slices and frontmatter fields.
// Handles nested structures in frontmatter fields recursively.
func (n Note) Clone() Note {
	// Deep copy frontmatter fields map with recursive copying
	fieldsCopy := deepCopyFields(n.Frontmatter.Fields)

	// Deep copy slices
	links := make([]Link, len(n.Links))
	copy(links, n.Links)

	headings := make([]Heading, len(n.Headings))
	copy(headings, n.Headings)

	tags := make([]string, len(n.Tags))
	copy(tags, n.Tags)

	tasks := make([]TaskItem, len(n.Tasks))
	copy(tasks, n.Tasks)

	backlinks := make([]Link, len(n.Backlinks))
	copy(backlinks, n.Backlinks)

	return Note{
		Path: n.Path, // strings are immutable in Go
		Frontmatter: Frontmatter{
			Fields: fieldsCopy,
		},
		Links:     links,
		Headings:  headings,
		Tags:      tags,
		Tasks:     tasks,
		Backlinks: backlinks,
	}
}

// deepCopyFields recursively deep-copies frontmatter fields to prevent
// template mutations from corrupting the index.
func deepCopyFields(fields map[string]any) map[string]any {
	if fields == nil {
		return nil
	}

	copied := make(map[string]any, len(fields))
	for k, v := range fields {
		copied[k] = deepCopyValue(v)
	}
	return copied
}

// deepCopyValue recursively copies values that might contain mutable
// references.
func deepCopyValue(val any) any {
	if val == nil {
		return nil
	}

	switch v := val.(type) {
	case []any:
		// Deep copy slices
		copySlice := make([]any, len(v))
		for i, item := range v {
			copySlice[i] = deepCopyValue(item)
		}
		return copySlice
	case []string:
		// Copy string slices
		copySlice := make([]string, len(v))
		copy(copySlice, v)
		return copySlice
	case map[string]any:
		// Deep copy nested maps
		return deepCopyFields(v)
	default:
		// Immutable types (strings, numbers, bools) can be copied directly
		return v
	}
}
