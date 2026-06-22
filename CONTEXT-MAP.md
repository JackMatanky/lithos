# Context Map

## Contexts

- [Settings](./crates/settings/CONTEXT.md) - locates vault root, configuration sources, precedence, and validated runtime settings
- [Indexer](./crates/indexer/CONTEXT.md) _(planned)_ - filesystem node scanning and indexing, classifying nodes by type and tracking index status
- [Note](./crates/note/CONTEXT.md) - parses and models notes, tasks, links, tags, and note metadata
- [Schema](./crates/schema/CONTEXT.md) - defines and resolves schema rules used for metadata validation
- [Template](./crates/template/CONTEXT.md) _(planned)_ - defines template assets and rendering constraints for note generation
- [DB](./crates/db/CONTEXT.md) - infrastructure context for persistence, transactions, and zero-copy data access
- [FS](./crates/fs/CONTEXT.md) - infrastructure context for safe file discovery, reads, and writes
- [Utils](./crates/utils/CONTEXT.md) - outward-facing utility contracts and dependency governance for reusable primitives
- [Support](./crates/support/CONTEXT.md) - crate-private implementation internals and internal support facade
- [App](./crates/app/CONTEXT.md) - composition root, orchestrator, and unified facade
- [CLI](./crates/cli/src/CONTEXT.md) - command-line entrypoints and user-facing orchestration of core contexts

## Relationships

- **Note -> Schema**: Note metadata is validated against schema definitions selected via File Class schema-name reference
- **Template -> Schema** _(planned)_: Template inputs and generated structures align with schema-defined property semantics
- **Settings -> Note**: Configuration controls note ingestion and interpretation behavior
- **Settings -> Schema**: Configuration controls schema loading and validation behavior
- **Settings -> Template** _(planned)_: Configuration controls template lookup and rendering behavior
- **Settings -> Indexer** _(planned)_: Configuration provides index scope specs that define which filesystem nodes are eligible for scanning
- **App -> Settings, Note, Schema, Template, DB, FS**: App orchestrates end-user workflows across contexts
- **CLI -> App**: CLI coordinates with the App facade
- **Schema (shared semantics)**: Schema uses a global Property Bank for reusable property definitions and resolves parent-child inheritance with explicit excludes
- **Note, Schema, Template -> DB (infrastructure)** _(Template planned)_: Business contexts persist/query through repository contracts backed by DB infrastructure
- **Note, Schema, Template -> FS (infrastructure)** _(Template planned)_: Business contexts ingest and materialize file-backed state through FS abstractions
- **Indexer -> Schema, Note, Template** _(planned)_: Indexer feeds classified filesystem nodes to downstream business contexts for content processing
- **Settings -> FS (infrastructure)**: Configuration loading depends on filesystem reads for vault root and selected config paths
- **Note, Schema, Template, Settings, DB -> Utils** _(Template planned)_: Contexts consume stable outward-facing utility contracts
- **DB, Schema, Settings -> Support (internal)**: Internal modules consume crate-private support internals
- **Support -> Utils (promotion path)**: Stabilized, outward-facing internals move from support into utils by explicit governance decisions

## Global Invariants

- **Filesystem Isolation**: All interaction with the filesystem MUST happen through the `FS` context:
  - File reads via `FileReader`.
  - File writes via `FsWriter`.
  - Directory scanning via `ScannerPort` (delegated to Indexer context).
- **Segregated Repository Pattern**: Business contexts (Note, Schema, Template _(planned)_, Config) MUST define their own segregated Repository interfaces (Read, Write, and Unified traits) to decouple domain logic from infrastructure.
