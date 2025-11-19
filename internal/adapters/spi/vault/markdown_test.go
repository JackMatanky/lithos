package vault

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/rs/zerolog"
)

// TestMarkdownParserAdapterConstruction tests adapter struct creation
func TestMarkdownParserAdapterConstruction(t *testing.T) {
	logger := zerolog.New(nil)

	// This will fail until MarkdownParserAdapter is defined
	adapter := NewMarkdownParserAdapter(logger)
	if adapter == nil {
		t.Error("Should create adapter")
	}
}

// TestMarkdownParserAdapterValidFrontmatter tests parsing of valid YAML frontmatter
func TestMarkdownParserAdapterValidFrontmatter(t *testing.T) {
	tests := []struct {
		name         string
		content      string
		expectFields []string
		expectValues map[string]any
	}{
		{
			name:         "simple frontmatter",
			content:      "---\ntitle: Test Note\nauthor: John Doe\n---\n# Content",
			expectFields: []string{"title", "author"},
			expectValues: map[string]any{
				"title":  "Test Note",
				"author": "John Doe",
			},
		},
		{
			name:         "nested YAML structure",
			content:      "---\ntitle: Complex\nconfig:\n  enabled: true\n  count: 42\n---\n# Content",
			expectFields: []string{"title", "config"},
			expectValues: map[string]any{
				"title": "Complex",
			},
		},
		{
			name:         "empty frontmatter",
			content:      "---\n---\n# Content",
			expectFields: []string{},
		},
	}

	logger := zerolog.New(nil)
	adapter := NewMarkdownParserAdapter(logger)
	ctx := context.Background()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := adapter.ParseFrontmatter(ctx, []byte(tt.content))
			if err != nil {
				t.Fatalf("Unexpected error: %v", err)
			}

			// Check that expected fields are present
			for _, field := range tt.expectFields {
				if _, exists := result[field]; !exists {
					t.Errorf("Expected field %s not found in result", field)
				}
			}

			// Check specific values for simple types
			for key, expected := range tt.expectValues {
				if result[key] != expected {
					t.Errorf("For key %s: expected %v, got %v", key, expected, result[key])
				}
			}
		})
	}
}

// TestMarkdownParserAdapterSyntacticValidation tests YAML structure validation
func TestMarkdownParserAdapterSyntacticValidation(t *testing.T) {
	tests := []struct {
		name    string
		content string
		wantErr bool
	}{
		{
			name:    "malformed YAML - invalid structure",
			content: "---\ninvalid: yaml: structure: bad\n---\n# Content",
			wantErr: true,
		},
		{
			name:    "malformed YAML - unclosed quotes",
			content: "---\ntitle: \"unclosed quote\nauthor: test\n---\n# Content",
			wantErr: true,
		},
		{
			name:    "malformed YAML - invalid indentation",
			content: "---\ntitle: Test\n config:\nenabled: true\n---\n# Content",
			wantErr: true,
		},
		{
			name:    "missing opening delimiter",
			content: "title: Test\nauthor: John\n---\n# Content",
			wantErr: false, // Should treat as no frontmatter
		},
		{
			name:    "missing closing delimiter",
			content: "---\ntitle: Test\nauthor: John\n# Content",
			wantErr: false, // Goldmark handles this gracefully - treats as no frontmatter
		},
	}

	logger := zerolog.New(nil)
	adapter := NewMarkdownParserAdapter(logger)
	ctx := context.Background()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := adapter.ParseFrontmatter(ctx, []byte(tt.content))
			if tt.wantErr && err == nil {
				t.Error("Expected error but got none")
			}
			if !tt.wantErr && err != nil {
				t.Errorf("Unexpected error: %v", err)
			}
		})
	}
}

