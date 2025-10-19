package domain

// Note represents a complete note combining file metadata and content metadata.
// It serves as the composite model for operations needing both file and content
// information, used by TemplateEngine, QueryService, and VaultIndexer.
type Note struct {
	File        // File identity and metadata (embedded struct)
	Frontmatter // Content metadata (embedded struct)
}

// NewNote creates Note from file and frontmatter components.
// Called by VaultIndexer when combining file system and content data.
func NewNote(file File, frontmatter Frontmatter) Note {
	return Note{
		File:        file,
		Frontmatter: frontmatter,
	}
}

// SchemaName returns the frontmatter FileClass for schema validation.
// Used by SchemaValidator for validation against Schema.Properties.
func (n *Note) SchemaName() string {
	return n.Frontmatter.SchemaName()
}
