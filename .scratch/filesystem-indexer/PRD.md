# PRD: Filesystem Indexer

**Status**: accepted
**Created**: 2026-06-07
**Triage**: ready-for-agent
**Supersedes**: `.scratch/centralized-discovery-processor/PRD.md` for filesystem node indexing scope

---

## Problem Statement

Lithos needs a single post-config component that indexes filesystem nodes and persists canonical filesystem identity. The existing historical centralized discovery PRD mixed several concerns that now have clearer owners: pre-config Vault Root discovery, Config processing, event-log infrastructure, filesystem indexing, and context-specific parsing.

The current `discovery/` context now explicitly owns only pre-config path discovery. It locates the Vault Root and Discovered Config Path metadata needed before Config can load. It must not become a grab bag for filesystem indexing. Continuing to describe node indexing as "discovery" would blur the newly established `Discovery -> Config -> Indexer -> Context processors` dependency direction.

Today, filesystem scanning and freshness behavior are still spread across Vault, Schema, and other context-specific code. That duplication makes it harder to maintain consistent identity, deletion detection, scan filtering, and path-key behavior. Schema, Note, and Template processors should not each own vault-wide scanning or node identity. They should consume a stable indexer result or query an indexer-owned repository after Config has produced narrowed runtime specs.

## Solution

Introduce a new Filesystem Indexer bounded context that runs after Config is resolved. The Indexer consumes Config-owned, execution-facing specs and owns filesystem node indexing for the configured Vault Root. It scans through FS-owned filesystem ports, compares scanned nodes against persisted index state, classifies node freshness, persists deltas, prunes deleted nodes, and returns a deterministic indexing result for downstream context processors.

The Indexer replaces the filesystem-node portion of the old centralized discovery processor design. It does not replace root/config path discovery. It does not parse Config files. It does not parse Schema, Note, or Template content. It provides canonical filesystem node state and routing-friendly output so downstream contexts can focus on context-owned parsing, validation, hashing, projection, and persistence.

The implementation follows hexagonal architecture:

- Indexer domain/application code defines domain models, service behavior, and ports.
- FS and DB details are adapters behind Indexer-owned ports.
- Context processors depend on Indexer contracts, not concrete scanner or storage implementation details.
- CLI command intent orchestrates the execution flow but does not own indexing rules.

## User Stories

1. As a Lithos maintainer, I want filesystem node indexing to live in a dedicated Indexer context, so that Discovery remains focused on pre-config path discovery.
2. As a Lithos maintainer, I want Config to resolve before indexing, so that scans use validated runtime specs rather than raw or unresolved configuration.
3. As an architecture reviewer, I want the dependency direction to remain `Discovery -> Config -> Indexer -> Context processors`, so that module interaction stays predictable.
4. As a Schema processor maintainer, I want Schema ingestion to consume indexed file metadata, so that Schema no longer owns vault-wide scanning.
5. As a Note processor maintainer, I want Note ingestion to consume indexed markdown file candidates, so that Note only handles note semantics and note persistence.
6. As a Template processor maintainer, I want Template processing to consume indexed template candidates, so that Template can focus on template semantics.
7. As a persistence maintainer, I want one canonical filesystem node identity model, so that file-backed contexts do not invent separate file identity systems.
8. As a query maintainer, I want filesystem node state indexed by path, parent, kind, and file format, so that downstream queries can resolve nodes efficiently.
9. As a cross-platform user, I want persisted paths to use `PathKey`, so that index storage remains portable across operating systems.
10. As a filesystem safety maintainer, I want disk access to go through the FS context, so that Vault Root validation and path rules remain centralized.
11. As a performance-focused engineer, I want freshness classification based on filesystem metadata before expensive context work runs, so that unchanged files can be skipped.
12. As a reliability-focused engineer, I want deletion detection and pruning owned by the Indexer, so that stale node records are removed deterministically.
13. As a test author, I want Indexer ports to be mockable, so that scan, compare, persist, and prune behavior can be tested without real disk or redb dependencies.
14. As a CLI user, I want an explicit indexing command, so that I can refresh filesystem node state without running every downstream context processor.
15. As a CLI user, I want command output that reports scanned, new, fresh, stale, deleted, and failed node counts, so that I can understand what changed.
16. As a CLI user, I want `--reindex` and targeted scope flags (`--path`, `--context`), so that I can choose between incremental freshness checking and a full re-index from scratch.
17. As an operator, I want actionable diagnostics for invalid paths, permission failures, and scan failures, so that I can fix vault issues without reading internal traces.
18. As a future file-watcher maintainer, I want the Indexer scope model to allow event-driven inputs later, so that watcher integration does not require redesigning indexing.
19. As a migration owner, I want the old Vault processor indexing behavior treated as prior art, so that implementation can migrate incrementally without preserving the Vault module as the long-term name.
20. As a domain maintainer, I want context content hashes to remain context-owned, so that the Indexer does not couple filesystem metadata to Schema, Note, or Template semantics.
21. As a storage adapter maintainer, I want redb table and transaction details hidden behind Indexer repository adapters, so that application logic depends on ports rather than storage primitives.
22. As a new contributor, I want names that distinguish indexed domain state from database read views, so that `*View` terminology remains unambiguous.

