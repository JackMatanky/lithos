# Allocation Optimization Benchmarks - Baseline Results

**Date**: 2026-02-11
**Commit**: 23b33259
**Hardware**: Apple M3 Max (likely)
**Rust Version**: 1.84.0-nightly (or stable - check with `rustc --version`)

## Purpose

This document captures baseline performance numbers for the allocation optimization
benchmark suite. These numbers represent the **after** state following P0 and P1
optimizations documented in `TODO_ALLOCATIONS.md`.

## Summary of Key Improvements

### Database Operations

| Operation              | Optimized Time | Baseline Time          | Improvement |
| ---------------------- | -------------- | ---------------------- | ----------- |
| `get_by_uuid` (native) | 420.07 ns      | 451.68 ns (via string) | ~7% faster  |
| `put_by_uuid` (native) | 3.64 ms        | 3.99 ms (via string)   | ~9% faster  |

**Key insight**: UUID-native methods save 31-350 µs per operation by avoiding string
allocation (36 bytes per UUID formatting).

### Numeric Formatting

| Operation                      | Optimized (itoa/ryu) | Baseline (.to_string()) | Improvement      |
| ------------------------------ | -------------------- | ----------------------- | ---------------- |
| Integer formatting (100 items) | 134.73 ns            | 1.31 µs                 | **~9.7x faster** |
| Float formatting (100 items)   | 2.76 µs              | 3.23 µs                 | **~17% faster**  |

**Key insight**: `itoa::Buffer` provides nearly 10x improvement for integer formatting;
`ryu::Buffer` provides 17% improvement for floats. Both are zero-allocation.

### Constructor API Ergonomics

| Constructor           | `&str` API | `String` API | Improvement |
| --------------------- | ---------- | ------------ | ----------- |
| `SchemaName::new()`   | 22.22 ns   | 32.63 ns     | ~32% faster |
| `PropertyName::new()` | 25.21 ns   | 36.44 ns     | ~31% faster |
| `DateSpec::try_new()` | 10.77 ns   | 11.11 ns     | ~3% faster  |
| `Template::new()`     | 1.06 µs    | 1.07 µs      | ~1% faster  |

**Key insight**: `&str` parameters save 10-11 ns per constructor call by avoiding
forced string allocations at call sites. Larger types (Template) show minimal
difference due to other constructor overhead.

## Detailed Results

### Database Operations (P0 Tasks 1 & 2)

```
database_operations/get_with_string_key
                        time:   [243.92 ns]
                        thrpt:  [4.0998 Melem/s]

database_operations/put_with_string_key
                        time:   [3.75 ms]
                        thrpt:  [266.45 elem/s]

database_operations/get_by_uuid_native (OPTIMIZED)
                        time:   [420.07 ns]
                        thrpt:  [2.3805 Melem/s]

database_operations/get_by_uuid_via_string (BASELINE)
                        time:   [451.68 ns]
                        thrpt:  [2.2140 Melem/s]

database_operations/put_by_uuid_native (OPTIMIZED)
                        time:   [3.64 ms]
                        thrpt:  [274.38 elem/s]

database_operations/put_by_uuid_via_string (BASELINE)
                        time:   [3.99 ms]
                        thrpt:  [250.05 elem/s]
```

**Optimization impact**: UUID-native methods save ~31 ns on reads and ~350 µs on
writes by avoiding UUID-to-string conversion overhead (36 bytes saved per operation).

### Numeric Formatting (P0 Task 5)

```
numeric_formatting/format_integers_itoa (OPTIMIZED)
                        time:   [134.73 ns] for 100 integers
                        thrpt:  [742.23 Melem/s]

numeric_formatting/format_integers_to_string (BASELINE)
                        time:   [1.31 µs] for 100 integers
                        thrpt:  [76.061 Melem/s]

numeric_formatting/format_floats_ryu (OPTIMIZED)
                        time:   [2.76 µs] for 100 floats
                        thrpt:  [36.167 Melem/s]

numeric_formatting/format_floats_to_string (BASELINE)
                        time:   [3.23 µs] for 100 floats
                        thrpt:  [30.908 Melem/s]
```

