package template

import (
	"context"
	"strings"
	"testing"
	"text/template"

	"github.com/jack/lithos/internal/domain"
)

// validateCustomFunctionsOutput validates the output of custom functions test.
func validateCustomFunctionsOutput(t *testing.T, got string) {
	t.Helper()

	if !strings.HasPrefix(got, "Now: ") {
		t.Errorf("Execute() = %q, expected to start with 'Now: '", got)
	}
	if !strings.Contains(got, "Upper: HELLO") {
		t.Errorf("Execute() = %q, expected to contain 'Upper: HELLO'", got)
	}
	if !strings.Contains(got, "Lower: world") {
		t.Errorf("Execute() = %q, expected to contain 'Lower: world'", got)
	}

	// Validate the date format in the now function
	parts := strings.Split(got, " ")
	if len(parts) < 2 {
		t.Errorf("Execute() = %q, expected format "+
			"'Now: YYYY-MM-DD Upper: HELLO Lower: world'", got)
		return
	}

	datePart := strings.TrimPrefix(parts[1], "Now: ")
	if len(datePart) != 10 || datePart[4] != '-' || datePart[7] != '-' {
		t.Errorf(
			"Execute() date part = %q, expected format YYYY-MM-DD",
			datePart,
		)
	}
}

// complexity.
//
//nolint:gocognit // Test function with multiple test cases requires higher
func TestGoTemplateExecutor_Execute(t *testing.T) {
	executor := NewGoTemplateExecutor()
	ctx := context.Background()

	tests := []struct {
		name        string
		template    *domain.Template
		data        interface{}
		want        string
		wantErr     bool
		errContains string
	}{
		{
			name: "successful execution of static template",
			template: &domain.Template{
				Name:    "static",
				Content: "Hello, World!",
				Parsed: func() *template.Template {
					tmpl, _ := template.New("static").Parse("Hello, World!")
					return tmpl
				}(),
			},
			data:    nil,
			want:    "Hello, World!",
			wantErr: false,
		},
		{
			name: "successful execution with custom functions",
			template: &domain.Template{
				Name: "functions",
				Content: "Now: {{now \"2006-01-02\"}} Upper: {{toUpper \"hello\"}} " +
					"Lower: {{toLower \"WORLD\"}}",
				Parsed: func() *template.Template {
					funcMap := NewFuncMap()
					tmpl, _ := template.New("functions").
						Funcs(funcMap).
						Parse("Now: {{now \"2006-01-02\"}} Upper: {{toUpper \"hello\"}} " +
							"Lower: {{toLower \"WORLD\"}}")
					return tmpl
				}(),
			},
			data:    nil,
			want:    "", // Will be validated in the test logic
			wantErr: false,
		},
		{
			name:        "nil template",
			template:    nil,
			data:        nil,
			want:        "",
			wantErr:     true,
			errContains: "cannot execute nil template",
		},
		{
			name: "template not parsed",
			template: &domain.Template{
				Name:    "unparsed",
				Content: "Hello",
				Parsed:  nil,
			},
			data:        nil,
			want:        "",
			wantErr:     true,
			errContains: "template must be parsed before execution",
		},
		{
			name: "external data not supported",
			template: &domain.Template{
				Name:    "static",
				Content: "Hello",
				Parsed: func() *template.Template {
					tmpl, _ := template.New("static").Parse("Hello")
					return tmpl
				}(),
			},
			data:        "some data",
			want:        "",
			wantErr:     true,
			errContains: "external data not supported in MVP",
		},
		{
			name: "template execution error",
			template: &domain.Template{
				Name:    "error",
				Content: "{{len .InvalidField}}",
				Parsed: func() *template.Template {
					tmpl, _ := template.New("error").
						Parse("{{len .InvalidField}}")
					return tmpl
				}(),
			},
			data:        nil,
			want:        "",
			wantErr:     true,
			errContains: "template execution failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := executor.Execute(ctx, tt.template, tt.data)

			if tt.wantErr {
				if err == nil {
					t.Errorf("Execute() expected error, got nil")
					return
				}
				if tt.errContains != "" &&
					!strings.Contains(err.Error(), tt.errContains) {
					t.Errorf(
						"Execute() error = %v, expected to contain %q",
						err,
						tt.errContains,
					)
				}
				return
			}

			if err != nil {
				t.Errorf("Execute() unexpected error = %v", err)
				return
			}

			if tt.name == "successful execution with custom functions" {
				validateCustomFunctionsOutput(t, got)
				return
			}

			if got != tt.want {
				t.Errorf("Execute() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestGoTemplateExecutor_Execute_ContextCancellation(t *testing.T) {
	executor := NewGoTemplateExecutor()

	// Create a template that would take time to execute (though in practice
	// it's fast)
	tmpl := &domain.Template{
		Name:    "test",
		Content: "Hello, World!",
		Parsed: func() *template.Template {
			tmpl, _ := template.New("test").Parse("Hello, World!")
			return tmpl
		}(),
	}

	// Test with canceled context (though execution is synchronous, this tests
	// the interface)
	ctx, cancel := context.WithCancel(context.Background())
	cancel() // Cancel immediately

	_, err := executor.Execute(ctx, tmpl, nil)
	// Currently, the executor doesn't check for context cancellation during
	// execution
	// This is acceptable for MVP as template execution is typically fast
	// In future versions, we could add context checking for long-running
	// templates
	if err != nil {
		t.Errorf(
			"Execute() with canceled context should not fail in MVP, got error: %v",
			err,
		)
	}
}

func TestNewGoTemplateExecutor(t *testing.T) {
	executor := NewGoTemplateExecutor()
	if executor == nil {
		t.Error("NewGoTemplateExecutor() returned nil")
	}
}
