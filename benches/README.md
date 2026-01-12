# Lithos Benchmarks

This directory contains performance benchmarks for the Lithos project.

## Structure

Benchmarks are organized by crate and category:

- `crates/app/benches/`: Application-level benchmarks (core operations, event bus).
- `crates/domain/benches/`: Domain-level micro-benchmarks (logic, conversions).
- `crates/adapters/benches/`: Adapter-level integration benchmarks (storage, I/O).

## Running Benchmarks

Benchmarks are orchestrated via `mise`:

```bash
# Run all benchmarks
mise run test:bench

# Run a specific package (e.g., app)
mise run tbap

# Run a specific benchmark group with Criterion flags
mise run test:bench core_ops --quick --noplot
```

## Performance Gates

- **Regression Detection**: Criterion compares current runs against stored baselines in `target/criterion/`.
- **Thresholds**: Defined in `lithos-test-utils::performance_gates`.
    - > 5% regression: Warning/Alert (`WARNING_THRESHOLD`)
    - > 10% regression: Block release (`BLOCKING_THRESHOLD`)

## Memory Profiling

Memory profiling is integrated via `dhat`. To run benchmarks with heap profiling:

```bash
# Run with dhat enabled (requires --features dhat-on)
cargo bench -p lithos-app --bench core_ops --features dhat-on
```

After running, `dhat-heap.json` will be generated in the root directory. You can view it using the [DHAT Viewer](https://valgrind.org/docs/manual/dh-manual.html).
