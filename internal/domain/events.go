package domain

import (
	"fmt"
	"strings"
	"time"
)

// DomainEvent represents a significant domain occurrence that other
// components can react to. All events carry a type, aggregate identifier,
// and timestamp for tracing.
type DomainEvent interface {
	EventType() string
	OccurredAt() time.Time
	AggregateID() string
}

// BaseEvent provides common fields and methods for all events.
type BaseEvent struct {
	eventType   string
	aggregateID string
	occurredAt  time.Time
}

// NoteIndexedEvent is emitted when a note is successfully indexed.
type NoteIndexedEvent struct {
	BaseEvent

	note      Note
	path      string
	fileClass string
}

// FrontmatterValidatedEvent captures the result of semantic frontmatter
// validation.
type FrontmatterValidatedEvent struct {
	BaseEvent

	note       Note
	schemaName string
	valid      bool
	errors     []string
}

// SchemaLoadedEvent is emitted when a schema is loaded into the registry.
type SchemaLoadedEvent struct {
	BaseEvent

	schemaName    string
	propertyCount int
}

// SchemasReloadedEvent represents a schema reload operation completing.
type SchemasReloadedEvent struct {
	BaseEvent

	schemaCount int
}

// NoteCreatedEvent tracks successful note creation.
type NoteCreatedEvent struct {
	BaseEvent

	noteID     string
	fileClass  string
	templateID string
}

// SchemaUpdatedEvent triggers reactive cache invalidation.
type SchemaUpdatedEvent struct {
	BaseEvent

	schemaName string
	operation  string // "created", "updated", "deleted"
}

// NewBaseEvent creates a new BaseEvent with validation.
func NewBaseEvent(
	eventType, aggregateID string,
	occurredAt time.Time,
) (BaseEvent, error) {
	if eventType == "" {
		return BaseEvent{}, fmt.Errorf("event type is required")
	}
	if aggregateID == "" {
		return BaseEvent{}, fmt.Errorf("aggregate id is required")
	}
	if occurredAt.IsZero() {
		occurredAt = time.Now()
	}
	return BaseEvent{
		eventType:   eventType,
		aggregateID: aggregateID,
		occurredAt:  occurredAt,
	}, nil
}

// EventType returns the event type string.
func (e BaseEvent) EventType() string {
	return e.eventType
}

// OccurredAt returns when the event occurred.
func (e BaseEvent) OccurredAt() time.Time {
	return e.occurredAt
}

// AggregateID returns the aggregate identifier for the event.
func (e BaseEvent) AggregateID() string {
	return e.aggregateID
}

// NewNoteIndexedEvent validates inputs and constructs the event.
func NewNoteIndexedEvent(
	note Note,
	occurredAt time.Time,
) (*NoteIndexedEvent, error) {
	if strings.TrimSpace(note.Path) == "" {
		return nil, fmt.Errorf("note path is required")
	}
	fileClass := note.FileClass()
	if fileClass == "" {
		return nil, fmt.Errorf("file class is required")
	}
	base, err := NewBaseEvent("NoteIndexed", note.Path, occurredAt)
	if err != nil {
		return nil, err
	}
	return &NoteIndexedEvent{
		BaseEvent: base,
		note:      note,
		path:      note.Path,
		fileClass: fileClass,
	}, nil
}

