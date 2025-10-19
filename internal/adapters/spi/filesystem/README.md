# LocalFileSystemAdapter

The `LocalFileSystemAdapter` provides a concrete implementation of the `FileSystemPort` interface using Go's standard library filesystem operations. It follows the hexagonal architecture pattern by implementing the SPI port with atomic write guarantees and safe defaults for vault operations.

## Overview

This adapter wraps Go's `os`, `io`, and `path/filepath` packages to provide:
- Safe file reading operations
- Atomic file writing using temp file + rename pattern
- Directory tree traversal with callback support
- Proper error handling and cleanup

## Features

- **Atomic Writes**: Uses temp file + rename pattern to ensure consistency
- **Directory Creation**: Automatically creates parent directories as needed
- **Thread-Safe**: All operations are safe for concurrent use
- **Cross-Platform**: Uses Go's standard library for portability
- **Zero Dependencies**: Pure stdlib implementation

## Usage

### Basic File Operations

```go
package main

import (
    "fmt"
    "github.com/jack/lithos/internal/adapters/spi/filesystem"
    "github.com/jack/lithos/internal/ports/spi"
)

func main() {
    // Create adapter instance
    fs := filesystem.NewLocalFileSystemAdapter()
    
    // Write file atomically
    data := []byte("Hello, World!")
    err := fs.WriteFileAtomic("/path/to/file.txt", data)
    if err != nil {
        panic(err)
    }
    
    // Read file
    content, err := fs.ReadFile("/path/to/file.txt")
    if err != nil {
        panic(err)
    }
    
    fmt.Printf("Content: %s\n", content)
}
```

### Directory Traversal

```go
// Walk directory tree
err := fs.Walk("/vault/root", func(path string, isDir bool) error {
    if !isDir && strings.HasSuffix(path, ".md") {
        fmt.Printf("Found markdown file: %s\n", path)
        
        // Process the file
        content, err := fs.ReadFile(path)
        if err != nil {
            return err
        }
        
        // Do something with content...
    }
    return nil
})

if err != nil {
    panic(err)
}
```

### Dependency Injection

```go
// In domain service
type VaultIndexer struct {
    fs spi.FileSystemPort
}

func NewVaultIndexer(fs spi.FileSystemPort) *VaultIndexer {
    return &VaultIndexer{fs: fs}
}

// In main/DI container
func main() {
    fs := filesystem.NewLocalFileSystemAdapter()
    indexer := NewVaultIndexer(fs)
    
    // Use indexer...
}
```

## Implementation Details

### Atomic Write Pattern

The `WriteFileAtomic` method ensures consistency using the following pattern:

1. Create temporary file in same directory as target
2. Write data to temporary file
3. Sync to ensure data is on disk
4. Close temporary file
5. Atomically rename temp file to target

This guarantees that readers never see partial writes and that the operation is atomic at the filesystem level.

### Error Handling

All errors from underlying filesystem operations are propagated without modification. The adapter doesn't wrap or transform errors, allowing calling code to use standard Go error handling patterns like `errors.Is()` and `errors.As()`.

### Directory Creation

The `WriteFileAtomic` method automatically creates parent directories with 0750 permissions if they don't exist. This ensures that vault operations can create nested directory structures as needed.

### File Permissions

- Files are created with default permissions (system umask applied)
- Directories are created with 0750 permissions
- Temporary files use the same permissions as the target

## Testing

The adapter includes comprehensive unit tests covering:

- Basic read/write operations
- Atomic write guarantees
- Directory creation
- Directory traversal
- Error conditions
- Edge cases
- Interface compliance

Run tests with:

```bash
go test ./internal/adapters/spi/filesystem/ -v
```

## Thread Safety

All operations are thread-safe and can be used concurrently from multiple goroutines. The atomic write pattern ensures that concurrent writes to the same file are handled safely.

## Performance Considerations

- **Read Operations**: Direct delegation to `os.ReadFile` for optimal performance
- **Write Operations**: Small overhead from temp file creation, but ensures consistency
- **Directory Traversal**: Uses `filepath.WalkDir` for efficient tree traversal
- **Memory Usage**: Minimal memory overhead, temporary files are cleaned up automatically

## Error Conditions

The adapter handles these error conditions:

- **File Not Found**: Propagated from `os.ReadFile`
- **Permission Denied**: Propagated from filesystem operations
- **Disk Full**: Detected during write operations
- **Invalid Paths**: Handled by underlying filesystem operations
- **Directory Creation Failures**: Propagated during atomic writes

## Security Considerations

- **Path Validation**: All paths are treated as user-controlled input
- **File Permissions**: Uses secure default permissions (0750 for directories)
- **Temporary Files**: Cleaned up automatically on error or success
- **No Path Traversal**: Relies on Go's standard library path handling

## Integration with Lithos

This adapter is specifically designed for Lithos vault operations:

- Supports the atomic write patterns required for vault consistency
- Handles the file permissions appropriate for note files
- Provides the directory traversal needed for vault indexing
- Integrates with the hexagonal architecture for domain isolation

The adapter can be injected into any domain service that requires filesystem access, maintaining the separation between domain logic and infrastructure concerns.