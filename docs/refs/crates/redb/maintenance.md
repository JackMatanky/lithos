# redb Maintenance & Operations

This guide provides procedures for managing `redb` database health, backups, and migrations.

---

## 1. Space Reclamation (`Database::compact`)
`redb` files can grow over time due to Copy-on-Write fragmentation.

### How it works
`compact()` moves pages from the end of the file into holes in earlier regions, then truncates the file.
- **Blocking:** It requires exclusive access (`&mut Database`) and **blocks all concurrent readers and writers.**
- **Automatic Truncation:** Trailing empty regions are automatically removed.
```rust
// Reclaim space when no transactions are active
let compacted: bool = db.compact()?;
```
> **Source:** [redb::Database::compact API](https://docs.rs/redb/latest/redb/struct.Database.html#method.compact)

---

## 2. Backup Strategies
Because `redb` uses a single-byte atomic flip for commits, simple file copies are often consistent, but structured approaches are safer.

### Consistent Backups via Savepoints
1. **Create Persistent Savepoint:** `begin_write().persistent_savepoint()`.
2. **File-Level Copy:** Use standard filesystem tools (`cp`, `rsync`) to copy the `.redb` file.
3. **Delete Savepoint:** Once copied, delete the savepoint to allow page reclamation.
> **Source:** [redb Design - Savepoints](https://github.com/cberner/redb/blob/master/docs/design.md#savepoints)

---

## 3. Database Repair
`redb` automatically handles repair upon opening.

- **Quick Repair:** (Default) Loads the allocator state from a system table. Fast and efficient.
- **Full Repair:** Rebuilds the allocator state by walking every B-tree in the database. Triggered if the allocator state is missing or the primary checksum fails.
> **Source:** [redb Design - Database Repair](https://github.com/cberner/redb/blob/master/docs/design.md#database-repair)

---

## 4. Migrations & Versioning
Major upgrades in `redb` (e.g., 1.x to 2.x) frequently include breaking changes to the internal B-tree format.

### Migration Path
There is no in-place upgrade for major format changes. The standard procedure is:
1. Open the database using the old library version.
2. Export all data to a neutral format (e.g., JSON, FlatBuffers).
3. Create a new database using the latest library version.
4. Import the data into the new database.
> **Source:** [redb 4.0.0 Release Notes](https://github.com/cberner/redb/releases/tag/v4.0.0)

---

## 5. Dynamic Shrinking Constraints
The database will only shrink its file size during `commit()` if:
1. **No Savepoints Exist:** Savepoints "pin" old pages, preventing them from being freed.
2. **Trailing Regions are Empty:** If a single page is allocated at the very end of the file, the file cannot be truncated. Use `compact()` to move these pages.
> **Source:** [redb Design - Database Header](https://github.com/cberner/redb/blob/master/docs/design.md#database-header-64-bytes)
