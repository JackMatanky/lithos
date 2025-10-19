# SPI Ports Package

The `spi` package defines Service Provider Interface (SPI) ports for the hexagonal architecture. These interfaces define contracts that infrastructure adapters must implement to provide services to the domain layer.

## Overview

SPI ports are "driven" interfaces that allow the domain to remain independent of external dependencies. They represent the needs of the domain and must be implemented by infrastructure adapters.

## FileSystemPort

The `FileSystemPort` provides safe file read/write/walk operations for domain services to interact with the vault without importing the `os` package directly.

### Interface Definition

```go
type FileSystemPort interface {
    ReadFile(path string) ([]byte, error)
    WriteFileAtomic(path string, data []byte) error
    Walk(root string, fn WalkFunc) error
}
```

### Methods

#### ReadFile

```go
ReadFile(path string) ([]byte, error)
```

Reads the contents of a file at the given path. Returns the file contents or an error if the file cannot be read.

**Parameters:**
- `path`: The file path to read

**Returns:**
- `[]byte`: The file contents
- `error`: Any error that occurred during reading

#### WriteFileAtomic

```go
WriteFileAtomic(path string, data []byte) error
```

Writes data to a file atomically using temp file + rename pattern. This ensures that concurrent readers never see partial writes. The file is created with appropriate permissions if it doesn't exist.

**Parameters:**
- `path`: The file path to write to
- `data`: The data to write

**Returns:**
- `error`: Any error that occurred during writing

**Guarantees:**
- Atomic write operation (temp file + rename)
- Directory creation if needed
- No partial writes visible to readers

#### Walk

```go
Walk(root string, fn WalkFunc) error
```

Traverses a directory tree starting at root, calling fn for each file and directory encountered. The fn callback receives the file path and a boolean indicating if it's a directory.

**Parameters:**
- `root`: The root directory to start walking from
- `fn`: The callback function to call for each file/directory

**Returns:**
- `error`: Any error that occurred during walking or returned by the callback

### WalkFunc Type

```go
type WalkFunc func(path string, isDir bool) error
```

The callback function type used by the Walk method. Return an error to stop walking and propagate the error up.

**Parameters:**
- `path`: The file or directory path
- `isDir`: True if the path is a directory, false if it's a file

**Returns:**
- `error`: Return an error to stop walking

## Usage Example

```go
// Dependency injection in domain service
type VaultService struct {
    fs spi.FileSystemPort
}

func NewVaultService(fs spi.FileSystemPort) *VaultService {
    return &VaultService{fs: fs}
}

func (v *VaultService) ReadNote(path string) ([]byte, error) {
    return v.fs.ReadFile(path)
}

func (v *VaultService) SaveNote(path string, content []byte) error {
    return v.fs.WriteFileAtomic(path, content)
}

func (v *VaultService) IndexVault(vaultRoot string) error {
    return v.fs.Walk(vaultRoot, func(path string, isDir bool) error {
        if !isDir && strings.HasSuffix(path, ".md") {
            // Process markdown file
            content, err := v.fs.ReadFile(path)
            if err != nil {
                return err
            }
            // Index the content...
        }
        return nil
    })
}
```

## Implementation

The `FileSystemPort` is implemented by:
- `LocalFileSystemAdapter` in `internal/adapters/spi/filesystem/`

## Design Principles

1. **Interface Segregation**: Small, focused interfaces with ≤3 methods
2. **Dependency Inversion**: Domain defines the interface, infrastructure implements it
3. **Atomic Operations**: WriteFileAtomic guarantees consistency
4. **Error Transparency**: Errors are propagated without modification
5. **Path Safety**: All paths are treated as user-controlled input

## Testing

The port interface itself doesn't require unit tests as it's a contract definition. However, implementations should be thoroughly tested with:

- File read/write operations
- Atomic write guarantees
- Directory traversal
- Error conditions
- Edge cases

See `internal/adapters/spi/filesystem/filesystem_test.go` for implementation testing examples.