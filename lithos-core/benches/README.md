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
**What it measures**: Markdown to Note transformation across complexity levels
- **Simple**: Minimal note (91B, 1 heading, 3 tasks) → ~13.5 µs, 6.8 MiB/s
- **Medium**: Typical note (500B, multiple sections, links) → ~18.3 µs, 27 MiB/s
- **Complex**: Dense note (2.4KB, deep hierarchy, many links) → ~47.9 µs, 50 MiB/s

**When to run**: After changes to:
- Note parser (`src/note/adapter/reader.rs`)
- Markdown processing logic (pulldown-cmark configuration)
- Task/field extraction regex patterns
- Note domain model structure
- Section construction logic

**Key metrics**:
- **Sub-linear scaling**: 5x size → 1.35x time, 27x size → 3.5x time
- **Fixed overhead**: ~10-13 µs (file I/O, Config, Note construction)
- **Throughput improves with size**: 7 MiB/s (simple) → 50 MiB/s (complex)
- **Regression threshold**: >20% latency increase for any size class

**Performance characteristics**:
- Fixed costs dominate for small notes (simple benchmark)
- O(n) parsing cost validates linear scaling assumption
- Throughput reaches 50+ MiB/s for realistic complex notes

---

### `schema_loader.rs` - Schema Ingestion Pipeline
**What it measures**: Complete schema ingestion pipeline with per-stage breakdown
- **Stage 1**: File I/O + Parsing (TOML/JSON → RawSchema, ~200-500 µs)
- **Stage 2**: PropertyBank validation (PropertySpec construction, ~20-30 µs)
- **Stage 3**: PropertyBank lookup (HashMap get performance, ~5-10 ns)
- **Stage 4**: Dereferencing ($ref resolution, ~50-150 µs)
- **Stage 5**: DAG construction (topological sort, ~30-80 µs)
- **Stage 6**: Property merging (inheritance resolution, ~40-100 µs)
- **Stage 7**: Full pipeline end-to-end (~400-900 µs total)
- Scaling behavior across vault sizes (5, 20, 40, 100 schemas)

**When to run**: After changes to:
- Schema ingestion (`src/schema/ingestor.rs`)
- PropertyBank validation (`src/schema/bank.rs`)
- PropertySpec validation logic (`src/schema/property_spec.rs`)
- Internal pipeline stages (`src/schema/dereferencer.rs`, `extender.rs`, `resolver.rs`)
- Config path resolution (`src/config/paths.rs`)
- Property domain model (`src/schema/property.rs`, `property_spec.rs`)

**Key metrics**:
- File I/O should dominate (~50% of total time)
- Dereferencing ~25%, DAG ~10%, Merge ~10%, Validation ~5%
- All stages should scale linearly (O(n)) with schema count
- PropertyBank lookup should be constant time (O(1), ~5-10 ns)
- Throughput should exceed 20K schemas/sec for large vaults

**Bottleneck identification**:
- If file I/O >60% → normal, serde-bound
- If dereferencing >35% → PropertySpec cloning overhead
- If DAG construction >20% → HashMap or algorithm issue
- If property merging >25% → Arc cloning overhead
- If PropertyBank validation >10% → PropertySpec construction regressed

**Implementation notes**:
- Uses `#[doc(hidden)] pub` pattern to access internal pipeline modules
- Comprehensive 270-line module documentation with methodology, interpretation guidance
- Realistic test data from example_vault
- Expected performance characteristics documented per stage
- Follows Criterion.rs and Rust ecosystem best practices

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
cargo bench --bench schema_loader

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

## Benchmark Methodology

### Test Data Model

All benchmarks use controlled test data to ensure reproducibility:

**Standard Test Note Structure** (used in storage benchmarks):
- **Links**: 2 links (1 wikilink, 1 markdown link)
- **Tags**: 3 hierarchical tags (e.g., `#status/active`, `#priority/high`)
- **Headings**: 2 headings (H1, H2)
- **Tasks**: 2 tasks (1 incomplete `- [ ]`, 1 complete `- [x]`)
- **Sections**: 1 section with content
- **Frontmatter**: Minimal HashMap (empty or 1-2 fields)
- **Serialized Size**: ~1.5-2 KB per note

**Simple Parsing Note** (used in note_parsing):
- 1 heading, 3 tasks, 2 list items
- ~100 bytes total

### Benchmark Configuration

**Criterion Settings** (consistent across all benchmarks):
- **Sample Size**: 100 iterations (10 for expensive batch operations)
- **Warmup**: 3 seconds per benchmark
- **Measurement Time**: 5 seconds (30 seconds for batch writes)
- **Confidence Interval**: 95%
- **Throughput Reporting**: Operations per second (or elements/second for batches)

### Statistical Analysis

Criterion automatically provides:
- **Time**: Mean, min, max with confidence intervals
- **Throughput**: Operations per second
- **Regression**: Change from previous run (if baseline exists)
- **Outlier Detection**:
  - **Low Mild**: Faster than expected (system optimization, cache effects)
  - **High Mild**: Slower than expected (GC pauses, OS scheduling)
  - **High Severe**: Significantly slower (I/O contention, memory pressure)

### Hardware Considerations

Baseline numbers assume:
- **CPU**: Apple Silicon (M3 Max in original baselines)
- **Storage**: SSD with good random read performance
- **Memory**: 16GB+ (test datasets fit in memory)

**Important**: Absolute numbers vary by hardware. Focus on relative performance ratios (e.g., zero-copy vs deserialization) rather than raw nanoseconds when comparing across machines.

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
- **Relative performance** (optimized vs baseline)
- **Regression detection** (new code vs old code)
- **Scaling behavior** (batch size, complexity)

### Regression Signals
Watch for:
- Zero-copy read approaching deserialization (ratio < 1.5x)
- Batch transactions approaching individual transaction performance
- UUID-native approaching string conversion performance
- Numeric formatting optimizations losing gains (itoa < 5x faster)

### What Performance Levels Mean

**Nanoseconds (ns)** - Hot path operations:
- Constructor calls, key formatting, UUID conversion
- Single-digit improvements matter in tight loops
- Optimizations compound when called thousands of times per session

**Microseconds (µs)** - Interactive operations:
- Parsing operations, numeric formatting loops
- 10-20% improvements accumulate across session
- Suitable for real-time operations (LSP, autocomplete)

**Milliseconds (ms)** - Database operations:
- Database writes (transaction + fsync overhead dominates)
- Focus on reducing write count, not per-op cost
- Batch when possible to amortize transaction overhead

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
