# Context Map

## Contexts

- [Config](./lithos-core/src/config/CONTEXT.md) - defines configuration sources, precedence, and validated runtime settings
- [Discovery](./lithos-core/src/discovery/CONTEXT.md) - locates the vault root and config file paths before configuration is loaded
- [Indexer](./lithos-core/src/indexer/CONTEXT.md) _(planned)_ - filesystem node scanning and indexing, classifying nodes by type and tracking index status
- [Note](./lithos-core/src/note/CONTEXT.md) - parses and models notes, tasks, links, tags, and note metadata
- [Schema](./lithos-core/src/schema/CONTEXT.md) - defines and resolves schema rules used for metadata validation
- [Template](./lithos-core/src/template/CONTEXT.md) _(planned)_ - defines template assets and rendering constraints for note generation
- [DB](./lithos-core/src/db/CONTEXT.md) - infrastructure context for persistence, transactions, and zero-copy data access
- [FS](./lithos-core/src/fs/CONTEXT.md) - infrastructure context for safe file discovery, reads, and writes
- [Utils](./lithos-core/src/utils/CONTEXT.md) - outward-facing utility contracts and dependency governance for reusable primitives
- [Support](./lithos-core/src/support/CONTEXT.md) - crate-private implementation internals and internal support facade
- [CLI](./lithos-cli/src/CONTEXT.md) - command-line entrypoints and user-facing orchestration of core contexts

## Relationships

- **Note -> Schema**: Note metadata is validated against schema definitions selected via File Class schema-name reference
- **Template -> Schema** _(planned)_: Template inputs and generated structures align with schema-defined property semantics
- **Discovery -> Config**: Discovery locates vault root and config file paths; Config loads and resolves the selected files
- **Config -> Note**: Configuration controls note ingestion and interpretation behavior
- **Config -> Schema**: Configuration controls schema loading and validation behavior
- **Config -> Template** _(planned)_: Configuration controls template lookup and rendering behavior
- **Config -> Indexer** _(planned)_: Configuration provides index scope specs that define which filesystem nodes are eligible for scanning
- **CLI -> Config, Note, Schema, Template** _(Template planned)_: CLI coordinates end-user workflows across business contexts
- **Schema (shared semantics)**: Schema uses a global Property Bank for reusable property definitions and resolves parent-child inheritance with explicit excludes
- **Note, Schema, Template -> DB (infrastructure)** _(Template planned)_: Business contexts persist/query through repository contracts backed by DB infrastructure
- **Note, Schema, Template -> FS (infrastructure)** _(Template planned)_: Business contexts ingest and materialize file-backed state through FS abstractions
- **Indexer -> Schema, Note, Template** _(planned)_: Indexer feeds classified filesystem nodes to downstream business contexts for content processing
- **Discovery -> FS (infrastructure)**: Discovery depends on filesystem sources and path rules to locate vault root and config files
- **Config -> FS (infrastructure)**: Configuration loading depends on filesystem reads for selected config paths
- **Note, Schema, Template, Config, DB -> Utils** _(Template planned)_: Contexts consume stable outward-facing utility contracts
- **DB, Schema, Config -> Support (internal)**: Internal modules consume crate-private support internals
- **Support -> Utils (promotion path)**: Stabilized, outward-facing internals move from support into utils by explicit governance decisions

## Global Invariants

- **Filesystem Isolation**: All interaction with the filesystem MUST happen through the `FS` context:
  - File reads via `FileReader`.
  - File writes via `FsWriter`.
  - Directory scanning via `ScannerPort` (delegated to Indexer context).
- **Segregated Repository Pattern**: Business contexts (Note, Schema, Template _(planned)_, Config) MUST define their own segregated Repository interfaces (Read, Write, and Unified traits) to decouple domain logic from infrastructure.
