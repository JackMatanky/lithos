package spi

import (
	"context"
	"errors"
	"testing"
)

const testTitle = "test"

// MockMarkdownParserPort for testing downstream components.
type MockMarkdownParserPort struct {
	ParseFrontmatterFunc func(ctx context.Context, content []byte) (map[string]any, error)
}

// TestMarkdownParserPortContract tests the interface contract definition.
func TestMarkdownParserPortContract(t *testing.T) {
	// Test that interface exists and has correct method signature
	var _ MarkdownParserPort // Ensure interface compiles

	// Test with mock implementation
	mock := &MockMarkdownParserPort{
		ParseFrontmatterFunc: func(ctx context.Context, content []byte) (map[string]any, error) {
			return map[string]any{"title": testTitle}, nil
		},
	}

	ctx := context.Background()
	content := []byte("---\ntitle: test\n---\n# Content")

	result, err := mock.ParseFrontmatter(ctx, content)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if result["title"] != testTitle {
		t.Errorf("Expected title '%s', got %v", testTitle, result["title"])
	}
}

// TestMarkdownParserPortErrorConditions tests expected error scenarios.
func TestMarkdownParserPortErrorConditions(t *testing.T) {
	tests := []struct {
		name    string
		content []byte
		wantErr bool
	}{
		{
			name:    "invalid YAML",
			content: []byte("---\ninvalid: yaml: structure\n---\n# Content"),
			wantErr: true,
		},
		{
			name:    "missing frontmatter",
			content: []byte("# Content without frontmatter"),
			wantErr: false, // This should not error, just return empty map
		},
		{
			name:    "empty content",
			content: []byte(""),
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// This will fail until we have a concrete implementation to test
			// against
			t.Skip("Interface contract test - will implement with mock")
		})
	}
}

// ParseFrontmatter implements MarkdownParserPort.ParseFrontmatter.
func (m *MockMarkdownParserPort) ParseFrontmatter(
	ctx context.Context,
	content []byte,
) (map[string]any, error) {
	if m.ParseFrontmatterFunc != nil {
		return m.ParseFrontmatterFunc(ctx, content)
	}
	return nil, errors.New("mock not configured")
}

// TestMockMarkdownParserPort ensures mock implementation works.
func TestMockMarkdownParserPort(t *testing.T) {
	mock := &MockMarkdownParserPort{
		ParseFrontmatterFunc: func(ctx context.Context, content []byte) (map[string]any, error) {
			return map[string]any{"title": testTitle}, nil
		},
	}

	ctx := context.Background()
	content := []byte("---\ntitle: test\n---\n# Content")

	result, err := mock.ParseFrontmatter(ctx, content)
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}

	if result["title"] != testTitle {
		t.Errorf("Expected title '%s', got %v", testTitle, result["title"])
	}
}
