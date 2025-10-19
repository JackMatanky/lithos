package errors

import (
	"errors"
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
