# Performance Testing & Profiling

This directory contains performance benchmarks and profiling utilities for Lithos.

## Quick Start

```bash
# Show performance dashboard
mise run perf-dashboard

# Run performance test (10k notes)
mise run perf-test

# Generate all profiles
mise run perf-profile-all

# Analyze CPU usage
mise run perf-analyze-cpu
```

## Test Vaults

Synthetic vaults are generated automatically during tests:

| Command | Notes | Use Case |
|---------|-------|----------|
| `mise run perf-test-small` | 250 | Quick smoke test |
| `mise run perf-test-medium` | 2,500 | Realistic mid-size vault |
| `mise run perf-test-large` | 10,000 | Large vault stress test |
| `mise run perf-test-massive` | 100,000 | Extreme edge case (slow!) |

## Profiling Workflow

### 1. Generate Baseline

```bash
# Save current performance as baseline
mise run perf-baseline
```

This creates a timestamped baseline file in `tests/artifacts/baseline/`.

### 2. Make Changes

Edit code, optimize algorithms, etc.

### 3. Compare Performance

```bash
# Compare with baseline
mise run perf-compare
```

### 4. Profile to Find Bottlenecks

```bash
# Generate CPU profile
mise run perf-profile-cpu

# Analyze interactively
mise run perf-analyze-cpu

# Or generate text report
mise run perf-report-cpu
```

### 5. Optimize Hot Paths

Use profiling data to identify:
- Functions taking >10% of total time
- Unexpected allocations in hot loops
- Lock contention in concurrent code
- Syscall blocking

### 6. Verify Improvement

```bash
# Run baseline again and compare
mise run perf-baseline
```

## Profile Types

### CPU Profile

Shows where the program spends CPU time.

```bash
# Generate
mise run perf-profile-cpu

# Analyze
mise run perf-analyze-cpu

# Commands in pprof:
(pprof) top10          # Top 10 functions by CPU time
(pprof) top10 -cum     # By cumulative time
(pprof) list <func>    # Source code with timing
(pprof) web            # Visual graph (needs graphviz)
```

### Memory Profile

Shows memory allocations and usage.

```bash
# Generate
mise run perf-profile-mem

# Analyze
mise run perf-analyze-mem

# Commands in pprof:
(pprof) top10 -alloc_space    # Allocated memory
(pprof) top10 -inuse_space    # Currently in-use memory
(pprof) list <func>           # Source code with allocations
```

### Block Profile

Shows where goroutines are blocked.

```bash
mise run perf-profile-block
go tool pprof tests/artifacts/profiles/block.prof
```

### Mutex Profile

Shows lock contention.

```bash
mise run perf-profile-mutex
go tool pprof tests/artifacts/profiles/mutex.prof
```

### Execution Trace

Visual timeline of goroutine execution, GC, syscalls.

```bash
mise run perf-trace
go tool trace tests/artifacts/profiles/trace.out
```

Opens web UI showing:
- Goroutine execution timeline
- GC pauses
- Syscall blocking
- Network/disk I/O

## Performance Targets

| Vault Size | Index Time | Throughput | Status |
|------------|------------|------------|--------|
| 1,000 notes | <2s | >500 notes/sec | ✅ Target |
| 10,000 notes | <20s | >500 notes/sec | ✅ Target |
| 50,000 notes | <100s | >500 notes/sec | ✅ Target |
| 100,000 notes | <200s | >500 notes/sec | ✅ Target |

## Benchmark Output

```bash
$ mise run perf-test-large

BenchmarkLargeVault-8    1   15234567890 ns/op   notes/sec=656.78   ms/total=15234.57
                                                  12345678 B/op      9876 allocs/op
```

Interpretation:
- **15234567890 ns/op**: 15.2 seconds total
- **notes/sec=656.78**: Throughput (custom metric)
- **ms/total=15234.57**: Total milliseconds (custom metric)
- **12345678 B/op**: 12 MB allocated per operation
- **9876 allocs/op**: Number of allocations

## Common Optimizations

### 1. Reduce Allocations

```go
// BEFORE: Allocating in loop
for _, file := range files {
    data := make([]byte, 1024)  // Allocation per iteration!
    process(file, data)
}

// AFTER: Reuse buffer
data := make([]byte, 1024)
for _, file := range files {
    process(file, data)
}
```

### 2. Batch Database Operations