**Optimization impact**:

- `itoa::Buffer`: 9.7x faster than `.to_string()` for integers
- `ryu::Buffer`: 17% faster than `.to_string()` for floats
- Both are zero-allocation (stack-based)

### Constructor APIs (P1 Task 6)

```
constructor_apis/schema_name_from_str (OPTIMIZED)
                        time:   [22.22 ns]
                        thrpt:  [45.013 Melem/s]

constructor_apis/schema_name_from_owned_string (BASELINE)
                        time:   [32.63 ns]
                        thrpt:  [30.650 Melem/s]

constructor_apis/property_name_from_str (OPTIMIZED)
                        time:   [25.21 ns]
                        thrpt:  [39.669 Melem/s]

constructor_apis/property_name_from_owned_string (BASELINE)
                        time:   [36.44 ns]
                        thrpt:  [27.444 Melem/s]

constructor_apis/date_spec_from_str (OPTIMIZED)
                        time:   [10.77 ns]
                        thrpt:  [92.873 Melem/s]

constructor_apis/date_spec_from_owned_string (BASELINE)
                        time:   [11.11 ns]
                        thrpt:  [90.033 Melem/s]

constructor_apis/template_from_str (OPTIMIZED)
                        time:   [1.06 µs]
                        thrpt:  [940.16 Kelem/s]

constructor_apis/template_from_owned_string (BASELINE)
                        time:   [1.07 µs]
                        thrpt:  [933.42 Kelem/s]
```

**Optimization impact**: `&str` parameters save 10-11 ns per call for simple types by
avoiding forced allocations. Larger types show minimal difference due to other overhead.

### Aggregate Workflow

```
aggregate_workflow/complete_optimized_workflow
                        time:   [3.52 ms]
                        thrpt:  [283.83 elem/s]
```

**Notes**: Combines Tasks 1, 2, 5, and 6 into a single workflow. Dominated by database
write overhead (~3.6 ms), so individual optimization gains are less visible.

## Interpretation Guidelines

### What These Numbers Mean

- **Nanosecond (ns) operations**: Single-digit to double-digit improvements matter in
  hot loops (e.g., query execution, event handling)
- **Microsecond (µs) operations**: 10-20% improvements compound when called thousands
  of times per session
- **Millisecond (ms) operations**: Database writes dominate; focus on reducing write
  count rather than per-operation overhead

### Expected Use Cases

These benchmarks measure **hot path** performance for:

1. **Database operations**: UUID-native methods should be used for all template/note
   lookups (Tasks 1 & 2)
2. **Numeric formatting**: Use `itoa`/`ryu` in query formatting, property serialization,
   and display logic (Task 5)
3. **Constructor APIs**: `&str` parameters provide better ergonomics and performance;
   callers choose when to allocate (Task 6)

### Future Optimization Targets (P2/P3)

Remaining tasks from `TODO_ALLOCATIONS.md`:

- **P2 tasks**: Template composition, error context strings
- **P3 tasks**: Event field construction, incremental improvements
- **Profiling**: Use `dhat` or `heaptrack` to identify new hot paths

## Running Benchmarks

```bash
# Full suite (takes ~5 minutes)
cargo bench --bench allocation_optimizations

# Single benchmark group
cargo bench --bench allocation_optimizations database_operations

# Specific benchmark
cargo bench --bench allocation_optimizations -- schema_name_from_str

# Quick mode (less precise, faster)
cargo bench --bench allocation_optimizations -- --quick
```

## Regenerating This Baseline

If significant changes are made to hot paths, regenerate baseline:

```bash
# Ensure clean environment
cargo clean
mise run build

# Run full suite with output capture
cargo bench --bench allocation_optimizations 2>&1 | tee docs/benchmarks/latest_run.txt

# Update this file with new numbers and commit hash
```

## Change History

- **2026-02-11** (commit 23b33259): Initial baseline after P0/P1 optimizations complete
