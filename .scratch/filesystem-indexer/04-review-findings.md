# Review Findings: Issue 04 — Indexer Application Service

Reviewer: adversarial (hostile)
Scope: service.rs, model.rs, write.rs:clear(), read.rs, testing.rs, repository.rs
Commit: `00a0f45f` (11 files, +1692/−108)
Rule: review only, no code changes.

---

## 🔴 CRITICAL: `build_file_record`/`build_dir_record` always generate new `FsRecordId` — identity instability

`service.rs:229-237`, `service.rs:275-278`

**Problem**: Both `build_file_record` and `build_dir_record` call `FsRecordId::new()` for **all** statuses — `New`, `Fresh`, and `Stale`. The `match` arms are identical:

```rust
// build_file_record
let id = match status {
    IndexStatus::New => FsRecordId::new(),
    IndexStatus::Fresh | IndexStatus::Stale => FsRecordId::new(),
};
```

The spec explicitly requires existing ID reuse for Fresh/Stale: *"For Fresh/Stale the existing ID is reused so the record identity is preserved across runs."* The code comment acknowledges this gap: *"the classify loop doesn't know the existing record ID at this point"* — but the assumption that `save_many_records` will fix it is wrong.

**Impact**: On every index run (including no-op runs where nothing changed), every record gets a new `FsRecordId`. The old record remains in the primary table (`FILES`/`DIRS`) because `save_file_in_tx`'s stale-index cleanup only fires when `load_file_delete_context(tx, record.id())` finds a record matching the **new** ID — which it never does. Secondary indexes accumulate stale entries for old IDs. After N runs, the database contains N copies of each file's record, N-1 of which are unreachable (only the latest ID has a path index entry). This is unbounded bloat.

**Consequence**: Parent-child relationships across runs are broken. A dir's `FsRecordId` changes every scan, so children records from a previous run now point to a non-existent parent ID. Any external consumer holding a `FsRecordId` reference finds it stale after the next index run.

**Fix**: Pass the existing record's `id` into `build_file_record`/`build_dir_record` for `Fresh`/`Stale` paths. `classify()` already has `existing` from `repo.find_file_by_path(&key)` — it just needs to pass `existing.as_ref().map(|r| r.id())` to the builder.

---

## 🔴 CRITICAL: `FsParentId::Root` → `to_storage_key()` is non-deterministic — parent-index queries always return empty

`model.rs:279-284`, `read.rs:126-127`, `write.rs:136-137`, `testing.rs:97`

**Problem**: `FsParentId::Root.to_storage_key()` calls `FsRecordId::default()` which calls `FsRecordId::new()` — a **new random UUIDv7 every invocation**. This breaks the write-vs-read contract:

- **Write** (write.rs:136): `parent_table.insert(record.parent_id().to_storage_key(), record.id())` → inserts under random key `X`
- **Read** (read.rs:126): `id_table.get(parent_id.to_storage_key())` → queries random key `Y`
- **Since `X ≠ Y`**, `list_files_by_parent(FsParentId::Root)` and `list_dirs_by_parent(FsParentId::Root)` return **empty** results for every call.

The spec says *"zero sentinel (FsRecordId::MAX)"* but the code doesn't define `MAX`. `FsRecordId` wraps `UuidV7` — no sentinel value exists.

**Impact**: Any consumer using `list_files_by_parent(FsParentId::Root)` silently gets no results. CLI commands, browsing, or content-indexing that depend on root-level parent queries are broken. Test coverage is absent for this path — that's how it went undetected.

**Fix**: Define a deterministic zero sentinel: e.g., `FsRecordId(UuidV7::from_u128(0))` as a `const ZERO: Self`, and use it in `to_storage_key()` for `Root`. Or use an `Option<FsRecordId>` index directly instead of a sentinel hack.

**Secondary issue**: `FsRecordId::default()` returning a random value violates the `Default` contract (callers expect idempotent zero-value). Mark `FsRecordId::default()` as `#[doc(hidden)]` or remove it if only `to_storage_key` uses it.

---

## 🔴 ARCHITECTURE: `IndexNode` typestate is a weak application of the pattern — a false abstraction

`service.rs:41-166`

