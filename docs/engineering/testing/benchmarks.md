---
title: "Benchmarks"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
scope: "Performance benchmark execution and regression posture"
---

# Benchmarks

## Primary command

```bash
mise run test:bench
```

## Benchmark policy

- Benchmark NFR-critical paths (indexing, rendering, heavy query flows).
- Keep scenarios representative of real workloads.
- Track regressions over time; avoid one-off benchmark interpretation.

## Practical guidance

- Run benchmarks after performance-sensitive changes.
- Compare against known baseline output where available.
- Investigate regressions before merge when critical paths degrade materially.

## CI posture

- Benchmarks may run conditionally (for example PR-focused workflows).
- Preserve benchmark artifacts where configured for trend analysis.
