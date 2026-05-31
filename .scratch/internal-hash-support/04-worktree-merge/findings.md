# Findings — Worktree Merge: split-hash-rs

## GitNexus Analysis

### Index State
- Index is **5 commits behind HEAD** (indexed at `4897d835`, divergence point)
- Still references old `support::hash.rs` — cannot see new `content_hash.rs`/`hash_index.rs`
- `Blake3Hash`: 2 matches (Struct + Impl, both in old `hash.rs`)
- `Blake3HashIndex`: 2 matches (Struct + Impl, both in old `hash.rs`)
- `hash_bytes`: called by 7 sibling test functions (all in old `hash.rs`)

### Consumer Map (from index, pre-split)
Symbols that reference `support::hash`:
- `config/processor.rs` — `ConfigFieldHashes` (re-exports via `support/mod.rs`)
- `config/views.rs` — `hash_raw_global`, `hash_raw_vault`
- `schema/delta.rs` — delta analysis
- `schema/property_bank_processor.rs` — property analysis
- `schema/raw/property.rs` — `compute_hashes`
- `schema/views/contracts.rs` — contracts
- `schema/views/hashes.rs` — `HashRecord`, `RawPropertyHashIndex`, `hash_record_content_matches`
- `schema/views/raw.rs` — `is_content_match`, `supports_zero_copy_staleness_checks`
- `schema/views/snapshots.rs` — `SchemaVersion`, `PropertyBankVersion`

### Divergence
- **Main has 0 commits since divergence**
- **Worktree has 4 commits** (all tested: 1391 unit + 152 doc + 3 integration + 3 e2e all pass)
- **No overlapping edit conflicts** — main is static, all changes are worktree-only

## File Change Analysis

### New files
- `lithos-core/src/support/content_hash.rs` — 432 lines (Blake3Hash + HashInput + hash_bytes + hash_structured + tests)
- `lithos-core/src/support/hash_index.rs` — 559 lines (Blake3HashIndex + diff helpers + tests)

### Deleted files
- `lithos-core/src/support/hash.rs` — 536 lines (split and removed)

### Modified files
- `lithos-core/src/support/mod.rs` — facade: `pub(crate) mod hash` → `pub(crate) mod content_hash; pub(crate) mod hash_index;` + re-exports
- 6 consumer files (import line updates)
- 6 doc comments in `schema/views/hashes.rs`
- `AGENTS.md` — minor update (2 lines changed)

### Unrelated scratch files
- `.scratch/fs-reader-scanner-split/` — 4 files from previous split merge
- `.scratch/internal-hash-support/04-split-hash-rs-into-content-hash-and-hash-index.md` — issue spec

## Risk Assessment

| Factor | Assessment |
|--------|-----------|
| Overlapping edits | None (main unchanged) |
| Consumer changes | 11 files, all import-only updates |
| Test coverage | 1391 unit + 152 doc tests passing |
| Behavioral change | None (documented, tested) |
| Overall risk | **LOW** |
