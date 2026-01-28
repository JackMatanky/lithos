---
name: hierarchical-configuration-management-with-figment
status: accepted
stakeholders: [Jack (Developer), Architects]
date_proposed: 2026-01-08
date_decided: 2026-01-11
date_implemented: 2026-01-11
---

# ADR 0005: Hierarchical Configuration Management with Figment

## Context

Lithos requires a sophisticated, 6-layer hierarchical configuration structure to balance global defaults with fine-grained project and vault-specific overrides. The priority chain (from lowest to highest) is:

1.  **Global Defaults**: Compiled into the binary (baked-in).
2.  **User Config**: Standard OS location (e.g., `~/.config/lithos/config.toml`).
3.  **Project Config**: Located at the root of the current project (`.lithos/config.toml`).
4.  **Vault Config**: Specific overrides for the currently active Obsidian vault.
5.  **Environment Variables**: Prefix `LITHOS_` (e.g., `LITHOS_LOG_LEVEL`).
6.  **CLI Flags**: Immediate overrides via command-line arguments.

Technical requirements include deep merging, type safety, diagnostic quality (line/column info), and extensibility for vault discovery.

## Decision

We will use **Figment** as the configuration management framework for Lithos Rust.

### Comparison of Candidates

| Feature            | **config-rs**      | **Figment**                              | **Serde + Custom Merging** |
| :----------------- | :----------------- | :--------------------------------------- | :------------------------- |
| **Logic Pattern**  | Sequential Builder | **Provider Pattern**                     | Manual Implementation      |
| **Merge Quality**  | Good               | **Excellent (Native nested merging)**    | Perfect                    |
| **Type Safety**    | Moderate           | **High (Type-safe providers)**           | High                       |
| **Error Feedback** | Basic              | **Rich (Includes line/column metadata)** | Variable                   |

## Alternatives Considered

### config-rs

Traditional linear builder pattern where each step overwrites global state. Harder to reason about "where" a specific value came from during debugging.

### Manual Serde Merging

Requires significant boilerplate for every new field and is prone to errors in nested structure merging.

## Technical Validation

### Research Findings

- **Provider Pattern**: Figment's provider pattern treats every source (file, env, flag) as a separate entity that "provides" data. This aligns with **ADR 0002** by reducing intermediate allocations.
- **Vault Discovery**: We can implement the `Provider` trait for a `VaultDiscovery` struct to walk up the tree and find the `.lithos/` directory dynamically.
- **Error Diagnostics**: Preserves line and column information, which is critical for telling the user _exactly_ where their TOML is malformed.

### Compatibility & Performance

- **Hexagonal Alignment**: The Figment loader is isolated in `adapters/spi/config`.
- **Ecosystem**: Built by the Rocket team, significantly more maintained and idiomatic than older alternatives.

## Consequences

- **Positive**: Robust hierarchical merging, excellent error messages, easy "Vault Discovery" implementation, boilerplate reduction.
- **Negative**: Requires a custom provider bridge for `Clap` flags.
