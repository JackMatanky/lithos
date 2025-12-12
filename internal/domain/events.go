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

type baseEvent struct {
	eventType   string
	aggregateID string
	occurredAt  time.Time
}

// NoteIndexedEvent is emitted when a note is successfully indexed.
type NoteIndexedEvent struct {
	baseEvent

	note      Note
	path      string
	fileClass string
}

// VaultIndexingSummary captures vault indexing metrics for event payloads.
type VaultIndexingSummary struct {
	ScannedCount        int
	IndexedCount        int
	ParseFailures       int
	CacheFailures       int
	ValidationSuccesses int
	ValidationFailures  int
}

// VaultIndexingCompleteEvent captures summary metrics for a full vault index.
type VaultIndexingCompleteEvent struct {
	baseEvent

	summary  VaultIndexingSummary
	duration time.Duration
}

// FrontmatterValidatedEvent captures the result of semantic frontmatter
// validation.
type FrontmatterValidatedEvent struct {
	baseEvent

	schemaName string
	valid      bool
	errors     []string
}

// SchemaLoadedEvent is emitted when a schema is loaded into the registry.
type SchemaLoadedEvent struct {
	baseEvent

	schemaName    string
	propertyCount int
}

// SchemasReloadedEvent represents a schema reload operation completing.
type SchemasReloadedEvent struct {
	baseEvent

	schemaCount int
}

// CommandIssuedEvent represents a command invocation emitted by drivers
// (e.g., CLICommander) for decoupled orchestration.
type CommandIssuedEvent struct {
	baseEvent

	command string
	payload map[string]string
}

func newBaseEvent(
	eventType, aggregateID string,
	occurredAt time.Time,
) (baseEvent, error) {
	if eventType == "" {
		return baseEvent{}, fmt.Errorf("event type is required")
	}
	if aggregateID == "" {
		return baseEvent{}, fmt.Errorf("aggregate id is required")
	}
	if occurredAt.IsZero() {
		occurredAt = time.Now()
	}
	return baseEvent{
		eventType:   eventType,
		aggregateID: aggregateID,
		occurredAt:  occurredAt,
	}, nil
}

// EventType returns the event type string.
func (e baseEvent) EventType() string {
	return e.eventType
}

// OccurredAt returns when the event occurred.
func (e baseEvent) OccurredAt() time.Time {
	return e.occurredAt
}

