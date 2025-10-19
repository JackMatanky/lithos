package domain

import (
	"testing"
	"time"
)

func TestNewNote(t *testing.T) {
	tests := []struct {
		name        string
		file        File
		frontmatter Frontmatter
		expected    Note
	}{
		{
			name: "note with complete data",
			file: File{
				Path:     "/vault/notes/contact.md",
				Basename: "contact",
				Folder:   "/vault/notes",
				ModTime:  time.Date(2024, 1, 1, 12, 0, 0, 0, time.UTC),
			},
			frontmatter: Frontmatter{
				FileClass: "contact",
				Fields: map[string]interface{}{
					"fileClass": "contact",
					"name":      "John Doe",
					"email":     "john@example.com",
				},
			},
			expected: Note{
				File: File{
					Path:     "/vault/notes/contact.md",
					Basename: "contact",
					Folder:   "/vault/notes",
					ModTime:  time.Date(2024, 1, 1, 12, 0, 0, 0, time.UTC),
				},
				Frontmatter: Frontmatter{
					FileClass: "contact",
					Fields: map[string]interface{}{
						"fileClass": "contact",
						"name":      "John Doe",
						"email":     "john@example.com",
					},
				},
			},
		},
		{
			name: "note without schema",
			file: File{
				Path:     "/vault/notes/draft.md",
				Basename: "draft",
				Folder:   "/vault/notes",
				ModTime:  time.Date(2024, 2, 15, 10, 30, 0, 0, time.UTC),
			},
			frontmatter: Frontmatter{
				FileClass: "",
				Fields: map[string]interface{}{
					"title":  "My Draft",
					"author": "Jane Smith",
				},
			},
			expected: Note{
				File: File{
					Path:     "/vault/notes/draft.md",
					Basename: "draft",
					Folder:   "/vault/notes",
					ModTime:  time.Date(2024, 2, 15, 10, 30, 0, 0, time.UTC),
				},
				Frontmatter: Frontmatter{
					FileClass: "",
					Fields: map[string]interface{}{
						"title":  "My Draft",
						"author": "Jane Smith",
					},
				},
			},
		},
		{
			name: "note with empty frontmatter",
			file: File{
				Path:     "/vault/simple.md",
				Basename: "simple",
				Folder:   "/vault",
				ModTime:  time.Date(2024, 3, 10, 14, 20, 0, 0, time.UTC),
			},
			frontmatter: Frontmatter{
				FileClass: "",
				Fields:    map[string]interface{}{},
			},
			expected: Note{
				File: File{
					Path:     "/vault/simple.md",
					Basename: "simple",
					Folder:   "/vault",
					ModTime:  time.Date(2024, 3, 10, 14, 20, 0, 0, time.UTC),
				},
				Frontmatter: Frontmatter{
					FileClass: "",
					Fields:    map[string]interface{}{},
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := NewNote(tt.file, tt.frontmatter)

			// Compare File fields
			if result.File.Path != tt.expected.File.Path {
				t.Errorf("File.Path = %q, want %q", result.File.Path, tt.expected.File.Path)
			}
			if result.File.Basename != tt.expected.File.Basename {
				t.Errorf("File.Basename = %q, want %q", result.File.Basename, tt.expected.File.Basename)
			}
			if result.File.Folder != tt.expected.File.Folder {
				t.Errorf("File.Folder = %q, want %q", result.File.Folder, tt.expected.File.Folder)
			}
			if !result.File.ModTime.Equal(tt.expected.File.ModTime) {
				t.Errorf("File.ModTime = %v, want %v", result.File.ModTime, tt.expected.File.ModTime)
			}

			// Compare Frontmatter fields
			if result.Frontmatter.FileClass != tt.expected.Frontmatter.FileClass {
				t.Errorf("Frontmatter.FileClass = %q, want %q", result.Frontmatter.FileClass, tt.expected.Frontmatter.FileClass)
			}

			// Compare Fields map
			if len(result.Frontmatter.Fields) != len(tt.expected.Frontmatter.Fields) {
				t.Errorf("Frontmatter.Fields length = %d, want %d",
					len(result.Frontmatter.Fields), len(tt.expected.Frontmatter.Fields))
				return
			}

			for key, expectedValue := range tt.expected.Frontmatter.Fields {
				if actualValue, exists := result.Frontmatter.Fields[key]; !exists {
					t.Errorf("Frontmatter.Fields[%q] missing", key)
				} else if actualValue != expectedValue {
					t.Errorf("Frontmatter.Fields[%q] = %v, want %v", key, actualValue, expectedValue)
				}
			}
		})
	}
}

func TestNoteSchemaName(t *testing.T) {
	tests := []struct {
		name     string
		note     Note
		expected string
	}{
		{
			name: "note with schema",
			note: Note{
				File: File{
					Path:     "/vault/contact.md",
					Basename: "contact",
					Folder:   "/vault",
					ModTime:  time.Now(),
				},
				Frontmatter: Frontmatter{
					FileClass: "contact",
					Fields: map[string]interface{}{
						"fileClass": "contact",
						"name":      "John Doe",
					},
				},
			},
			expected: "contact",
		},
		{
			name: "note without schema",
			note: Note{
				File: File{
					Path:     "/vault/note.md",
					Basename: "note",
					Folder:   "/vault",
					ModTime:  time.Now(),
				},
				Frontmatter: Frontmatter{
					FileClass: "",
					Fields: map[string]interface{}{
						"title": "Simple Note",
					},
				},
			},
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.note.SchemaName()
			if result != tt.expected {
				t.Errorf("SchemaName() = %q, want %q", result, tt.expected)
			}
		})
	}
}

func TestNoteEmbeddedStructAccess(t *testing.T) {
	// Test that embedded structs are accessible
	file := File{
		Path:     "/vault/test.md",
		Basename: "test",
		Folder:   "/vault",
		ModTime:  time.Date(2024, 1, 1, 12, 0, 0, 0, time.UTC),
	}

	frontmatter := Frontmatter{
		FileClass: "project",
		Fields: map[string]interface{}{
			"fileClass": "project",
			"title":     "Test Project",
		},
	}

	note := NewNote(file, frontmatter)

	// Test direct access to embedded File fields
	if note.Path != "/vault/test.md" {
		t.Errorf("note.Path = %q, want %q", note.Path, "/vault/test.md")
	}
	if note.Basename != "test" {
		t.Errorf("note.Basename = %q, want %q", note.Basename, "test")
	}
	if note.Folder != "/vault" {
		t.Errorf("note.Folder = %q, want %q", note.Folder, "/vault")
	}
	if !note.ModTime.Equal(file.ModTime) {
		t.Errorf("note.ModTime = %v, want %v", note.ModTime, file.ModTime)
	}

	// Test direct access to embedded Frontmatter fields
	if note.FileClass != "project" {
		t.Errorf("note.FileClass = %q, want %q", note.FileClass, "project")
	}
	if len(note.Fields) != 2 {
		t.Errorf("note.Fields length = %d, want %d", len(note.Fields), 2)
	}
	if note.Fields["title"] != "Test Project" {
		t.Errorf("note.Fields[title] = %v, want %v", note.Fields["title"], "Test Project")
	}
}
