package events_test

import (
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/require"
)

func TestCommandIssuedEvent(t *testing.T) {
	now := time.Now()
	payload := map[string]string{"request_id": "abc-123"}
	event := events.MustNewCommandIssuedEvent("IndexVault", payload, now)

	require.Equal(t, "CommandIssued", event.EventType())
	require.Equal(t, "IndexVault", event.AggregateID())
	require.Equal(t, "IndexVault", event.Command())
	require.Equal(t, "abc-123", event.Payload()["request_id"])
	require.Equal(t, now, event.OccurredAt())

	t.Run("empty command", func(t *testing.T) {
		_, err := events.NewCommandIssuedEvent("", nil, time.Now())
		require.Error(t, err)
	})
}

func TestFileDiscoveredEvent(t *testing.T) {
	now := time.Now()
	content := []byte("test content")
	event := events.MustNewFileDiscoveredEvent(
		"notes/test.md",
		12,
		content,
		now,
	)

	require.Equal(t, "FileDiscovered", event.EventType())
	require.Equal(t, "notes/test.md", event.AggregateID())
	require.Equal(t, "notes/test.md", event.Path())
	require.Equal(t, 12, event.Size())
	require.Equal(t, content, event.Content())
	require.Equal(t, now, event.OccurredAt())
}

func TestFileParseRequestedEvent(t *testing.T) {
	now := time.Now()
	content := []byte("test content")
	event := events.MustNewFileParseRequestedEvent(
		"notes/test.md",
		content,
		now,
	)

	require.Equal(t, "FileParseRequested", event.EventType())
	require.Equal(t, "notes/test.md", event.AggregateID())
	require.Equal(t, content, event.Content())
	require.Equal(t, now, event.OccurredAt())
}

func TestNoteParsedEvent(t *testing.T) {
	now := time.Now()
	note := mustNewTestNote(t, "notes/test.md", "note")
	event := events.MustNewNoteParsedEvent(note, now)

	require.Equal(t, "NoteParsed", event.EventType())
	require.Equal(t, "notes/test.md", event.AggregateID())
	require.Equal(t, note.Path, event.Note().Path)
	require.Equal(t, now, event.OccurredAt())
}

func TestFrontmatterValidationRequestedEvent(t *testing.T) {
	now := time.Now()
	note := mustNewTestNote(t, "notes/test.md", "note")
	event := events.MustNewFrontmatterValidationRequestedEvent(note, now)

	require.Equal(t, "FrontmatterValidationRequested", event.EventType())
	require.Equal(t, "notes/test.md", event.AggregateID())
	require.Equal(t, note.Path, event.Note().Path)
	require.Equal(t, now, event.OccurredAt())
}

func TestNoteCacheRequestedEvent(t *testing.T) {
	now := time.Now()
	note := mustNewTestNote(t, "notes/test.md", "note")
	event := events.MustNewNoteCacheRequestedEvent(note, now)

	require.Equal(t, "NoteCacheRequested", event.EventType())
	require.Equal(t, "notes/test.md", event.AggregateID())
	require.Equal(t, note.Path, event.Note().Path)
	require.Equal(t, now, event.OccurredAt())
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
