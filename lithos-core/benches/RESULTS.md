# Benchmark Results & Historical Tracking

**Purpose**: Track performance baselines, regressions, and optimization impacts across all benchmark suites.

---

## Current Baselines (2026-02-11)

### Database Storage Layer (`db_storage.rs`)

| Benchmark                | Performance   | Notes                                    |
| ------------------------ | ------------- | ---------------------------------------- |
| **Zero-Copy Read**       | 457 ns        | Sub-microsecond, LSP-ready               |
| **Full Deserialization** | 785 ns        | 1.7x slower than zero-copy (small notes) |
| **Single Write**         | 3.7 ms        | Transaction-dominated, includes fsync    |
| **Batch Write (100)**    | 399 ms        | ~250 notes/sec throughput                |
| **Batch Write (500)**    | 1.95 s        | ~257 notes/sec throughput                |
| **Batch Write (1000)**   | 4.33 s        | ~231 notes/sec, I/O contention visible   |
| **Delete Single**        | ~4.0 ms       | Similar to single write                  |
| **Cache Hot Read**       | ~460 ns       | Minimal cache effect (redb optimized)    |
| **Cache Cold Read**      | ~470 ns       | 2% degradation only                      |
| **Txn Overhead (batch)** | ~399 ms (100) | Comparable to individual txns            |
| **Txn Overhead (indiv)** | ~370 ms (100) | Transaction batching benefit unclear     |

**Hardware**: Apple M3 Max (likely)
**Commit**: 6adf9230, b45c13b0

### Database Key Handling (`db_key_handling.rs`)

| Benchmark                      | Optimized   | Baseline (via-string)  | Improvement         |
| ------------------------------ | ----------- | ---------------------- | ------------------- |
| **UUID get (native)**          | 420 ns      | 452 ns                 | ~7% faster          |
| **UUID put (native)**          | 3.64 ms     | 3.99 ms                | ~9% faster          |
| **UUID delete (native)**       | ~3.6-4.0 ms | ~4.0-4.2 ms            | ~5-10% faster       |
| **Key formatting (optimized)** | 243 ns      | N/A (baseline removed) | Pre-allocation wins |

**Optimization Impact**: 36 bytes saved per UUID operation (string allocation avoided)
**Commit**: 23b33259, b45c13b0

### String Construction (`string_construction.rs`)

#### Numeric Formatting

| Benchmark               | Optimized (itoa/ryu) | Baseline (.to_string()) | Improvement      |
| ----------------------- | -------------------- | ----------------------- | ---------------- |
| **Integer (100 items)** | 135 ns (742 Melem/s) | 1.31 µs (76 Melem/s)    | **~9.7x faster** |
| **Float (100 items)**   | 2.76 µs (36 Melem/s) | 3.23 µs (31 Melem/s)    | **~17% faster**  |

#### Constructor APIs

| Benchmark               | Optimized (&str) | Baseline (String) | Improvement |
| ----------------------- | ---------------- | ----------------- | ----------- |
| **SchemaName::new()**   | 22.22 ns         | 32.63 ns          | ~32% faster |
| **PropertyName::new()** | 25.21 ns         | 36.44 ns          | ~31% faster |
| **DateSpec::try_new()** | 10.77 ns         | 11.11 ns          | ~3% faster  |
| **Template::new()**     | 1.06 µs          | 1.07 µs           | ~1% faster  |

**Aggregate Workflow**: 3.52 ms (combines all optimizations, database-dominated)
**Commit**: 23b33259, b45c13b0

### Note Parsing (`note_parsing.rs`)

| Benchmark           | Performance | Throughput | Notes                     |
| ------------------- | ----------- | ---------- | ------------------------- |
| **Ingest Markdown** | 3.5 µs      | 26.1 MiB/s | Simple 6-line note sample |

**Input**: 1 heading, 3 tasks, 2 list items (~100 bytes)
**Commit**: b45c13b0

---

## Optimization History

### Phase 1: P0 Critical Hot Paths (Complete)

**Date**: 2026-02-11
**Scope**: Database key formatting, UUID-native methods, numeric formatting

#### Task 1: Database Key Formatting (commit 094553e2)

- **Change**: Replaced `format!("{table}:{key}")` with pre-allocated `write!()` buffers
- **Impact**: 100% of database operations (15 allocation sites)
- **Savings**: 36-100 bytes per operation
- **Status**: ✅ Complete

#### Task 2: UUID-Native Methods (commit 5e94edf7)

- **Change**: Added `get_by_uuid`, `put_by_uuid`, `delete_by_uuid` methods
- **Impact**: All ID-based queries (8 allocation sites)
- **Savings**: 36 bytes per UUID operation
- **Performance**: 7-9% faster than string conversion
- **Status**: ✅ Complete

#### Task 3: Command Allocations (commit 62321251)

- **Change**: Documented as architectural constraint (redb transaction model)
- **Impact**: Note/template command operations
- **Status**: ✅ Accepted as necessary

#### Task 4: Template HashMap (commit b2569d43)