// TestMarkdownParserAdapterLineNumberErrorReporting tests structured error messages
func TestMarkdownParserAdapterLineNumberErrorReporting(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewMarkdownParserAdapter(logger)
	ctx := context.Background()

	// Content with error on line 3
	content := "---\ntitle: Test\ninvalid: yaml: structure\nauthor: John\n---\n# Content"

	_, err := adapter.ParseFrontmatter(ctx, []byte(content))
	if err == nil {
		t.Error("Expected error for malformed YAML")
		return
	}

	// Error should contain line number information
	errMsg := err.Error()
	if !strings.Contains(errMsg, "line") && !strings.Contains(errMsg, "3") {
		t.Errorf("Error message should contain line number information, got: %s", errMsg)
	}
}

// TestMarkdownParserAdapterMissingFrontmatter tests handling of content without frontmatter
func TestMarkdownParserAdapterMissingFrontmatter(t *testing.T) {
	tests := []struct {
		name    string
		content string
	}{
		{
			name:    "no frontmatter",
			content: "# Just a heading\n\nSome content here.",
		},
		{
			name:    "empty content",
			content: "",
		},
		{
			name:    "only whitespace",
			content: "   \n\t\n  ",
		},
	}

	logger := zerolog.New(nil)
	adapter := NewMarkdownParserAdapter(logger)
	ctx := context.Background()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := adapter.ParseFrontmatter(ctx, []byte(tt.content))
			if err != nil {
				t.Errorf("Should not error for missing frontmatter: %v", err)
			}

			if len(result) != 0 {
				t.Errorf("Expected empty map for missing frontmatter, got: %v", result)
			}
		})
	}
}

// TestMarkdownParserAdapterEdgeCases tests various edge cases
func TestMarkdownParserAdapterEdgeCases(t *testing.T) {
	tests := []struct {
		name    string
		content string
		wantErr bool
	}{
		{
			name:    "special characters in YAML",
			content: "---\ntitle: \"Special chars: @#$%^&*()\"\nauthor: \"John O'Doe\"\n---\n# Content",
			wantErr: false,
		},
		{
			name:    "unicode in YAML",
			content: "---\ntitle: \"测试文档\"\nauthor: \"José María\"\n---\n# Content",
			wantErr: false,
		},
		{
			name:    "multiple frontmatter blocks",
			content: "---\ntitle: First\n---\n# Content\n---\ntitle: Second\n---\nMore content",
			wantErr: false, // Should only parse first block
		},
	}

	logger := zerolog.New(nil)
	adapter := NewMarkdownParserAdapter(logger)
	ctx := context.Background()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := adapter.ParseFrontmatter(ctx, []byte(tt.content))
			if tt.wantErr && err == nil {
				t.Error("Expected error but got none")
			}
			if !tt.wantErr && err != nil {
				t.Errorf("Unexpected error: %v", err)
			}
		})
	}
}

// TestMarkdownParserAdapterContextCancellation tests context handling
func TestMarkdownParserAdapterContextCancellation(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewMarkdownParserAdapter(logger)

	// Create cancelled context
	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	content := "---\ntitle: Test\n---\n# Content"

	_, err := adapter.ParseFrontmatter(ctx, []byte(content))
	if err == nil {
		t.Error("Expected context cancellation error")
	}

	if err != context.Canceled {
		t.Errorf("Expected context.Canceled, got: %v", err)
	}
}

// TestMarkdownParserAdapterContextTimeout tests timeout handling
func TestMarkdownParserAdapterContextTimeout(t *testing.T) {
	logger := zerolog.New(nil)
	adapter := NewMarkdownParserAdapter(logger)

	// Create context with very short timeout
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Nanosecond)
	defer cancel()

	// Wait for timeout
	time.Sleep(1 * time.Millisecond)

	content := "---\ntitle: Test\n---\n# Content"

	_, err := adapter.ParseFrontmatter(ctx, []byte(content))
	if err == nil {
		t.Error("Expected context timeout error")
	}

	if err != context.DeadlineExceeded {
		t.Errorf("Expected context.DeadlineExceeded, got: %v", err)
	}
}
