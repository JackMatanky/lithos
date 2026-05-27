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
| ~~Keep `Present::view()` dead but link to issue~~ | **REVERSED** — remove entirely. Nothing calls it; can be re-added when a consumer exists |
| Remove `DiscoveryResult::is_cold_start()` | Dead code — no callers, no planned usage; can be reintroduced when needed |
| Eliminate `raw.clone()` in `create()` | Move raw after view creation — saves heap allocation |
| Eliminate `delta.clone()` in `update()` | Destructure self before persist — saves PropertyDelta clone |
| Accept `metadata.clone()` in `sync_metadata` | `update_metadata` takes `FileMetadata` by value; changing views API is out of scope for this refactor |

### Decisions from Post-05 Design Review

| Decision | Rationale |
|----------|-----------|
| Remove `Present::view()` | Genuinely dead — the builder reads the view from `bank_discovery.view()` before constructing `Present`, and `FetchReady` consumes it via destructuring. No code path reads it back from the status |
| Keep `Present::new()` | Needed — `view` field is private for encapsulation; `schema::builder` (sibling module) cannot use struct-literal syntax |
| Add `Init` stage marker | `<Comparison, Unknown>` conflated entry with comparison — `Init` makes pipeline entry explicit |
| `from_fs_file` renamed to `from_discovery` on `<Init, Unknown>` | Pairs semantically with `Init`; signals pipeline intent rather than parameter type |
| Keep `Option<RawPropertyBankView>` out of processor | Would push view-existence to runtime checks, defeating the type-state pattern's compile-time safety |

## Issues Encountered

| Issue | Resolution |
|-------|------------|
| `sync_metadata` clones forced by `update_metadata` API | Accept as cosmetic-only cost — `FileMetadata` is a small Copy-like struct |
| `FsFile` clone at builder boundary still needed | `from_fs_file()` takes `FsFile` by value; no way around this without changing constructor API to accept reference + clone internally |
| Builder still calls `bank_discovery.entry().clone()` for `from_fs_file()` | Keep — `entry()` returns `&FsFile` but `from_fs_file()` needs owned `FsFile`. A separate `from(&FsFile)` constructor could be added later |

## Implementation Notes (post-hoc)

### Surprises during implementation

1. **`persist_raw_property_bank` orphaned**: After inlining persist logic into `create()`/`update()` (because `self` was destructured and methods like `self.path_key.clone()` no longer worked on a consumed `self`), the shared `persist_raw_property_bank()` method became unreferenced and was removed. The plan didn't anticipate this — the extra dead code removal was a side benefit.

2. **Metadata accessor removal was automatic**: The plan listed `Missing::metadata()` and `Present::metadata()` as Phase 5 items, but removing `FileMetadata` from the structs in Phase 3 naturally removed the accessors too. No separate Phase 5 step needed.

3. **`sync_metadata` clones unchanged**: As predicted, `sync_metadata` still clones `FileMetadata` because `update_metadata()` takes `FileMetadata` by value. This is the one clone that survived the refactor.

4. **Test-only tests**: The `create_and_update_persist_equivalent_hash_view_for_same_raw_property_bank` integration test still works unchanged — the refactored `create()` and `update()` produce the same hash views.

### What didn't get implemented (and why it's OK)

| Item | Reason |
|------|--------|
| `sync_metadata` clone removal | Blocked by `update_metadata(…)` API shape (views module, out of scope) |
| `FsFile` clone at builder boundary | Blocked by `from_fs_file()` ownership requirement |
| `cargo clippy --all-targets --all-features -- -D warnings` | Plain `cargo clippy -p lithos-core` sufficient (no features, no excluded targets) |

### Planned future work (from post-05 design review)

| Item | Status |
|------|--------|
| Remove `Present::view()` dead accessor | **Implemented** |
| Rename entry stage `Comparison` → `Init` | **Implemented** |
| Rename `from_fs_file(…)` → `from_discovery(…)` on `<Init, Unknown>` | **Implemented** |
| Remove `impl Missing` block (`Missing::new()` → `Missing`) | **Implemented** (follow-up) |
| Embed `Option<RawPropertyBankView>` in processor | **Rejected** — would lose type-state safety |

## Resources
- `lithos-core/src/schema/property_bank_processor.rs` — main target
- `lithos-core/src/schema/builder.rs` — builder orchestration
- `lithos-core/src/schema/discovery.rs` — discovery types
- `lithos-core/src/schema/delta.rs` — `into_changed_name_set()`
- `lithos-core/src/schema/views/raw.rs` — `try_from_raw_with_hashes()`
- `lithos-core/src/schema/views/contracts.rs` — `update_metadata()`
- `docs/engineering/testing/unit.md` — unit test standards
