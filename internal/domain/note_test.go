package domain

import (
	"encoding/json"
	"reflect"
	"testing"
	"time"
)

const (
	testContent1 = "content1"
	testContent2 = "content2"
	testContent3 = "content3"
)

// TestNewNoteID tests the NewNoteID constructor function.
func TestNewNoteID(t *testing.T) {
	tests := []struct {
		name     string
		value    string
		expected NoteID
	}{
		{
			name:     "creates valid NoteID from string",
			value:    "test-note",
			expected: NoteID("test-note"),
		},
		{
			name:     "handles empty string",
			value:    "",
			expected: NoteID(""),
		},
		{
			name:     "handles special characters",
			value:    "note/with/path.md",
			expected: NoteID("note/with/path.md"),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := NewNoteID(tt.value)
			if result != tt.expected {
				t.Errorf(
					"NewNoteID(%q) = %v, want %v",
					tt.value,
					result,
					tt.expected,
				)
			}
		})
	}
}

// TestNoteID_String tests the String method of NoteID.
func TestNoteID_String(t *testing.T) {
	tests := []struct {
		name     string
		noteID   NoteID
		expected string
	}{
		{
			name:     "returns underlying string value",
			noteID:   NoteID("test-note"),
			expected: "test-note",
		},
		{
			name:     "handles empty NoteID",
			noteID:   NoteID(""),
			expected: "",
		},
		{
			name:     "handles special characters",
			noteID:   NoteID("note/with/path.md"),
			expected: "note/with/path.md",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.noteID.String()
			if result != tt.expected {
				t.Errorf(
					"NoteID(%q).String() = %q, want %q",
					tt.noteID,
					result,
					tt.expected,
				)
			}
		})
	}
}

// TestNoteID_AsMapKey tests that NoteID can be used as a map key.
func TestNoteID_AsMapKey(t *testing.T) {
	// Test that NoteID can be used as a map key
	noteMap := make(map[NoteID]string)

	id1 := NoteID("note1")
	id2 := NoteID("note2")
	id3 := NoteID("note1") // Same value as id1

	noteMap[id1] = testContent1
	noteMap[id2] = testContent2
	noteMap[id3] = testContent3 // Should overwrite id1

	if len(noteMap) != 2 {
		t.Errorf("Expected map length 2, got %d", len(noteMap))
	}

	if noteMap[id1] != testContent3 {
		t.Errorf(
			"Expected id1 to be overwritten to %q, got %q",
			testContent3,
			noteMap[id1],
		)
	}

	if noteMap[id2] != testContent2 {
		t.Errorf(
			"Expected id2 to remain %q, got %q",
			testContent2,
			noteMap[id2],
		)
	}
}

// TestNewFrontmatter tests the NewFrontmatter constructor function.
func TestNewFrontmatter(t *testing.T) {
	tests := []struct {
		name     string
		fields   map[string]interface{}
		expected Frontmatter
	}{
		{
			name: "creates Frontmatter with fileClass present",
			fields: map[string]interface{}{
				"fileClass": "contact",
				"name":      "John Doe",
				"email":     "john@example.com",
			},
			expected: Frontmatter{
				FileClass: "contact",
				Fields: map[string]interface{}{
					"fileClass": "contact",
					"name":      "John Doe",
					"email":     "john@example.com",
				},
			},
		},
		{
			name: "creates Frontmatter with fileClass missing",
			fields: map[string]interface{}{
				"name":  "Jane Doe",
				"email": "jane@example.com",
			},
			expected: Frontmatter{
				FileClass: "",
				Fields: map[string]interface{}{
					"name":  "Jane Doe",
					"email": "jane@example.com",
				},
			},
		},
		{
			name:   "handles empty fields map",
			fields: map[string]interface{}{},
			expected: Frontmatter{
				FileClass: "",
				Fields:    map[string]interface{}{},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := NewFrontmatter(tt.fields)
			if result.FileClass != tt.expected.FileClass {
				t.Errorf(
					"NewFrontmatter(%v).FileClass = %q, want %q",
					tt.fields,
					result.FileClass,
					tt.expected.FileClass,
				)
			}
			if len(result.Fields) != len(tt.expected.Fields) {
				t.Errorf(
					"NewFrontmatter(%v).Fields length = %d, want %d",
					tt.fields,
					len(result.Fields),
					len(tt.expected.Fields),
				)
			}
			for k, v := range tt.expected.Fields {
				if result.Fields[k] != v {
					t.Errorf(
						"NewFrontmatter(%v).Fields[%q] = %v, want %v",
						tt.fields,
						k,
						result.Fields[k],
						v,
					)
				}
			}
		})
	}
}