// AggregateID returns the aggregate identifier for the event.
func (e baseEvent) AggregateID() string {
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
	base, err := newBaseEvent("NoteIndexed", note.Path, occurredAt)
	if err != nil {
		return nil, err
	}
	return &NoteIndexedEvent{
		baseEvent: base,
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

// NewVaultIndexingCompleteEvent validates and constructs the event.
func NewVaultIndexingCompleteEvent(
	summary VaultIndexingSummary,
	duration time.Duration,
	occurredAt time.Time,
) (*VaultIndexingCompleteEvent, error) {
	if summary.IndexedCount < 0 {
		return nil, fmt.Errorf("indexed count must be >= 0")
	}
	if summary.ScannedCount < 0 {
		return nil, fmt.Errorf("scanned count must be >= 0")
	}
	base, err := newBaseEvent("VaultIndexingComplete", "vault", occurredAt)
	if err != nil {
		return nil, err
	}
	return &VaultIndexingCompleteEvent{
		baseEvent: base,
		summary:   summary,
		duration:  duration,
	}, nil
}

// MustNewVaultIndexingCompleteEvent panics when creation fails.
func MustNewVaultIndexingCompleteEvent(
	summary VaultIndexingSummary,
	duration time.Duration,
	occurredAt time.Time,
) *VaultIndexingCompleteEvent {
	event, err := NewVaultIndexingCompleteEvent(summary, duration, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// NotesIndexed returns the number of notes processed.
func (e *VaultIndexingCompleteEvent) NotesIndexed() int {
	return e.summary.IndexedCount
}

// Summary returns the indexing summary payload.
func (e *VaultIndexingCompleteEvent) Summary() VaultIndexingSummary {
	return e.summary
}

// Duration returns the total time spent indexing.
func (e *VaultIndexingCompleteEvent) Duration() time.Duration {
	return e.duration
}

// ScannedCount returns how many files were scanned.
func (e *VaultIndexingCompleteEvent) ScannedCount() int {
	return e.summary.ScannedCount
}

// ParseFailures returns parse failure count.
func (e *VaultIndexingCompleteEvent) ParseFailures() int {
	return e.summary.ParseFailures
}

// CacheFailures returns cache failure count.
func (e *VaultIndexingCompleteEvent) CacheFailures() int {
	return e.summary.CacheFailures
}

// ValidationSuccesses returns validation success count.
func (e *VaultIndexingCompleteEvent) ValidationSuccesses() int {
	return e.summary.ValidationSuccesses
}

// ValidationFailures returns validation failure count.
func (e *VaultIndexingCompleteEvent) ValidationFailures() int {
	return e.summary.ValidationFailures
}

// NewFrontmatterValidatedEvent constructs a validation event.
func NewFrontmatterValidatedEvent(
	noteID, schemaName string,
	valid bool,
	validationErrors []string,
	occurredAt time.Time,
) (*FrontmatterValidatedEvent, error) {
	if noteID == "" {
		return nil, fmt.Errorf("note id is required")
	}
	if schemaName == "" {
		return nil, fmt.Errorf("schema name is required")
	}
	base, err := newBaseEvent("FrontmatterValidated", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of errors slice to prevent external mutation.
	errs := make([]string, len(validationErrors))
	copy(errs, validationErrors)
	return &FrontmatterValidatedEvent{
		baseEvent:  base,
		schemaName: schemaName,
		valid:      valid,
		errors:     errs,
	}, nil
}

// MustNewFrontmatterValidatedEvent panics when creation fails.
func MustNewFrontmatterValidatedEvent(
	noteID, schemaName string,
	valid bool,
	validationErrors []string,
	occurredAt time.Time,
) *FrontmatterValidatedEvent {
	event, err := NewFrontmatterValidatedEvent(
		noteID,
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
	base, err := newBaseEvent("SchemaLoaded", schemaName, occurredAt)
	if err != nil {
		return nil, err
	}
	return &SchemaLoadedEvent{
		baseEvent:     base,
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
	base, err := newBaseEvent("SchemasReloaded", "schemas", occurredAt)
	if err != nil {
		return nil, err
	}
	return &SchemasReloadedEvent{baseEvent: base, schemaCount: schemaCount}, nil
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

// NewCommandIssuedEvent constructs a command event with optional payload.
func NewCommandIssuedEvent(
	command string,
	payload map[string]string,
	occurredAt time.Time,
) (*CommandIssuedEvent, error) {
	if strings.TrimSpace(command) == "" {
		return nil, fmt.Errorf("command name is required")
	}
	base, err := newBaseEvent("CommandIssued", command, occurredAt)
	if err != nil {
		return nil, err
	}
	payloadCopy := make(map[string]string, len(payload))
	for k, v := range payload {
		payloadCopy[k] = v
	}
	return &CommandIssuedEvent{
		baseEvent: base,
		command:   command,
		payload:   payloadCopy,
	}, nil
}

// MustNewCommandIssuedEvent panics when construction fails.
func MustNewCommandIssuedEvent(
	command string,
	payload map[string]string,
	occurredAt time.Time,
) *CommandIssuedEvent {
	event, err := NewCommandIssuedEvent(command, payload, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Command returns the command identifier.
func (e *CommandIssuedEvent) Command() string {
	return e.command
}

// Payload returns a defensive copy of the payload map.
func (e *CommandIssuedEvent) Payload() map[string]string {
	copyPayload := make(map[string]string, len(e.payload))
	for k, v := range e.payload {
		copyPayload[k] = v
	}
	return copyPayload
}
