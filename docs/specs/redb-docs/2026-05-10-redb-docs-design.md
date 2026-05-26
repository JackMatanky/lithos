# Design Spec: redb Documentation Reference

## Overview
Create a comprehensive, modular reference for the `redb` crate in `docs/refs/crates/redb/`. This documentation will serve as a standalone resource within the codebase, reducing the need to consult external official documentation.

## Goals
- Provide clear, source-backed information on `redb` architecture and usage.
- Include detailed comparative analysis with other embedded databases.
- Provide full code snippets for advanced patterns.
- Ensure all information is attributed to exact sources.

## Documentation Structure

### 1. `README.md` (Index)
- **Content:** Overview, design principles, navigation links, quick reference table.
- **Source:** https://github.com/cberner/redb

### 2. `concurrency.md`
- **Content:** Transaction model, MVCC, single-writer handling, durability modes.
- **Source:** https://github.com/cberner/redb/blob/master/docs/design.md, https://docs.rs/redb/latest/redb/enum.Durability.html

### 3. `performance.md`
- **Content:** Zero-copy reads, in-place updates, `insert_reserve`, batching, tuning (page/cache size).
- **Source:** https://docs.rs/redb/latest/redb/struct.AccessGuard.html, https://docs.rs/redb/latest/redb/trait.MutInPlaceValue.html

### 4. `errors_safety.md`
- **Content:** Error types, crash safety (1PC+C vs 2PC), power failure assumptions, mmap safety.
- **Source:** https://docs.rs/redb/latest/redb/enum.DatabaseError.html, https://github.com/cberner/redb/blob/master/docs/design.md#commit-strategies

### 5. `maintenance.md`
- **Content:** Backups, persistent savepoints, dynamic shrinking, `Database::compact()`, repair.
- **Source:** https://docs.rs/redb/latest/redb/struct.Database.html#method.compact, https://docs.rs/redb/latest/redb/struct.ReadTransaction.html#method.persistent_savepoint

### 6. `advanced.md`
- **Content:** Custom `Value` implementation, multimap tables, secondary indexes, compound keys.
- **Source:** https://docs.rs/redb/latest/redb/trait.Value.html, https://docs.rs/redb/latest/redb/struct.MultimapTable.html

### 7. `comparisons.md`
- **Content:** Detailed analysis vs Sled, LMDB, SQLite. Pros/Cons for performance, safety, and features.
- **Source:** https://github.com/cberner/redb (README comparisons)

## Implementation Plan
1. Create directory structure.
2. Research each section to gather detailed snippets and specific source links.
3. Draft each markdown file following the approved design.
4. Create the central index (`README.md`).
5. Self-review for completeness, source accuracy, and code quality.

## Success Criteria
- All 7 files created and populated.
- Every major claim has an exact source link.
- Advanced patterns include compilable (or nearly compilable) Rust snippets.
- Navigation between files is seamless.