- **Change**: Used borrowed `HashMap<&str, &Template>` keys
- **Impact**: Template composition operations
- **Savings**: ~400-1000 bytes per composition (20 templates)
- **Status**: ✅ Complete

#### Task 5: Numeric Formatting (commit be7433d7)

- **Change**: Replaced `.to_string()` with `itoa`/`ryu` buffers
- **Impact**: Query parameter formatting (4 sites)
- **Savings**: 10-19 bytes per query
- **Performance**: itoa 9.7x faster, ryu 17% faster
- **Status**: ✅ Complete

### Phase 2: P1 API Ergonomics (Complete)

**Date**: 2026-02-11

#### Task 6: Constructor API Design (commit eb68d1cf)

- **Change**: `new(name: String)` → `new(name: &str)` for 14 constructors
- **Impact**: All domain entity creation
- **Savings**: 10-11 ns per call (avoided forced allocations)
- **Performance**: 30-32% faster for small types
- **Status**: ✅ Complete

**Total Impact**: 50-80% reduction in hot-path allocations (estimated)

---

## Regression Detection Guidelines

### What Constitutes a Regression

**Critical (investigate immediately)**:

- Zero-copy read approaching deserialization (ratio < 1.5x)
- UUID-native slower than string conversion (ratio < 1.0)
- Numeric formatting advantage disappearing (itoa < 5x faster)
- Single write > 5 ms (transaction overhead growing)

**Significant (investigate soon)**:

- ±20% change in any storage benchmark
- ±10% change in key handling benchmarks
- Batch write scaling becoming super-linear

**Noise (monitor, may be benign)**:

- ±5-10% variance in individual runs
- Cache benchmark fluctuations
- Small constructor API changes

### When to Update This Document

**Must update**:

- After running benchmarks following significant database changes
- After optimization work affecting hot paths
- When baseline hardware changes
- Before/after major refactors

**How to update**:

1. Run benchmark suite: `cargo bench`
2. Record new numbers in appropriate section above
3. Note commit hash and date
4. Document any significant changes in "Optimization History"
5. Update regression thresholds if warranted

---

## Performance Context & Interpretation

### Expected Use Cases

**Zero-copy reads (457 ns)**:

- LSP hover operations
- Autocomplete candidate generation
- Real-time diagnostics
- **Requirement**: < 1 µs for interactive feel

**Batch writes (4.3s for 1000 notes)**:

- Vault indexing on startup
- Bulk import operations
- Background re-indexing
- **Requirement**: < 5s for 1000 complex notes

**Single writes (3.7 ms)**:

- Individual note updates
- Template modifications
- Schema changes
- **Requirement**: < 5 ms for interactive editing

### What These Numbers Mean

**Nanoseconds (ns)**:

- Constructor calls, key formatting, UUID conversion
- Single-digit improvements matter in hot loops
- Optimizations compound when called thousands of times

**Microseconds (µs)**:

- Parsing operations, numeric formatting loops
- 10-20% improvements accumulate across session
- Suitable for interactive operations

**Milliseconds (ms)**:

- Database writes (transaction + fsync overhead)
- Focus on reducing write count, not per-op cost
- Batch when possible

### Known Limitations

**Small note bias**: Test notes are smaller than typical production notes (1-2KB vs 5-10KB). Zero-copy advantage will increase with note complexity.

**Cache saturation**: 100-note test sets fit entirely in redb cache. Production vaults with 10,000+ notes may show different cache behavior.

**Synthetic data**: Benchmark notes have controlled complexity. Production notes vary widely in link count, tag count, and frontmatter size.

**Single-threaded**: Benchmarks don't model concurrent reader/writer scenarios.

---

## Running Benchmarks

### Full Suite

```bash
# Run all benchmarks (takes ~10-15 minutes)
cargo bench

# Save baseline for future comparison
cargo bench -- --save-baseline main-2026-02-11
```

### Individual Suites

```bash
# Storage infrastructure
cargo bench --bench db_storage

# Key handling optimization tracking
cargo bench --bench db_key_handling

# String/numeric formatting
cargo bench --bench string_construction

# Markdown parsing
cargo bench --bench note_parsing
```

### Quick Validation

```bash
# Single benchmark for smoke test
cargo bench --bench db_key_handling -- uuid_handling/get_by_uuid_native --quick

# Specific benchmark group
cargo bench --bench db_storage read_zero_copy
```

### Comparison Against Baseline

```bash
# Compare current code against saved baseline
cargo bench -- --baseline main-2026-02-11

# View HTML reports with plots
open target/criterion/report/index.html
```

---

## Related Documentation

- `README.md` - Benchmark organization and usage guide
- `db_storage.rs` - Storage layer methodology and interpretation
- `db_key_handling.rs` - Optimization validation approach
- `string_construction.rs` - API-level performance tracking
- `note_parsing.rs` - Ingestion performance baseline

---

## Document History

- **2026-02-11**: Initial consolidated results document (commit 7a3f6f8f)
  - Combined data from `docs/benchmarks/BASELINE.md` and `docs/benchmarks/phase6-db.md`
  - Added P0/P1 optimization tracking
  - Established regression detection guidelines
