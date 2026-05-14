# Criterion 0.8.2 Upgrade - Research Findings

## Breaking Changes from 0.5 → 0.8.2

### 0.6.0 (2025-05-17)
- **MSRV bumped to 1.80** → Not blocking (ours is 1.92)
- **`criterion::black_box()` → `std::hint::black_box()`** → Not breaking (still re-exported), but recommended migration
- **clap unpinned** → No impact

### 0.7.0 (2025-07-25)
- Just version alignment with criterion-plot, no API changes

### 0.8.0 (2025-11-29) — BREAKING
- **Drop async-std support** → We use `async_tokio`, no impact
- **MSRV to 1.86** → Not blocking
- `Throughput::ElementsAndBytes` added (new feature, not needed)
- alloca-based memory layout randomization (internal)

### 0.8.1 (2025-12-07)
- Fix homepage link, typo

### 0.8.2 (2026-02-04) — target version
- Fix panic with uniform iteration durations
- Fix alloca on unsupported targets

## Inventory

### Workspace Cargo.toml (line 72)
```toml
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
```
→ Change to: `version = "0.8.2"`

### lithos-core/Cargo.toml (line 59)
```toml
criterion = { workspace = true, features = ["html_reports"] }
```
→ No version change needed (workspace inheritance)

### Benchmark files - all import `black_box` from criterion
All 4 benches import `black_box` from `criterion`. In 0.8.2, `criterion::black_box` is still exported as a re-export of `std::hint::black_box`. The 0.6.0 changelog recommends switching to `std::hint::black_box()`.

### Features check
- `html_reports` ✓ Still available in 0.8.x
- `async_tokio` ✓ Still available in 0.8.x (only async-std was dropped)

## Verdict
Clean upgrade - no API breakage for our usage. Only changes needed:
1. Version bump `0.5` → `0.8.2` in workspace Cargo.toml
2. Optional: migrate `criterion::black_box` → `std::hint::black_box` across 4 bench files
