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

// TestMarkdownParserPortErrorConditions exercises contract-level error
// scenarios
// using a configurable mock to ensure interface consumers can rely on
// deterministic behaviors (error vs empty-map semantics).
func TestMarkdownParserPortErrorConditions(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name      string
		mockErr   error
		mockValue map[string]any
		wantErr   bool
		wantLen   int
	}{
		{
			name:    "invalid YAML propagates error",
			mockErr: errors.New("frontmatter parsing failed"),
			wantErr: true,
		},
		{
			name:      "missing frontmatter returns empty map",
			mockValue: map[string]any{},
			wantLen:   0,
		},
		{
			name:      "empty content treated as no frontmatter",
			mockValue: map[string]any{},
			wantLen:   0,
		},
	}

	for _, tt := range tests {
		mock := &MockMarkdownParserPort{
			ParseFrontmatterFunc: func(ctx context.Context, content []byte) (map[string]any, error) {
				return tt.mockValue, tt.mockErr
			},
		}

		result, err := mock.ParseFrontmatter(
			context.Background(),
			[]byte("ignored"),
		)
		if tt.wantErr {
			if err == nil {
				t.Fatalf("expected error for %s scenario", tt.name)
			}
			continue
		}

		if err != nil {
			t.Fatalf("unexpected error for %s scenario: %v", tt.name, err)
		}
		if len(result) != tt.wantLen {
			t.Fatalf(
				"expected %d frontmatter entries for %s, got %d",
				tt.wantLen,
				tt.name,
				len(result),
			)
		}
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
