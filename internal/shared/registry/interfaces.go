// Package registry provides thread-safe generic storage for shared resources.
//
// This package implements a CQRS-aware registry pattern with separate read and
// write interfaces. It uses sync.RWMutex for concurrent access and supports
// generic types for type-safe storage of schemas, templates, and other shared
// resources.
//
// Thread safety is guaranteed for all operations. The registry is designed for
// in-memory storage with optional JSON persistence for index files.
package registry

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
