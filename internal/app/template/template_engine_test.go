package template

import (
	"context"
	"strings"
	"testing"

	"github.com/jack/lithos/internal/domain"
)

func TestTemplateEngine_ProcessTemplate(t *testing.T) {
	parser := NewStaticTemplateParser()
	executor := NewGoTemplateExecutor()
	engine := NewTemplateEngine(parser, executor)
	ctx := context.Background()

	tests := []struct {
		name         string
		content      string
		templateName string
		want         string
		wantErr      bool
		errContains  string
	}{
		{
			name:         "successful processing of static template",
			content:      "Hello, World!",
			templateName: "static",
			want:         "Hello, World!",
			wantErr:      false,
		},
		{
			name: "successful processing with custom functions",
			content: "Now: {{now \"2006-01-02\"}} Upper: {{toUpper \"hello\"}} " +
				"Lower: {{toLower \"WORLD\"}}",
			templateName: "functions",
			want:         "", // Will be validated in test logic
			wantErr:      false,
		},
		{
			name:         "empty content",
			content:      "",
			templateName: "empty",
			want:         "",
			wantErr:      true,
			errContains:  "cannot process empty template content",
		},
		{
			name:         "template execution error",
			content:      "{{len .InvalidField}}",
			templateName: "error",
			want:         "",
			wantErr:      true,
			errContains:  "failed to execute parsed template",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := engine.ProcessTemplate(ctx, tt.content, tt.templateName)

			if tt.wantErr {
				if err == nil {
					t.Errorf("ProcessTemplate() expected error, got nil")
					return
				}
				if tt.errContains != "" &&
					!strings.Contains(err.Error(), tt.errContains) {
					t.Errorf(
						"ProcessTemplate() error = %v, expected to contain %q",
						err,
						tt.errContains,
					)
				}
				return
			}

			if err != nil {
				t.Errorf("ProcessTemplate() unexpected error = %v", err)
				return
			}

			if tt.name == "successful processing with custom functions" {
				validateTemplateEngineCustomFunctionsOutput(t, got)
				return
			}

			if got != tt.want {
				t.Errorf("ProcessTemplate() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestTemplateEngine_ExecuteParsedTemplate(t *testing.T) {
	parser := NewStaticTemplateParser()
	executor := NewGoTemplateExecutor()
	engine := NewTemplateEngine(parser, executor)
	ctx := context.Background()

	tests := []struct {
		name        string
		template    *domain.Template
		want        string
		wantErr     bool
		errContains string
	}{
		{
			name:        "nil template",
			template:    nil,
			want:        "",
			wantErr:     true,
			errContains: "cannot execute nil template",
		},
		{
			name: "template not parsed",
			template: &domain.Template{
				Name:    "test",
				Content: "Hello",
				Parsed:  nil,
			},
			want:        "",
			wantErr:     true,
			errContains: "template must be parsed before execution",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := engine.ExecuteParsedTemplate(ctx, tt.template)

			if tt.wantErr {
				if err == nil {
					t.Errorf("ExecuteParsedTemplate() expected error, got nil")
					return
				}
				if tt.errContains != "" &&
					!strings.Contains(err.Error(), tt.errContains) {
					t.Errorf(
						"ExecuteParsedTemplate() error = %v, expected to contain %q",
						err,
						tt.errContains,
					)
				}
				return
			}

			if err != nil {
				t.Errorf("ExecuteParsedTemplate() unexpected error = %v", err)
				return
			}

			if got != tt.want {
				t.Errorf("ExecuteParsedTemplate() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestTemplateEngine_ProcessTemplateFromPath(t *testing.T) {
	parser := NewStaticTemplateParser()
	executor := NewGoTemplateExecutor()
	engine := NewTemplateEngine(parser, executor)
	ctx := context.Background()

	_, err := engine.ProcessTemplateFromPath(ctx, "dummy-path")
	if err == nil {
		t.Error("ProcessTemplateFromPath() expected error, got nil")
		return
	}

	if !strings.Contains(err.Error(), "not implemented") {
		t.Errorf(
			"ProcessTemplateFromPath() error = %v, expected to contain 'not implemented'",
			err,
		)
	}
}

func TestNewTemplateEngine(t *testing.T) {
	parser := NewStaticTemplateParser()
	executor := NewGoTemplateExecutor()
	engine := NewTemplateEngine(parser, executor)

	if engine == nil {
		t.Error("NewTemplateEngine() returned nil")
	}
}

// validateTemplateEngineCustomFunctionsOutput validates the output of custom
// functions test.
func validateTemplateEngineCustomFunctionsOutput(t *testing.T, got string) {
	t.Helper()

	if !strings.HasPrefix(got, "Now: ") {
		t.Errorf("ProcessTemplate() = %q, expected to start with 'Now: '", got)
	}
	if !strings.Contains(got, "Upper: HELLO") {
		t.Errorf(
			"ProcessTemplate() = %q, expected to contain 'Upper: HELLO'",
			got,
		)
	}
	if !strings.Contains(got, "Lower: world") {
		t.Errorf(
			"ProcessTemplate() = %q, expected to contain 'Lower: world'",
			got,
		)
	}

	// Validate the date format in the now function
	parts := strings.Split(got, " ")
	if len(parts) < 2 {
		t.Errorf(
			"ProcessTemplate() = %q, expected format 'Now: YYYY-MM-DD Upper: HELLO Lower: world'",
			got,
		)
		return
	}

	datePart := strings.TrimPrefix(parts[1], "Now: ")
	if len(datePart) != 10 || datePart[4] != '-' || datePart[7] != '-' {
		t.Errorf(
			"ProcessTemplate() date part = %q, expected format YYYY-MM-DD",
			datePart,
		)
	}
}