// TestFrontmatter_SchemaName tests the SchemaName method of Frontmatter.
func TestFrontmatter_SchemaName(t *testing.T) {
	tests := []struct {
		name        string
		frontmatter Frontmatter
		expected    string
	}{
		{
			name: "returns FileClass when present",
			frontmatter: Frontmatter{
				FileClass: "contact",
				Fields:    map[string]interface{}{"fileClass": "contact"},
			},
			expected: "contact",
		},
		{
			name: "returns empty string when FileClass is empty",
			frontmatter: Frontmatter{
				FileClass: "",
				Fields:    map[string]interface{}{},
			},
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.frontmatter.SchemaName()
			if result != tt.expected {
				t.Errorf(
					"Frontmatter.SchemaName() = %q, want %q",
					result,
					tt.expected,
				)
			}
		})
	}
}

// TestNewNote tests the NewNote constructor function.
func TestNewNote(t *testing.T) {
	id := NoteID("test-note")
	frontmatter := Frontmatter{
		FileClass: "contact",
		Fields: map[string]interface{}{
			"fileClass": "contact",
			"name":      "John Doe",
		},
	}

	note := NewNote(id, time.Now(), frontmatter)

	if note.ID != id {
		t.Errorf("NewNote ID = %v, want %v", note.ID, id)
	}
	if note.Frontmatter.FileClass != frontmatter.FileClass {
		t.Errorf("NewNote Frontmatter.FileClass = %q, want %q",
			note.Frontmatter.FileClass, frontmatter.FileClass)
	}
}

// TestNote_SchemaName tests the SchemaName method of Note.
func TestNote_SchemaName(t *testing.T) {
	frontmatter := Frontmatter{
		FileClass: "contact",
		Fields:    map[string]interface{}{},
	}
	note := NewNote(NoteID("test"), time.Now(), frontmatter)

	result := note.SchemaName()
	expected := "contact"

	if result != expected {
		t.Errorf("Note.SchemaName() = %q, want %q", result, expected)
	}
}

// TestNote_NoFileField tests that Note struct has no File field.
func TestNote_NoFileField(t *testing.T) {
	// This test verifies that the Note struct does not have a File field
	// We use reflection to inspect the struct fields
	note := NewNote(NoteID("test"), time.Now(), Frontmatter{})

	v := reflect.ValueOf(note)
	typ := v.Type()

	//nolint:intrange // false positive, cannot range over NumField()
	for i := 0; i < typ.NumField(); i++ {
		field := typ.Field(i)
		if field.Name == "File" {
			t.Errorf(
				"Note struct should not have a File field, but found: %s",
				field.Name,
			)
		}
	}
}

// TestNote_JSONSerialization tests that Note can be serialized to and from
// JSON.
func TestNote_JSONSerialization(t *testing.T) {
	original := NewNote(
		NoteID("test-note"),
		time.Now(),
		Frontmatter{
			FileClass: "contact",
			Fields: map[string]interface{}{
				"fileClass": "contact",
				"name":      "John Doe",
				"email":     "john@example.com",
			},
		},
	)

	// Serialize to JSON
	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("Failed to marshal Note to JSON: %v", err)
	}

	// Deserialize from JSON
	var deserialized Note
	err = json.Unmarshal(data, &deserialized)
	if err != nil {
		t.Fatalf("Failed to unmarshal Note from JSON: %v", err)
	}

	// Verify the round-trip
	if deserialized.ID != original.ID {
		t.Errorf("Deserialized ID = %v, want %v", deserialized.ID, original.ID)
	}
	if deserialized.Frontmatter.FileClass != original.Frontmatter.FileClass {
		t.Errorf(
			"Deserialized FileClass = %q, want %q",
			deserialized.Frontmatter.FileClass,
			original.Frontmatter.FileClass,
		)
	}
}
