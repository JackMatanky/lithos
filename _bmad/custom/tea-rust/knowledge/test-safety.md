# TEA Knowledge: Safety Invariants & Determinism

## CONTEXT

- **Applies to**: Core architecture and test stability
- **Purpose**: Ensure predictable, reproducible, and high-performance tests
- **Key Constraints**: Sync-first, Fixed Seeds, RAII

## SYNC-FIRST ARCHITECTURE

Lithos follows a **sync-first architecture**. The core domain and business logic is entirely synchronous with no async dependencies.

- **Zero async in domain**: `lithos-core` has zero async dependencies (no `tokio`, `async-trait`, etc.)
- **Synchronous tests**: All tests in `lithos-core` are standard synchronous Rust tests.
- **Filesystem operations**: Use `std::fs` and `std::io` directly.
- **Database operations**: `redb` and `moka` are synchronous.

## DETERMINISM

Tests must produce the same result regardless of the environment or execution order.

- **Fixed Seeds**: Use deterministic seeds for any randomness or UUID generation in fixtures.
- **Proptest Seeds**: Use `.prop_with_config()` to set deterministic seeds for property tests.
- **Temporary Directories**: Use `tempfile::TempDir` for automatic cleanup via RAII.

## VALIDATION CHECKLIST

- [ ] No `tokio::test` in `lithos-core` (unless specifically justified for infra adapters)
- [ ] Random data uses fixed seeds
- [ ] Filesystem tests use `TempDir`
- [ ] No reliance on system clock for logic (mock time if needed)

## CORRECT EXAMPLES

### Deterministic Proptest

```rust
use proptest::prelude::*;
use proptest::test_runner::Config;

proptest! {
    #![proptest_config(Config {
        // Ensure deterministic failures
        rng_seed: [0; 32],
        .. Config::default()
    })]
    #[test]
    fn my_deterministic_test(s: String) {
        // ...
    }
}
```

## RELATED MODULES

- See `test-unit.md` for unit testing rules
- See `fixtures.md` for `TempDir` usage
- See `anti-patterns.md` for flakiness patterns
