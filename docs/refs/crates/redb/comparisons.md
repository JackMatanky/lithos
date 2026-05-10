# redb Comparison with Alternatives

`redb` is part of a landscape of high-performance embedded databases. This guide compares it technically with its most common alternatives.

---

## 1. Feature & Performance Matrix

| Feature | redb | LMDB | Sled | SQLite |
| :--- | :--- | :--- | :--- | :--- |
| **Pure Rust** | ✅ Yes | ❌ No (C) | ✅ Yes | ❌ No (C) |
| **Concurrency** | 1W, N Readers | 1W, N Readers | NW, NR | 1W, N Readers |
| **Isolation** | Serializable | Serializable | Snapshot | Serializable |
| **Storage** | CoW B-tree | CoW B-tree | Log-Structured | B-tree |
| **Zero-Copy** | ✅ Yes | ✅ Yes | ❌ No | ❌ No |
| **ACID** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |

---

## 2. Technical Deep Dive

### redb vs LMDB
- **Inspiration:** `redb` was designed to bring LMDB-like performance to pure Rust.
- **Safety:** LMDB's C API allows for many memory safety issues (dangling pointers to mmap'd regions). `redb` provides a safe Rust API that prevents these.
- **Read Speed:** LMDB is slightly faster for pure random reads because its internal architecture is simpler and specialized for read-heavy workloads.
- **Write Speed:** `redb` often outperforms LMDB in individual write latency.

### redb vs Sled
- **Architecture:** Sled uses a log-structured architecture (LSM-like), which is excellent for high write throughput but can suffer from "background maintenance noise" (compaction stalls). `redb` uses a predictable CoW B-tree.
- **Stability:** `redb` prioritizes file format stability. Sled is currently undergoing significant internal re-architecture.

### redb vs SQLite
- **Philosophy:** SQLite is a relational database with a full SQL engine. `redb` is a minimal key-value store.
- **Overhead:** SQLite has significant overhead for simple KV lookups due to the SQL parsing and planning layer. `redb` lookups are direct B-tree traversals.

---

## 3. When to Choose redb
- You need **Pure Rust** for easy cross-compilation.
- You require **Zero-Copy** performance for large read-heavy datasets.
- You want **Serializable** isolation without the overhead of a full SQL engine.
- You need a **Stable** file format that won't change between library versions.

---

## 🔗 Sources
- [redb GitHub README - Benchmarks](https://github.com/cberner/redb#benchmarks)
- [redb Design Document](https://github.com/cberner/redb/blob/master/docs/design.md)
- [LMDB Technical Docs](https://www.lmdb.tech/doc/)
- [Sled GitHub Repository](https://github.com/spacejam/sled)