**Problem**: The code's two-state typestate (`Scanned` → `Classified`) is a weak use of the pattern that adds ceremony without meaningful compile-time safety. Compare to the **typestate builder pattern** from the Apollo reference (`Builder<NameSet, AgeSet>`):

### What's wrong

1. **The typestate lives only inside a single loop iteration.** Each iteration creates an `IndexNode`, calls `classify()`, then immediately consumes it. The guarantee ("PathKey resolved before entry consumed") is enforced within a 30-line loop body that one person wrote at once. This is not a meaningful constraint — no external API boundary is protected. You'd catch a misuse during code review, not at compile time across modules.

2. **The state types ARE the data carriers, not markers.** `Scanned(FsNode)` — the struct variant IS the content. `Classified` holds the actual data. This is documented as intentional ("No PhantomData, no Option"), but it means the "state" and the "data" are the same thing. Contrast with `Builder<MissingName, MissingAge>` where `name: Option<String>` becomes `NameSet` + guaranteed `name: String` only after `.name()` is called. Here there's no incremental building, no partial state, no compile-time prevention of invalid construction — just a before/after transition.

3. **The typestate coexists with `IndexedEntry` — a runtime tagged union.** `IndexNode<Classified>` stores `IndexedEntry`, which is `enum { File(FileIndexEntry), Dir(DirIndexEntry) }`. Every accessor (`entry_id()`, `is_dir()`, `into_entry()`) pattern-matches on this enum at runtime. If the typestate were doing its job, the file-vs-dir discrimination would be encoded in the type, not checked with `match`. The typestate guarantees ordering (`classify()` before consumption) but NOT the more important invariant (file vs dir separation in the accumulation loop). That's still a runtime branch in the service loop.

4. **The `file`-vs-`dir` branch in the service loop is structurally duplicated** — lines 357-375 vs 377-398 are nearly identical. The only differences are:
   - `FsNode::File` vs `FsNode::Dir` constructor
   - `IndexedEntry::File(f)` vs `IndexedEntry::Dir(d)` extraction
   - `dir_ids.insert(pk, id)` only for dirs
   This is 38 lines of duplicated match logic that a proper typestate could eliminate by encoding the type distinction at compile time.

### What a typestate builder would look like here

Instead of `IndexNode<Scanned> → IndexNode<Classified>`, the code could use:

```rust
// Two independent state axes: Type (File|Dir) × Classification (Unclassified|Classified)
struct Unclassified;
struct ClassifiedState;
struct AsFile;
struct AsDir;

struct EntryBuilder<T, C> {
    inner: ...,
    _type: PhantomData<(T, C)>,
}

// Only AsFile + Classified can produce FileIndexEntry
impl EntryBuilder<AsFile, ClassifiedState> {
    fn build(self) -> FileIndexEntry { ... }
}
// Only AsDir + Classified can produce DirIndexEntry
impl EntryBuilder<AsDir, ClassifiedState> {
    fn build(self) -> DirIndexEntry { ... }
}
```

This would eliminate the `IndexedEntry` union enum AND the runtime `match` in the loop — the type system would enforce file-vs-dir separation at compile time. The service loop would be:

```rust
Ok(ScanEntry::File(node)) => {
    let key = node.path().as_key(&self.vault_root)?;
    let builder = EntryBuilder::new(node, &key);
    let classified = builder.classify(parent_id, &self.repo)?;
    let entry = classified.build();  // returns FileIndexEntry, guaranteed
    // no match, no runtime branch
    seen_paths.insert(key);
    counters.increment(entry.status());
    indexed_files.push(entry);
}
Ok(ScanEntry::Dir(node)) => {
    let key = node.path().as_key(&self.vault_root)?;
    let builder = EntryBuilder::new(node, &key);
    let classified = builder.classify(parent_id, &self.repo)?;
    let entry = classified.build();  // returns DirIndexEntry, guaranteed
    seen_paths.insert(key.clone());
    dir_ids.insert(key, entry.id());
    counters.increment(entry.status());
    indexed_dirs.push(entry);
}
```

No `match` on `IndexedEntry`. No `panic!` in `into_entry()`. No runtime is-dir check. The type system encodes the distinction.

### Existing missed opportunity: why `classify()` takes `&R` not `self`

The `classify()` signature already uses `self` (consumes the `Scanned` state) — so it's already linear. A builder would extend this naturally.

