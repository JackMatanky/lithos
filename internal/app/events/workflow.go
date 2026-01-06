package events

import (
	"fmt"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
)

// CommandIssuedEvent represents a command invocation emitted by drivers
// (e.g., CLICommander) for decoupled orchestration.
type CommandIssuedEvent struct {
	domain.BaseEvent

	command string
	payload map[string]string
}

// FileDiscoveredEvent is emitted when a new file is discovered during vault
// scanning.
type FileDiscoveredEvent struct {
	domain.BaseEvent

	path    string
	size    int
	content []byte
}

// FileParseRequestedEvent requests parsing of a discovered file.
type FileParseRequestedEvent struct {
	domain.BaseEvent

	content []byte
}

// NoteParsedEvent is emitted when a file has been successfully parsed into a
// Note.
type NoteParsedEvent struct {
	domain.BaseEvent

	note domain.Note
}

// FrontmatterValidationRequestedEvent requests validation of frontmatter.
type FrontmatterValidationRequestedEvent struct {
	domain.BaseEvent

	note domain.Note
}

// NoteCacheRequestedEvent requests caching of a validated note.
type NoteCacheRequestedEvent struct {
	domain.BaseEvent

	note domain.Note
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
	base, err := domain.NewBaseEvent("CommandIssued", command, occurredAt)
	if err != nil {
		return nil, err
	}
	payloadCopy := make(map[string]string, len(payload))
	for k, v := range payload {
		payloadCopy[k] = v
	}
	return &CommandIssuedEvent{
		BaseEvent: base,
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
	base, err := domain.NewBaseEvent("FileDiscovered", path, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of content
	contentCopy := make([]byte, len(content))
	copy(contentCopy, content)
	return &FileDiscoveredEvent{
		BaseEvent: base,
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
	base, err := domain.NewBaseEvent("FileParseRequested", path, occurredAt)
	if err != nil {
		return nil, err
	}
	// Defensive copy of content
	contentCopy := make([]byte, len(content))
	copy(contentCopy, content)
	return &FileParseRequestedEvent{
		BaseEvent: base,
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
	note domain.Note,
	occurredAt time.Time,
) (*NoteParsedEvent, error) {
	if note.Path == "" {
		return nil, fmt.Errorf("note path is required")
	}
	base, err := domain.NewBaseEvent("NoteParsed", note.Path, occurredAt)
	if err != nil {
		return nil, err
	}
	return &NoteParsedEvent{
		BaseEvent: base,
		note:      note,
	}, nil
}

// MustNewNoteParsedEvent panics when construction fails.
func MustNewNoteParsedEvent(
	note domain.Note,
	occurredAt time.Time,
) *NoteParsedEvent {
	event, err := NewNoteParsedEvent(note, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Note returns a copy of the parsed note.
func (e *NoteParsedEvent) Note() domain.Note {
	return e.note
}

// NewFrontmatterValidationRequestedEvent constructs a validation request event.
func NewFrontmatterValidationRequestedEvent(
	note domain.Note,
	occurredAt time.Time,
) (*FrontmatterValidationRequestedEvent, error) {
	if note.Path == "" {
		return nil, fmt.Errorf("note path is required")
	}
	base, err := domain.NewBaseEvent(
		"FrontmatterValidationRequested",
		note.Path,
		occurredAt,
	)
	if err != nil {
		return nil, err
	}
	return &FrontmatterValidationRequestedEvent{
		BaseEvent: base,
		note:      note,
	}, nil
}

// MustNewFrontmatterValidationRequestedEvent panics when construction fails.
func MustNewFrontmatterValidationRequestedEvent(
	note domain.Note,
	occurredAt time.Time,
) *FrontmatterValidationRequestedEvent {
	event, err := NewFrontmatterValidationRequestedEvent(note, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Note returns a copy of the note to validate.
func (e *FrontmatterValidationRequestedEvent) Note() domain.Note {
	return e.note
}

// NewNoteCacheRequestedEvent constructs a cache request event.
func NewNoteCacheRequestedEvent(
	note domain.Note,
	occurredAt time.Time,
) (*NoteCacheRequestedEvent, error) {
	if note.Path == "" {
		return nil, fmt.Errorf("note path is required")
	}
	base, err := domain.NewBaseEvent(
		"NoteCacheRequested",
		note.Path,
		occurredAt,
	)
	if err != nil {
		return nil, err
	}
	return &NoteCacheRequestedEvent{
		BaseEvent: base,
		note:      note,
	}, nil
}

// MustNewNoteCacheRequestedEvent panics when construction fails.
func MustNewNoteCacheRequestedEvent(
	note domain.Note,
	occurredAt time.Time,
) *NoteCacheRequestedEvent {
	event, err := NewNoteCacheRequestedEvent(note, occurredAt)
	if err != nil {
		panic(err)
	}
	return event
}

// Note returns a copy of the note to cache.
func (e *NoteCacheRequestedEvent) Note() domain.Note {
	return e.note
}
