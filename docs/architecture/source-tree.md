# Source Tree

```plaintext
lithos/
├── cmd/
│   └── lithos/
│       └── main.go
├── internal/
│   ├── domain/              # Core models (File, Frontmatter, Note, Schema, Property)
│   ├── app/                 # Domain services & orchestrators (template, indexing, schema, query, command)
│   │   ├── command/
│   │   ├── indexing/
│   │   ├── query/
│   │   ├── schema/
│   │   └── template/
│   ├── ports/
│   │   ├── api/             # CLICommandPort and related contracts
│   │   └── spi/             # FileSystemPort, Cache ports, SchemaLoaderPort, etc.
│   ├── adapters/
│   │   ├── api/             # Cobra CLI today; Bubble Tea/LSP post-MVP
│   │   └── spi/
│   │       ├── cache/
│   │       ├── config/
│   │       ├── filesystem/
│   │       ├── interactive/
│   │       ├── schema/
│   │       └── template/
│   └── shared/              # Cross-cutting helpers (logger, errors, registry, utilities)
├── pkg/                     # Reserved for future public modules
├── templates/               # Default template pack shipped with CLI
├── schemas/                 # User-defined schemas + property banks
├── testdata/                # Golden vault used in automated tests
├── .lithos/                 # Runtime cache (ignored in version control)
└── docs/                    # PRD, architecture, elicitation summaries
```

---
