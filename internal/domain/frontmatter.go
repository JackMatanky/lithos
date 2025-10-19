package domain

// Frontmatter represents parsed YAML frontmatter from a note file.
// It separates content metadata from file system metadata, containing
// both the raw parsed fields and extracted schema reference.
type Frontmatter struct {
	FileClass string                 // Optional schema reference from fields["fileClass"]
	Fields    map[string]interface{} // Complete parsed YAML frontmatter
}

// NewFrontmatter creates Frontmatter from parsed YAML fields.
// Called by adapter during note content parsing.
func NewFrontmatter(fields map[string]interface{}) Frontmatter {
	return Frontmatter{
		FileClass: extractFileClass(fields),
		Fields:    fields,
	}
}

// extractFileClass extracts schema reference from frontmatter fields.
// Returns the value of "fileClass" field as string, or empty string if not
// present.
func extractFileClass(fields map[string]interface{}) string {
	if fields == nil {
		return ""
	}

	if fileClass, exists := fields["fileClass"]; exists {
		if str, ok := fileClass.(string); ok {
			return str
		}
	}

	return ""
}

// SchemaName returns the FileClass for schema validation.
// Used by SchemaValidator for validation against Schema.Properties.
func (f Frontmatter) SchemaName() string {
	return f.FileClass
}
