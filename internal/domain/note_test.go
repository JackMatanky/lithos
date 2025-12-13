package domain

import (
	"errors"
	"reflect"
	"testing"
)

const (
	testTitle = "John Doe"
)

// TestNote_Validate tests the Note.Validate() method with various scenarios.
func TestNote_Validate(t *testing.T) {
	tests := []struct {
		name        string
		note        Note
		expectError bool
		errorField  string
	}{
		{
			name: "valid note with all required fields",
			note: Note{
				Path: "notes/test.md",
				Frontmatter: Frontmatter{
					Fields: map[string]any{"title": "Test"},
				},
				Links:     []Link{},
				Headings:  []Heading{},
				Tags:      []string{},
				Tasks:     []TaskItem{},
				Backlinks: []Link{},
			},
			expectError: false,
		},
		{
			name: "invalid note with empty path",
			note: Note{
				Path: "",
				Frontmatter: Frontmatter{
					Fields: map[string]any{"title": "Test"},
				},
				Links:     []Link{},
				Headings:  []Heading{},
				Tags:      []string{},
				Tasks:     []TaskItem{},
				Backlinks: []Link{},
			},
			expectError: true,
			errorField:  "Path",
		},
		{
			name: "invalid note with whitespace-only path",
			note: Note{
				Path: "   ",
				Frontmatter: Frontmatter{
					Fields: map[string]any{"title": "Test"},
				},
				Links:     []Link{},
				Headings:  []Heading{},
				Tags:      []string{},
				Tasks:     []TaskItem{},
				Backlinks: []Link{},
			},
			expectError: true,
			errorField:  "Path",
		},
		{
			name: "invalid note with nil frontmatter fields",
			note: Note{
				Path: "notes/test.md",
				Frontmatter: Frontmatter{
					Fields: nil,
				},
				Links:     []Link{},
				Headings:  []Heading{},
				Tags:      []string{},
				Tasks:     []TaskItem{},
				Backlinks: []Link{},
			},
			expectError: true,
			errorField:  "Frontmatter",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.note.Validate()

			switch {
			case tt.expectError && err == nil:
				t.Errorf(
					"Expected validation error for field %s, but got nil",
					tt.errorField,
				)
			case tt.expectError && err != nil:
				var validationErr NoteValidationError
				if !errors.As(err, &validationErr) {
					t.Errorf(
						"Expected NoteValidationError, got %T: %v",
						err,
						err,
					)
				} else if validationErr.Field != tt.errorField {
					t.Errorf(
						"Expected error field %s, got %s",
						tt.errorField,
						validationErr.Field,
					)
				}
			case !tt.expectError && err != nil:
				t.Errorf("Expected no validation error, but got: %v", err)
			}
		})
	}
}

// TestNewNote tests the NewNote constructor function.
func TestNewNote(t *testing.T) {
	tests := []struct {
		name        string
		path        string
		frontmatter Frontmatter
		links       []Link
		headings    []Heading
		tags        []string
		tasks       []TaskItem
		expectError bool
	}{
		{
			name: "creates valid note with all parameters",
			path: "notes/test.md",
			frontmatter: Frontmatter{
				Fields: map[string]any{"title": "Test Note"},
			},
			links:       []Link{{Text: "link1", Destination: "dest1"}},
			headings:    []Heading{{Level: 1, Text: "Heading 1"}},
			tags:        []string{"tag1", "tag2"},
			tasks:       []TaskItem{{Text: "Task 1", IsChecked: false}},
			expectError: false,
		},
		{
			name: "creates note with empty slices",
			path: "notes/test.md",
			frontmatter: Frontmatter{
				Fields: map[string]any{"title": "Test Note"},
			},
			links:       nil,
			headings:    nil,
			tags:        nil,
			tasks:       nil,
			expectError: false,
		},
		{
			name: "fails with empty path",
			path: "",
			frontmatter: Frontmatter{
				Fields: map[string]any{"title": "Test Note"},
			},
			links:       []Link{},
			headings:    []Heading{},
			tags:        []string{},
			tasks:       []TaskItem{},
			expectError: true,
		},
		{
			name: "fails with nil frontmatter",
			path: "notes/test.md",
			frontmatter: Frontmatter{
				Fields: nil,
			},
			links:       []Link{},
			headings:    []Heading{},
			tags:        []string{},
			tasks:       []TaskItem{},
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			note, err := NewNote(
				tt.path,
				tt.frontmatter,
				tt.links,
				tt.headings,
				tt.tags,
				tt.tasks,
			)

			switch {
			case tt.expectError && err == nil:
				t.Errorf("Expected error, but got nil")
			case tt.expectError && note.Path != "":
				t.Errorf(
					"Expected empty note on error, but got path: %s",
					note.Path,
				)
			case !tt.expectError && err != nil:
				t.Errorf("Expected no error, but got: %v", err)
			case !tt.expectError && note.Path != tt.path:
				t.Errorf("Expected path %s, got %s", tt.path, note.Path)
			case !tt.expectError && err == nil:
				// Verify defensive copying - slices should be non-nil even if
				// input was nil
				if note.Links == nil {
					t.Error("Expected Links slice to be non-nil")
				}
				if note.Headings == nil {
					t.Error("Expected Headings slice to be non-nil")
				}
				if note.Tags == nil {
					t.Error("Expected Tags slice to be non-nil")
				}
				if note.Tasks == nil {
					t.Error("Expected Tasks slice to be non-nil")
				}
				if note.Backlinks == nil {
					t.Error("Expected Backlinks slice to be non-nil")
				}

				// Verify Backlinks start empty
				if len(note.Backlinks) != 0 {
					t.Errorf(
						"Expected Backlinks to be empty, got length %d",
						len(note.Backlinks),
					)
				}

				// Verify slice contents match input
				if tt.links != nil && !reflect.DeepEqual(note.Links, tt.links) {
					t.Errorf("Expected Links %v, got %v", tt.links, note.Links)
				}
				if tt.headings != nil &&
					!reflect.DeepEqual(note.Headings, tt.headings) {
					t.Errorf(
						"Expected Headings %v, got %v",
						tt.headings,
						note.Headings,
					)
				}
				if tt.tags != nil && !reflect.DeepEqual(note.Tags, tt.tags) {
					t.Errorf("Expected Tags %v, got %v", tt.tags, note.Tags)
				}
				if tt.tasks != nil && !reflect.DeepEqual(note.Tasks, tt.tasks) {
					t.Errorf("Expected Tasks %v, got %v", tt.tasks, note.Tasks)
				}
			}
		})
	}
}

