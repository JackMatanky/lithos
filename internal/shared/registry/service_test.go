package registry

import (
	"sync"
	"testing"
)

const testValue1 = "value1"

// TestRegistryInterfaceDefinition verifies that the Registry interface is
// properly defined and can be used as a generic type parameter.
func TestRegistryInterfaceDefinition(t *testing.T) {
	// GREEN: Registry interface is now defined and can be used
	var reg Registry[string]
	_ = reg // Use the variable to avoid unused variable error

	// Test passes if compilation succeeds and interface is defined
	t.Log("Registry interface successfully defined with CQRS pattern")
}

// TestNewConstructor verifies that New() creates a working registry instance.
func TestNewConstructor(t *testing.T) {
	reg := New[string]()
	if reg == nil {
		t.Error("New() should return a non-nil registry")
	}

	// Verify it implements the Registry interface
	var _ = reg

	// Verify empty registry behavior
	exists := reg.Exists("nonexistent")
	if exists {
		t.Error("Empty registry should not contain any keys")
	}

	keys := reg.ListKeys()
	if len(keys) != 0 {
		t.Errorf("Empty registry should have 0 keys, got %d", len(keys))
	}
}

// TestRegisterAndGet verifies Register stores values and Get retrieves them.
func TestRegisterAndGet(t *testing.T) {
	reg := New[string]()

	// Register a value
	err := reg.Register("key1", testValue1)
	if err != nil {
		t.Errorf("Register should not return error, got %v", err)
	}

	// Get the value
	value, err := reg.Get("key1")
	if err != nil {
		t.Errorf("Get should not return error for existing key, got %v", err)
	}
	if value != testValue1 {
		t.Errorf("Get returned wrong value: got %q, want %q", value, testValue1)
	}

	// Verify Exists returns true
	if !reg.Exists("key1") {
		t.Error("Exists should return true for registered key")
	}

	// Test non-existent key
	_, err = reg.Get("nonexistent")
	if err == nil {
		t.Error("Get should return error for non-existent key")
	}

	if reg.Exists("nonexistent") {
		t.Error("Exists should return false for non-existent key")
	}
}

// TestListKeys verifies ListKeys returns all registered keys sorted.
func TestListKeys(t *testing.T) {
	reg := New[string]()

	// Register multiple keys
	if err := reg.Register("zebra", "animal"); err != nil {
		t.Errorf("Register failed: %v", err)
	}
	if err := reg.Register("apple", "fruit"); err != nil {
		t.Errorf("Register failed: %v", err)
	}
	if err := reg.Register("banana", "fruit"); err != nil {
		t.Errorf("Register failed: %v", err)
	}

	keys := reg.ListKeys()
	expected := []string{"apple", "banana", "zebra"}

	if len(keys) != len(expected) {
		t.Errorf(
			"ListKeys returned wrong number of keys: got %d, want %d",
			len(keys),
			len(expected),
		)
	}

	for i, key := range expected {
		if i >= len(keys) || keys[i] != key {
			t.Errorf(
				"ListKeys returned wrong order: got %v, want %v",
				keys,
				expected,
			)
			break
		}
	}
}

// TestClear verifies Clear removes all keys and values.
func TestClear(t *testing.T) {
	reg := New[string]()

	// Register some values
	if err := reg.Register("key1", "value1"); err != nil {
		t.Errorf("Register failed: %v", err)
	}
	if err := reg.Register("key2", "value2"); err != nil {
		t.Errorf("Register failed: %v", err)
	}

	// Verify they exist
	if !reg.Exists("key1") || !reg.Exists("key2") {
		t.Error("Keys should exist before clear")
	}

	// Clear the registry
	reg.Clear()

	// Verify they're gone
	if reg.Exists("key1") || reg.Exists("key2") {
		t.Error("Keys should not exist after clear")
	}

	keys := reg.ListKeys()
	if len(keys) != 0 {
		t.Errorf("Registry should be empty after clear, got %d keys", len(keys))
	}
}

