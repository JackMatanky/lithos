# Lithos Core Benchmarks

This directory contains focused performance benchmarks organized by concern.

## Benchmark Files

### `db_storage.rs` - Storage Layer Performance
**What it measures**: Core redb+rkyv storage infrastructure
- Zero-copy reads vs full deserialization
- Single writes vs batch writes
- Delete operations
- Cache effectiveness (hot vs cold reads)
- Transaction overhead (individual vs batch)

**When to run**: After changes to:
- Database layer (`src/db/`)
- rkyv serialization settings
- Transaction boundaries
- Note domain model (impacts storage size)

**Key metrics**:
- Zero-copy should be faster than deserialization
- Batch transactions should dominate individual transactions
- Cache should show significant hot vs cold differences

---

### `db_key_handling.rs` - Key Formatting Strategies
**What it measures**: Database key construction and UUID handling
- UUID-native methods vs string conversion
- Key formatting performance

**When to run**: After changes to:
- Database key formatting logic
- UUID handling methods
- UUID optimization tracking

**Key metrics**:
- UUID-native should be 7-9% faster than string conversion
- 36 bytes saved per UUID operation

---

### `string_construction.rs` - String & Numeric Formatting
**What it measures**: API-level string construction patterns
- Numeric formatting: `itoa`/`ryu` vs `.to_string()`
- Constructor APIs: `&str` vs `String` parameters
- Aggregate workflow combining all optimizations

**When to run**: After changes to:
- Constructor API signatures
- Numeric formatting code
- String/numeric optimization tracking

**Key metrics**:
- `itoa`: ~9.7x faster than `.to_string()` for integers
- `ryu`: ~17% faster than `.to_string()` for floats
- `&str` constructors: 30-32% faster than forced `String` allocation

---

### `note_parsing.rs` - Markdown Parsing
**What it measures**: Markdown to Note transformation
- Note ingestion from markdown to structured domain objects

**When to run**: After changes to:
- Note parser (`src/note/parser.rs`)
- Markdown processing logic
- Note domain model structure

**Key metrics**:
- Typical: ~3-5 µs for simple notes
- Scales with markdown complexity

---

## Running Benchmarks

```bash
# Run all benchmarks (takes ~10-15 minutes)
cargo bench

# Run specific benchmark file
cargo bench --bench db_storage
cargo bench --bench db_key_handling
cargo bench --bench string_construction
cargo bench --bench note_parsing

# Run specific benchmark group
cargo bench --bench db_storage read_zero_copy
cargo bench --bench db_key_handling uuid_handling

# Quick mode (less precise, faster)
cargo bench --bench db_key_handling -- --quick

# Save baseline for comparison
cargo bench --bench db_storage -- --save-baseline before_changes

# Compare against baseline
cargo bench --bench db_storage -- --baseline before_changes
```

## Benchmark Organization Principles

Each benchmark file follows these principles:

1. **Single Concern**: Measures one aspect of performance (storage, key handling, string construction, parsing)
2. **Clear Purpose**: File name describes what it measures, not implementation details
3. **Minimal Overlap**: Each benchmark is in exactly one file
4. **Cohesive Grouping**: Related benchmarks live together
5. **Self-Contained**: Documentation within benchmarks, not dependent on external planning docs

## Interpreting Results

### Focus on Trends, Not Absolutes
Raw numbers vary by CPU, filesystem, and SSD. What matters:
- Relative performance (optimized vs baseline)
- Regression detection (new code vs old code)
- Scaling behavior (batch size, complexity)

### Regression Signals
Watch for:
- Zero-copy approaching deserialization performance
- Batch transactions approaching individual transaction performance
- UUID-native approaching string conversion performance
- Numeric formatting optimizations losing gains

### Expected Ranges
See `RESULTS.md` for detailed baseline numbers, optimization history, and interpretation guidelines.

## Adding New Benchmarks

When adding a new benchmark:

1. **Choose the right file**: Place in the file that matches its concern
2. **Consider splitting**: If a file grows beyond ~500 lines or spans multiple concerns, split it
3. **Update this README**: Document what the new benchmark measures
4. **Update Cargo.toml**: Add `[[bench]]` entry if creating a new file
5. **Cross-reference**: Link to related docs in `docs/` (ADRs, crate references, etc.)

## Related Documentation

- `RESULTS.md` - Performance baselines, optimization history, regression guidelines
- `docs/refs/crates/rkyv.md` - rkyv serialization guidelines
- `docs/refs/crates/redb.md` - redb database patterns

## Benchmark History

### 2026-02-11: Reorganization
- Split `redb_rkyv.rs` → `db_storage.rs` (storage infrastructure)
- Split `allocation_optimizations.rs` → `db_key_handling.rs` + `string_construction.rs`
- Renamed `note_ingest.rs` → `note_parsing.rs` (clearer naming)
- Each file now has single, well-defined purpose
- Updated all benchmarks to use UUID-native methods where available
