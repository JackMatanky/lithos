// Package domain provides core domain types and business logic for Lithos.
package domain

// Note represents a core business entity for a markdown note.
// It is a pure domain aggregate root combining identity and content metadata.
// Infrastructure concerns (file paths, modification times) are kept in DTOs.
type Note struct {
	// ID is the abstract identifier for this note.
	// Opaque to the domain - could represent file path, UUID, or database key.
	ID NoteID
	// Frontmatter contains content metadata from YAML frontmatter.
	// Composed (not embedded) to maintain clean domain boundaries.
	Frontmatter Frontmatter
}

// NoteID represents an opaque domain identifier for notes.
// It abstracts the storage mechanism (file paths, UUIDs, database keys)
// from the domain logic.
type NoteID string

// Frontmatter represents note content metadata extracted from YAML frontmatter.
// It is a pure data structure with no behavior (anemic model).
type Frontmatter struct {
	// FileClass is the schema reference extracted from Fields["fileClass"].
	// Used for validation lookup. Empty if not present in Fields.
	FileClass string
	// Fields contains the complete parsed YAML frontmatter as a flexible map.
	// Preserves all user-defined fields without filtering.
	Fields map[string]interface{}
}

// NewNoteID creates a new NoteID from a string value.
// The domain doesn't know or care what this string represents -
// it could be a file path, UUID, or database key.
func NewNoteID(value string) NoteID {
	return NoteID(value)
}

// String returns the string representation of the NoteID.
// This implements the Stringer interface for logging and debugging.
func (id NoteID) String() string {
	return string(id)
}

// NewFrontmatter creates a new Frontmatter from parsed YAML fields.
// It creates a defensive copy of the fields map to ensure immutability.
// Frontmatter instances are immutable after construction - helper methods never
// mutate Fields. This guarantees that adapter-parsed data remains unchanged
// during domain operations. See docs/architecture/data-models.md#frontmatter
// for entity enrichment pattern.
func NewFrontmatter(fields map[string]interface{}) Frontmatter {
	fieldsCopy := make(map[string]interface{})
	for k, v := range fields {
		fieldsCopy[k] = v
	}
	return Frontmatter{
		FileClass: extractFileClass(fieldsCopy),
		Fields:    fieldsCopy,
	}
}

// extractFileClass extracts the fileClass from the fields map.
// Returns empty string if fileClass key is missing or not a string.
func extractFileClass(fields map[string]interface{}) string {
	if fc, ok := fields["fileClass"].(string); ok {
		return fc
	}
	return ""
}

// SchemaName returns the schema name (FileClass) for this frontmatter.
// This method provides a consistent interface for schema resolution.
// TODO: Update to use FileClass() method once consumers are migrated.
func (f Frontmatter) SchemaName() string {
	return f.FileClass
}

// NewNote creates a new Note from its constituent parts.
// This is the aggregate root constructor for the Note entity.
// Infrastructure concerns (paths, timestamps) are handled by DTOs.
func NewNote(id NoteID, frontmatter Frontmatter) Note {
	return Note{
		ID:          id,
		Frontmatter: frontmatter,
	}
}

// SchemaName returns the schema name for this note.
// Delegates to the Frontmatter's SchemaName method.
func (n Note) SchemaName() string {
	return n.Frontmatter.SchemaName()
}

// Get retrieves a field value from the frontmatter.
// Returns the value and true if the field exists, nil and false otherwise.
// Part of generic field access helpers in
// docs/architecture/data-models.md#frontmatter.
func (f Frontmatter) Get(key string) (interface{}, bool) {
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

// IsString checks if the field exists and is of string type.
// Enables safe type checking before casting. Part of type inspector helpers
// in docs/architecture/data-models.md#frontmatter.
func (f Frontmatter) IsString(key string) bool {
	val, ok := f.Fields[key]
	if !ok {
		return false
	}
	_, ok = val.(string)
	return ok
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
	case []interface{}, []string:
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
func (f Frontmatter) IsBool(key string) bool {
	val, ok := f.Fields[key]
	if !ok {
		return false
	}
	_, ok = val.(bool)
	return ok
}

// IsMap checks if the field exists and is of map type.
// Enables safe type checking before casting. Part of type inspector helpers
// in docs/architecture/data-models.md#frontmatter.
func (f Frontmatter) IsMap(key string) bool {
	val, ok := f.Fields[key]
	if !ok {
		return false
	}
	_, ok = val.(map[string]interface{})
	return ok
}

// GetFileClass retrieves the fileClass field using the configured key.
// Returns the fileClass value if it exists and is a string, empty string
// otherwise. Part of domain-level delegation helpers in
// docs/architecture/data-models.md#frontmatter.
func (f Frontmatter) GetFileClass() string {
	// For now, use default key until config singleton is available
	key := "fileClass" // defaultFileClassKey from config.go
	if val, ok := f.Fields[key].(string); ok {
		return val
	}
	return ""
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
// Handles []string, []interface{} (with string elements), and single strings.
// Used internally by Aliases() method.
func stringSliceFrom(val interface{}) []string {
	if val == nil {
		return []string{}
	}

	// Handle []string directly
	if strSlice, ok := val.([]string); ok {
		return strSlice
	}

	// Handle []interface{} with string elements
	if anySlice, ok := val.([]interface{}); ok {
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
