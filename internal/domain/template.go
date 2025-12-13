package domain

// Template represents an executable template for note generation.
// It is a pure data structure containing only the template identity and raw
// content. This follows the anemic domain model pattern where business logic
// resides in services. FilePath and Parsed fields are intentionally omitted as
// they represent infrastructure concerns handled by adapters (FileMetadata) and
// services (TemplateEngine) respectively.
type Template struct {
	// ID is the template name used for identification and composition.
	// It enables Go's text/template {{template "name"}} directive
	// functionality.
	ID TemplateID
	// Content contains the raw template text with Go text/template syntax and
	// Lithos functions. This includes directives like {{prompt}}, {{now}}, and
	// {{template}} references.
	Content string
}

// TemplateID represents a template name used for identification and
// composition.
// It wraps a string to provide type safety and domain-specific semantics.
// TemplateID is meaningful in the domain (unlike NoteID which is opaque)
// because Go's text/template requires names for {{template "name"}} references.
type TemplateID string

// NewTemplate creates a new Template with the given ID and content.
// The content should contain valid Go text/template syntax.
// This constructor ensures Template remains a pure data structure with no
// behavior.
func NewTemplate(id TemplateID, content string) Template {
	return Template{
		ID:      id,
		Content: content,
	}
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
