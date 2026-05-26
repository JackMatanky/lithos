# Context Map

## Contexts

- [Config](./lithos-core/src/config/CONTEXT.md) - defines configuration sources, precedence, and validated runtime settings
- [Note](./lithos-core/src/note/CONTEXT.md) - parses and models notes, tasks, links, tags, and note metadata
- [Schema](./lithos-core/src/schema/CONTEXT.md) - defines and resolves schema rules used for metadata validation
- [Template](./lithos-core/src/template/CONTEXT.md) - defines template assets and rendering constraints for note generation
- [DB](./lithos-core/src/db/CONTEXT.md) - infrastructure context for persistence, transactions, and zero-copy data access
- [FS](./lithos-core/src/fs/CONTEXT.md) - infrastructure context for safe file discovery, reads, and writes
- [Utils](./lithos-core/src/utils/CONTEXT.md) - outward-facing utility contracts and dependency governance for reusable primitives
- [Support](./lithos-core/src/support/CONTEXT.md) - crate-private implementation internals and internal support facade
- [CLI](./lithos-cli/src/CONTEXT.md) - command-line entrypoints and user-facing orchestration of core contexts

## Relationships

- **Note -> Schema**: Note metadata is validated against schema definitions selected via File Class schema-name reference
- **Template -> Schema**: Template inputs and generated structures align with schema-defined property semantics
- **Config -> Note**: Configuration controls note ingestion and interpretation behavior
- **Config -> Schema**: Configuration controls schema loading and validation behavior
- **Config -> Template**: Configuration controls template lookup and rendering behavior
- **CLI -> Config, Note, Schema, Template**: CLI coordinates end-user workflows across business contexts
- **Schema (shared semantics)**: Schema uses a global Property Bank for reusable property definitions and resolves parent-child inheritance with explicit excludes
- **Note, Schema, Template -> DB (infrastructure)**: Business contexts persist/query through repository contracts backed by DB infrastructure
- **Note, Schema, Template -> FS (infrastructure)**: Business contexts ingest and materialize file-backed state through FS abstractions
- **Config -> FS (infrastructure)**: Configuration loading depends on filesystem sources and path rules
- **Note, Schema, Template, Config, DB -> Utils**: Contexts consume stable outward-facing utility contracts
- **DB, Schema, Config -> Support (internal)**: Internal modules consume crate-private support internals
- **Support -> Utils (promotion path)**: Stabilized, outward-facing internals move from support into utils by explicit governance decisions

## Global Invariants

- **Filesystem Isolation**: All interaction with the filesystem MUST happen through the `FS` context:
  - File reads via `FsReader`.
  - File writes via `FsWriter`.
  - Directory scanning via `DirScanner`.
- **Segregated Repository Pattern**: Business contexts (Note, Schema, Template, Config) MUST define their own segregated Repository interfaces (Read, Write, and Unified traits) to decouple domain logic from infrastructure.
