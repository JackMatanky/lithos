package spi_test

import (
	"context"
	"io"
	"testing"

	vaultAdapter "github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	spi "github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
)

// TestMarkdownParserPortAdapterContract verifies that the concrete adapter
// satisfies the MarkdownParserPort contract for critical error scenarios.
func TestMarkdownParserPortAdapterContract(t *testing.T) {
	t.Parallel()

	var parser spi.MarkdownParserPort = vaultAdapter.NewMarkdownParserAdapter(zerolog.New(io.Discard))
	ctx := context.Background()

	tests := []struct {
		name    string
		content string
		wantErr bool
		wantLen int
	}{
		{
			name:    "invalid YAML surfaces error",
			content: "---\ninvalid: yaml: structure\n---\n# Content",
			wantErr: true,
		},
		{
			name:    "missing frontmatter produces empty map",
			content: "# Content without frontmatter",
			wantLen: 0,
		},
		{
			name:    "empty file behaves like missing frontmatter",
			content: "",
			wantLen: 0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := parser.ParseFrontmatter(ctx, []byte(tt.content))
			if tt.wantErr {
				if err == nil {
					t.Fatalf("expected error for %s", tt.name)
				}
				return
			}

			if err != nil {
				t.Fatalf("unexpected error for %s: %v", tt.name, err)
			}
			if len(result) != tt.wantLen {
				t.Fatalf(
					"expected %d frontmatter fields for %s, got %d",
					tt.wantLen,
					tt.name,
					len(result),
				)
			}
		})
	}
}
