# TEA Knowledge: Performance & Benchmarking

## CONTEXT

- **Applies to**: Performance-critical paths and regression prevention
- **Purpose**: Measure execution time, memory usage, and throughput
- **Tools**: `criterion`, `black_box`

## VALIDATION CHECKLIST

### Benchmark Design

- [ ] Benchmarks are representative of real-world usage
- [ ] Uses `criterion` for statistical significance
- [ black_box` is used to prevent compiler optimizations from skewing results
- [ ] Warmup and measurement phases are configured appropriately

### Memory Profiling

- [ ] Allocation tracking for high-frequency operations
- [ ] Zero-copy patterns are verified for performance-critical paths

## ANTI-PATTERNS (FLAG THESE)

- ❌ **Naive benchmarking** → Using `Instant::now()` without accounting for JIT/warmup
- ❌ **Optimized-away benchmarks** → Compiler removing code that doesn't affect output
- ❌ **Shared state skew** → Benchmarks interfering with each other's cache/state

## CORRECT EXAMPLES

### Basic Benchmark Structure

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_note_creation(c: &mut Criterion) {
    let title = "Test Note Title";
    let content = "Test note content";

    c.bench_function("note_creation", |b| {
        b.iter(|| {
            let note = Note::new(
                NoteId::new_random(),
                NoteTitle::new(black_box(title)).unwrap(),
                NoteContent::new(black_box(content)).unwrap(),
                Timestamp::now(),
            );
            black_box(note)
        })
    });
}

criterion_group!(benches, benchmark_note_creation);
criterion_main!(benches);
```

### Comparative Benchmarks

```rust
fn benchmark_search_algorithms(c: &mut Criterion) {
    let notes: Vec<Note> = (0..1000).map(|i| create_test_note(i)).collect();
    let search_term = "test";

    c.bench_function("linear_search", |b| {
        b.iter(|| {
            notes.iter().find(|note| note.content().contains(search_term))
        })
    });

    let indexed_notes = create_indexed_notes(&notes);

    c.bench_function("indexed_search", |b| {
        b.iter(|| {
            indexed_notes.get(search_term)
        })
    });
}
```

### Allocation Tracking

```rust
#[test]
fn test_memory_efficiency() {
    let notes: Vec<Note> = (0..1000).map(|i| create_test_note(i)).collect();

    // Use criterion's memory profiling
    let mut group = criterion::BenchmarkGroup::new("memory_usage");
    group.measured_time = false;

    group.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let processed: Vec<ProcessedNote> = notes
                .iter()
                .map(|note| process_note(black_box(note)))
                .collect();
            black_box(processed.len())
        })
    });

    group.finish();
}
```

## RELATED MODULES

- See `anti-patterns.md` for performance anti-patterns
- See `unit.md` for standard unit testing
