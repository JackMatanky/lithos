# redb Error Handling & Safety

`redb` is designed for high reliability in systems with potentially unstable power or hardware. This guide catalog's common errors and analyzes the safety guarantees provided by the engine.

---

## 1. Error Hierarchy
`redb` uses specialized error enums to provide clear feedback on what failed.

- **`DatabaseError`:** Failures opening or creating a database.
  - `DatabaseAlreadyOpen`: Another process is using the file.
  - `UpgradeRequired`: File format version is too old.
- **`TableError`:** Failures during table operations.
  - `TableDoesNotExist`: Attempted to open a missing table.
  - `TableAlreadyExists`: Conflict during creation.
- **`StorageError`:** Critical low-level failures.
  - `Corrupted`: Checksum mismatch or invalid metadata detected.
  - `Io(std::io::Error)`: OS-level I/O failure.
- **`TransactionError`:** Issues during commit or rollback.
> **Source:** [redb Error APIs](https://docs.rs/redb/latest/redb/index.html#enums)

---

## 2. Crash Safety Analysis
`redb` maintains consistency even during sudden power loss or process crashes.

### Design Assumptions
- **Single-Byte Atomicity:** Assumes that writing a single byte is atomic (the "god byte" flip).
- **Powersafe Overwrite:** Assumes that writing to a range of bytes does not corrupt data outside that range.
> **Source:** [redb Design - Assumptions](https://github.com/cberner/redb/blob/master/docs/design.md#assumptions-about-underlying-media)

### Recovery Mechanism
- **1-Phase Commit + Checksum (1PC+C):** If a crash occurs during a commit, the new transaction slot will have an invalid checksum. `redb` will automatically detect this on next open and ignore the partial write, effectively rolling back to the last good state.

---

## 3. Memory Safety & Mmap
Unlike C-based databases (like LMDB), `redb` is written in pure Rust, preventing many common classes of memory safety bugs.

- **Mmap Safety:** `redb` manages its memory maps carefully to avoid use-after-free or data races.
- **Alignment:** **CRITICAL:** `redb` does **not** guarantee the alignment of stored bytes. If your `Value` implementation expects alignment (e.g., casting to a `u64`), you must either:
  1. Use unaligned loads (e.g., `u64::from_le_bytes`).
  2. Copy the data to an aligned buffer before use.
> **Source:** [redb GitHub - Benchmarks/Alignment](https://github.com/cberner/redb#benchmarks)

---

## 4. Platform-Specific Considerations
- **Windows:** `redb` uses standard file locking and `mmap`.
- **WASI:** Support is present but limited by the capabilities of the WASI runtime environment.
- **Embedded:** Minimal dependencies make it suitable for a wide range of platforms.
