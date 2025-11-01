// Package domain provides core domain types and business logic for Lithos.
package domain

import "encoding/json"

// Note represents a core business entity for a markdown note.
// It is an aggregate root combining identity and content metadata.
type Note struct {
	// ID is the abstract identifier for this note.
	// Opaque to the domain - could represent file path, UUID, or database key.
	ID NoteID
	// Path is the vault-relative file path for this note.
	// Used for path-based queries and navigation.
	Path string
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
// It extracts the fileClass for convenience while preserving all original
// fields.
func NewFrontmatter(fields map[string]interface{}) Frontmatter {
	return Frontmatter{
		FileClass: extractFileClass(fields),
		Fields:    fields,
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
func (f Frontmatter) SchemaName() string {
	return f.FileClass
}

// NewNote creates a new Note from its constituent parts.
// This is the aggregate root constructor for the Note entity.
func NewNote(id NoteID, frontmatter Frontmatter) Note {
	return Note{
		ID:          id,
		Path:        string(id), // NoteID contains the vault-relative path
		Frontmatter: frontmatter,
	}
}

// UnmarshalJSON implements custom JSON unmarshaling for Note.
// Ensures Path field is populated from ID for backward compatibility.
func (n *Note) UnmarshalJSON(data []byte) error {
	// Define a temporary struct to avoid recursion
	aux := &struct { //nolint:exhaustruct // Custom unmarshaling struct for backward compatibility
		ID          NoteID      `json:"id"`
		Path        *string     `json:"path,omitempty"` // Optional for backward compatibility
		Frontmatter Frontmatter `json:"frontmatter"`
	}{}

	if err := json.Unmarshal(data, aux); err != nil {
		return err
	}

	// Set values
	n.ID = aux.ID
	n.Frontmatter = aux.Frontmatter

	// Ensure Path is set from ID if not present in JSON (backward
	// compatibility)
	if aux.Path == nil || *aux.Path == "" {
		n.Path = string(n.ID)
	} else {
		n.Path = *aux.Path
	}

	return nil
}

// SchemaName returns the schema name for this note.
// Delegates to the Frontmatter's SchemaName method.
func (n Note) SchemaName() string {
	return n.Frontmatter.SchemaName()
}
