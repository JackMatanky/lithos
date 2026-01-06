package domain_test

import (
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/require"
)

func TestDomainEventImplementations(t *testing.T) {
	note := mustNewTestNote(t, "notes/contact.md", "contact")
	events := []domain.DomainEvent{
		domain.MustNewNoteIndexedEvent(note, time.Now()),
		domain.MustNewFrontmatterValidatedEvent(
			domain.Note{Path: "note-999"},
			"contact",
			true,
			nil,
			time.Now(),
		),
		domain.MustNewSchemaLoadedEvent("contact", 10, time.Now()),
		domain.MustNewSchemasReloadedEvent(5, time.Now()),
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

func mustNewTestNote(t *testing.T, path, fileClass string) domain.Note {
	t.Helper()
	frontmatter := domain.NewFrontmatter(map[string]interface{}{
		"file_class": fileClass,
	})
	note, err := domain.NewNote(path, frontmatter, nil, nil, nil, nil)
	require.NoError(t, err)
	return note
}
