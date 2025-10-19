package template

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"
)

func TestStaticTemplateParser_Parse(t *testing.T) {
	tests := []struct {
		name            string
		templateContent string
		wantErr         bool
		errContains     string
	}{
		{
			name:            "valid static template",
			templateContent: "Hello, World!",
			wantErr:         false,
		},
		{
			name:            "valid template with variables",
			templateContent: "Hello, {{.Name}}!",
			wantErr:         false,
		},
		{
			name:            "valid template with range",
			templateContent: "Items: {{range .Items}}{{.}} {{end}}",
			wantErr:         false,
		},
		{
			name:            "invalid template syntax - unclosed action",
			templateContent: "Hello, {{.Name",
			wantErr:         true,
			errContains:     "unclosed action",
		},
		{
			name:            "invalid template syntax - bad action",
			templateContent: "Hello, {{.}}{{",
			wantErr:         true,
			errContains:     "unclosed action",
		},
		{
			name:            "empty template",
			templateContent: "",
			wantErr:         false,
		},
	}

	parser := NewStaticTemplateParser()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			result := parser.Parse(ctx, tt.templateContent)

			if tt.wantErr {
				if result.IsOk() {
					t.Errorf("Parse() expected error, got success")
				}
				if tt.errContains != "" {
					err := result.Error()
					if err == nil ||
						tt.errContains != "" &&
							!contains(err.Error(), tt.errContains) {
						t.Errorf(
							"Parse() error = %v, want error containing %q",
							err,
							tt.errContains,
						)
					}
				}
			} else {
				if result.IsErr() {
					t.Errorf("Parse() unexpected error = %v", result.Error())
				}
				template := result.Value()
				if template == nil {
					t.Errorf("Parse() returned nil template")
				}
			}
		})
	}
}

func TestStaticTemplateParser_Parse_ContextCancellation(t *testing.T) {
	parser := NewStaticTemplateParser()

	// Create a context that's already canceled
	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	result := parser.Parse(ctx, "Hello, World!")

	if result.IsOk() {
		t.Errorf("Parse() expected error due to canceled context, got success")
	}

	err := result.Error()
	if !errors.Is(err, context.Canceled) {
		t.Errorf("Parse() error = %v, want %v", err, context.Canceled)
	}
}

func TestStaticTemplateParser_Parse_ContextTimeout(t *testing.T) {
	parser := NewStaticTemplateParser()

	// Create a context with a very short timeout
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Nanosecond)
	defer cancel()

	// Wait for the context to timeout
	time.Sleep(1 * time.Millisecond)

	result := parser.Parse(ctx, "Hello, World!")

	if result.IsOk() {
		t.Errorf("Parse() expected error due to timeout context, got success")
	}

	err := result.Error()
	if !errors.Is(err, context.DeadlineExceeded) {
		t.Errorf("Parse() error = %v, want %v", err, context.DeadlineExceeded)
	}
}