## Implementation Decisions

### 1. Boundary And Dependency Direction

The canonical pipeline is:

`Discovery -> Config -> Indexer -> Context processors`

- Discovery locates the Vault Root and Discovered Config Path metadata before Config is loaded.
- Config parses, validates, merges, hashes, and resolves configuration, then exposes narrowed execution-facing specs.
- Indexer scans and indexes filesystem nodes using those specs.
- Schema, Note, and Template processors consume indexed node results and perform context-owned parsing, validation, hashing, and persistence.

The Indexer must not import from Discovery for indexing behavior. It may receive a Vault Root value that originated from Discovery through Config/orchestration, but root discovery algorithms remain outside Indexer scope.

The Discovery context must not import Indexer. Discovery remains pre-config and path-only.

### 2. Module Scope

The Indexer context should be introduced as a new top-level core context. The expected internal modules are:

- `model`: filesystem node identity, node structs, classification status, scan scope, and result contracts.
- `repository`: Indexer-owned read/write/unified repository ports.
- `service` or `processor`: application service that coordinates scan, compare, persist, prune, and result construction.
- `scanner`: adapter-facing seam that translates between FS scanning and Indexer domain scan records.
- `storage`: redb repository adapter implementing Indexer repository ports.
- `routing`: optional partitioning helpers or output contracts consumed by context processors.
- `error`: Indexer-specific domain/application errors, with adapters translating infrastructure failures into Indexer errors.

The long-term context name should be `indexer`, not `discovery` and not `vault`. The existing Vault processor and storage code are prior art and possible migration source, but not the target module language.

The application composition root should be introduced separately as a minimal
`app` module in `lithos-core`. It should not revive the deprecated
`application` module, and `lithos-core` should remain a library crate rather
than gaining its own `main.rs`. The process entrypoint remains `lithos-cli`.
The `app` module exists to compose core ports/adapters and expose reusable
execution flows to executable adapters.

The initial `app` module should stay intentionally small. Its module-level
documentation should describe likely growth paths without requiring all of
them immediately:

- `commands`: typed app commands, such as `IndexCommand`.
- `flows` or `services`: execution flows, such as `run_index`.
- `composition`: construction of core adapters from runtime resources.
- `diagnostics`: app-level result summaries, not CLI formatting.

### 3. Hexagonal Architecture Alignment

The Indexer follows a ports-and-adapters structure.

Domain/application-owned contracts:

- Filesystem node models and newtypes.
- Indexing scope and classification types.
- Indexing result contracts.
- Repository ports for persisted node state.
- Scanner port for obtaining filesystem entries.
- Application service that orchestrates the use case.

Adapters:

- walkdir adapter implements the Indexer's `ScannerPort` directly, translating `ScanFilters` into walkdir `filter_entry` predicates.
- redb adapter implements Indexer repository ports using DB infrastructure and Indexer table definitions.
- CLI adapter maps command intent into Config/Indexer execution flow and user-facing diagnostic output.

Third-party or infrastructure details must not leak into Indexer domain contracts. The application service must not expose `walkdir`, `redb`, raw table definitions, raw transactions, or adapter-specific errors in its public contract. Adapters translate infrastructure errors into Indexer-owned errors.

This PRD intentionally keeps the existing project repository pattern: each context defines segregated `ReadRepository`, `WriteRepository`, and `Repository` traits. That aligns with ADR 016 while preserving hexagonal architecture's port ownership rule.

