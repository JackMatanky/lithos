# Registry Package

The `registry` package provides thread-safe generic storage for shared resources in Lithos. It implements a CQRS-aware registry pattern with separate read and write interfaces, ensuring type-safe concurrent access to schemas, templates, and other shared data.

## Features

- **Generic Types**: Type-safe storage for any data type using Go generics
- **Thread-Safe**: Uses `sync.RWMutex` for concurrent read/write access
- **CQRS Pattern**: Separate interfaces for read and write operations
- **JSON Persistence**: Optional serialization/deserialization for index files
- **Zero Dependencies**: Pure Go standard library implementation

## Interfaces

### Reader[T any]

Provides read-only access to registry entries:

```go
type Reader[T any] interface {
    Get(key string) T                    // Retrieve value by key (returns zero value if not found)
    Exists(key string) bool             // Check if key exists
    ListKeys() []string                 // Get all keys
}
```

### Writer[T any]

Provides write-only access to registry entries:

```go
type Writer[T any] interface {
    Register(key string, value T)       // Store value with key
    Clear()                             // Remove all entries
}
```

### Registry[T any]

Combines read and write access:

```go
type Registry[T any] interface {
    Reader[T]
    Writer[T]
    Persister
}
```

### Persister

Provides persistence operations:

```go
type Persister interface {
    SaveIndex() ([]byte, error)         // Serialize to JSON
    LoadIndex(data []byte) error        // Deserialize from JSON
}
```

## Usage

### Basic Usage

```go
// Create a string registry
reg := registry.New[string]()

// Register values
reg.Register("greeting", "Hello, World!")
reg.Register("farewell", "Goodbye!")

// Retrieve values
greeting := reg.Get("greeting")  // "Hello, World!"
exists := reg.Exists("greeting")  // true
keys := reg.ListKeys()           // ["greeting", "farewell"]
```

### CQRS Pattern

```go
// In a service, use read interface for queries
type SchemaService struct {
    reader registry.Reader[Schema]
}

// In a loader, use write interface for updates
type SchemaLoader struct {
    writer registry.Writer[Schema]
}
```

### Persistence

```go
// Save registry to JSON
data, err := reg.SaveIndex()
if err != nil {
    log.Fatal(err)
}

// Load registry from JSON
err = reg.LoadIndex(data)
if err != nil {
    log.Fatal(err)
}
```

### Thread Safety

All operations are thread-safe and can be used concurrently:

```go
reg := registry.New[int]()

// Safe for concurrent use
go func() {
    reg.Register("counter", 1)
}()

go func() {
    value := reg.Get("counter")
    // value will be 1 or 0 depending on timing
}()
```

## Examples

### Schema Registry

```go
type Schema struct {
    Name string
    Fields []Field
}

reg := registry.New[Schema]()

schema := Schema{
    Name: "note",
    Fields: []Field{...},
}

reg.Register("note", schema)
retrieved := reg.Get("note")
```

### Template Registry

```go
reg := registry.New[Template]()

template := Template{...}
reg.Register("default.md", template)
```

## Error Handling

The registry uses Go's zero value semantics - `Get()` returns the zero value for the type if a key doesn't exist. Use `Exists()` to check presence:

```go
value := reg.Get("missing")  // returns ""
if reg.Exists("missing") {
    // key exists
} else {
    // key doesn't exist, value is zero value
}
```

Persistence operations return errors for JSON marshaling/unmarshaling failures.

## Performance

- Read operations use `RLock()` for concurrent access
- Write operations use `Lock()` for exclusive access
- JSON serialization uses `encoding/json` from stdlib
- Memory usage scales with number of stored items

## Thread Safety Guarantees

- **Reads**: Multiple concurrent readers allowed
- **Writes**: Exclusive write access
- **Read-Write**: Reads block during writes, writes block all operations
- **Persistence**: Save/Load operations are not concurrent with other operations

## Integration

This registry package is used by:

- **Schema Service**: Stores loaded JSON schemas
- **Template Service**: Stores loaded template definitions
- **Future Services**: Any component requiring thread-safe shared storage

## Best Practices

1. **Type Safety**: Use specific types, not `interface{}`
2. **Key Naming**: Use consistent key naming conventions
3. **Error Handling**: Check `Exists()` before assuming non-zero values
4. **CQRS**: Prefer read interfaces for query operations
5. **Persistence**: Use persistence for index files, not primary storage</content>
</xai:function_call"> 

<xai:function_call name="bash">
<parameter name="command">cd /Users/jack/Documents/41_personal/lithos && golangci-lint run --disable=golines ./internal/shared/registry/