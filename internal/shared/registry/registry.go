package registry

import (
	"encoding/json"
	"sync"
)

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
