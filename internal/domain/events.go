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

	note       Note
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

// FileParseRequestedEvent requests parsing of a discovered file.
type FileParseRequestedEvent struct {
	baseEvent

	content []byte
}

// NoteParsedEvent is emitted when a file has been successfully parsed into a
// Note.
type NoteParsedEvent struct {
	baseEvent

	note Note
}

// FrontmatterValidationRequestedEvent requests validation of frontmatter.
type FrontmatterValidationRequestedEvent struct {
	baseEvent

	note Note
}

// NoteCacheRequestedEvent requests caching of a validated note.
type NoteCacheRequestedEvent struct {
	baseEvent

	note Note
}

// FileDiscoveredEvent is emitted when a new file is discovered during vault
// scanning.
type FileDiscoveredEvent struct {
	baseEvent

	path    string
	size    int
	content []byte
}

// LookupPerformedEvent tracks individual lookup operations in template
// functions.
type LookupPerformedEvent struct {
	baseEvent

	noteID      string
	resultCount int
	duration    time.Duration
	lookupType  string // "basename" or "id"
}

// QueryPerformedEvent tracks query operations with filter criteria.
type QueryPerformedEvent struct {
	baseEvent

	filterCriteria map[string]any
	resultCount    int
	duration       time.Duration
	queryType      string // "path" or "frontmatter"
}

// SchemaLookupEvent tracks schema resolution lookups.
type SchemaLookupEvent struct {
	baseEvent

	noteID     string
	schemaName string
	found      bool
	duration   time.Duration
}

// ValidationPerformedEvent tracks validation outcomes.
type ValidationPerformedEvent struct {
	baseEvent

	noteID     string
	schemaName string
	valid      bool
	duration   time.Duration
	errors     []string
}

// ValidationFailedEvent tracks validation failures with remediation hints.
type ValidationFailedEvent struct {
	baseEvent

	noteID           string
	schemaName       string
	errors           []string
	remediationHints []string
	duration         time.Duration
}

// NoteCreatedEvent tracks successful note creation.
type NoteCreatedEvent struct {
	baseEvent

	noteID     string
	fileClass  string
	templateID string
}

