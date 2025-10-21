package registry

import (
	"encoding/json"
	"reflect"
	"sync"
	"testing"
)

const testRegistryValue = "value1"

func TestRegistry_Get(t *testing.T) {
	reg := New[string]()

	// Test getting non-existent key returns zero value
	if got := reg.Get("nonexistent"); got != "" {
		t.Errorf("Get(nonexistent) = %v, want empty string", got)
	}

	// Test getting existing key
	reg.Register("key1", testRegistryValue)
	if got := reg.Get("key1"); got != testRegistryValue {
		t.Errorf("Get(key1) = %v, want %s", got, testRegistryValue)
	}
}

func TestRegistry_Exists(t *testing.T) {
	reg := New[int]()

	// Test non-existent key
	if reg.Exists("nonexistent") {
		t.Error("Exists(nonexistent) = true, want false")
	}

	// Test existing key
	reg.Register("key1", 42)
	if !reg.Exists("key1") {
		t.Error("Exists(key1) = false, want true")
	}
}

func TestRegistry_ListKeys(t *testing.T) {
	reg := New[bool]()

	// Test empty registry
	keys := reg.ListKeys()
	if len(keys) != 0 {
		t.Errorf("ListKeys() on empty registry = %v, want empty slice", keys)
	}

	// Test with entries
	reg.Register("key1", true)
	reg.Register("key2", false)
	reg.Register("key3", true)

	keys = reg.ListKeys()
	if len(keys) != 3 {
		t.Errorf("ListKeys() = %v, want 3 keys", keys)
	}

	// Check all keys are present (order doesn't matter)
	keyMap := make(map[string]bool)
	for _, key := range keys {
		keyMap[key] = true
	}

	expectedKeys := []string{"key1", "key2", "key3"}
	for _, expected := range expectedKeys {
		if !keyMap[expected] {
			t.Errorf("ListKeys() missing expected key: %s", expected)
		}
	}
}

func TestRegistry_Register(t *testing.T) {
	reg := New[float64]()

	// Test registering and retrieving
	reg.Register("pi", 3.14159)
	if got := reg.Get("pi"); got != 3.14159 {
		t.Errorf("Get(pi) after Register = %v, want 3.14159", got)
	}

	// Test overwriting existing value
	reg.Register("pi", 3.14)
	if got := reg.Get("pi"); got != 3.14 {
		t.Errorf("Get(pi) after overwrite = %v, want 3.14", got)
	}
}

func TestRegistry_Clear(t *testing.T) {
	reg := New[string]()

	// Add some entries
	reg.Register("key1", "value1")
	reg.Register("key2", "value2")

	// Verify entries exist
	if !reg.Exists("key1") || !reg.Exists("key2") {
		t.Error("Entries should exist before Clear")
	}

	// Clear and verify
	reg.Clear()

	if reg.Exists("key1") || reg.Exists("key2") {
		t.Error("Entries should not exist after Clear")
	}

	keys := reg.ListKeys()
	if len(keys) != 0 {
		t.Errorf("ListKeys() after Clear = %v, want empty slice", keys)
	}
}

func TestRegistry_SaveIndex(t *testing.T) {
	reg := New[int]()

	reg.Register("a", 1)
	reg.Register("b", 2)

	data, err := reg.SaveIndex()
	if err != nil {
		t.Fatalf("SaveIndex() error = %v", err)
	}

	// Verify it's valid JSON
	var result map[string]int
	if unmarshalErr := json.Unmarshal(data, &result); unmarshalErr != nil {
		t.Fatalf("SaveIndex() produced invalid JSON: %v", unmarshalErr)
	}

	expected := map[string]int{"a": 1, "b": 2}
	if !reflect.DeepEqual(result, expected) {
		t.Errorf("SaveIndex() = %v, want %v", result, expected)
	}
}

func TestRegistry_LoadIndex(t *testing.T) {
	reg := New[string]()

	// Prepare JSON data
	data := `{"key1":"value1","key2":"value2"}`

	err := reg.LoadIndex([]byte(data))
	if err != nil {
		t.Fatalf("LoadIndex() error = %v", err)
	}

	// Verify entries were loaded
	if got := reg.Get("key1"); got != "value1" {
		t.Errorf("Get(key1) after LoadIndex = %v, want value1", got)
	}
	if got := reg.Get("key2"); got != "value2" {
		t.Errorf("Get(key2) after LoadIndex = %v, want value2", got)
	}
}

func TestRegistry_LoadIndex_InvalidJSON(t *testing.T) {
	reg := New[int]()

	invalidJSON := `{"invalid": json}`

	err := reg.LoadIndex([]byte(invalidJSON))
	if err == nil {
		t.Error("LoadIndex() with invalid JSON should return error")
	}
}

func TestRegistry_ThreadSafety(t *testing.T) {
	reg := New[int]()
	const numGoroutines = 100
	const numOperations = 100

	var wg sync.WaitGroup
	wg.Add(numGoroutines * 2) // readers and writers

	// Start writers
	for i := 0; i < numGoroutines; i++ { //nolint:intrange // need index for goroutine ID
		go func(id int) {
			defer wg.Done()
			for j := 0; j < numOperations; j++ { //nolint:intrange // need index for operation count
				reg.Register(string(rune('a'+id)), id*j)
			}
		}(i)
	}

	// Start readers
	for i := 0; i < numGoroutines; i++ { //nolint:intrange // need index for goroutine ID
		go func(id int) {
			defer wg.Done()
			for j := 0; j < numOperations; j++ { //nolint:intrange // need index for operation count
				_ = reg.Get(string(rune('a' + id)))
				_ = reg.Exists(string(rune('a' + id)))
				_ = reg.ListKeys()
			}
		}(i)
	}

	wg.Wait()

	// Verify no race conditions caused panics or corruption
	// (This test passes if it completes without panicking)
}

func TestRegistry_GenericTypes(t *testing.T) {
	// Test with different types
	stringReg := New[string]()
	intReg := New[int]()
	boolReg := New[bool]()

	// String registry
	stringReg.Register("hello", "world")
	if got := stringReg.Get("hello"); got != "world" {
		t.Errorf("String registry Get = %v, want world", got)
	}

	// Int registry
	intReg.Register("answer", 42)
	if got := intReg.Get("answer"); got != 42 {
		t.Errorf("Int registry Get = %v, want 42", got)
	}

	// Bool registry
	boolReg.Register("flag", true)
	if got := boolReg.Get("flag"); got != true {
		t.Errorf("Bool registry Get = %v, want true", got)
	}
}

func TestRegistry_ZeroValues(t *testing.T) {
	reg := New[int]()

	// Non-existent key should return zero value for int (0)
	if got := reg.Get("missing"); got != 0 {
		t.Errorf("Get(missing) for int registry = %v, want 0", got)
	}

	reg.Register("zero", 0)
	if got := reg.Get("zero"); got != 0 {
		t.Errorf("Get(zero) = %v, want 0", got)
	}
	if !reg.Exists("zero") {
		t.Error("Exists(zero) should return true even for zero value")
	}
}
