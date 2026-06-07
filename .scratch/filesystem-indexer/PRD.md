# PRD: Filesystem Indexer

**Status**: draft
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
16. As a CLI user, I want `--force` and targeted scope flags, so that I can choose between normal freshness checking and explicit full scans.
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

- FS adapter wraps the FS context's `DirScanner` and `DirScanInput` behavior.
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

### 5. Identity Model: Evaluate `FsNodeId` Against `FileId` / `DirId`

The PRD recommends using a single canonical `FsNodeId` with file/dir-specific node structs:

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

`FileNode` should carry at minimum:

- `FsNodeId`
- parent `FsNodeId` for the containing directory, when present
- `PathKey`
- `FileName`
- `FileFormat`
- `FileMetadata`
- recorded/indexed timestamp if needed for observability

`DirNode` should carry at minimum:

- `FsNodeId`
- parent `FsNodeId`, when present
- `PathKey`
- `DirName`
- `DirMetadata`
- recorded/indexed timestamp if needed for observability

The Indexer must not store context-owned content hashes in `FileNode`. Schema, Note, and Template own content hashing and semantic freshness checks after filesystem freshness has been classified.

### 7. Indexing Result Contract

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

`IndexResult` should include:

- file entries for new, fresh, and stale files
- directory entries for new, fresh, and stale directories
- deleted node records
- summary counts
- non-fatal per-node failures when the indexing run can continue safely

### 8. Freshness Classification

The initial freshness model is filesystem metadata based.

For files, a node is `Fresh` only when stored metadata matches scanned metadata. The comparison should include size and available timestamps. A node is `Stale` when size or timestamp differs, or when the indexing scope bypasses freshness checks.

For directories, a node is `Fresh` only when stored directory metadata matches scanned directory metadata. Directory size is not portable and must not be used.

Content hashing remains out of Indexer scope. The Indexer is allowed to prevent unnecessary downstream work by classifying filesystem-level freshness, but it must not decide Schema, Note, or Template semantic freshness.

### 9. Scope Model

The Indexer should accept an explicit `IndexScope` separate from Config. Config describes stable runtime boundaries. CLI and orchestration provide run-specific intent.

Initial scopes:

- Full Vault scan with normal freshness checking.
- Full Vault scan with freshness bypass.
- Context scan for a configured context boundary.
- Targeted scan for one file or subtree.

Future-compatible scope:

- Event-driven scan from filesystem watcher events.

The Indexer should always prefer the narrowest correct scope. If Config detects a localized context boundary change, orchestration should request a context scan rather than a full Vault scan.

### 10. Repository Ports And Storage Adapter

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

Storage adapter implementation should use Indexer-owned table definitions. redb primitives and table mechanics stay inside the adapter. Public repository ports expose Indexer domain errors, not redb errors.

### 11. Scanner Port And FS Adapter

The Indexer should define a scanner port instead of depending directly on concrete `DirScanner` from application logic. The FS adapter can wrap `DirScanner` and translate `DirScanInput` output into Indexer scan records.

This preserves the global invariant that filesystem interaction happens through the FS context while keeping Indexer application logic testable. Tests can provide an in-memory scanner implementation with deterministic file and directory entries.

### 12. Context Processor Consumption

Schema, Note, and Template processors should consume Indexer output or query Indexer repository ports rather than performing their own vault-wide scans.

Context routing should use Config-resolved boundaries and Indexer result entries. Routing should not live in Discovery. It may live in Indexer if it is purely filesystem-node partitioning, or in orchestration if routing requires cross-context dependency decisions.

The initial contract should be conservative: Indexer returns all indexed file and directory entries in scope, plus deleted records. Orchestration or a small routing module partitions entries for Schema, Note, and Template based on Config specs.

### 13. CLI Command Intent And Execution Flow

The CLI remains a thin orchestration layer. It defines command intent and diagnostic output, then delegates to core contexts.

Recommended command surface:

- `lithos index`: resolve Discovery and Config, run the Indexer, print summary diagnostic output.
- `lithos index --force`: bypass freshness checks for the selected scope.
- `lithos index --path <path>`: run a targeted scan for one file or directory subtree.
- `lithos index --context <schema|note|template>`: run indexing for the configured context boundary when that boundary can be derived from Config.
- `lithos index --dry-run`: scan and classify without persisting changes, useful for diagnostics.
- `lithos index --format <human|json>`: choose diagnostic output format.
- `lithos index status`: report persisted index summary without scanning, including node counts by kind/format and last indexed timestamp when available.
- `lithos index explain <path>`: show how a path resolves to a node, classification inputs, and downstream routing candidates, if present.

The exact command shape can be refined during CLI implementation, but these intents should be represented in the PRD so Indexer scope and result contracts are designed with CLI observability in mind.

CLI execution flow for `lithos index`:

1. Parse command intent and flags.
2. Run Discovery to resolve Vault Root and config paths.
3. Run Config to load, validate, merge, hash, and produce narrowed specs.
4. Map CLI flags plus Config specs into `IndexScope`.
5. Construct Indexer adapters.
6. Run Indexer service.
7. Print diagnostic output from `IndexResult`.

CLI must not own filesystem freshness rules, path-key conversion invariants, repository transaction semantics, or context routing semantics. CLI errors should map Indexer errors into actionable diagnostic output.

### 14. Event Sourcing And Restartability

Generic event-log infrastructure belongs to the Event Sourcing Foundation PRD. The Filesystem Indexer PRD may leave hooks for future restartability but should not define shared event primitives.

If Indexer-specific events are added later, they should reuse foundation contracts and remain owned by the Indexer context. They should model Indexer transitions only, not Schema, Note, Template, or Config transitions.

For this PRD, restartability is optional and deferred unless implementation sequencing requires it.

### 15. Migration Notes

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
4. Scanner adapter tests prove FS `DirScanner` output is translated into Indexer scan records without leaking FS adapter details into domain contracts.
5. Application-service tests classify missing persisted nodes as `New`.
6. Application-service tests classify metadata-matching nodes as `Fresh`.
7. Application-service tests classify changed metadata nodes as `Stale`.
8. Application-service tests classify forced scans as `Stale` even when metadata matches.
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

## Further Notes

- This PRD should be implemented after root/config discovery and Config pipeline contracts can produce the narrowed specs the Indexer consumes.
- This PRD should respect ADR 016, ADR 018, ADR 019, and ADR 020.
- The Indexer context may require a new `CONTEXT.md` glossary when implementation begins. Candidate terms include Filesystem Node, File Node, Directory Node, Index Scope, Index Status, Indexed Node, and Deleted Node.
- If the choice of `FsNodeId` proves too weak for file-only or directory-only compile-time safety, implementation can add thin `FileNodeId` and `DirNodeId` wrappers over `FsNodeId` without changing the underlying canonical identity space.
