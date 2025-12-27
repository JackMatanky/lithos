package domain_test

import (
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/require"
)

func TestDomainEventImplementations(t *testing.T) {
	note := mustNewTestNote(t, "notes/contact.md", "contact")
	summary := domain.VaultIndexingSummary{ScannedCount: 42, IndexedCount: 42}
	events := []domain.DomainEvent{
		domain.MustNewNoteIndexedEvent(note, time.Now()),
		domain.MustNewVaultIndexingCompleteEvent(
			summary,
			time.Second,
			time.Now(),
		),
		domain.MustNewFrontmatterValidatedEvent(
			domain.Note{Path: "note-999"},
			"contact",
			true,
			nil,
			time.Now(),
		),
		domain.MustNewSchemaLoadedEvent("contact", 10, time.Now()),
		domain.MustNewSchemasReloadedEvent(5, time.Now()),
		domain.MustNewCommandIssuedEvent(
			"IndexVault",
			map[string]string{"request_id": "abc"},
			time.Now(),
		),
		domain.MustNewLookupPerformedEvent(
			"note-123",
			1,
			time.Millisecond*100,
			"basename",
			time.Now(),
		),
		domain.MustNewQueryPerformedEvent(
			map[string]any{"author": "John"},
			5,
			time.Millisecond*50,
			"frontmatter",
			time.Now(),
		),
		domain.MustNewSchemaLookupEvent(
			"note-456",
			"contact",
			true,
			time.Millisecond*25,
			time.Now(),
		),
		domain.MustNewValidationPerformedEvent(
			"note-789",
			"contact",
			true,
			time.Millisecond*75,
			nil,
			time.Now(),
		),
		domain.MustNewValidationFailedEvent(
			"note-101",
			"meeting",
			[]string{"missing title"},
			[]string{"Add title field"},
			time.Millisecond*60,
			time.Now(),
		),
		domain.MustNewNoteCreatedEvent(
			"note-202",
			"contact",
			"contact-template",
			time.Now(),
		),
		domain.MustNewSchemaUpdatedEvent("contact", "updated", time.Now()),
	}

	for _, evt := range events {
		require.NotEmpty(t, evt.EventType())
		require.False(t, evt.OccurredAt().IsZero())
		require.NotEmpty(t, evt.AggregateID())
	}
}

func TestNoteIndexedEvent(t *testing.T) {
	now := time.Now()
	note := mustNewTestNote(t, "notes/meeting.md", "meeting")
	event := domain.MustNewNoteIndexedEvent(note, now)

	require.Equal(t, "NoteIndexed", event.EventType())
	require.Equal(t, "notes/meeting.md", event.AggregateID())
	require.Equal(t, now, event.OccurredAt())
	require.Equal(t, "notes/meeting.md", event.Path())
	require.Equal(t, "meeting", event.FileClass())
	require.Equal(t, note.Path, event.Note().Path)
}

func TestNoteIndexedEventValidation(t *testing.T) {
	t.Run("empty path", func(t *testing.T) {
		note := domain.Note{
			Path: "",
			Frontmatter: domain.NewFrontmatter(
				map[string]interface{}{"file_class": "contact"},
			),
		}
		_, err := domain.NewNoteIndexedEvent(note, time.Now())
		require.Error(t, err)
	})

	t.Run("missing file class", func(t *testing.T) {
		note := domain.Note{
			Path:        "notes/test.md",
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{}),
		}
		_, err := domain.NewNoteIndexedEvent(note, time.Now())
		require.Error(t, err)
	})
}

func TestVaultIndexingCompleteEvent(t *testing.T) {
	now := time.Now()
	summary := domain.VaultIndexingSummary{
		ScannedCount:        260,
		IndexedCount:        250,
		ParseFailures:       2,
		CacheFailures:       1,
		ValidationSuccesses: 248,
		ValidationFailures:  2,
	}
	event := domain.MustNewVaultIndexingCompleteEvent(
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

func TestVaultIndexingCompleteValidation(t *testing.T) {
	summary := domain.VaultIndexingSummary{ScannedCount: 1, IndexedCount: -1}
	_, err := domain.NewVaultIndexingCompleteEvent(
		summary,
		time.Second,
		time.Now(),
	)
	require.Error(t, err)
}

func TestFrontmatterValidatedEvent(t *testing.T) {
	now := time.Now()
	errors := []string{"missing title"}
	event := domain.MustNewFrontmatterValidatedEvent(
		domain.Note{Path: "note-33"},
		"contact",
		false,
		errors,
		now,
	)

	require.Equal(t, "FrontmatterValidated", event.EventType())
	require.Equal(t, "note-33", event.AggregateID())
	require.Equal(t, "note-33", event.NoteID())
	require.Equal(t, "contact", event.SchemaName())
	require.False(t, event.IsValid())
	require.Equal(t, errors, event.Errors())
	require.Equal(t, errors, event.ValidationErrors())
	require.Equal(t, "note-33", event.Note().Path)
}

func TestFrontmatterValidatedValidation(t *testing.T) {
	_, err := domain.NewFrontmatterValidatedEvent(
		domain.Note{Path: ""},
		"contact",
		true,
		nil,
		time.Now(),
	)
	require.Error(t, err)
}

func TestSchemaLoadedEvent(t *testing.T) {
	now := time.Now()
	event := domain.MustNewSchemaLoadedEvent("contact", 12, now)

	require.Equal(t, "SchemaLoaded", event.EventType())
	require.Equal(t, "contact", event.AggregateID())
	require.Equal(t, "contact", event.SchemaName())
	require.Equal(t, 12, event.PropertyCount())
}

func TestSchemaLoadedValidation(t *testing.T) {
	_, err := domain.NewSchemaLoadedEvent("", 1, time.Now())
	require.Error(t, err)
}

func TestSchemasReloadedEvent(t *testing.T) {
	now := time.Now()
	event := domain.MustNewSchemasReloadedEvent(7, now)

	require.Equal(t, "SchemasReloaded", event.EventType())
	require.Equal(t, "schemas", event.AggregateID())
	require.Equal(t, 7, event.SchemaCount())
}

func TestSchemasReloadedValidation(t *testing.T) {
	_, err := domain.NewSchemasReloadedEvent(0, time.Now())
	require.Error(t, err)
}

func TestCommandIssuedEvent(t *testing.T) {
	now := time.Now()
	payload := map[string]string{"request_id": "abc-123"}
	event := domain.MustNewCommandIssuedEvent("IndexVault", payload, now)

	require.Equal(t, "CommandIssued", event.EventType())
	require.Equal(t, "IndexVault", event.AggregateID())
	require.Equal(t, "IndexVault", event.Command())
	require.Equal(t, "abc-123", event.Payload()["request_id"])
	require.Equal(t, now, event.OccurredAt())

	t.Run("empty command", func(t *testing.T) {
		_, err := domain.NewCommandIssuedEvent("", nil, time.Now())
		require.Error(t, err)
	})
}

func mustNewTestNote(t *testing.T, path, fileClass string) domain.Note {
	t.Helper()
	frontmatter := domain.NewFrontmatter(map[string]interface{}{
		"file_class": fileClass,
	})
	note, err := domain.NewNote(path, frontmatter, nil, nil, nil, nil)
	require.NoError(t, err)
	return note
}
