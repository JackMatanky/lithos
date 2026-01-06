package events_test

import (
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/stretchr/testify/require"
)

func TestVaultIndexingCompleteEvent(t *testing.T) {
	now := time.Now()
	summary := events.VaultIndexingSummary{
		ScannedCount:        260,
		IndexedCount:        250,
		ParseFailures:       2,
		CacheFailures:       1,
		ValidationSuccesses: 248,
		ValidationFailures:  2,
	}
	event := events.MustNewVaultIndexingCompleteEvent(
		summary,
		1500*time.Millisecond,
		now,
	)

	require.Equal(t, "VaultIndexingComplete", event.EventType())
	require.Equal(t, "vault", event.AggregateID())
	require.Equal(t, summary.IndexedCount, event.NotesIndexed())
	require.Equal(t, summary, event.Summary())
	require.Equal(t, summary.ScannedCount, event.ScannedCount())
	require.Equal(t, summary.ParseFailures, event.ParseFailures())
	require.Equal(t, summary.CacheFailures, event.CacheFailures())
	require.Equal(t, summary.ValidationSuccesses, event.ValidationSuccesses())
	require.Equal(t, summary.ValidationFailures, event.ValidationFailures())
	require.Equal(t, 1500*time.Millisecond, event.Duration())
	require.Equal(t, now, event.OccurredAt())
}

func TestLookupPerformedEvent(t *testing.T) {
	now := time.Now()
	event := events.MustNewLookupPerformedEvent(
		"note-123",
		1,
		time.Millisecond*100,
		"basename",
		now,
	)

	require.Equal(t, "LookupPerformed", event.EventType())
	require.Equal(t, "note-123", event.AggregateID())
	require.Equal(t, "note-123", event.NoteID())
	require.Equal(t, 1, event.ResultCount())
	require.Equal(t, time.Millisecond*100, event.Duration())
	require.Equal(t, "basename", event.LookupType())
	require.Equal(t, now, event.OccurredAt())
}

func TestQueryPerformedEvent(t *testing.T) {
	now := time.Now()
	filter := map[string]any{"author": "John"}
	event := events.MustNewQueryPerformedEvent(
		filter,
		5,
		time.Millisecond*50,
		"frontmatter",
		now,
	)

	require.Equal(t, "QueryPerformed", event.EventType())
	require.Equal(t, "query", event.AggregateID())
	require.Equal(t, filter, event.FilterCriteria())
	require.Equal(t, 5, event.ResultCount())
	require.Equal(t, time.Millisecond*50, event.Duration())
	require.Equal(t, "frontmatter", event.QueryType())
	require.Equal(t, now, event.OccurredAt())
}

func TestSchemaLookupEvent(t *testing.T) {
	now := time.Now()
	event := events.MustNewSchemaLookupEvent(
		"note-456",
		"contact",
		true,
		time.Millisecond*25,
		now,
	)

	require.Equal(t, "SchemaLookup", event.EventType())
	require.Equal(t, "note-456", event.AggregateID())
	require.Equal(t, "note-456", event.NoteID())
	require.Equal(t, "contact", event.SchemaName())
	require.True(t, event.Found())
	require.Equal(t, time.Millisecond*25, event.Duration())
	require.Equal(t, now, event.OccurredAt())
}

func TestValidationPerformedEvent(t *testing.T) {
	now := time.Now()
	errs := []string{"error 1"}
	event := events.MustNewValidationPerformedEvent(
		"note-789",
		"contact",
		true,
		time.Millisecond*75,
		errs,
		now,
	)

	require.Equal(t, "ValidationPerformed", event.EventType())
	require.Equal(t, "note-789", event.AggregateID())
	require.Equal(t, "note-789", event.NoteID())
	require.Equal(t, "contact", event.SchemaName())
	require.True(t, event.IsValid())
	require.Equal(t, time.Millisecond*75, event.Duration())
	require.Equal(t, errs, event.Errors())
	require.Equal(t, now, event.OccurredAt())
}

func TestValidationFailedEvent(t *testing.T) {
	now := time.Now()
	errs := []string{"missing title"}
	hints := []string{"Add title field"}
	event := events.MustNewValidationFailedEvent(
		"note-101",
		"meeting",
		errs,
		hints,
		time.Millisecond*60,
		now,
	)

	require.Equal(t, "ValidationFailed", event.EventType())
	require.Equal(t, "note-101", event.AggregateID())
	require.Equal(t, "note-101", event.NoteID())
	require.Equal(t, "meeting", event.SchemaName())
	require.Equal(t, errs, event.Errors())
	require.Equal(t, hints, event.RemediationHints())
	require.Equal(t, time.Millisecond*60, event.Duration())
	require.Equal(t, now, event.OccurredAt())
}