// SchemaUpdatedEvent triggers reactive cache invalidation.
type SchemaUpdatedEvent struct {
	baseEvent

	schemaName string
	operation  string // "created", "updated", "deleted"
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
	base, err := newBaseEvent("FrontmatterValidated", note.Path, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of errors slice to prevent external mutation.
	errs := make([]string, len(validationErrors))
	copy(errs, validationErrors)
	return &FrontmatterValidatedEvent{
		baseEvent:  base,
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

// NewFileDiscoveredEvent constructs a file discovery event.
func NewFileDiscoveredEvent(
	path string,
	size int,
	content []byte,
	occurredAt time.Time,
) (*FileDiscoveredEvent, error) {
	if path == "" {
		return nil, fmt.Errorf("file path is required")
	}
	if size < 0 {
		return nil, fmt.Errorf("file size must be >= 0")
	}
	base, err := newBaseEvent("FileDiscovered", path, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of content
	contentCopy := make([]byte, len(content))
	copy(contentCopy, content)
	return &FileDiscoveredEvent{
		baseEvent: base,
		path:      path,
		size:      size,
		content:   contentCopy,
	}, nil
}

// MustNewFileDiscoveredEvent panics when construction fails.
func MustNewFileDiscoveredEvent(
	path string,
	size int,
	content []byte,
	occurredAt time.Time,
) *FileDiscoveredEvent {
	event, err := NewFileDiscoveredEvent(path, size, content, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Path returns the vault-relative file path.
func (e *FileDiscoveredEvent) Path() string {
	return e.path
}

// Size returns the file size in bytes.
func (e *FileDiscoveredEvent) Size() int {
	return e.size
}

// Content returns a defensive copy of the file content.
func (e *FileDiscoveredEvent) Content() []byte {
	contentCopy := make([]byte, len(e.content))
	copy(contentCopy, e.content)
	return contentCopy
}

// NewFileParseRequestedEvent constructs a parse request event.
func NewFileParseRequestedEvent(
	path string,
	content []byte,
	occurredAt time.Time,
) (*FileParseRequestedEvent, error) {
	if path == "" {
		return nil, fmt.Errorf("file path is required")
	}
	base, err := newBaseEvent("FileParseRequested", path, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of content
	contentCopy := make([]byte, len(content))
	copy(contentCopy, content)
	return &FileParseRequestedEvent{
		baseEvent: base,
		content:   contentCopy,
	}, nil
}

// MustNewFileParseRequestedEvent panics when construction fails.
func MustNewFileParseRequestedEvent(
	path string,
	content []byte,
	occurredAt time.Time,
) *FileParseRequestedEvent {
	event, err := NewFileParseRequestedEvent(path, content, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Content returns a defensive copy of the file content to parse.
func (e *FileParseRequestedEvent) Content() []byte {
	contentCopy := make([]byte, len(e.content))
	copy(contentCopy, e.content)
	return contentCopy
}

// NewNoteParsedEvent constructs a note parsed event.
func NewNoteParsedEvent(
	note Note,
	occurredAt time.Time,
) (*NoteParsedEvent, error) {
	if note.Path == "" {
		return nil, fmt.Errorf("note path is required")
	}
	base, err := newBaseEvent("NoteParsed", note.Path, occurredAt)
	if err != nil {
		return nil, err
	}
	return &NoteParsedEvent{
		baseEvent: base,
		note:      note,
	}, nil
}

// MustNewNoteParsedEvent panics when construction fails.
func MustNewNoteParsedEvent(
	note Note,
	occurredAt time.Time,
) *NoteParsedEvent {
	event, err := NewNoteParsedEvent(note, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Note returns a copy of the parsed note.
func (e *NoteParsedEvent) Note() Note {
	return e.note
}

// NewFrontmatterValidationRequestedEvent constructs a validation request event.
func NewFrontmatterValidationRequestedEvent(
	note Note,
	occurredAt time.Time,
) (*FrontmatterValidationRequestedEvent, error) {
	if note.Path == "" {
		return nil, fmt.Errorf("note path is required")
	}
	base, err := newBaseEvent(
		"FrontmatterValidationRequested",
		note.Path,
		occurredAt,
	)
	if err != nil {
		return nil, err
	}
	return &FrontmatterValidationRequestedEvent{
		baseEvent: base,
		note:      note,
	}, nil
}

// MustNewFrontmatterValidationRequestedEvent panics when construction fails.
func MustNewFrontmatterValidationRequestedEvent(
	note Note,
	occurredAt time.Time,
) *FrontmatterValidationRequestedEvent {
	event, err := NewFrontmatterValidationRequestedEvent(note, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Note returns a copy of the note to validate.
func (e *FrontmatterValidationRequestedEvent) Note() Note {
	return e.note
}

// NewNoteCacheRequestedEvent constructs a cache request event.
func NewNoteCacheRequestedEvent(
	note Note,
	occurredAt time.Time,
) (*NoteCacheRequestedEvent, error) {
	if note.Path == "" {
		return nil, fmt.Errorf("note path is required")
	}
	base, err := newBaseEvent("NoteCacheRequested", note.Path, occurredAt)
	if err != nil {
		return nil, err
	}
	return &NoteCacheRequestedEvent{
		baseEvent: base,
		note:      note,
	}, nil
}

// MustNewNoteCacheRequestedEvent panics when construction fails.
func MustNewNoteCacheRequestedEvent(
	note Note,
	occurredAt time.Time,
) *NoteCacheRequestedEvent {
	event, err := NewNoteCacheRequestedEvent(note, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Note returns a copy of the note to cache.
func (e *NoteCacheRequestedEvent) Note() Note {
	return e.note
}

// NewLookupPerformedEvent constructs a lookup event.
func NewLookupPerformedEvent(
	noteID string,
	resultCount int,
	duration time.Duration,
	lookupType string,
	occurredAt time.Time,
) (*LookupPerformedEvent, error) {
	if noteID == "" {
		return nil, fmt.Errorf("note ID is required")
	}
	if resultCount < 0 {
		return nil, fmt.Errorf("result count must be >= 0")
	}
	if lookupType == "" {
		return nil, fmt.Errorf("lookup type is required")
	}
	base, err := newBaseEvent("LookupPerformed", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	return &LookupPerformedEvent{
		baseEvent:   base,
		noteID:      noteID,
		resultCount: resultCount,
		duration:    duration,
		lookupType:  lookupType,
	}, nil
}

// MustNewLookupPerformedEvent panics when construction fails.
func MustNewLookupPerformedEvent(
	noteID string,
	resultCount int,
	duration time.Duration,
	lookupType string,
	occurredAt time.Time,
) *LookupPerformedEvent {
	event, err := NewLookupPerformedEvent(
		noteID,
		resultCount,
		duration,
		lookupType,
		occurredAt,
	)
	if err != nil {
		panic(err)
	}
	return event
}

// NoteID returns the identifier of the looked-up note.
func (e *LookupPerformedEvent) NoteID() string {
	return e.noteID
}

// ResultCount returns the number of results found.
func (e *LookupPerformedEvent) ResultCount() int {
	return e.resultCount
}

// Duration returns the lookup duration.
func (e *LookupPerformedEvent) Duration() time.Duration {
	return e.duration
}

// LookupType returns the type of lookup performed.
func (e *LookupPerformedEvent) LookupType() string {
	return e.lookupType
}

// NewQueryPerformedEvent constructs a query event.
func NewQueryPerformedEvent(
	filterCriteria map[string]any,
	resultCount int,
	duration time.Duration,
	queryType string,
	occurredAt time.Time,
) (*QueryPerformedEvent, error) {
	if len(filterCriteria) == 0 {
		return nil, fmt.Errorf("filter criteria is required")
	}
	if resultCount < 0 {
		return nil, fmt.Errorf("result count must be >= 0")
	}
	if queryType == "" {
		return nil, fmt.Errorf("query type is required")
	}
	base, err := newBaseEvent("QueryPerformed", "query", occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of filter criteria
	criteria := make(map[string]any, len(filterCriteria))
	for k, v := range filterCriteria {
		criteria[k] = v
	}
	return &QueryPerformedEvent{
		baseEvent:      base,
		filterCriteria: criteria,
		resultCount:    resultCount,
		duration:       duration,
		queryType:      queryType,
	}, nil
}

// MustNewQueryPerformedEvent panics when construction fails.
func MustNewQueryPerformedEvent(
	filterCriteria map[string]any,
	resultCount int,
	duration time.Duration,
	queryType string,
	occurredAt time.Time,
) *QueryPerformedEvent {
	event, err := NewQueryPerformedEvent(
		filterCriteria,
		resultCount,
		duration,
		queryType,
		occurredAt,
	)
	if err != nil {
		panic(err)
	}
	return event
}

// FilterCriteria returns a defensive copy of the filter criteria.
func (e *QueryPerformedEvent) FilterCriteria() map[string]any {
	criteria := make(map[string]any, len(e.filterCriteria))
	for k, v := range e.filterCriteria {
		criteria[k] = v
	}
	return criteria
}

// ResultCount returns the number of results found.
func (e *QueryPerformedEvent) ResultCount() int {
	return e.resultCount
}

// Duration returns the query duration.
func (e *QueryPerformedEvent) Duration() time.Duration {
	return e.duration
}

// QueryType returns the type of query performed.
func (e *QueryPerformedEvent) QueryType() string {
	return e.queryType
}

// NewSchemaLookupEvent constructs a schema lookup event.
func NewSchemaLookupEvent(
	noteID string,
	schemaName string,
	found bool,
	duration time.Duration,
	occurredAt time.Time,
) (*SchemaLookupEvent, error) {
	if noteID == "" {
		return nil, fmt.Errorf("note ID is required")
	}
	base, err := newBaseEvent("SchemaLookup", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	return &SchemaLookupEvent{
		baseEvent:  base,
		noteID:     noteID,
		schemaName: schemaName,
		found:      found,
		duration:   duration,
	}, nil
}

// MustNewSchemaLookupEvent panics when construction fails.
func MustNewSchemaLookupEvent(
	noteID string,
	schemaName string,
	found bool,
	duration time.Duration,
	occurredAt time.Time,
) *SchemaLookupEvent {
	event, err := NewSchemaLookupEvent(
		noteID,
		schemaName,
		found,
		duration,
		occurredAt,
	)
	if err != nil {
		panic(err)
	}
	return event
}

// NoteID returns the identifier of the note.
func (e *SchemaLookupEvent) NoteID() string {
	return e.noteID
}

// SchemaName returns the resolved schema name.
func (e *SchemaLookupEvent) SchemaName() string {
	return e.schemaName
}

// Found indicates if the schema was found.
func (e *SchemaLookupEvent) Found() bool {
	return e.found
}

// Duration returns the lookup duration.
func (e *SchemaLookupEvent) Duration() time.Duration {
	return e.duration
}

// NewValidationPerformedEvent constructs a validation event.
func NewValidationPerformedEvent(
	noteID string,
	schemaName string,
	valid bool,
	duration time.Duration,
	errors []string,
	occurredAt time.Time,
) (*ValidationPerformedEvent, error) {
	if noteID == "" {
		return nil, fmt.Errorf("note ID is required")
	}
	if schemaName == "" {
		return nil, fmt.Errorf("schema name is required")
	}
	base, err := newBaseEvent("ValidationPerformed", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of errors
	errs := make([]string, len(errors))
	copy(errs, errors)
	return &ValidationPerformedEvent{
		baseEvent:  base,
		noteID:     noteID,
		schemaName: schemaName,
		valid:      valid,
		duration:   duration,
		errors:     errs,
	}, nil
}

// MustNewValidationPerformedEvent panics when construction fails.
func MustNewValidationPerformedEvent(
	noteID string,
	schemaName string,
	valid bool,
	duration time.Duration,
	errors []string,
	occurredAt time.Time,
) *ValidationPerformedEvent {
	event, err := NewValidationPerformedEvent(
		noteID,
		schemaName,
		valid,
		duration,
		errors,
		occurredAt,
	)
	if err != nil {
		panic(err)
	}
	return event
}

// NoteID returns the identifier of the validated note.
func (e *ValidationPerformedEvent) NoteID() string {
	return e.noteID
}

// SchemaName returns the schema used for validation.
func (e *ValidationPerformedEvent) SchemaName() string {
	return e.schemaName
}

// IsValid indicates if validation succeeded.
func (e *ValidationPerformedEvent) IsValid() bool {
	return e.valid
}

// Duration returns the validation duration.
func (e *ValidationPerformedEvent) Duration() time.Duration {
	return e.duration
}

// Errors returns a defensive copy of validation errors.
func (e *ValidationPerformedEvent) Errors() []string {
	errs := make([]string, len(e.errors))
	copy(errs, e.errors)
	return errs
}

// NewValidationFailedEvent constructs a validation failure event.
func NewValidationFailedEvent(
	noteID string,
	schemaName string,
	errors []string,
	remediationHints []string,
	duration time.Duration,
	occurredAt time.Time,
) (*ValidationFailedEvent, error) {
	if noteID == "" {
		return nil, fmt.Errorf("note ID is required")
	}
	if schemaName == "" {
		return nil, fmt.Errorf("schema name is required")
	}
	if len(errors) == 0 {
		return nil, fmt.Errorf("errors are required for failure event")
	}
	base, err := newBaseEvent("ValidationFailed", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copies
	errs := make([]string, len(errors))
	copy(errs, errors)
	hints := make([]string, len(remediationHints))
	copy(hints, remediationHints)
	return &ValidationFailedEvent{
		baseEvent:        base,
		noteID:           noteID,
		schemaName:       schemaName,
		errors:           errs,
		remediationHints: hints,
		duration:         duration,
	}, nil
}

// MustNewValidationFailedEvent panics when construction fails.
func MustNewValidationFailedEvent(
	noteID string,
	schemaName string,
	errors []string,
	remediationHints []string,
	duration time.Duration,
	occurredAt time.Time,
) *ValidationFailedEvent {
	event, err := NewValidationFailedEvent(
		noteID,
		schemaName,
		errors,
		remediationHints,
		duration,
		occurredAt,
	)
	if err != nil {
		panic(err)
	}
	return event
}

// NoteID returns the identifier of the note that failed validation.
func (e *ValidationFailedEvent) NoteID() string {
	return e.noteID
}

// SchemaName returns the schema used for validation.
func (e *ValidationFailedEvent) SchemaName() string {
	return e.schemaName
}

// Errors returns a defensive copy of validation errors.
func (e *ValidationFailedEvent) Errors() []string {
	errs := make([]string, len(e.errors))
	copy(errs, e.errors)
	return errs
}

// RemediationHints returns a defensive copy of remediation hints.
func (e *ValidationFailedEvent) RemediationHints() []string {
	hints := make([]string, len(e.remediationHints))
	copy(hints, e.remediationHints)
	return hints
}

// Duration returns the validation duration.
func (e *ValidationFailedEvent) Duration() time.Duration {
	return e.duration
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
	base, err := newBaseEvent("NoteCreated", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	return &NoteCreatedEvent{
		baseEvent:  base,
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
	base, err := newBaseEvent("SchemaUpdated", schemaName, occurredAt)
	if err != nil {
		return nil, err
	}
	return &SchemaUpdatedEvent{
		baseEvent:  base,
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