### 4. Domain Naming: Nodes, Not Views

The Indexer should not use `FileView` and `DirView` as the central domain names. In this codebase, `*View` tends to imply a database read projection or persisted read-only representation. The Indexer output will be used throughout the rest of the system after indexing, so the core type names should not imply read-only DB projection.

Preferred names:

- `FileNode`
- `DirNode`
- `FsNode`

Rationale:

- "Node" precisely describes durable filesystem graph/tree state.
- It is short and easy to use in downstream context contracts.
- It avoids conflict with `*View` read-model language.
- It avoids `BaseFile` / `BaseDir`, where "base" is less specific and already overloaded by base schema concepts.

`FileNode` and `DirNode` are domain state, not adapter DTOs. Storage adapters may archive and persist them, but the names should remain meaningful outside DB reads.

### 5. Identity Model: `FsNodeId` Generation And Storage

**Accepted decision**: `FsNodeId` is a newtype over `UuidV7`, following the same pattern as every other context ID in the codebase (`UuidV7` is the project-wide canonical identity primitive in `lithos-core::utils`). The Indexer typestate pipeline generates `FsNodeId` for each new node at the point of first classification (i.e., when a scanned node has no matching persisted record). Generation uses `UuidV7::new()` directly — no `IdPort` abstraction is needed because `UuidV7` is already a stable shared utility, not a hard third-party dependency. Tests construct known IDs via `UuidV7::parse()` or fixed-byte construction. The domain stores `FsNodeId` as the canonical identity for the lifetime of the node.

The `IdPort` / clock-port pattern remains a valid fallback if generation behavior ever needs to be injected (e.g., for deterministic bulk-import scenarios), but is not required for the initial design.

#### 5b. `FsNodeId` vs. `FileId` / `DirId` Trade-off Analysis

The PRD uses a single canonical `FsNodeId` with file/dir-specific node structs:

- `FileNode { id: FsNodeId, ... }`
- `DirNode { id: FsNodeId, ... }`
- `FsNodeKind::{File, Dir}` where a generic node classification is required.

Tradeoffs:

`FsNodeId` benefits:

- One identity space for the filesystem node graph.
- Simpler parent-child relationships, deletion records, and index result contracts.
- Easier future event-driven indexing because file and directory changes share one node identity model.
- Less duplicated repository surface for generic node operations.

`FsNodeId` costs:

- File-only and directory-only APIs need typed wrappers or runtime kind checks.
- A caller can theoretically pass a directory node ID into a file lookup unless the repository method shape prevents it.

Separate `FileId` / `DirId` benefits:

- Stronger compile-time distinction for file-only and dir-only operations.
- Impossible to call a file-specific API with a directory-specific ID.

Separate `FileId` / `DirId` costs:

- More duplicated repository methods and table/index handling.
- Harder generic deletion/pruning output contracts.
- Harder to model the filesystem as one node graph.

Decision for initial design: use `FsNodeId` as canonical identity, plus typed `FileNode` and `DirNode` structs to preserve domain shape. Where compile-time distinction is important, methods should return file/dir-specific structs rather than exposing untyped node payloads.

### 6. Node Model And Path Taxonomy

Indexer path behavior must follow the accepted three-tier path taxonomy:

- Filesystem I/O uses `FsPath`, `FilePath`, and `DirPath`.
- Config/display uses relative config path types.
- Repository and storage boundaries use `PathKey`.

The Indexer converts filesystem paths to `PathKey` only with an explicit Vault Root. Rootless path-key conversion is not allowed.

**Accepted decision**: `FileNode` and `DirNode` fields are canonicalised against the previously locked `FileView` / `DirView` shape from the centralized-discovery PRD, with names updated to match current domain language:

```rust
pub struct FileNode {
    id: FsNodeId,
    parent_id: Option<FsNodeId>,
    path: PathKey,              // vault-relative, forward-slash normalised
    name: FileName,
    format: FileFormat,
    metadata: FileMetadata,
    #[rkyv(with = rkyv::with::AsUnixTime)]
    recorded_at: SystemTime,    // when node was persisted
}

pub struct DirNode {
    id: FsNodeId,
    parent_id: Option<FsNodeId>,
    path: PathKey,              // vault-relative, forward-slash normalised
    name: DirName,
    metadata: DirMetadata,
    #[rkyv(with = rkyv::with::AsUnixTime)]
    recorded_at: SystemTime,    // when node was persisted
}
```