**Verdict**: The current `IndexNode` typestate adds ~100 lines of ceremony for a guarantee that any Rust programmer would get right in a 30-line loop. It's a false abstraction — it looks type-safe but the critical runtime distinction (file vs dir) still uses tagged unions. The architectural cost (complexity, code size, reader confusion) outweighs the benefit.

---

## 🔴 ARCHITECTURE: `IndexerService` is generic but monomorphizes a 1400-line machine — binary bloat

`service.rs:306-310`

**Problem**: `IndexerService<S: ScannerPort, R: Repository>` is generic over both the scanner and repository types. Every instantiation with a different `ScannerPort` or `Repository` monomorphizes the entire 1400-line `run()` method, including the fused loop (100 lines), `detect_deletions` (25 lines), `persist` (15 lines), ALL helper functions, and ALL 200+ lines of test infrastructure.

In production there's exactly one scan adapter (`WalkdirAdapter`) and exactly two repo adapters (`RedbRepository` for prod, `InMemoryRepository` for tests). That's two monomorphizations of a 1400-line module. For the binary, the test code is stripped, so the prod binary has one copy. But the compile time penalty is real — every change to `service.rs` forces recompilation of both monomorphized copies.

**Hexagonal architecture approach**: The boundary ports should use `dyn`:

```rust
pub(crate) struct IndexerService {
    vault_root: DirPath,
    scanner: Box<dyn ScannerPort>,
    repo: Box<dyn Repository>,
}

impl IndexerService {
    pub(crate) fn new(
        vault_root: DirPath,
        scanner: Box<dyn ScannerPort>,
        repo: Box<dyn Repository>,
    ) -> Self { ... }
}
```

This eliminates monomorphization overhead at the cost of two vtable dispatches per loop iteration (one for `scanner.walk()`, one for each `repo.find_file_by_path()`). For a disk-bound operation like filesystem scanning, the vtable cost is negligible. The mental model also becomes simpler — `IndexerService` is a concrete type, not a generic with two phantom type parameters.

**Counter-argument**: The current design allows zero-cost inlining of `find_file_by_path` for the `InMemoryRepository` in tests. But the test-repo is a HashMap lookup — inlining it doesn't matter when the test also creates real files on disk.

---

## 🔴 ARCHITECTURE: Data stored THREE times during a single run — memory multiplier

`service.rs:344-352`, `service.rs:405-411`, `service.rs:469-472`

**Problem**: During `run()`, every entry's record data exists in memory in three forms:

1. **`indexed_files: Vec<FileIndexEntry>`** — each `FileIndexEntry` wraps a `FileRecord` (full record data, ~100+ bytes per file). Accumulated during the scan loop.
2. **Cloned records in `persist()`** — `indexed.files().iter().map(|f| f.node().clone())` clones every `FileRecord` into a `Vec<FileRecord>` for `save_many_records`. The clone (line 470) allocates a full copy of each record.
3. **`IndexResult`** — the originals are consumed via `into_boxed_slice()` at line 427, so they survive as the return value.

For a vault with 100k files, that's ~30MB per run just for record data (assuming ~100 bytes/record × 3 copies). For a CLI tool running on a developer's laptop, this is wasteful. For an agent that runs continuously, it adds unnecessary GC pressure.

**Fix**: Build `IndexedNodes` once, share it between `persist` and the return value. `persist` can extract records from `IndexedNodes` without cloning by consuming it (or by taking references to `node()` and having the repo serialize from refs). Alternatively, build `IndexedNodes` lazily and avoid the clone in the non-dry-run path:

```rust
let indexed = IndexedNodes::new(
    indexed_files.into_boxed_slice(),
    indexed_dirs.into_boxed_slice(),
);

if !opts.dry_run() {
    self.persist_from_slices(indexed.files(), indexed.dirs(), &deleted)?;
}

// No second allocation — indexed is used directly
```

---

## 🔴 ARCHITECTURE: `derive_parent_id` couples `IndexerService` to walkdir's ordering — undocumented contract violation

`service.rs:437-454`

**Problem**: `derive_parent_id` panics with `expect("parent directory must be classified before child")` when a parent isn't in `dir_ids`. This assumes the scanner yields entries in breadth-first / parent-before-child order. This is walkdir's behavior, **not** `ScannerPort`'s contract.

