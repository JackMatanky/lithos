package events

import (
	"fmt"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
)

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
	domain.BaseEvent

	summary  VaultIndexingSummary
	duration time.Duration
}

// LookupPerformedEvent tracks individual lookup operations in template
// functions.
type LookupPerformedEvent struct {
	domain.BaseEvent

	noteID      string
	resultCount int
	duration    time.Duration
	lookupType  string // "basename" or "id"
}

// QueryPerformedEvent tracks query operations with filter criteria.
type QueryPerformedEvent struct {
	domain.BaseEvent

	filterCriteria map[string]any
	resultCount    int
	duration       time.Duration
	queryType      string // "path" or "frontmatter"
}

// SchemaLookupEvent tracks schema resolution lookups.
type SchemaLookupEvent struct {
	domain.BaseEvent

	noteID     string
	schemaName string
	found      bool
	duration   time.Duration
}

// ValidationPerformedEvent tracks validation outcomes.
type ValidationPerformedEvent struct {
	domain.BaseEvent

	noteID     string
	schemaName string
	valid      bool
	duration   time.Duration
	errors     []string
}

// ValidationFailedEvent tracks validation failures with remediation hints.
type ValidationFailedEvent struct {
	domain.BaseEvent

	noteID           string
	schemaName       string
	errors           []string
	remediationHints []string
	duration         time.Duration
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
	base, err := domain.NewBaseEvent(
		"VaultIndexingComplete",
		"vault",
		occurredAt,
	)
	if err != nil {
		return nil, err
	}
	return &VaultIndexingCompleteEvent{
		BaseEvent: base,
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
	base, err := domain.NewBaseEvent("LookupPerformed", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	return &LookupPerformedEvent{
		BaseEvent:   base,
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
	base, err := domain.NewBaseEvent("QueryPerformed", "query", occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of filter criteria
	criteria := make(map[string]any, len(filterCriteria))
	for k, v := range filterCriteria {
		criteria[k] = v
	}
	return &QueryPerformedEvent{
		BaseEvent:      base,
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
	base, err := domain.NewBaseEvent("SchemaLookup", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	return &SchemaLookupEvent{
		BaseEvent:  base,
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
	base, err := domain.NewBaseEvent("ValidationPerformed", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of errors
	errs := make([]string, len(errors))
	copy(errs, errors)
	return &ValidationPerformedEvent{
		BaseEvent:  base,
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
	base, err := domain.NewBaseEvent("ValidationFailed", noteID, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copies
	errs := make([]string, len(errors))
	copy(errs, errors)
	hints := make([]string, len(remediationHints))
	copy(hints, remediationHints)
	return &ValidationFailedEvent{
		BaseEvent:        base,
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
