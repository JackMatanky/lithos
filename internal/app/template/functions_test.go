package template

import (
	"strings"
	"testing"
	"text/template"
)

func TestNow(t *testing.T) {
	tests := []struct {
		name     string
		layout   string
		wantFunc func(string) bool // Function to validate the result
	}{
		{
			name:   "default layout",
			layout: "",
			wantFunc: func(result string) bool {
				// Should match YYYY-MM-DD HH:MM:SS format
				return len(result) == 19 && strings.Contains(result, "-") &&
					strings.Contains(result, ":")
			},
		},
		{
			name:   "date only layout",
			layout: "2006-01-02",
			wantFunc: func(result string) bool {
				// Should match YYYY-MM-DD format
				return len(result) == 10 && strings.Count(result, "-") == 2
			},
		},
		{
			name:   "time only layout",
			layout: "15:04:05",
			wantFunc: func(result string) bool {
				// Should match HH:MM:SS format
				return len(result) == 8 && strings.Count(result, ":") == 2
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := now(tt.layout)
			if !tt.wantFunc(result) {
				t.Errorf("now(%q) = %q, validation failed", tt.layout, result)
			}

			// Ensure result is not empty
			if result == "" {
				t.Errorf("now(%q) returned empty string", tt.layout)
			}
		})
	}
}

func TestToLower(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  string
	}{
		{
			name:  "uppercase string",
			input: "HELLO",
			want:  "hello",
		},
		{
			name:  "mixed case string",
			input: "HeLLo WoRlD",
			want:  "hello world",
		},
		{
			name:  "already lowercase",
			input: "hello",
			want:  "hello",
		},
		{
			name:  "empty string",
			input: "",
			want:  "",
		},
		{
			name:  "string with numbers",
			input: "Hello123",
			want:  "hello123",
		},
		{
			name:  "unicode characters",
			input: "HÉLLÖ WÖRLD",
			want:  "héllö wörld",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := toLower(tt.input)
			if got != tt.want {
				t.Errorf("toLower(%q) = %q, want %q", tt.input, got, tt.want)
			}
		})
	}
}

func TestToUpper(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  string
	}{
		{
			name:  "lowercase string",
			input: "hello",
			want:  "HELLO",
		},
		{
			name:  "mixed case string",
			input: "HeLLo WoRlD",
			want:  "HELLO WORLD",
		},
		{
			name:  "already uppercase",
			input: "HELLO",
			want:  "HELLO",
		},
		{
			name:  "empty string",
			input: "",
			want:  "",
		},
		{
			name:  "string with numbers",
			input: "Hello123",
			want:  "HELLO123",
		},
		{
			name:  "unicode characters",
			input: "héllö wörld",
			want:  "HÉLLÖ WÖRLD",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := toUpper(tt.input)
			if got != tt.want {
				t.Errorf("toUpper(%q) = %q, want %q", tt.input, got, tt.want)
			}
		})
	}
}

func TestNewFuncMap(t *testing.T) {
	funcMap := NewFuncMap()

	// Verify all expected functions are present
	expectedFuncs := []string{"now", "toLower", "toUpper"}

	for _, funcName := range expectedFuncs {
		if _, exists := funcMap[funcName]; !exists {
			t.Errorf("NewFuncMap() missing expected function: %s", funcName)
		}
	}

	// Verify the function map has the expected number of functions
	// (This ensures we don't accidentally add or remove functions)
	if len(funcMap) != len(expectedFuncs) {
		t.Errorf(
			"NewFuncMap() has %d functions, expected %d",
			len(funcMap),
			len(expectedFuncs),
		)
	}
}

func TestFuncMapIntegration(t *testing.T) {
	funcMap := NewFuncMap()

	// Create a template with the function map
	tmpl := template.New("test").Funcs(funcMap)

	// Test template with now function
	nowTemplate := `Date: {{now "2006-01-02"}}`
	parsed, err := tmpl.Parse(nowTemplate)
	if err != nil {
		t.Fatalf("Failed to parse template with now function: %v", err)
	}

	var buf strings.Builder
	err = parsed.Execute(&buf, nil)
	if err != nil {
		t.Fatalf("Failed to execute template with now function: %v", err)
	}

	result := buf.String()
	// Should contain a date in YYYY-MM-DD format
	if len(result) != 16 || !strings.Contains(result, "Date: ") {
		t.Errorf(
			"Template execution result = %q, expected format 'Date: YYYY-MM-DD'",
			result,
		)
	}

	// Test template with toLower function
	buf.Reset()
	lowerTemplate := `Result: {{ "HELLO" | toLower }}`
	parsed2, err := tmpl.Parse(lowerTemplate)
	if err != nil {
		t.Fatalf("Failed to parse template with toLower function: %v", err)
	}

	err = parsed2.Execute(&buf, nil)
	if err != nil {
		t.Fatalf("Failed to execute template with toLower function: %v", err)
	}

	result2 := buf.String()
	expected2 := "Result: hello"
	if result2 != expected2 {
		t.Errorf(
			"Template execution result = %q, expected %q",
			result2,
			expected2,
		)
	}

	// Test template with toUpper function
	buf.Reset()
	upperTemplate := `Result: {{ "world" | toUpper }}`
	parsed3, err := tmpl.Parse(upperTemplate)
	if err != nil {
		t.Fatalf("Failed to parse template with toUpper function: %v", err)
	}

	err = parsed3.Execute(&buf, nil)
	if err != nil {
		t.Fatalf("Failed to execute template with toUpper function: %v", err)
	}

	result3 := buf.String()
	expected3 := "Result: WORLD"
	if result3 != expected3 {
		t.Errorf(
			"Template execution result = %q, expected %q",
			result3,
			expected3,
		)
	}
}

func TestFunctionExtensibility(t *testing.T) {
	// Test that we can add new functions to the map
	funcMap := NewFuncMap()

	// Add a mock function for testing extensibility
	funcMap["testFunc"] = func() string { return "test" }

	// Verify the new function was added
	if _, exists := funcMap["testFunc"]; !exists {
		t.Error(
			"Failed to add new function to FuncMap - extensibility test failed",
		)
	}

	// Verify original functions are still present
	expectedFuncs := []string{"now", "toLower", "toUpper", "testFunc"}
	if len(funcMap) != len(expectedFuncs) {
		t.Errorf(
			"FuncMap has %d functions after extension, expected %d",
			len(funcMap),
			len(expectedFuncs),
		)
	}
}