// MustNewNoteIndexedEvent panics when construction fails.
func MustNewNoteIndexedEvent(
	note Note,
	occurredAt time.Time,
) *NoteIndexedEvent {
	event, err := NewNoteIndexedEvent(note, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Path returns the vault-relative path of the indexed note.
func (e *NoteIndexedEvent) Path() string {
	return e.path
}

// FileClass returns the schema identifier associated with the note.
func (e *NoteIndexedEvent) FileClass() string {
	return e.fileClass
}

// Note returns a copy of the indexed note.
func (e *NoteIndexedEvent) Note() Note {
	return e.note
}

// NewFrontmatterValidatedEvent constructs a validation event.
func NewFrontmatterValidatedEvent(
	note Note,
	schemaName string,
	valid bool,
	validationErrors []string,
	occurredAt time.Time,
) (*FrontmatterValidatedEvent, error) {
	if note.Path == "" {
		return nil, fmt.Errorf("note path is required")
	}
	if schemaName == "" {
		return nil, fmt.Errorf("schema name is required")
	}
	base, err := NewBaseEvent("FrontmatterValidated", note.Path, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of errors slice to prevent external mutation.
	errs := make([]string, len(validationErrors))
	copy(errs, validationErrors)
	return &FrontmatterValidatedEvent{
		BaseEvent:  base,
		note:       note,
		schemaName: schemaName,
		valid:      valid,
		errors:     errs,
	}, nil
}

// MustNewFrontmatterValidatedEvent panics when creation fails.
func MustNewFrontmatterValidatedEvent(
	note Note,
	schemaName string,
	valid bool,
	validationErrors []string,
	occurredAt time.Time,
) *FrontmatterValidatedEvent {
	event, err := NewFrontmatterValidatedEvent(
		note,
		schemaName,
		valid,
		validationErrors,
		occurredAt,
	)
	if err != nil {
		panic(err)
	}
	return event
}

// SchemaName returns the schema used for validation.
func (e *FrontmatterValidatedEvent) SchemaName() string {
	return e.schemaName
}

// IsValid indicates whether validation succeeded.
func (e *FrontmatterValidatedEvent) IsValid() bool {
	return e.valid
}

// Errors returns a defensive copy of validation errors.
func (e *FrontmatterValidatedEvent) Errors() []string {
	copyErrs := make([]string, len(e.errors))
	copy(copyErrs, e.errors)
	return copyErrs
}

// Note returns a copy of the validated note.
func (e *FrontmatterValidatedEvent) Note() Note {
	return e.note
}

// NoteID returns the path of the validated note (for backward compatibility).
func (e *FrontmatterValidatedEvent) NoteID() string {
	return e.note.Path
}

// ValidationErrors returns validation errors (for backward compatibility).
func (e *FrontmatterValidatedEvent) ValidationErrors() []string {
	return e.Errors()
}

// NewSchemaLoadedEvent constructs the event after validation.
func NewSchemaLoadedEvent(
	schemaName string,
	propertyCount int,
	occurredAt time.Time,
) (*SchemaLoadedEvent, error) {
	if schemaName == "" {
		return nil, fmt.Errorf("schema name is required")
	}
	if propertyCount < 0 {
		return nil, fmt.Errorf("property count must be >= 0")
	}
	base, err := NewBaseEvent("SchemaLoaded", schemaName, occurredAt)
	if err != nil {
		return nil, err
	}
	return &SchemaLoadedEvent{
		BaseEvent:     base,
		schemaName:    schemaName,
		propertyCount: propertyCount,
	}, nil
}

// MustNewSchemaLoadedEvent panics when creation fails.
func MustNewSchemaLoadedEvent(
	schemaName string,
	propertyCount int,
	occurredAt time.Time,
) *SchemaLoadedEvent {
	event, err := NewSchemaLoadedEvent(schemaName, propertyCount, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// SchemaName returns the schema identifier that was loaded.
func (e *SchemaLoadedEvent) SchemaName() string {
	return e.schemaName
}

// PropertyCount returns the number of properties registered for the schema.
func (e *SchemaLoadedEvent) PropertyCount() int {
	return e.propertyCount
}

// NewSchemasReloadedEvent constructs the event.
func NewSchemasReloadedEvent(
	schemaCount int,
	occurredAt time.Time,
) (*SchemasReloadedEvent, error) {
	if schemaCount <= 0 {
		return nil, fmt.Errorf("schema count must be > 0")
	}
	base, err := NewBaseEvent("SchemasReloaded", "schemas", occurredAt)
	if err != nil {
		return nil, err
	}
	return &SchemasReloadedEvent{BaseEvent: base, schemaCount: schemaCount}, nil
}

// MustNewSchemasReloadedEvent panics when creation fails.
func MustNewSchemasReloadedEvent(
	schemaCount int,
	occurredAt time.Time,
) *SchemasReloadedEvent {
	event, err := NewSchemasReloadedEvent(schemaCount, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// SchemaCount returns the number of schemas refreshed.
func (e *SchemasReloadedEvent) SchemaCount() int {
	return e.schemaCount
}

// NewNoteCreatedEvent constructs a note creation event.
func NewNoteCreatedEvent(
	noteID string,
	fileClass string,
	templateID string,
	occurredAt time.Time,
) (*NoteCreatedEvent, error) {
	if noteID == "" {
		return nil, fmt.Errorf("note ID is required")
	}
	if fileClass == "" {
		return nil, fmt.Errorf("file class is required")
	}
	base, err := NewBaseEvent("NoteCreated", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	return &NoteCreatedEvent{
		BaseEvent:  base,
		noteID:     noteID,
		fileClass:  fileClass,
		templateID: templateID,
	}, nil
}

// MustNewNoteCreatedEvent panics when construction fails.
func MustNewNoteCreatedEvent(
	noteID string,
	fileClass string,
	templateID string,
	occurredAt time.Time,
) *NoteCreatedEvent {
	event, err := NewNoteCreatedEvent(noteID, fileClass, templateID, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// NoteID returns the identifier of the created note.
func (e *NoteCreatedEvent) NoteID() string {
	return e.noteID
}

// FileClass returns the file class of the created note.
func (e *NoteCreatedEvent) FileClass() string {
	return e.fileClass
}

// TemplateID returns the template used to create the note.
func (e *NoteCreatedEvent) TemplateID() string {
	return e.templateID
}

// NewSchemaUpdatedEvent constructs a schema update event.
func NewSchemaUpdatedEvent(
	schemaName string,
	operation string,
	occurredAt time.Time,
) (*SchemaUpdatedEvent, error) {
	if schemaName == "" {
		return nil, fmt.Errorf("schema name is required")
	}
	if operation == "" {
		return nil, fmt.Errorf("operation is required")
	}
	base, err := NewBaseEvent("SchemaUpdated", schemaName, occurredAt)
	if err != nil {
		return nil, err
	}
	return &SchemaUpdatedEvent{
		BaseEvent:  base,
		schemaName: schemaName,
		operation:  operation,
	}, nil
}

// MustNewSchemaUpdatedEvent panics when construction fails.
func MustNewSchemaUpdatedEvent(
	schemaName string,
	operation string,
	occurredAt time.Time,
) *SchemaUpdatedEvent {
	event, err := NewSchemaUpdatedEvent(schemaName, operation, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// SchemaName returns the name of the updated schema.
func (e *SchemaUpdatedEvent) SchemaName() string {
	return e.schemaName
}

// Operation returns the type of operation performed.
func (e *SchemaUpdatedEvent) Operation() string {
	return e.operation
}
