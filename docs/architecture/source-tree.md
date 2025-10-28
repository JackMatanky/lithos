# Source Tree

```plaintext
lithos/
├── cmd/
│   └── lithos/
│       └── main.go
├── internal/
│   ├── domain/              # Core models
│   │   ├── config.go        # Config model
│   │   ├── note.go          # Note models: Note, NoteID, Frontmatter
│   │   ├── property.go      # Property model
│   │   ├── property_bank.go # PropertyBank singleton model
│   │   ├── property_spec.go # PropertySpec models: StringSpec, NumberSpec, DateSpec, BooleanSpec, FileSpec
│   │   ├── schema.go        # Schema model
│   │   └── template.go      # Template models: Template, TemplateID
│   ├── app/                 # Domain services & orchestrators (template, indexing, schema, query, command)
│   │   ├── command/
│   │   ├── indexing/
│   │   ├── query/
│   │   │   └── service.go   # QueryService
│   │   ├── schema/
│   │   │   ├── engine.go    # SchemaEngine
│   │   │   ├── resolver.go  # SchemaResolver
│   │   │   └── validator.go # SchemaValidator
│   │   └── template/
│   ├── ports/
│   │   ├── api/             # Drivers: CLIPort and related contracts
│   │   │   ├── cli.go       # Cobra CLI
│   │   │   └── command.go   # Command Handler
│   │   └── spi/             # CacheWriter, CacheReader, VaultReader, VaultWriter, SchemaPort, etc.
│   ├── adapters/
│   │   ├── api/             # Drivers: Cobra CLI today; Bubble Tea/LSP post-MVP
│   │   │   └── cli/         # Cobra CLI
│   │   └── spi/
│   │       ├── cache/
│   │       ├── config/
│   │       ├── interactive/
│   │       ├── schema/
│   │       └── template/
│   └── shared/              # Cross-cutting helpers (logger, errors, registry, utilities)
│       ├── errors/
│       ├── logger/
│       └── registry/
├── pkg/                     # Reserved for future public modules
├── templates/               # Default template pack shipped with CLI
├── schemas/                 # User-defined schemas + property bank
├── testdata/                # Golden vault used in automated tests
├── tests/                   # Test suite
│   ├── e2e/                 # End-to-end tests
│   ├── integration/         # Integration tests
│   └── utils/
│       └── mocks.go         # Mock port implementations for testing
├── .lithos/                 # Runtime cache (ignored in version control)
└── docs/                    # PRD, architecture, elicitation summaries
```

---
