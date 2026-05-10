# redb - Modular Reference Documentation

**Version:** 3.1.x (Stable)
**Goal:** This documentation serves as a standalone, comprehensive reference for the `redb` embedded database within the Lithos project. It aims to replace the need for consulting external official documentation for most development and operational tasks.

---

## 🧭 Navigation
- [**Core Concepts & API**](api_reference.md) - Deep dive into `Value`, `Key`, `Table`, and `AccessGuard`.
- [**Concurrency & Transactions**](concurrency.md) - MVCC internals, locking mechanics, and durability levels.
- [**Performance & Optimization**](performance.md) - Tuning guide for cache, page sizes, and zero-allocation patterns.
- [**Error Handling & Safety**](errors_safety.md) - Comprehensive error catalog and crash safety analysis.
- [**Maintenance & Operations**](maintenance.md) - Ops manual: backup, compaction, repair, and migrations.
- [**Advanced Patterns**](advanced.md) - Secondary indexes, compound keys, and `rkyv` integration.
- [**Comparison with Alternatives**](comparisons.md) - Detailed analysis vs LMDB, Sled, and SQLite.
- [**Internal Design**](DESIGN.md) - Technical specification of the file format and B-tree layout.

---

## 🚀 Quick Start (Lithos Pattern)

### 1. Define your Table
```rust
use redb::TableDefinition;

// Pattern: Namespace tables to avoid collisions
pub const NOTES_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("lithos:note:v1");
```

### 2. Basic CRUD
```rust
use redb::{Database, ReadableTable};

fn example(db: Database) -> Result<(), redb::Error> {
    // WRITE: Atomic transaction
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(NOTES_TABLE)?;
        table.insert("note_1", b"content")?;
    }
    write_txn.commit()?;

    // READ: Zero-copy access
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(NOTES_TABLE)?;
    if let Some(guard) = table.get("note_1")? {
        let value: &[u8] = guard.value(); // Zero-copy view!
        println!("Found: {:?}", value);
    }
    Ok(())
}
```

---

## 🏗️ Core Design Principles
1. **Pure Rust:** No C dependencies, making cross-compilation seamless.
2. **Zero-Copy:** Data is memory-mapped and returned as views (`AccessGuard`), eliminating deserialization overhead for read-heavy workloads.
3. **ACID Compliance:** Fully ACID-compliant transactions with **Serializable** isolation.
4. **Crash Safe:** Design based on double-buffered headers and checksummed B-trees ensures data integrity even during power failure.
5. **MVCC:** Multi-Version Concurrency Control allows multiple concurrent readers to proceed without being blocked by a writer.

---

## 📚 Key Technical Metrics (at a glance)
- **Concurrency:** 1 writer, N readers (non-blocking).
- **Isolation:** Serializable.
- **Storage:** Copy-on-Write (CoW) B-Trees.
- **Repair:** Automatic rollback to last consistent state on crash.
- **File Growth:** Region-based allocation with dynamic shrinking support.

---

## 🔗 Primary Sources
- [redb Design Spec](https://github.com/cberner/redb/blob/master/docs/design.md)
- [redb GitHub Repository](https://github.com/cberner/redb)
- [redb API Reference (docs.rs)](https://docs.rs/redb/latest/redb/)

---
*Last Updated: 2026-05-10*