The Indexer must not store context-owned content hashes in `FileNode`. Schema, Note, and Template own content hashing and semantic freshness checks after filesystem freshness has been classified.

### 7. Indexing Result Contract

**Error handling boundary (accepted decision)**: Per-node I/O failures (permission denied, unreadable file, symlink loop) are non-fatal. They are accumulated in `IndexResult` as per-node failure records and do not abort the run. Only two categories cause a hard abort: configuration errors (invalid Vault Root, missing Config specs) and repository initialization failures (unable to open or create redb tables). A partially-indexed result with failures reported is preferred over aborting because one unreadable node in an irrelevant subdirectory should not prevent the rest of the vault from being indexed.

`IndexedFile` and `IndexedDir` were placeholder names and should not be used as persistent domain entities. The PRD uses these clearer output terms instead:

- `IndexResult`
- `FileIndexEntry`
- `DirIndexEntry`
- `IndexStatus`

`FileIndexEntry` represents a file node as classified in the current indexing run. It should include:

- current `FileNode`
- current `FilePath` for immediate context reads
- `IndexStatus`

`DirIndexEntry` represents a directory node as classified in the current indexing run. It should include:

- current `DirNode`
- current `DirPath` if downstream orchestration needs disk access
- `IndexStatus`

`IndexStatus` is the per-run classification:

- `New`: node did not exist in the persisted index.
- `Fresh`: node existed and filesystem metadata matched persisted metadata.
- `Stale`: node existed and filesystem metadata changed, or the caller explicitly bypassed freshness checks.

Deleted nodes should be separate from live entries because no current filesystem path/metadata exists for them. Use deleted-node records rather than fake file/dir entries. A deletion record should carry the `FsNodeId`, previous `PathKey`, and previous kind when available.

**Accepted decision**: `FileIndexEntry` and `IndexResult` follow the locked Discovery Result Contract pattern — embedded node struct, not flattened fields; deleted nodes in a separate collection, not mixed into live entries:

```rust
pub struct FileIndexEntry {
    node: FileNode,       // embedded, not flattened
    path: FilePath,       // live filesystem path for immediate context reads
    status: IndexStatus,
}

pub struct DirIndexEntry {
    node: DirNode,        // embedded, not flattened
    path: DirPath,
    status: IndexStatus,
}
```

`IndexResult` should include:

- file entries for new, fresh, and stale files
- directory entries for new, fresh, and stale directories
- deleted node IDs in a separate `Vec<FsNodeId>` (not mixed with live entries)
- summary counts
- non-fatal per-node failures when the indexing run can continue safely

### 8. Freshness Classification

The initial freshness model is filesystem metadata based.

For files, a node is `Fresh` only when stored metadata matches scanned metadata. The comparison should include size and available timestamps. A node is `Stale` when size or timestamp differs, or when the indexing scope bypasses freshness checks.

For directories, a node is `Fresh` only when stored directory metadata matches scanned directory metadata. Directory size is not portable and must not be used.

Content hashing remains out of Indexer scope. The Indexer is allowed to prevent unnecessary downstream work by classifying filesystem-level freshness, but it must not decide Schema, Note, or Template semantic freshness.

### 9. Scope Model

The Indexer should accept an explicit `IndexScope` separate from Config. Config describes stable runtime boundaries. CLI and orchestration provide run-specific intent.

**Accepted decision**: `IndexScope` is a two-variant enum:

```rust
pub enum IndexScope {
    Full { filters: ScanFilters },
    Partial { root: PathKey, filters: ScanFilters },
}
```

`Full` is the default: scan the entire configured Vault Root. `Partial` narrows the scan to a subtree rooted at `root`, which corresponds to a context boundary path or a targeted path supplied by the CLI. Each context maps to a path, so context-scoped and targeted scans are both expressed as `Partial` with the appropriate root.

