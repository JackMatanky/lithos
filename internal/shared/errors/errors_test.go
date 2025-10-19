package errors

import (
	"errors"
	"strings"
	"testing"
)

func TestResult_IsOk(t *testing.T) {
	tests := []struct {
		name   string
		result Result[int]
		want   bool
	}{
		{
			name:   "ok result returns true",
			result: Ok(42),
			want:   true,
		},
		{
			name:   "error result returns false",
			result: Err[int](errors.New("test error")),
			want:   false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.result.IsOk(); got != tt.want {
				t.Errorf("Result.IsOk() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestResult_IsErr(t *testing.T) {
	tests := []struct {
		name   string
		result Result[int]
		want   bool
	}{
		{
			name:   "ok result returns false",
			result: Ok(42),
			want:   false,
		},
		{
			name:   "error result returns true",
			result: Err[int](errors.New("test error")),
			want:   true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.result.IsErr(); got != tt.want {
				t.Errorf("Result.IsErr() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestResult_Unwrap(t *testing.T) {
	tests := []struct {
		name    string
		result  Result[string]
		wantVal string
		wantErr bool
	}{
		{
			name:    "ok result unwraps value and nil error",
			result:  Ok("success"),
			wantVal: "success",
			wantErr: false,
		},
		{
			name:    "error result unwraps zero value and error",
			result:  Err[string](errors.New("test error")),
			wantVal: "",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			gotVal, gotErr := tt.result.Unwrap()
			if gotVal != tt.wantVal {
				t.Errorf(
					"Result.Unwrap() gotVal = %v, want %v",
					gotVal,
					tt.wantVal,
				)
			}
			if (gotErr != nil) != tt.wantErr {
				t.Errorf(
					"Result.Unwrap() gotErr = %v, wantErr %v",
					gotErr,
					tt.wantErr,
				)
			}
		})
	}
}

func TestResult_Value(t *testing.T) {
	t.Run("ok result returns value", func(t *testing.T) {
		result := Ok("test value")
		if got := result.Value(); got != "test value" {
			t.Errorf("Result.Value() = %v, want %v", got, "test value")
		}
	})

	t.Run("error result panics on Value()", func(t *testing.T) {
		defer func() {
			if r := recover(); r == nil {
				t.Errorf("Result.Value() should panic on error result")
			}
		}()
		result := Err[string](errors.New("test error"))
		_ = result.Value()
	})
}

func TestResult_Error(t *testing.T) {
	t.Run("ok result returns nil error", func(t *testing.T) {
		result := Ok(42)
		if got := result.Error(); got != nil {
			t.Errorf("Result.Error() = %v, want nil", got)
		}
	})

	t.Run("error result returns error", func(t *testing.T) {
		testErr := errors.New("test error")
		result := Err[int](testErr)
		if got := result.Error(); !errors.Is(got, testErr) {
			t.Errorf("Result.Error() = %v, want %v", got, testErr)
		}
	})
}

func TestOk(t *testing.T) {
	result := Ok("test")
	if !result.IsOk() {
		t.Error("Ok() should create ok result")
	}
	if result.IsErr() {
		t.Error("Ok() should not create error result")
	}
	if val := result.Value(); val != "test" {
		t.Errorf("Ok() value = %v, want %v", val, "test")
	}
}

func TestErr(t *testing.T) {
	testErr := errors.New("test error")
	result := Err[string](testErr)
	if result.IsOk() {
		t.Error("Err() should not create ok result")
	}
	if !result.IsErr() {
		t.Error("Err() should create error result")
	}
	if err := result.Error(); !errors.Is(err, testErr) {
		t.Errorf("Err() error = %v, want %v", err, testErr)
	}
}

func TestWrap(t *testing.T) {
	originalErr := errors.New("original error")
	wrappedErr := Wrap(originalErr, "additional context")

	errStr := wrappedErr.Error()
	if !strings.Contains(errStr, "additional context") {
		t.Errorf("Wrap() error message should contain context: %s", errStr)
	}
	if !strings.Contains(errStr, "original error") {
		t.Errorf(
			"Wrap() error message should contain original error: %s",
			errStr,
		)
	}

	// Test that wrapped error still contains original
	if !errors.Is(wrappedErr, originalErr) {
		t.Error("Wrap() should preserve original error for errors.Is()")
	}
}

func TestWrapWithContext(t *testing.T) {
	originalErr := errors.New("original error")
	context := map[string]interface{}{
		"operation": "read",
		"path":      "/test/file",
		"attempt":   3,
	}
	wrappedErr := WrapWithContext(originalErr, context)

	errStr := wrappedErr.Error()
	expectedParts := []string{
		"operation=read",
		"path=/test/file",
		"attempt=3",
		"original error",
	}
	for _, part := range expectedParts {
		if !strings.Contains(errStr, part) {
			t.Errorf(
				"WrapWithContext() error message should contain %s: %s",
				part,
				errStr,
			)
		}
	}

	// Test that wrapped error still contains original
	if !errors.Is(wrappedErr, originalErr) {
		t.Error(
			"WrapWithContext() should preserve original error for errors.Is()",
		)
	}
}

func TestJoinErrors(t *testing.T) {
	err1 := errors.New("error 1")
	err2 := errors.New("error 2")
	joinedErr := JoinErrors(err1, err2)

	// Test that joined error contains both
	if !errors.Is(joinedErr, err1) {
		t.Error("JoinErrors() should contain first error")
	}
	if !errors.Is(joinedErr, err2) {
		t.Error("JoinErrors() should contain second error")
	}

	errStr := joinedErr.Error()
	if !strings.Contains(errStr, "error 1") ||
		!strings.Contains(errStr, "error 2") {
		t.Errorf("JoinErrors() should contain both error messages: %s", errStr)
	}
}

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

func TestResultWithDifferentTypes(t *testing.T) {
	// Test Result[T] with different generic types
	intResult := Ok(42)
	if val := intResult.Value(); val != 42 {
		t.Errorf("Result[int] value = %v, want 42", val)
	}

	stringResult := Ok("hello")
	if val := stringResult.Value(); val != "hello" {
		t.Errorf("Result[string] value = %v, want 'hello'", val)
	}

	boolResult := Ok(true)
	if val := boolResult.Value(); val != true {
		t.Errorf("Result[bool] value = %v, want true", val)
	}
}
