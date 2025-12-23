# Performance Testing & Profiling Quick Guide

## Quick Commands

```bash
# 1. Test current performance (10k notes)
go test -v -run TestLargeVaultPerformance ./tests/performance

# 2. Run benchmarks with memory stats
go test -bench=BenchmarkLargeVault -benchmem ./tests/performance

# 3. Generate CPU profile
go test -cpuprofile=cpu.prof -bench=BenchmarkLargeVault ./tests/performance
go tool pprof cpu.prof

# 4. Generate memory profile
go test -memprofile=mem.prof -bench=BenchmarkLargeVault ./tests/performance
go tool pprof mem.prof

# 5. Generate execution trace
go test -trace=trace.out -bench=BenchmarkLargeVault ./tests/performance
go tool trace trace.out
```

## Synthetic Vault Sizes

| Test Function | Notes | Use Case |
|---------------|-------|----------|
| `GenerateTestVault` | 250 | Quick tests |
| `GenerateLargeVault` | 10,000 | Realistic large vault |
| `GenerateMassiveVault` | 100,000 | Stress testing |

## Performance Targets

| Vault Size | Index Time | Throughput |
|------------|------------|------------|
| 1,000 | <2s | >500 notes/sec |
| 10,000 | <20s | >500 notes/sec |
| 100,000 | <200s | >500 notes/sec |

## LSP Latency Targets

| Operation | Target | Acceptable |
|-----------|--------|------------|
| Autocomplete | <50ms | <100ms |
| Go to Definition | <100ms | <200ms |
| Hover | <100ms | <200ms |
| Diagnostics | <500ms | <1s |

## When to Optimize

1. **Profile first** - Don't guess
2. **Optimize hot paths** - >10% of total time
3. **Measure improvement** - Compare before/after
4. **Real bottlenecks** - Disk I/O, DB queries, not language

## Decision: Go vs Rust

**Stay with Go** unless profiling proves:
- [ ] Go is the bottleneck (not I/O)
- [ ] 10ms+ latency in critical paths
- [ ] Memory usage >1 GB for small vaults

**None of these will happen.**

See `docs/performance-analysis.md` for full analysis.
