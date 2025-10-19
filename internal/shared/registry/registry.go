// Package registry provides thread-safe generic storage for shared resources.
//
// This package implements a CQRS-aware registry pattern with separate read and write
// interfaces. It uses sync.RWMutex for concurrent access and supports generic types
// for type-safe storage of schemas, templates, and other shared resources.
//
// Thread safety is guaranteed for all operations. The registry is designed for
// in-memory storage with optional JSON persistence for index files.
package registry

import (
	"encoding/json"
	"sync"
)

// Reader provides read-only access to registry entries.
type Reader[T any] interface {
	// Get retrieves a value by key. Returns the zero value if not found.
	Get(key string) T

	// Exists checks if a key exists in the registry.
	Exists(key string) bool

	// ListKeys returns all keys in the registry.
	ListKeys() []string
}

// Writer provides write-only access to registry entries.
type Writer[T any] interface {
	// Register stores a value with the given key.
	Register(key string, value T)

	// Clear removes all entries from the registry.
	Clear()
}

// Persister provides persistence operations for registry indexes.
type Persister interface {
	// SaveIndex serializes the registry to JSON.
	SaveIndex() ([]byte, error)

	// LoadIndex deserializes the registry from JSON.
	LoadIndex(data []byte) error
}

// Registry combines read and write access to registry entries.
type Registry[T any] interface {
	Reader[T]
	Writer[T]
	Persister
}

// registry implements the Registry interface with thread-safe operations.
type registry[T any] struct {
	mu    sync.RWMutex
	store map[string]T
}

// New creates a new thread-safe registry for type T.
func New[T any]() Registry[T] {
	return &registry[T]{
		mu:    sync.RWMutex{},
		store: make(map[string]T),
	}
}

// Get retrieves a value by key. Returns the zero value if not found.
func (r *registry[T]) Get(key string) T {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.store[key]
}

// Exists checks if a key exists in the registry.
func (r *registry[T]) Exists(key string) bool {
	r.mu.RLock()
	defer r.mu.RUnlock()
	_, exists := r.store[key]
	return exists
}

// ListKeys returns all keys in the registry.
func (r *registry[T]) ListKeys() []string {
	r.mu.RLock()
	defer r.mu.RUnlock()
	keys := make([]string, 0, len(r.store))
	for key := range r.store {
		keys = append(keys, key)
	}
	return keys
}

// Register stores a value with the given key.
func (r *registry[T]) Register(key string, value T) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.store[key] = value
}

// Clear removes all entries from the registry.
func (r *registry[T]) Clear() {
	r.mu.Lock()
	defer r.mu.Unlock()
	clear(r.store)
}

// SaveIndex serializes the registry to JSON.
func (r *registry[T]) SaveIndex() ([]byte, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return json.Marshal(r.store)
}

// LoadIndex deserializes the registry from JSON.
func (r *registry[T]) LoadIndex(data []byte) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	return json.Unmarshal(data, &r.store)
}
