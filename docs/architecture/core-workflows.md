# Core Workflows

## Template Generation (`lithos new`)

```mermaid
sequenceDiagram
    participant User
    participant CLI as Cobra CLI
    participant CO as CommandOrchestrator
    participant TE as TemplateEngineService
    participant INT as InteractivePort
    participant QS as QueryService
    participant SV as SchemaValidatorService
    participant FS as FileSystemPort

    User->>CLI: lithos new daily-note
    CLI->>CO: New(ctx,"daily-note")
    CO->>TE: Execute(templateID)
    TE->>INT: Prompt/Suggester for inputs
    TE->>QS: lookup()/query() for related notes
    QS-->>TE: Note metadata
    TE->>SV: Validate rendered frontmatter
    SV-->>TE: Validation result
    TE->>FS: Write file via atomic rename
    FS-->>TE: Write confirmation
    TE-->>CO: RenderResult (path, warnings)
    CO-->>CLI: Success summary
    CLI-->>User: Display output + hints
```

**Notes:** Any validation errors bubble back through `RenderResult` for the CLI to display with remediation. Context cancellation propagates downward if the user aborts.

## Template Discovery (`lithos find`)

```mermaid
sequenceDiagram
    participant User
    participant CLI as Cobra CLI
    participant CO as CommandOrchestrator
    participant TE as TemplateEngineService
    participant TR as TemplateRepositoryPort
    participant INT as InteractivePort
    participant QS as QueryService
    participant SV as SchemaValidatorService
    participant FS as FileSystemPort

    User->>CLI: lithos find
    CLI->>CO: Find(ctx)
    CO->>TE: Launch finder
    TE->>TR: ListTemplates()
    TR-->>TE: Template metadata list
    TE->>INT: Fuzzy finder (select template)
    INT-->>TE: Selected template ID
    TE->>QS: Optional lookups for preview
    QS-->>TE: Preview data (if requested)
    TE->>SV: Validate rendered result
    SV-->>TE: Validation result
    TE->>FS: Write generated file
    FS-->>TE: Confirmation
    TE-->>CO: RenderResult
    CO-->>CLI: Output summary
    CLI-->>User: Display generated note path
```

**Notes:** If the user exits the fuzzy finder without selection, `RenderResult` returns a canceled status and no file is written.

## Vault Indexing (`lithos index`)

```mermaid
sequenceDiagram
    participant User
    participant CLI as Cobra CLI
    participant CO as CommandOrchestrator
    participant VI as VaultIndexingService
    participant FS as FileSystemPort
    participant SV as SchemaValidatorService
    participant CC as CacheCommandPort
    participant REG as QueryService (refresh)

    User->>CLI: lithos index
    CLI->>CO: Index(ctx)
    CO->>VI: Rebuild(vaultPath)
    VI->>FS: Walk vault (ignore .lithos)
    FS-->>VI: Markdown file list
    VI->>FS: Read file contents
    VI->>SV: Validate frontmatter
    SV-->>VI: Validation result
    VI->>CC: Store(note) via atomic rename
    CC-->>VI: Write confirmation
    VI->>REG: Refresh in-memory indices
    VI-->>CO: IndexStats (count, duration, warnings)
    CO-->>CLI: Summary + follow-up guidance
    CLI-->>User: Display stats & next steps
```

**Notes:** Failed validations are reported as warnings (file path + reason) and do not block indexing. Subsequent commands reuse refreshed QueryService indices.

---
