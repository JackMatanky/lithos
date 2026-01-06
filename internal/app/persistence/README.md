# Persistence Package

The persistence package provides transaction management for multi-backend cache systems, supporting both sequential and parallel write strategies.

## Overview

Lithos maintains two cache backends:
- **Hot Cache**: BoltDB (fast, limited storage)
- **Deep Cache**: SQLite (slower, unlimited storage)

Writing to both backends requires transactional guarantees to ensure consistency.

## Architecture

### CacheTransaction

`CacheTransaction` coordinates write operations across multiple backends using a pluggable `WriteStrategy`.

```go
type CacheTransaction struct {
    writers    []spi.CacheWriterPort
    strategy   WriteStrategy
    operations []PersistenceOperation
}
```

**Key Features:**
- **Operation Staging**: Queue write/delete operations before execution
- **Strategy Pattern**: Pluggable execution strategies (sequential vs parallel)
- **Automatic Rollback**: On strategy execution failure

### WriteStrategy Interface

```go
type WriteStrategy interface {
    Execute(ctx, operations, writers) error
    Name() string
    Describe() string
}
```

## Provided Strategies

### SequentialWriteStrategy

Executes operations one-by-one on each writer in order.

**Use Case:**
- Debugging (predictable execution order)
- Simple transaction scenarios
- Single backend

**Pros:**
- Simple execution model
- Easy to debug
- Predictable order

**Cons:**
- Slower than parallel for multiple backends
- No performance benefit from concurrency

**Example:**
```go
tx := NewCacheTransaction(&SequentialWriteStrategy{}, boltWriter, sqliteWriter)
tx.AddWrite(note, metadata)
tx.AddDelete("old.md")
err := tx.Commit(ctx)
```

### ParallelWriteStrategy

Executes operations concurrently on all writers with coordinated rollback.

**Use Case:**
- Production (maximum throughput)
- Multi-backend writes
- Performance-critical operations

**Pros:**
- **2-3x faster** than sequential for multi-backend writes
- Utilizes goroutines and channels (idiomatic Go)
- Coordinated rollback on failure

**Cons:**
- More complex execution model
- Requires careful goroutine management
- Slightly higher memory overhead

**Performance:**
```
Sequential: ~20ms per multi-backend write
Parallel:   ~8ms per multi-backend write (2.5x improvement)
```

**Example:**
```go
tx := NewCacheTransaction(&ParallelWriteStrategy{}, boltWriter, sqliteWriter)
tx.AddWrite(note, metadata)
err := tx.Commit(ctx) // Executes concurrently on both backends
```

## Operation Types

### WriteOperation

Writes a note to cache with metadata.

```go
type WriteOperation struct {
    Note     domain.Note
    Metadata spi.CacheWriteMetadata
}
```

**Rollback Behavior:** Deletes the note from cache

### DeleteOperation

Removes a note from cache.

```go
type DeleteOperation struct {
    NotePath string
}
```

**Rollback Behavior:** No-op (delete is complex without read-before-delete)

## Usage Patterns

### Basic Write Transaction

```go
func persistNote(ctx context.Context, note domain.Note) error {
    tx := NewCacheTransaction(
        &ParallelWriteStrategy{}, // Use parallel for speed
        boltWriter,
        sqliteWriter,
    )

    // Stage operation
    tx.AddWrite(note, spi.CacheWriteMetadata{})

    // Execute (writes to both backends concurrently)
    return tx.Commit(ctx)
}
```

### Multiple Operations

```go
func batchUpdate(ctx context.Context, notes []domain.Note) error {
    tx := NewCacheTransaction(
        &ParallelWriteStrategy{},
        boltWriter,
        sqliteWriter,
    )

    // Stage multiple operations
    for _, note := range notes {
        tx.AddWrite(note, spi.CacheWriteMetadata{})
    }

    // Execute all in single transaction
    return tx.Commit(ctx)
}
```

### Mixed Write/Delete

```go
func migrateNotes(ctx context.Context, oldPath string, newNote domain.Note) error {
    tx := NewCacheTransaction(
        &ParallelWriteStrategy{},
        boltWriter,
        sqliteWriter,
    )

    tx.AddDelete(oldPath)
    tx.AddWrite(newNote, spi.CacheWriteMetadata{})

    return tx.Commit(ctx)
}
```

### Conditional Rollback

```go
tx := NewCacheTransaction(&ParallelWriteStrategy{}, boltWriter, sqliteWriter)
tx.AddWrite(note, metadata)

if someCondition {
    tx.Rollback() // Clears staged operations
    return nil
}

return tx.Commit(ctx)
}
```

## Design Decisions

### Why Strategy Pattern?

The transaction system needs flexibility in execution:
- **Testing**: Use sequential strategy for predictable test execution
- **Production**: Use parallel strategy for maximum performance
- **Future**: Add new strategies (retry, exponential backoff, etc.)

### Why Not Unit of Work?

The Unit of Work pattern was over-engineered for this use case:
- Too complex for simple write coordination
- Abstract base classes not idiomatic in Go
- Strategy pattern provides same flexibility with less code

### Why Goroutines and Channels?

Go's native concurrency primitives are ideal for parallel I/O:
- **Goroutines**: Lightweight threads for concurrent backend writes
- **Channels**: Safe communication between goroutines
- **Context**: Cancellation and timeout support

This follows Go idioms rather than thread pools or executors.

## Thread Safety

- **CacheTransaction**: Protected by `sync.Mutex` for staged operations
- **WriteStrategy implementations**: Stateless, thread-safe
- **ParallelWriteStrategy**: Uses `sync.WaitGroup` for goroutine coordination

## Error Handling

### Strategy Execution Failures

If a strategy fails during execution:
1. All operations are rolled back (for ParallelWriteStrategy)
2. Error includes strategy name for debugging
3. Transaction remains in failed state

### Individual Operation Failures

- **Sequential**: Fails immediately, stops remaining operations
- **Parallel**: Records all errors, returns `errors.Join()` of all failures

## Performance Benchmarks

See `tests/performance/transaction_bench_test.go` for benchmarks.

### Benchmark Results (Apple M4 Pro)

```
BenchmarkParallelWriteStrategy-12          100    8500000 ns/op    (8.5ms)
BenchmarkSequentialWriteStrategy-12         50   21000000 ns/op   (21.0ms)

Speedup: 2.47x
```

## Testing

Unit tests cover:
- Operation staging and execution
- Strategy implementations
- Rollback behavior
- Concurrent execution safety

Integration tests verify:
- Real BoltDB and SQLite backends
- Multi-backend coordination
- Failure scenarios

## Future Enhancements

Potential strategies to add:
- **RetryStrategy**: Retry failed operations with exponential backoff
- **CircuitBreakerStrategy**: Skip failed backends temporarily
- **AsyncStrategy**: Fire-and-forget writes with event confirmation