// TestNote_WithBacklinks tests the WithBacklinks enrichment helper.
func TestNote_WithBacklinks(t *testing.T) {
	// Create a base note
	note, err := NewNote(
		"notes/test.md",
		Frontmatter{Fields: map[string]any{"title": "Test"}},
		nil,
		nil,
		nil,
		nil,
	)
	if err != nil {
		t.Fatalf("Failed to create test note: %v", err)
	}

	// Verify Backlinks start empty
	if len(note.Backlinks) != 0 {
		t.Errorf(
			"Expected initial Backlinks to be empty, got %d",
			len(note.Backlinks),
		)
	}

	// Add backlinks
	backlinks := []Link{
		{Text: "ref1", Destination: "notes/test.md", IsWikilink: true},
		{Text: "ref2", Destination: "notes/test.md", IsWikilink: false},
	}
	enrichedNote := note.WithBacklinks(backlinks)

	// Verify original note unchanged
	if len(note.Backlinks) != 0 {
		t.Error("Original note Backlinks should remain unchanged")
	}

	// Verify enriched note has backlinks
	if len(enrichedNote.Backlinks) != 2 {
		t.Errorf("Expected 2 backlinks, got %d", len(enrichedNote.Backlinks))
	}

	if !reflect.DeepEqual(enrichedNote.Backlinks, backlinks) {
		t.Errorf(
			"Expected backlinks %v, got %v",
			backlinks,
			enrichedNote.Backlinks,
		)
	}

	// Verify other fields unchanged
	if enrichedNote.Path != note.Path {
		t.Error("Path should remain unchanged")
	}
	if !reflect.DeepEqual(enrichedNote.Frontmatter, note.Frontmatter) {
		t.Error("Frontmatter should remain unchanged")
	}
}

// TestNote_DelegationHelpers tests the delegation helper methods.
func TestNote_DelegationHelpers(t *testing.T) {
	// Create note with test frontmatter
	fields := map[string]any{
		"fileClass": "contact",
		"title":     testTitle,
		"aliases":   []any{"JD", "Johnny"},
	}
	note, err := NewNote(
		"notes/test.md",
		Frontmatter{Fields: fields},
		nil,
		nil,
		nil,
		nil,
	)
	if err != nil {
		t.Fatalf("Failed to create test note: %v", err)
	}

	// Test FileClass
	if fileClass := note.FileClass(); fileClass != "contact" {
		t.Errorf("Expected FileClass 'contact', got '%s'", fileClass)
	}

	// Test Title
	if title := note.Title(); title != testTitle {
		t.Errorf("Expected Title '%s', got '%s'", testTitle, title)
	}

	// Test Aliases
	expectedAliases := []string{"JD", "Johnny"}
	if aliases := note.Aliases(); !reflect.DeepEqual(aliases, expectedAliases) {
		t.Errorf("Expected Aliases %v, got %v", expectedAliases, aliases)
	}

	// Test HasFrontmatterField
	if !note.HasFrontmatterField("title") {
		t.Error("Expected HasFrontmatterField('title') to be true")
	}
	if note.HasFrontmatterField("nonexistent") {
		t.Error("Expected HasFrontmatterField('nonexistent') to be false")
	}

	// Test GetFrontmatterString
	if val, ok := note.GetFrontmatterString("title"); !ok || val != testTitle {
		t.Errorf(
			"Expected GetFrontmatterString('title') to return '%s', true; got '%s', %v",
			testTitle,
			val,
			ok,
		)
	}
	if val, ok := note.GetFrontmatterString("nonexistent"); ok || val != "" {
		t.Errorf(
			"Expected GetFrontmatterString('nonexistent') to return '', false; got '%s', %v",
			val,
			ok,
		)
	}
	if val, ok := note.GetFrontmatterString("aliases"); ok || val != "" {
		t.Errorf(
			"Expected GetFrontmatterString('aliases') to return '', false (not a string); got '%s', %v",
			val,
			ok,
		)
	}
}

// TestNote_SchemaName tests the SchemaName method (delegates to Frontmatter).
func TestNote_SchemaName(t *testing.T) {
	tests := []struct {
		name     string
		fields   map[string]any
		expected string
	}{
		{
			name: "returns fileClass when present",
			fields: map[string]any{
				"fileClass": "contact",
				"title":     "Test",
			},
			expected: "contact",
		},
		{
			name:     "returns empty when fileClass missing",
			fields:   map[string]any{"title": "Test"},
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			note, err := NewNote(
				"notes/test.md",
				Frontmatter{Fields: tt.fields},
				nil,
				nil,
				nil,
				nil,
			)
			if err != nil {
				t.Fatalf("Failed to create test note: %v", err)
			}

			if result := note.SchemaName(); result != tt.expected {
				t.Errorf(
					"Expected SchemaName '%s', got '%s'",
					tt.expected,
					result,
				)
			}
		})
	}
}