// TestConcurrentReads verifies that multiple goroutines can read concurrently
// without blocking each other.
func TestConcurrentReads(t *testing.T) {
	reg := New[string]()

	// Register some test data
	if err := reg.Register("key1", testValue1); err != nil {
		t.Errorf("Register failed: %v", err)
	}
	if err := reg.Register("key2", "value2"); err != nil {
		t.Errorf("Register failed: %v", err)
	}

	const numGoroutines = 10
	const numReads = 100

	var wg sync.WaitGroup
	results := make(chan string, numGoroutines*numReads)

	// Start multiple goroutines reading concurrently
	for i := range numGoroutines {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			for range numReads {
				value, err := reg.Get("key1")
				if err != nil {
					t.Errorf("Goroutine %d: Get failed: %v", id, err)
					return
				}
				results <- value
			}
		}(i)
	}

	wg.Wait()
	close(results)

	// Verify all reads returned the correct value
	count := 0
	for result := range results {
		if result != testValue1 {
			t.Errorf("Expected '%s', got '%s'", testValue1, result)
		}
		count++
	}

	expectedCount := numGoroutines * numReads
	if count != expectedCount {
		t.Errorf("Expected %d results, got %d", expectedCount, count)
	}
}

// TestConcurrentAccess verifies that concurrent read/write operations work
// correctly.
func TestConcurrentAccess(t *testing.T) {
	reg := New[string]()

	const numOperations = 50

	// Start multiple goroutines performing mixed operations
	var wg sync.WaitGroup

	for range 3 {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := range numOperations {
				key := "key"
				// Alternate between read and write
				if j%2 == 0 {
					_ = reg.Register(key, "value") // Write
				} else {
					_, _ = reg.Get(key) // Read
				}
			}
		}()
	}

	wg.Wait()

	// Verify final state
	value, err := reg.Get("key")
	if err != nil {
		t.Errorf("Final Get failed: %v", err)
	}
	if value != "value" {
		t.Errorf("Expected 'value', got '%s'", value)
	}
}

// TestGenericInstantiation verifies that Registry works with different types.
func TestGenericInstantiation(t *testing.T) {
	// Test with string type
	stringReg := New[string]()
	if err := stringReg.Register("key1", "string_value"); err != nil {
		t.Errorf("String registry Register failed: %v", err)
	}
	value, err := stringReg.Get("key1")
	if err != nil {
		t.Errorf("String registry Get failed: %v", err)
	}
	if value != "string_value" {
		t.Errorf("Expected 'string_value', got '%s'", value)
	}

	// Test with int type
	intReg := New[int]()
	intRegErr := intReg.Register("key2", 42)
	if intRegErr != nil {
		t.Errorf("Int registry Register failed: %v", intRegErr)
	}
	intValue, intErr := intReg.Get("key2")
	if intErr != nil {
		t.Errorf("Int registry Get failed: %v", intErr)
	}
	if intValue != 42 {
		t.Errorf("Expected 42, got %d", intValue)
	}

	// Test with custom struct type
	type CustomStruct struct {
		Name  string
		Value int
	}

	customReg := New[CustomStruct]()
	customValue := CustomStruct{Name: "test", Value: 123}
	customRegErr := customReg.Register("key3", customValue)
	if customRegErr != nil {
		t.Errorf("Custom struct registry Register failed: %v", customRegErr)
	}
	retrieved, customErr := customReg.Get("key3")
	if customErr != nil {
		t.Errorf("Custom struct registry Get failed: %v", customErr)
	}
	if retrieved != customValue {
		t.Errorf("Expected %+v, got %+v", customValue, retrieved)
	}

	// Verify type safety - this should not compile if uncommented:
	// stringReg.Register("wrong_type", 123) // This would be a compile error
}

// BenchmarkConcurrentReads benchmarks concurrent read performance.
func BenchmarkConcurrentReads(b *testing.B) {
	reg := New[string]()

	// Pre-populate with test data
	for i := range 100 {
		_ = reg.Register(
			"key"+string(rune(i)),
			"value"+string(rune(i)),
		) // Ignore error for benchmark
	}

	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			_, _ = reg.Get("key50") // Ignore error for benchmark performance
		}
	})
}

// BenchmarkWriteContention benchmarks write performance under contention.
func BenchmarkWriteContention(b *testing.B) {
	reg := New[string]()

	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		i := 0
		for pb.Next() {
			_ = reg.Register(
				"key"+string(rune(i%100)),
				"value"+string(rune(i%100)),
			) // Ignore error for benchmark
			i++
		}
	})
}

// BenchmarkListKeys benchmarks ListKeys performance with large registry.
func BenchmarkListKeys(b *testing.B) {
	reg := New[string]()

	// Pre-populate with many keys
	for i := range 1000 {
		_ = reg.Register(
			"key"+string(rune(i)),
			"value"+string(rune(i)),
		) // Ignore error for benchmark
	}

	b.ResetTimer()
	for range b.N {
		keys := reg.ListKeys()
		_ = keys // Prevent optimization
	}
}