`ScanFilters` is an Indexer-owned type carrying narrowing criteria (extension, name patterns, or any other predicate mappable to walkdir's `filter_entry`). Filters are embedded in each variant rather than passed as a separate argument, so the scanner adapter receives one coherent input. The walkdir adapter is responsible for translating `ScanFilters` into concrete walkdir predicates.

Future-compatible scope:

- Event-driven scan from filesystem watcher events (deferred).

The Indexer should always prefer the narrowest correct scope. If Config detects a localized context boundary change, orchestration should request a `Partial` scan rather than `Full`.

### 10. Table Naming And Cross-Context Access

**Accepted decision**: The Indexer defines two redb tables: `FILES` and `DIRS` (not `file_nodes` / `dir_nodes`, not the Vault's `file_views` / `dir_views`). These are the canonical filesystem node tables for the entire codebase.

**Accepted decision**: The Indexer owns `FILES` and `DIRS` exclusively — it is the only context with write access. Other contexts (Schema, Note, Template) resolve path → `FsNodeId` → `FileNode` / `DirNode` by calling the Indexer's `ReadRepository` port at the **application-service level**, not by accessing the raw redb tables directly from their own storage adapters. This is the correct dependency direction (`Indexer → Context processors`) enforced at the storage level, not just the application level.

Rationale: redb is a KV store with no joins. Every cross-context lookup that needs to go from a path or ID to a filesystem node must traverse `FILES` / `DIRS` regardless of architecture. Routing that traversal through an Indexer port costs nothing in query performance and prevents downstream adapters from coupling to the raw table key schema. If the `FILES` / `DIRS` key layout changes, only the Indexer adapter needs updating.

Known tradeoff: downstream context services take a dependency on `IndexerReadRepository`. If this proves too rigid (e.g., a context needs a combined traversal that the port does not expose), the escape hatch is promoting `FILES` / `DIRS` definitions to a shared schema layer and allowing read-only table accessors. This tradeoff is recorded in ADR 022.

### 10b. Repository Ports And Storage Adapter

The Indexer owns repository ports. They should follow the project pattern:

- `ReadRepository`
- `WriteRepository`
- `Repository: ReadRepository + WriteRepository`

Read operations should support:

- lookup by `FsNodeId`
- lookup by `PathKey`
- listing paths by kind
- listing file nodes by format
- listing child nodes by parent
- loading persisted paths for deletion detection and pruning

Write operations should support:

- saving file nodes
- saving directory nodes
- saving batches atomically
- deleting nodes by ID
- pruning batches atomically

**Accepted decision**: The Indexer storage adapter defines the following redb tables, migrated and renamed from the Vault's table inventory. `PATH_BY_FILE_ID` and `PATH_BY_DIR_ID` are dropped (the primary nodes carry enough data for deletion without a reverse index at this stage):

| Table | Key | Value | Purpose |
|---|---|---|---|
| `FILES` | `FsNodeId` | `&[u8]` (rkyv `FileNode`) | Primary file node store |
| `DIRS` | `FsNodeId` | `&[u8]` (rkyv `DirNode`) | Primary directory node store |
| `FILE_ID_BY_PATH` | `PathKey` string | `FsNodeId` | Path → ID resolution |
| `DIR_ID_BY_PATH` | `PathKey` string | `FsNodeId` | Path → ID resolution |
| `FILE_IDS_BY_BASENAME` | `&str` | `FsNodeId` | Wikilink-style lookup |
| `FILE_IDS_BY_PARENT` | `FsNodeId` (parent) | `FsNodeId` (child) | Child listing queries |
| `FILE_IDS_BY_FORMAT` | `&str` | `FsNodeId` | Format-filtered queries |

All tables are updated atomically within the same `redb::WriteTransaction`. Indexes must never diverge from primary data.

redb primitives and table mechanics stay inside the adapter. Public repository ports expose Indexer domain errors, not redb errors.

### 11. Scanner Port And FS Adapter

**Accepted decision**: The Indexer owns a `ScannerPort` trait (its driven port for filesystem access). The walkdir adapter implements `ScannerPort` directly, wrapping walkdir and translating `ScanFilters` into walkdir `filter_entry` predicates. The existing FS-context `DirScanner` is not the seam; since filesystem scanning will only ever be performed by the Indexer, there is no reason to route through an intermediate FS-level abstraction. The walkdir adapter is the sole concrete implementation of `ScannerPort`.

This preserves hexagonal architecture's port ownership rule (the Indexer defines the port, the adapter conforms to it) while removing an unnecessary indirection. Indexer application logic remains testable: tests provide an in-memory `ScannerPort` implementation with deterministic file and directory entries, with no walkdir or real-disk dependency.

### 12. Context Processor Consumption

Schema, Note, and Template processors should consume Indexer output or query Indexer repository ports rather than performing their own vault-wide scans.

Context routing should use Config-resolved boundaries and Indexer result entries. Routing must not live in Discovery. `lithos-core::app` owns routing orchestration because it composes Config specs with Indexer results and downstream context execution order. Indexer should return filesystem truth through `IndexResult`; it should not own Schema, Note, or Template routing semantics.

The initial contract should be conservative: Indexer returns all indexed file and directory entries in scope, plus deleted records. `lithos-core::app` partitions entries for Schema, Note, and Template based on Config specs.

**Accepted decision on run options**: `IndexScope` answers *what* to scan. A separate `IndexOptions { reindex: bool, dry_run: bool }` answers *how* to treat the results. `reindex: true` discards all persisted state before the scan so every node is classified `New`; it is not a freshness bypass — it is a full re-index from scratch. `dry_run: true` classifies nodes without persisting changes. These are orthogonal to scope and should not be embedded in `IndexScope` variants.

**Accepted decision**: `lithos-core::app` routes only for the requested command. A `lithos index` run returns index diagnostics only; it does not automatically trigger Schema, Note, or Template ingestion. Follow-on commands request downstream context processing explicitly. This keeps the Indexer as a standalone, independently executable use case and avoids coupling index runs to full ingestion pipelines.

### 13. CLI Command Intent And Execution Flow

The CLI remains a thin executable adapter. It defines user-facing command syntax and diagnostic output, then delegates typed command intent to `lithos-core::app`. The app composition root wires core adapters and runs the execution flow.

Recommended command surface:

- `lithos index`: resolve Discovery and Config, run the Indexer, print summary diagnostic output.
- `lithos index --reindex`: discard all persisted index state and re-index from scratch, bypassing incremental freshness checks entirely. The flag name is provisional; exact CLI ergonomics can be refined during implementation.
- `lithos index --path <path>`: run a targeted scan for one file or directory subtree.
- `lithos index --context <schema|note|template>`: run indexing for the configured context boundary when that boundary can be derived from Config.
- `lithos index --dry-run`: scan and classify without persisting changes, useful for diagnostics.
- `lithos index --format <human|json>`: choose diagnostic output format.
- `lithos index status`: report persisted index summary without scanning, including node counts by kind/format and last indexed timestamp when available.
- `lithos index explain <path>`: show how a path resolves to a node, classification inputs, and downstream routing candidates, if present.

The exact command shape can be refined during CLI implementation, but these intents should be represented in the PRD so Indexer scope and result contracts are designed with CLI observability in mind.

CLI execution flow for `lithos index`:

1. Parse command intent and flags.
2. Convert CLI input into an app-level `IndexCommand`.
3. Call the app composition root.
4. App runs Discovery to resolve Vault Root and config paths.
5. App runs Config to load, validate, merge, hash, and produce narrowed specs.
6. App maps command intent plus Config specs into `IndexScope`.
7. App constructs Indexer adapters.
8. App runs the Indexer service.
9. App owns routing orchestration for downstream context execution.
10. CLI renders diagnostic output from the app result.

CLI must not own filesystem freshness rules, path-key conversion invariants, repository transaction semantics, or context routing semantics. CLI errors should map Indexer errors into actionable diagnostic output.

### 14. Event Sourcing And Restartability

Generic event-log infrastructure belongs to the Event Sourcing Foundation PRD. The Filesystem Indexer PRD may leave hooks for future restartability but should not define shared event primitives.

If Indexer-specific events are added later, they should reuse foundation contracts and remain owned by the Indexer context. They should model Indexer transitions only, not Schema, Note, Template, or Config transitions.

For this PRD, restartability is optional and deferred unless implementation sequencing requires it.

### 15. Migration Notes

**Accepted decision**: `lithos-core::application` was a shell module with no exports or live imports. It was deleted before Indexer implementation to avoid carrying dead code into the scaffolding phase. Confirmed clean deletion: all 1588 tests pass after removal.

**Accepted decision**: `lithos-core::vault` is kept in place until the Indexer storage adapter passes full integration tests. Once the adapter is proven stable, Vault is deleted in a dedicated follow-on PR. This keeps Indexer issues focused and makes the deletion reviewable in isolation.

The existing Vault processor contains useful prior art:

- scanning directories and files through `DirScanner`
- converting filesystem paths to `PathKey`
- constructing file and directory records
- comparing metadata
- pruning deleted records
- routing markdown candidates
- repository table/index patterns

The Indexer should not preserve Vault names as the final API. It should migrate or replace Vault indexing behavior under the new context boundary.

The old centralized discovery PRD remains historical context. Its root/config discovery, config pipeline, event sourcing, and structured format selector portions have moved to separate artifacts. This PRD owns only the filesystem indexer scope.

## Testing Decisions

Good tests assert externally observable behavior and domain invariants, not private implementation order. Tests should use ports to avoid coupling application-service tests to concrete FS or redb adapters.

Test areas:

1. Domain model construction rejects invalid node/path states and preserves valid `FsNodeId`, `FileNode`, and `DirNode` fields.
2. `FsNodeId` identity behavior is stable, unique, ordered where needed, and compatible with DB key wrappers.
3. Path conversion tests prove filesystem paths convert to `PathKey` only with an explicit Vault Root.
4. Scanner adapter tests prove the walkdir `ScannerPort` implementation translates `ScanFilters` into correct walkdir traversal without leaking walkdir details into domain contracts.
5. Application-service tests classify missing persisted nodes as `New`.
6. Application-service tests classify metadata-matching nodes as `Fresh`.
7. Application-service tests classify changed metadata nodes as `Stale`.
8. Application-service tests classify all nodes as `New` when `IndexOptions { reindex: true }` is set, regardless of stored metadata.
9. Pruning tests remove persisted nodes missing from the current scan and report deleted node records.
10. Repository contract tests prove primary node records and path/parent/format indexes stay consistent after save, batch save, delete, and prune operations.
11. Dry-run tests prove classification can run without persisting changes.
12. Scope tests prove full, context, and targeted scans use the expected scan boundaries.
13. CLI mapping tests prove command flags map to the expected `IndexScope` without duplicating domain rules.
14. CLI diagnostic tests prove human and JSON output report summary counts and actionable errors.

Prior art:

- FS scanner tests for `DirScanner` and `DirScanInput`.
- Vault repository tests for path, parent, basename, and format indexes.
- Schema discovery tests for combining filesystem scan results with cached repository state.
- ADR 016 repository trait split and existing read/write storage adapter tests.
- ADR 019 and ADR 020 path-key and path taxonomy tests.

## Out of Scope

- Pre-config Vault Root and config path discovery.
- Config parsing, validation, merge, hashing, or boundary-change detection.
- Schema content parsing, Property Bank processing, inheritance resolution, or schema semantic validation.
- Note content parsing, frontmatter extraction, task/link/tag modeling, or note semantic validation.
- Template parsing, rendering, input validation, or generated note behavior.
- Context-owned content hashing and semantic freshness checks.
- Generic event sourcing infrastructure.
- Full filesystem watcher implementation.
- Parallel execution strategy for downstream processors.
- Rewriting CLI architecture beyond adding Indexer command intent and orchestration.

## Deferred Actions

**Accepted decision**: The `Indexer` context is not added to `CONTEXT-MAP.md` as a PRD-time artifact. A `(planned)` entry should be added to `CONTEXT-MAP.md` alongside a new `lithos-core/src/indexer/CONTEXT.md` stub as part of the first implementation issue (Indexer scaffolding). This ensures the context map entry is tied to real module structure and the glossary exists when the module is first opened.

## Further Notes

- This PRD should be implemented after root/config discovery and Config pipeline contracts can produce the narrowed specs the Indexer consumes.
- This PRD should respect ADR 016, ADR 018, ADR 019, and ADR 020.
- The Indexer context may require a new `CONTEXT.md` glossary when implementation begins. Candidate terms include Filesystem Node, File Node, Directory Node, Index Scope, Index Status, Indexed Node, and Deleted Node.
- If the choice of `FsNodeId` proves too weak for file-only or directory-only compile-time safety, implementation can add thin `FileNodeId` and `DirNodeId` wrappers over `FsNodeId` without changing the underlying canonical identity space.