The `ScannerPort` trait (`port.rs:42-48`) documents nothing about traversal order. Its doc comment says *"the adapter does not know about vaults, PathKey, or IndexScope"* — but says nothing about "parents before children." A scanner backed by a database (e.g., "re-scan these specific paths from a previous index") might yield entries in insertion order. A network scanner might yield entries as they arrive. Both would cause an unrecoverable panic.

**Fix**: Either:
- Document the ordering requirement on `ScannerPort::walk()` as a contract (with `#[doc = "..."]`), or
- Make `derive_parent_id` fallible (`Result<FsParentId, ...>`) so the service can gracefully handle out-of-order entries (e.g., deferring to a second pass).

---

## 🟡 ARCHITECTURE: `run()` is 100 lines — the fused loop body duplicates file and dir handling

`service.rs:354-401`

**Problem**: The `match entry` in the fused loop has two arms (`File` and `Dir`) that are 19 lines each and structurally identical except for:
- `FsNode::File(node)` vs `FsNode::Dir(node)` constructor
- `IndexedEntry::File(f)` vs `IndexedEntry::Dir(d)` extraction
- `dir_ids.insert(pk, id)` on line 395 (dir-only)

The duplicated code performs the exact same operations: compute key, derive parent, wrap in IndexNode, classify, extract entry, update seen_paths, increment counter, push to collection. This is a textbook extract-method opportunity.

**Fix**: Extract the common classification pattern into a generic handler. The only type-specific operations are the extraction and dir-ID registration, which can be closures:

```rust
fn classify_entry<T, F>(
    node: FsNode,
    vault_root: &DirPath,
    repo: &impl ReadRepository,
    dir_ids: &HashMap<PathKey, FsRecordId>,
    on_classified: F,
) -> Result<..., IndexerError>
where F: FnOnce(PathKey, ...) -> T
```

Or, as argued above, use a proper typestate builder that eliminates the branch entirely.

---

## 🟡 ARCHITECTURE: `persist()` takes `&self` and calls `WriteRepository` methods — should be `&mut self`

`service.rs:464-477`