```go
// BEFORE: N+1 queries
for _, note := range notes {
    db.Query("SELECT * FROM metadata WHERE note_id = ?", note.ID)
}

// AFTER: Single batch query
ids := extractIDs(notes)
db.Query("SELECT * FROM metadata WHERE note_id IN (?)", ids)
```

### 3. Parallelize I/O

```go
// BEFORE: Sequential
for _, file := range files {
    processFile(file)
}

// AFTER: Parallel (with concurrency limit)
var wg sync.WaitGroup
sem := make(chan struct{}, runtime.NumCPU())
for _, file := range files {
    wg.Add(1)
    go func(f string) {
        defer wg.Done()
        sem <- struct{}{}
        defer func() { <-sem }()
        processFile(f)
    }(file)
}
wg.Wait()
```

## Artifacts Directory Structure

```
tests/artifacts/
├── profiles/
│   ├── cpu.prof         # CPU profile
│   ├── mem.prof         # Memory profile
│   ├── block.prof       # Block profile
│   ├── mutex.prof       # Mutex profile
│   └── trace.out        # Execution trace
├── reports/
│   ├── cpu-report.txt   # CPU text report
│   ├── cpu-top.txt      # CPU top functions
│   ├── mem-report.txt   # Memory text report
│   ├── mem-alloc.txt    # Allocated space
│   └── mem-inuse.txt    # In-use space
├── baseline/
│   ├── baseline_20250101_120000.txt
│   └── baseline_20250102_150000.txt
└── performance/
    └── performance-results.txt
```

## Interpreting Results

### Good Performance Indicators

- ✅ Throughput >500 notes/sec
- ✅ Linear scaling with vault size
- ✅ <1ms GC pauses
- ✅ No goroutine leaks
- ✅ Minimal lock contention

### Red Flags

- ❌ Throughput <500 notes/sec on modern hardware
- ❌ Quadratic or exponential scaling
- ❌ >10ms GC pauses
- ❌ Growing goroutine count
- ❌ High lock contention (>5% time in mutex)

### Typical Bottlenecks

1. **Disk I/O** (not language-related)
   - Solution: Batch reads, memory-mapped files
2. **Database writes** (SQLite/BoltDB, not language-related)
   - Solution: WAL mode, larger batches
3. **JSON/YAML marshaling** (library-dependent)
   - Solution: Use faster libraries (`sonic-go`, `goccy/go-yaml`)
4. **Regex compilation** (can be expensive)
   - Solution: Pre-compile and reuse regex patterns

## Continuous Monitoring

### Before Each Release

```bash
# 1. Save baseline
mise run perf-baseline

# 2. Run full benchmark suite
mise run perf-test-small
mise run perf-test-medium
mise run perf-test-large

# 3. Generate profiles
mise run perf-profile-all

# 4. Compare with previous release
mise run perf-compare
```

### In CI/CD

Add to GitHub Actions:

```yaml
- name: Performance Regression Test
  run: |
    mise run perf-test-large
    # Fail if throughput <500 notes/sec
```

## Further Reading

- [Performance Analysis Guide](../../docs/performance-analysis.md) - Full Go vs Rust analysis
- [Performance Quick Guide](../../PERFORMANCE_GUIDE.md) - Quick reference
- [Go Profiling Blog](https://go.dev/blog/pprof) - Official Go profiling guide
- [pprof Documentation](https://github.com/google/pprof/blob/main/doc/README.md)

## FAQ

### Q: Should I rewrite in Rust for better performance?

**A: No.** See [performance-analysis.md](../../docs/performance-analysis.md) for detailed analysis. Go is fast enough for this I/O-bound workload.

### Q: How do I test with my real vault?

Copy notes to testdata:
```bash
cp -r ~/path/to/vault testdata/my-vault/
# Then modify benchmark to use testdata/my-vault/
```

### Q: Why is the massive vault test so slow?

Generating 100,000 markdown files with frontmatter takes time. This is intentional for stress testing edge cases.

### Q: Can I run benchmarks in parallel?

No - benchmarks should run serially for accurate timing. Use `-bench=.` to run all, but they execute sequentially.

### Q: What's a "good" allocation count?

Depends on operation complexity. For indexing:
- <100 allocs per note: Excellent
- 100-500 allocs per note: Good
- 500-1000 allocs per note: Acceptable
- >1000 allocs per note: Investigate

### Q: Should I optimize preemptively?

**No.** Profile first, optimize second. "Premature optimization is the root of all evil." - Donald Knuth
