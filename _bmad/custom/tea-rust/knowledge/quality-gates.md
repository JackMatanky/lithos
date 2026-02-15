# TEA Knowledge: Quality Gates

## CONTEXT

- **Applies to**: Project-wide standards and enforcement rules
- **Purpose**: Define acceptable thresholds for correctness, coverage, and performance
- **Enforcement**: Automated in CI/CD pipelines

## QUALITY TARGETS

### Coverage Thresholds

| Category           | Target |
| :----------------- | :----- |
| **Critical Paths** | 100%   |
| **Public APIs**    | 100%   |
| **Error Handling** | 95%    |
| **Business Logic** | 90%    |

### Performance Requirements

| Operation             | P50 Latency | P99 Latency |
| :-------------------- | :---------- | :---------- |
| **Note Creation**     | < 5ms       | < 20ms      |
| **Vault Indexing**    | < 50ms      | < 200ms     |
| **Search (1k items)** | < 1ms       | < 5ms       |

## VALIDATION CHECKLIST

### Coverage Quality

- [ ] Branch coverage is monitored (>= 85%)
- [ ] Condition coverage is monitored (>= 80%)
- [ ] Mutation score is assessed periodically (>= 70%)

### Regression Detection

- [ ] Performance baselines are established
- [ ] Regressions > 5% trigger automated alerts

## CORRECT EXAMPLES

### Performance Regression Detector

```rust
pub struct PerformanceRegressionDetector {
    baselines: HashMap<String, PerformanceBaseline>,
    tolerance: f64,
}

impl PerformanceRegressionDetector {
    pub fn check_regression(&self, current: &PerformanceMetrics) -> Vec<RegressionAlert> {
        let mut alerts = Vec::new();
        for (metric, current_value) in current.metrics() {
            if let Some(baseline) = self.baselines.get(metric) {
                let regression = (current_value - baseline.value) / baseline.value;
                if regression > self.tolerance {
                    alerts.push(RegressionAlert::new(metric, regression));
                }
            }
        }
        alerts
    }
}
```

### Coverage Exclusion Pattern

```rust
// Exclude certain code from coverage if justified
#[cfg(not(test))]
pub mod production_only_code {
    // Code that cannot be meaningfully tested
}
```

## RELATED MODULES

- See `ci.md` for integration into the pipeline
- See `benchmarks.md` for performance measuring