**Problem**: `persist()` calls `self.repo.save_many_records()` and `self.repo.delete_many_records()`. Both are `WriteRepository` methods that take `&self`. This means the write operations are interior-mutability-based (`RwLock` for `InMemoryRepository`, redb's transaction engine for `RedbRepository`). The service itself gives no compile-time indication that `run()` has side effects — it takes `&self`.

If the repo was `&mut R` in `persist()`, the borrow checker would prevent calling `run()` concurrently or while holding a read reference. Currently, nothing prevents:

```rust
let service = IndexerService::new(vault, scanner, repo);
// No compile error, even though these interfere:
thread::spawn(|| service.run(&scope, opts));
thread::spawn(|| service.detect_deletions(&seen));
```

With `&mut self`, the call to `run()` would statically prevent concurrent access. The trade-off is ergonomic (callers can't share the service), but for a short-lived pipeline service that's used once, `&mut self` is appropriate.

---

## 🟡 RISK: `derive_parent_id` has two `expect` panic paths tied to walkdir ordering

`service.rs:174-197`

**Problem**: Two `#[expect(clippy::expect_used)]` panics:

1. `PathKey::try_new(&s[..pos]).expect("parent of valid path is a valid path")` — asserts that stripping the last segment of a valid `PathKey` yields a valid `PathKey`. This invariant holds for current `PathKey` validation (non-empty, no trailing slash), but is tight coupling to `PathKey`'s internal invariants. A future `PathKey` change (e.g., supporting absolute paths with leading `/`) could break this without a compile-time signal — only a runtime panic.

2. `dir_ids.get(&pk).copied().expect("parent directory must be classified before child")` — assumes `ScannerPort` yields parents before children. This is walkdir's guarantee, not `ScannerPort`'s contract. The trait docs for `ScannerPort` don't mention ordering requirements. An alternative adapter (e.g., database-backed scanner, network scanner) could yield entries in arbitrary order, causing an unrecoverable panic.

**Risk**: Both are programmer-error-level assumptions gated on `clippy::expect_used` suppression. The second one is more concerning because `ScannerPort` has no ordering contract.

**Mitigation**: Document the ordering requirement on `ScannerPort::walk()` and add a `#[must_use]` assertion or test. For path derivation (case 1), add a `parent_key()` method to `PathKey` that returns `Option<&str>` without the `try_new` round-trip.

---

## 🟡 RISK: `persist` clones all entries when `!dry_run` — unnecessary allocation

`service.rs:406-411`

**Problem**:
```rust
if !opts.dry_run() {
    let indexed = IndexedNodes::new(
        indexed_files.clone().into_boxed_slice(),  // full clone
        indexed_dirs.clone().into_boxed_slice(),   // full clone
    );
    self.persist(&indexed, &deleted)?;
}
// Later: consumes originals
Ok(IndexResult::new(
    IndexedNodes::new(
        indexed_files.into_boxed_slice(),  // move
        indexed_dirs.into_boxed_slice(),   // move
    ),
```

For a vault with 100k files, this clones every entry just to build `IndexedNodes` for `persist`, then again for `IndexResult`. If `persist` takes a slice, the `IndexedNodes` construction can be deferred to after the persist check and shared with the return value. Alternatively, extract the record extraction from `persist` into a helper that takes the vectors directly.

---

## 🟡 RISK: `detect_deletions` silently skips type-changed paths (file↔dir) and duplicates

`service.rs:445-454`, `read.rs:208-231`

**Problem**: Two issues:

1. **`all_paths()` can return duplicate paths** (read.rs:218-226): It iterates `FILE_ID_BY_PATH` then `DIR_ID_BY_PATH`. If a path exists in both (file-to-dir conversion or data integrity issue), the path appears twice. The outer loop processes it twice — on the second pass, `find_file_by_path` returns `Some` again (the file-side still exists), adding the same file ID to `DeletedNodes` again. This means `DeletedNodes` can contain duplicate IDs.

2. **Type-changed paths are silently skipped**: If `notes.md` was a file but is now a directory, `all_paths()` has the path from `FILE_ID_BY_PATH`. `seen_paths` has the path from the new scan (which sees it as a dir). Since the path IS in `seen`, it won't be processed as deleted. The old file record remains orphaned in `FILES` and its secondary indexes. If the path is NOT in `seen` (it was deleted entirely), `find_file_by_path` returns the file record, but `find_dir_by_path` also fails because no dir record exists for this path. This works for the current case, but edge cases around type migration are untested.

**Fix**: Deduplicate in `all_paths()` or the processing loop. Accept type orphans as a known limitation but document it explicitly.

---

## 🟡 RISK: `clear()` via `delete_table`+`open_table` — transaction rollback depends on redb internals

`write.rs:284-311`

**Problem**: `clear()` drops all 8 tables then immediately re-opens them within a single write transaction. If any `open_table` call fails (disk full, corruption), the tables are deleted but not recreated. Transaction rollback should revert the deletes, but this relies on redb's log being replayed correctly in all failure modes.

**Mitigation**: No action required — single-transaction guarantees should cover this. Document that `clear()` is unsafe for concurrent readers (they see deleted tables until commit). Consider if this matters in practice (clear happens only during reindex, which is exclusive).

---

## 🔵 NIT: `IndexNode` module-level doc says `Scanned` wraps raw `FsNode` — wrong

`service.rs:15-17`

```
//! `IndexNode<Classified>` wraps a raw `FsNode`. `IndexNode<Classified>`
//! carries the resolved `PathKey` and classified `IndexedEntry`.
```

Line 15 says `<Classified>` wraps `FsNode`. That's wrong — `IndexNode<Scanned>` wraps `FsNode`. `IndexNode<Classified>` wraps `Classified { entry, path_key }`. This is a documentation error that would confuse a first-time reader.

---

## 🔵 NIT: `run()` method signature asymmetry — `&scope` vs `opts` by value

`service.rs:332-336`

```rust
pub(crate) fn run(&self, scope: &IndexScope, opts: IndexOptions) -> ...
```

`scope` is passed as `&IndexScope` (to satisfy `clippy::needless_pass_by_value`) but `opts` is passed by value. If `IndexOptions` is `Copy`, this is fine. If not, it's inconsistent.

---

## 🔵 NIT: Module-level `clippy::arithmetic_side_effects` suppression is too broad

`service.rs:20-23`

```
#![expect(clippy::arithmetic_side_effects, reason = "counter increments...")]
```

`expect` at the crate/module level suppresses the lint for ALL arith in the module (1405 lines), not just the counters. Use per-expression `#[expect(...)]` on the specific increments, or a local group.

---

## 🔵 NIT: Test quality gaps

`service.rs` (test section), `read.rs`, `write.rs`

| Gap | Location | Problem |
|-----|----------|---------|
| `persists_indexed_entries` never queries repo | `service.rs:1227-1240` | Checks `new_count() == 1` but never calls `repo.find_file` to verify persistence |
| `dry_run_skips_persistence` never verifies repo emptiness | `service.rs:1243-1256` | Checks `new_count() == 1` but never verifies repo is unchanged |
| `reindex_clears_repo_before_scan` no repo emptiness check | `service.rs:1106-1126` | Only checks `new_count() == 2` |
| No `list_files_by_parent(Root)` test | read.rs, testing.rs | Would have caught the non-deterministic `to_storage_key` bug |
| No scanner-error stream test | `service.rs` | `Err(e) => return Err(e.into())` path never tested |
| `classified_entry_id` suppresses the ID | `service.rs:934` | `let _id = classified.entry_id()` — no assertion on the ID value or type |
| `is_dir` tests two behaviors in one function | `service.rs:938-958` | Tests both file (is_dir=false) and dir (is_dir=true) in same test |
| `full_integration_mixed_entries` doesn't check parent-child correctness | `service.rs:1308-1335` | Never verifies `sub/c.md` has `FsParentId::Id(sub_id)` |
| No test for duplicate paths in `all_paths` | read.rs | If a path exists in both file and dir tables, behavior is undefined |
| No test for concurrent `run()` | service.rs | No guard against calling `run()` while holding a read ref |

---

## 🔵 NIT: `run()` computes path key twice per entry

`service.rs:358` and `service.rs:94`

The service loop calls `node.path().as_key(&self.vault_root)?` to compute `key` for `derive_parent_id`, then `classify()` calls `file.path().as_key(vault_root)?` again internally. For 100k files, this is 200k redundant path operations. `classify` could accept the pre-computed `PathKey` as a parameter.

---

## 🔵 NIT: `classify` consumes `self` on error — cannot retry

`service.rs:86-132`

```rust
pub(crate) fn classify<R: ReadRepository>(self, ...) -> Result<IndexNode<Classified>, IndexerError>
```

On `Err`, the `IndexNode<Scanned>` is consumed (moved into the `match`). The caller has no access to the original `FsNode` for error logging or graceful degradation. Unlikely to matter in current architecture, but prevents circuit-breaking patterns.

---

## 🔵 NIT: `deserialize_file`/`deserialize_dir` not `#[inline]`

`read.rs:35-43`

These are called once per record on every read. Adding `#[inline]` would let the compiler specialize the `rkyv` deserialization for the specific return type.

---

## Summary

| Severity | Count | Key finding |
|----------|-------|-------------|
| 🔴 bug | 2 | Record ID instability (all entries get new IDs every run); `Root` parent-index sentinel non-deterministic |
| 🔴 architecture | 4 | `IndexNode` typestate is a weak/false abstraction versus proper typestate builder; monomorphization bloat on 1400-line module; triple data storage per run; `derive_parent_id` couples to undocumented walkdir ordering |
| 🟡 risk | 4 | `expect` panic on walkdir coupling; redundant clone in persist; type-change orphans/missing dedup; clear rollback assumptions |
| 🔵 nit | 8 | Test coverage gaps, redundant path computation, broad lint suppression, doc error, inline opportunities |

**Verdict**: The two 🔴 bugs are correctness issues. The four 🔴 architecture findings suggest the design was over-abstracted in one dimension (per-entry linear typestate that adds ceremony without eliminating the critical runtime branch) and under-abstracted in another (duplicated file/dir handling in the loop, triple data allocation, missed builder opportunity). The typestate, as implemented, is an abstraction with a costly surface area (~100 dedicated lines + 2 state types + 1 tagged union) that fails to eliminate the one runtime dispatch that matters (file vs dir).
