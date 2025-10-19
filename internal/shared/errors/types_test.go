package errors

import (
	"errors"
	"testing"
)

func TestValidationError_Error(t *testing.T) {
	err := NewValidationError("email", "invalid format", "test@")
	errStr := err.Error()

	expected := "[ValidationError] field 'email': invalid format"
	if errStr != expected {
		t.Errorf("ValidationError.Error() = %v, want %v", errStr, expected)
	}
}

func TestNotFoundError_Error(t *testing.T) {
	err := NewNotFoundError("user", "123")
	errStr := err.Error()

	expected := "[NotFoundError] user '123' not found"
	if errStr != expected {
		t.Errorf("NotFoundError.Error() = %v, want %v", errStr, expected)
	}
}

func TestConfigurationError_Error(t *testing.T) {
	err := NewConfigurationError("database.url", "missing required value")
	errStr := err.Error()

	expected := "[ConfigurationError] key 'database.url': missing required value"
	if errStr != expected {
		t.Errorf("ConfigurationError.Error() = %v, want %v", errStr, expected)
	}
}

func TestTemplateError_Error(t *testing.T) {
	tests := []struct {
		name     string
		error    TemplateError
		expected string
	}{
		{
			name:     "with line number",
			error:    NewTemplateError("welcome.tmpl", 5, "undefined variable"),
			expected: "[TemplateError] template 'welcome.tmpl' line 5: undefined variable",
		},
		{
			name:     "without line number",
			error:    NewTemplateError("welcome.tmpl", 0, "syntax error"),
			expected: "[TemplateError] template 'welcome.tmpl': syntax error",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.error.Error(); got != tt.expected {
				t.Errorf(
					"TemplateError.Error() = %v, want %v",
					got,
					tt.expected,
				)
			}
		})
	}
}

func TestSchemaError_Error(t *testing.T) {
	err := NewSchemaError("user.schema", "invalid field type")
	errStr := err.Error()

	expected := "[SchemaError] schema 'user.schema': invalid field type"
	if errStr != expected {
		t.Errorf("SchemaError.Error() = %v, want %v", errStr, expected)
	}
}

func TestStorageError_Error(t *testing.T) {
	tests := []struct {
		name     string
		error    StorageError
		expected string
	}{
		{
			name: "with cause",
			error: NewStorageError(
				"write",
				"/data/cache",
				errors.New("disk full"),
			),
			expected: "[StorageError] write '/data/cache': disk full",
		},
		{
			name:     "without cause",
			error:    NewStorageError("read", "/data/cache", nil),
			expected: "[StorageError] read '/data/cache' failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.error.Error(); got != tt.expected {
				t.Errorf("StorageError.Error() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestFileSystemError_Error(t *testing.T) {
	tests := []struct {
		name     string
		error    FileSystemError
		expected string
	}{
		{
			name: "with cause",
			error: NewFileSystemError(
				"open",
				"/tmp/test.txt",
				errors.New("permission denied"),
			),
			expected: "[FileSystemError] open '/tmp/test.txt': permission denied",
		},
		{
			name:     "without cause",
			error:    NewFileSystemError("stat", "/tmp/test.txt", nil),
			expected: "[FileSystemError] stat '/tmp/test.txt' failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.error.Error(); got != tt.expected {
				t.Errorf(
					"FileSystemError.Error() = %v, want %v",
					got,
					tt.expected,
				)
			}
		})
	}
}

func TestErrorTypesImplementErrorInterface(t *testing.T) {
	// Test that all error types implement the error interface
	var err error

	// ValidationError
	err = ValidationError{}
	_ = err.Error()

	// NotFoundError
	err = NotFoundError{}
	_ = err.Error()

	// ConfigurationError
	err = ConfigurationError{}
	_ = err.Error()

	// TemplateError
	err = TemplateError{}
	_ = err.Error()

	// SchemaError
	err = SchemaError{}
	_ = err.Error()

	// StorageError
	err = StorageError{}
	_ = err.Error()

	// FileSystemError
	err = FileSystemError{}
	_ = err.Error()
}
