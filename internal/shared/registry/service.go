// Package registry provides a generic, thread-safe in-memory registry
// with CQRS-aware interfaces for storing and retrieving values by key.
//
// The registry uses Go generics for type safety and sync.RWMutex for
// thread-safe concurrent access. It follows CQRS principles with separate
// Reader and Writer interfaces to enable different access patterns.
package registry

import (
	"sort"
	"sync"

	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// Reader provides read-only access to registry values.
// It uses RLock for concurrent read operations.
type Reader[T any] interface {
	// Get retrieves a value by key. Returns the zero value and ErrNotFound
	// if the key does not exist.
	Get(key string) (T, error)

	// Exists checks if a key exists in the registry.
	Exists(key string) bool

	// ListKeys returns all keys in the registry, sorted alphabetically.
	ListKeys() []string
}

// Writer provides write-only access to registry values.
// It uses Lock for exclusive write operations.
type Writer[T any] interface {
	// Register stores a value with the given key.
	// Overwrites existing values if the key already exists.
	Register(key string, value T) error

	// Clear removes all keys and values from the registry.
	Clear()
}

// Registry combines Reader and Writer interfaces for full registry access.
type Registry[T any] interface {
	Reader[T]
	Writer[T]
}

// registry is the unexported implementation of the Registry interface.
// It uses sync.RWMutex for thread-safe concurrent access.
type registry[T any] struct {
	mu    sync.RWMutex
	items map[string]T
}

// New creates a new Registry instance with the specified type parameter.
// It initializes an empty map for storing key-value pairs.
func New[T any]() Registry[T] {
	return &registry[T]{
		mu:    sync.RWMutex{},
		items: make(map[string]T),
	}
}

// Get retrieves a value by key. Returns the zero value and ErrNotFound
// if the key does not exist. Uses RLock for concurrent read access.
func (r *registry[T]) Get(key string) (T, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()

	value, exists := r.items[key]
	if !exists {
		var zero T
		return zero, errors.ErrNotFound
	}
	return value, nil
}

// Exists checks if a key exists in the registry. Uses RLock for concurrent read
// access.
func (r *registry[T]) Exists(key string) bool {
	r.mu.RLock()
	defer r.mu.RUnlock()

	_, exists := r.items[key]
	return exists
}

// ListKeys returns all keys in the registry, sorted alphabetically.
// Uses RLock for concurrent read access.
func (r *registry[T]) ListKeys() []string {
	r.mu.RLock()
	defer r.mu.RUnlock()

	keys := make([]string, 0, len(r.items))
	for k := range r.items {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

// Register stores a value with the given key. Overwrites existing values.
// Uses Lock for exclusive write access.
func (r *registry[T]) Register(key string, value T) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.items[key] = value
	return nil
}

// Clear removes all keys and values from the registry.
// Uses Lock for exclusive write access.
func (r *registry[T]) Clear() {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.items = make(map[string]T)
}
