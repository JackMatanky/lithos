# redb Concurrency & Transactions

`redb` uses Multi-Version Concurrency Control (MVCC) and a sequential writer model to provide robust ACID transactions with high performance.

---

## 1. MVCC Implementation
Multi-Version Concurrency Control allows multiple concurrent readers to access the database without being blocked by a writer.

### Epoch-based Reclamation
Reclamation of freed pages is managed through an epoch-based system.
1. **Reader Registration:** When a `ReadTransaction` starts, it is assigned a `TransactionId` (the current database version). It increments a reference count for this ID in the `TransactionTracker`.
2. **CoW Pinning:** Because `redb` uses Copy-on-Write (CoW) B-trees, any page referenced by an active reader's root is "pinned" and cannot be modified or reused.
3. **Reclamation:** When a writer commits, it pushes unreachable pages into a "pending free" queue. These pages are only made available to the allocator once the `oldest_live_read_transaction` has passed the writer's transaction ID.
> **Source:** [redb/src/transaction_tracker.rs](https://github.com/cberner/redb/blob/master/src/transaction_tracker.rs)

---

## 2. Isolation Level: Serializable
`redb` provides **Serializable** isolation, the highest level of isolation defined by the SQL standard.

- **One Writer:** Only one `WriteTransaction` can be active at a time. Attempts to begin a second writer will block or return an error depending on the API used.
- **Snapshot Reads:** Every reader sees a consistent, immutable snapshot of the database at the moment the transaction began.
- **Sequential Writes:** All writes are applied in a strict linear order.
> **Source:** [redb Design - MVCC](https://github.com/cberner/redb/blob/master/docs/design.md#mvcc-multi-version-concurrency-control)

---

## 3. Durability Modes
You can tune the trade-off between write performance and crash safety using the `Durability` enum.

| Mode | Safety Guarantee | Performance |
| :--- | :--- | :--- |
| **`Immediate`** | **ACID.** Calls `fsync` on every commit. No data loss on power failure. | Lowest (Disk I/O bound) |
| **`Eventual`** | Calls `fsync` asynchronously in the background. Small window of data loss on crash. | High |
| **`None`** | Never calls `fsync`. Data loss likely on crash (rolls back to last durable commit). | Highest |

### Recommended Usage
- **Primary Data:** Use `Durability::Immediate`.
- **Caches/Metadata:** Use `Durability::Eventual` or `None`.
> **Source:** [redb::Durability API](https://docs.rs/redb/latest/redb/enum.Durability.html)

---

## 4. Crash Safety Design
`redb` is crash-safe by design, even in `Durability::None` mode.

### Double-Buffered Super-Header
The database file starts with a super-header containing two "commit slots."
1. **1-Phase Commit + Checksum (1PC+C):** By default, `redb` writes new data and checksums, then flips a single "god byte" to point to the new slot.
2. **Automatic Rollback:** If a crash occurs mid-write, the checksum for the new slot will fail. Upon reopening, `redb` detects the corruption and automatically rolls back to the previous consistent slot.
3. **2-Phase Commit (2PC):** Can be enabled for higher security (mitigates theoretical non-cryptographic checksum collision attacks) by adding an extra `fsync` before flipping the primary bit.
> **Source:** [redb Design - Commit Strategies](https://github.com/cberner/redb/blob/master/docs/design.md#commit-strategies)

---

## 5. Deadlock Prevention
Since `redb` only supports a single concurrent writer, it is immune to write-write deadlocks. However, application-level deadlocks can still occur if multiple threads coordinate writes with other external locks.
**Best Practice:** Always acquire the `redb` write transaction *after* other application locks to maintain a consistent lock hierarchy.
