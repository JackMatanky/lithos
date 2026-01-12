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
mise run test:benchmark

# Run a specific benchmark group
mise run test:benchmark event_bus
```

## Performance Gates (NFR2)

- **Regression Detection**: Criterion compares current runs against stored baselines in `target/criterion/`.
- **Thresholds**:
    - > 5% regression: Warning/Alert
    - > 10% regression: Block release
