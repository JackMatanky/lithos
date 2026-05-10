# Findings - redb Documentation

## Core Research
- **1-Phase Commit (Default):** redb uses 1PC+C (1-phase + checksum) by default. Updates inactive slot, flips primary bit, calls `fsync`. (Source: https://docs.rs/redb/latest/src/redb/transactions.rs.html)
- **Dynamic Shrinking:** `redb` can shrink its file size dynamically after data removal and multiple commits. (Source: https://docs.rs/redb/latest/src/redb/db.rs.html)
- **MVCC Isolation:** Single isolation level: serializable. All writes applied sequentially. (Source: docs/refs/crates/redb/design.md)
- **Non-Durable Commit:** Updates in-memory flags, no `fsync`. Consistent state on crash but may roll back to last durable commit. (Source: https://docs.rs/redb/latest/src/redb/tree_store/page_store/page_manager.rs.html)

## Error Types
- `DatabaseError`: General database-level errors (opening, creating).
- `TableError`: Table-specific errors (doesn't exist, already exists).
- `StorageError`: Low-level I/O errors.
- `TransactionError`: Commit/Rollback failures.

## Performance Tips
- Page size should align with OS page size (usually 4KB).
- Use `insert_reserve` for zero-allocation writes.
- Multimap tables for secondary indexes.
- Batch operations to amortize `fsync` costs.
