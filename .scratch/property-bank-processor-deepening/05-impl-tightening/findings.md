# Findings & Decisions — PropertyBankProcessor Review

## Requirements (from review findings)

1. **Fix builder metadata clone** (`builder.rs:144`): `file.metadata().clone()` should be eliminated
2. **Resolve redundant FileMetadata in status**: Status structs carry `FileMetadata` that duplicates `self.file.metadata()` on processor root — should be removed
3. **Fix dead code expectations**: `Missing::metadata()`, `Present::metadata()`, `Present::view()`, `DiscoveryResult::is_cold_start()` — decide whether to remove or link to issues
4. **Split dual-assertion test**: `persists_view_with_rooted_path_key_when_constructing_new_bank` tests two things
5. **Use shared test fixture**: First test duplicates `make_fixture()` inline
6. **Eliminate production clones**:
   - `property_bank_processor.rs:733`: `self.status.raw.clone()` in `create()`
   - `property_bank_processor.rs:781`: `self.status.delta.clone()` in `update()`
   - `property_bank_processor.rs:650,674`: `self.status.metadata.clone()` in `sync_metadata`

## Research Findings

### Metadata in Status vs. Processor Root

The `FileMetadata` in status structs is **always identical** to `self.file.metadata()` throughout the pipeline because:
- Both originate from the same `PropertyBankDiscovery` entry during builder construction
- The pipeline is synchronous — no thread can update the file metadata mid-transition
- `StaleContent` / `StaleTimestamps` statuses that call `raw.metadata().clone()` get the same metadata injected via `RawPropertyBank::with_metadata()` at parse time

**Conclusion**: `FileMetadata` in status is fully redundant. All comparisons can use `self.file.metadata()`. This eliminates:
- Builder: `file.metadata().clone()`
- `sync_metadata`: `self.status.metadata.clone()` (×2)

### `try_from_raw_with_hashes` Signature
`RawPropertyBankView::try_from_raw_with_hashes(&RawPropertyBank, PathKey, HashRecord)` takes `raw: &RawPropertyBank` by reference.
→ We can borrow `raw` to create the view, THEN move it into `PropertyBank::try_from(raw)`.
→ **Eliminates `raw.clone()` in `create()`.**

### `into_changed_name_set` Signature
`PropertyDelta::into_changed_name_set(self)` takes `self` by value (consumes).
→ Current code clones `delta` because `self.persist()` later borrows `self.status.delta`.
→ **Fix**: destructure `self` before calling persist, extract `delta` by value, reconstruct processor without raw/delta fields for persist.

### `update_metadata` Signature
`Version::update_metadata(&mut self, FileMetadata)` takes `FileMetadata` by value.
→ Clone is forced by API — but becomes unnecessary once status no longer carries `FileMetadata`.
→ Instead pass `self.file.metadata()` directly. However, `update_metadata` takes ownership, which requires cloning still.
→ Alternatively: change `update_metadata` to accept `&FileMetadata` and clone internally. But that's a changing API in the views module, which is out of scope for this refactor.

**Revised conclusion**: The `sync_metadata` clones can only be eliminated if `update_metadata` accepts a reference. Since that's in the views module (separate concern), we have two options:
1. Accept the clone in `sync_metadata` (it's a `FileMetadata` — small struct, trivial cost)
2. Change `update_metadata` signature as a companion change

Option 1 is the pragmatic choice. The real value is eliminating the builder metadata clone and the `raw.clone()`.

## Technical Decisions

| Decision | Rationale |
|----------|-----------|
| Remove FileMetadata from ALL status structs | Eliminates builder clone AND is conceptually cleaner; `FsFile` at root is the single source of truth for file metadata |
| Remove `Missing::metadata()` | Dead code — removing metadata from Missing makes this entirely unused |
| Remove `Present::metadata()` | Accessor was "reserved for future use" — keep decoupled from metadata removal |
| Keep `Present::view()` dead but link to issue | Builder may need view access in future for diagnostics; annotate with `#[expect(dead_code, reason="TODO(#XXX): exposed for caller diagnostics")]` |
| Remove `DiscoveryResult::is_cold_start()` | Dead code — no callers, no planned usage; can be reintroduced when needed |
| Eliminate `raw.clone()` in `create()` | Move raw after view creation — saves heap allocation |
| Eliminate `delta.clone()` in `update()` | Destructure self before persist — saves PropertyDelta clone |
| Accept `metadata.clone()` in `sync_metadata` | `update_metadata` takes `FileMetadata` by value; changing views API is out of scope for this refactor |

## Issues Encountered

| Issue | Resolution |
|-------|------------|
| `sync_metadata` clones forced by `update_metadata` API | Accept as cosmetic-only cost — `FileMetadata` is a small Copy-like struct |
| `FsFile` clone at builder boundary still needed | `from_fs_file()` takes `FsFile` by value; no way around this without changing constructor API to accept reference + clone internally |
| Builder still calls `bank_discovery.entry().clone()` for `from_fs_file()` | Keep — `entry()` returns `&FsFile` but `from_fs_file()` needs owned `FsFile`. A separate `from(&FsFile)` constructor could be added later |

## Resources
- `lithos-core/src/schema/property_bank_processor.rs` — main target
- `lithos-core/src/schema/builder.rs` — builder orchestration
- `lithos-core/src/schema/discovery.rs` — discovery types
- `lithos-core/src/schema/delta.rs` — `into_changed_name_set()`
- `lithos-core/src/schema/views/raw.rs` — `try_from_raw_with_hashes()`
- `lithos-core/src/schema/views/contracts.rs` — `update_metadata()`
- `docs/engineering/testing/unit.md` — unit test standards
