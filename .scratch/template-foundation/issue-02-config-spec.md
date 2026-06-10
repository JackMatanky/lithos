# Template Configuration Spec

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Define `TemplateConfigSpec` — the template context's narrow view into the global `Config`. This follows the same pattern as `SchemaConfigSpec`: the template context only sees the fields it actually needs, with no direct dependency on the full `Config` shape.

```rust
pub struct TemplateConfigSpec {
    /// Vault root directory.
    root: DirPath,
    /// Relative path to the template directory from vault root.
    directory: RelativeDirPath,
}
```

`TemplateConfigSpec` should expose a constructor (likely `From<&Config>` or a dedicated `from_config` method) and accessors for `root` and `directory`, resolving the full template directory path on demand.

Template discovery is scoped to `.md` files at any depth within the configured template directory.

## Acceptance criteria

- [ ] `TemplateConfigSpec` is defined with `root: DirPath` and `directory: RelativeDirPath` private fields
- [ ] A constructor exists that can be built from the application `Config` (matching the `SchemaConfigSpec` pattern)
- [ ] `TemplateConfigSpec` exposes accessors sufficient for the processor and service to resolve the full template directory path
- [ ] Unit tests cover construction from a representative config value and accessor correctness

## Blocked by

- `issue-01-domain-models.md` (needs `DirPath`/`RelativeDirPath` already in scope, but more importantly this spec feeds the processor which depends on domain models)