func TestStaticTemplateParser_Execute(t *testing.T) {
	tests := []struct {
		name            string
		templateContent string
		data            interface{}
		wantOutput      string
		wantErr         bool
		validateOutput  func(string) bool // Optional validation function for dynamic content
	}{
		{
			name:            "static template execution",
			templateContent: "Hello, World!",
			data:            nil,
			wantOutput:      "Hello, World!",
			wantErr:         false,
		},
		{
			name:            "template with simple variable",
			templateContent: "Hello, {{.Name}}!",
			data:            map[string]string{"Name": "Alice"},
			wantOutput:      "Hello, Alice!",
			wantErr:         false,
		},
		{
			name:            "template with range",
			templateContent: "Items: {{range .Items}}{{.}} {{end}}",
			data: map[string][]string{
				"Items": {"apple", "banana", "cherry"},
			},
			wantOutput: "Items: apple banana cherry ",
			wantErr:    false,
		},
		{
			name:            "empty template",
			templateContent: "",
			data:            nil,
			wantOutput:      "",
			wantErr:         false,
		},
		{
			name:            "template with missing variable renders no value",
			templateContent: "Hello, {{.MissingField}}!",
			data:            map[string]string{"Name": "Alice"},
			wantOutput:      "Hello, <no value>!",
			wantErr:         false,
		},
		{
			name:            "template with now function",
			templateContent: `Date: {{now "2006-01-02"}}`,
			data:            nil,
			wantOutput:      "",
			wantErr:         false,
			validateOutput: func(output string) bool {
				// Should start with "Date: " and contain a valid date
				if !strings.HasPrefix(output, "Date: ") {
					return false
				}
				dateStr := strings.TrimPrefix(output, "Date: ")
				_, err := time.Parse("2006-01-02", dateStr)
				return err == nil
			},
		},
		{
			name:            "template with toLower function",
			templateContent: `Result: {{ "HELLO" | toLower }}`,
			data:            nil,
			wantOutput:      "Result: hello",
			wantErr:         false,
		},
		{
			name:            "template with toUpper function",
			templateContent: `Result: {{ "world" | toUpper }}`,
			data:            nil,
			wantOutput:      "Result: WORLD",
			wantErr:         false,
		},
		{
			name:            "template with multiple functions",
			templateContent: `{{ "HELLO" | toLower }} {{now "2006-01-02"}} {{ "world" | toUpper }}`,
			data:            nil,
			wantOutput:      "",
			wantErr:         false,
			validateOutput: func(output string) bool {
				parts := strings.Split(output, " ")
				if len(parts) != 3 {
					return false
				}
				// Check "hello"
				if parts[0] != "hello" {
					return false
				}
				// Check date format
				_, err := time.Parse("2006-01-02", parts[1])
				if err != nil {
					return false
				}
				// Check "WORLD"
				if parts[2] != "WORLD" {
					return false
				}
				return true
			},
		},
	}

	parser := NewStaticTemplateParser()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()

			// First parse the template
			parseResult := parser.Parse(ctx, tt.templateContent)
			if parseResult.IsErr() {
				t.Fatalf("Parse() unexpected error = %v", parseResult.Error())
			}

			tmpl := parseResult.Value()

			// Then execute the template
			execResult := parser.Execute(ctx, tmpl, tt.data)

			if tt.wantErr {
				if execResult.IsOk() {
					t.Errorf(
						"Execute() expected error, got success with output: %q",
						execResult.Value(),
					)
				}
			} else {
				if execResult.IsErr() {
					t.Errorf("Execute() unexpected error = %v", execResult.Error())
				}
				output := execResult.Value()

				// Use validation function if provided, otherwise check exact
				// match
				if tt.validateOutput != nil {
					if !tt.validateOutput(output) {
						t.Errorf("Execute() output = %q, validation failed", output)
					}
				} else if output != tt.wantOutput {
					t.Errorf("Execute() output = %q, want %q", output, tt.wantOutput)
				}
			}
		})
	}
}

func TestStaticTemplateParser_Execute_ContextCancellation(t *testing.T) {
	parser := NewStaticTemplateParser()

	// Parse a simple template first
	ctx := context.Background()
	parseResult := parser.Parse(ctx, "Hello, World!")
	if parseResult.IsErr() {
		t.Fatalf("Parse() unexpected error = %v", parseResult.Error())
	}
	tmpl := parseResult.Value()

	// Create a canceled context for execution
	cancelledCtx, cancel := context.WithCancel(context.Background())
	cancel()

	result := parser.Execute(cancelledCtx, tmpl, nil)

	if result.IsOk() {
		t.Errorf(
			"Execute() expected error due to canceled context, got success",
		)
	}

	err := result.Error()
	if !errors.Is(err, context.Canceled) {
		t.Errorf("Execute() error = %v, want %v", err, context.Canceled)
	}
}

// contains is a helper function to check if a string contains a substring.
func contains(s, substr string) bool {
	return len(s) >= len(substr) &&
		(substr == "" || indexOf(s, substr) >= 0)
}

// indexOf finds the index of substr in s, or -1 if not found.
func indexOf(s, substr string) int {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return i
		}
	}
	return -1
}
