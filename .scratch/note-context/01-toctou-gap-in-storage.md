# TOCTOU Gap in Note Storage

## Problem
A Time-Of-Check to Time-Of-Use (TOCTOU) gap exists in `crates/note/src/storage/write.rs:63-91`. Specifically, the code currently performs checks and persistence operations across multiple isolated transactions instead of executing them atomically. If another process or thread modifies the same records between the time the checks are made and the writes are committed, it can result in an inconsistent state or data races.

## Proposed Solution
With the adoption of ADR 025, `traces-db` now exposes a closure-based transaction capability: `Store::write<R, F, E>`.

We should fix this gap by wrapping the validation and persistence logic inside a single `store.write(|tx| { ... })` closure.

Steps for the fix:
1. Update `crates/note/src/storage/write.rs` to group the existence check and the write operations.
2. Use the `Store::write` capability provided by ADR 025 to wrap the entire operation, passing the transaction context downward.
3. Ensure that any domain-level constraint checks are performed inside this transaction so they are atomic.

> Note: this issue was identified during the adversarial review of the `indexer` crate (`06.8-adversarial-review.md`, defect A14).
