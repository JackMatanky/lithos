package errors

import (
	"errors"
	"testing"
)

const (
	testErrorMessage = "boom"
)

func TestResultStateQueries(t *testing.T) {
	ok := Ok(42)
	if !ok.IsOk() || ok.IsErr() {
		t.Fatalf("expected ok result to report IsOk")
	}

	err := errors.New(testErrorMessage)
	bad := Err[int](err)
	if !bad.IsErr() || bad.IsOk() {
		t.Fatalf("expected error result to report IsErr")
	}
	if !errors.Is(bad.Error(), err) {
		t.Fatalf("expected contained error to match original")
	}
	if !errors.Is(bad.Err(), err) {
		t.Fatalf("Err() should mirror Error()")
	}
}

func TestUnwrapAndValue(t *testing.T) {
	value, err := Ok("hello").Unwrap()
	if err != nil || value != "hello" {
		t.Fatalf("unexpected unwrap values: %q, %v", value, err)
	}

	defer func() {
		if recover() == nil {
			t.Fatalf("Value() should panic when called on error result")
		}
	}()

	_ = Err[string](errors.New("boom")).Value()
}

func TestErrPanicsOnNil(t *testing.T) {
	defer func() {
		if recover() == nil {
			t.Fatalf("Err(nil) must panic to prevent silent misuse")
		}
	}()
	_ = Err[int](nil)
}

func TestValueOr(t *testing.T) {
	if got := Ok("value").ValueOr("fallback"); got != "value" {
		t.Fatalf("expected ok result to preserve value")
	}
	if got := Err[string](errors.New("boom")).ValueOr("fallback"); got != "fallback" {
		t.Fatalf("expected error result to return fallback value")
	}
}

func TestMap(t *testing.T) {
	mapped := Map(Ok(21), func(v int) int { return v * 2 })
	if mappedValue := mapped.Value(); mappedValue != 42 {
		t.Fatalf("expected mapped value to equal 42, got %d", mappedValue)
	}

	err := errors.New(testErrorMessage)
	propagated := Map(Err[int](err), func(v int) int { return v * 2 })
	if !errors.Is(propagated.Err(), err) {
		t.Fatalf("expected original error to propagate through Map")
	}
}

func TestAndThen(t *testing.T) {
	toString := func(v int) Result[string] {
		return Ok("value: " + string(rune(v)))
	}

	ok := AndThen(Ok(65), toString)
	if val := ok.Value(); val != "value: A" {
		t.Fatalf("unexpected AndThen value: %s", val)
	}

	err := errors.New(testErrorMessage)
	propagated := AndThen(Err[int](err), toString)
	if !errors.Is(propagated.Err(), err) {
		t.Fatalf("expected error to propagate across AndThen")
	}
}

func TestOrElse(t *testing.T) {
	recovered := Err[int](errors.New("boom")).OrElse(
		func(err error) Result[int] {
			if err.Error() != testErrorMessage {
				t.Fatalf("unexpected error passed to OrElse: %v", err)
			}
			return Ok(1)
		},
	)
	if value := recovered.Value(); value != 1 {
		t.Fatalf("expected recovery branch to return 1, got %d", value)
	}

	unchanged := Ok(2).OrElse(func(err error) Result[int] {
		t.Fatalf("OrElse should not run when result is ok")
		return Err[int](err)
	})
	if value := unchanged.Value(); value != 2 {
		t.Fatalf("expected ok branch to remain unchanged")
	}
}

func TestInspect(t *testing.T) {
	var seenValue int
	var seenErr error

	Ok(9).Inspect(func(v int) { seenValue = v })
	Err[int](
		errors.New("boom"),
	).InspectErr(func(err error) { seenErr = err })

	if seenValue != 9 {
		t.Fatalf("Inspect should observe contained value")
	}
	if seenErr == nil || seenErr.Error() != "boom" {
		t.Fatalf("InspectErr should observe contained error")
	}

	Ok(1).InspectErr(func(err error) {
		t.Fatalf("InspectErr must not run on ok result")
	})
	Err[int](errors.New("boom")).Inspect(func(v int) {
		t.Fatalf("Inspect must not run on error result")
	})
}
