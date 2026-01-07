# ADR 004: Hierarchical Configuration Management with Figment

## Status
Accepted

## Context
Lithos requires a sophisticated, 6-layer hierarchical configuration structure to balance global defaults with fine-grained project and vault-specific overrides. The priority chain (from lowest to highest) is:
1.  **Global Defaults**: Compiled into the binary (baked-in).
2.  **User Config**: Standard OS location (e.g., `~/.config/lithos/config.toml`).
3.  **Project Config**: Located at the root of the current project (`.lithos/config.toml`).
4.  **Vault Config**: Specific overrides for the currently active Obsidian vault.
5.  **Environment Variables**: Prefix `LITHOS_` (e.g., `LITHOS_LOG_LEVEL`).
6.  **CLI Flags**: Immediate overrides via command-line arguments.

Technical requirements include:
-   **Deep Merging**: Correctly merging nested TOML tables across all layers.
-   **Type Safety**: Direct mapping to `serde`-derived Rust structs.
-   **Diagnostic Quality**: Clear feedback (line/column) when a user's TOML is malformed.
-   **Extensibility**: Ability to inject custom logic (e.g., "Vault Discovery") into the configuration loading process.

## Decision
We will use **Figment** as the configuration management framework for Lithos Rust.

### Comparison of Candidates

| Feature | **config-rs** | **Figment** | **Serde + Custom Merging** |
| :--- | :--- | :--- | :--- |
| **Logic Pattern** | Sequential Builder | **Provider Pattern** | Manual Implementation |
| **Merge Quality** | Good (some edge cases with maps) | **Excellent (Native nested merging)** | Perfect (but manually coded) |
| **Type Safety** | Moderate (loose typing until final) | **High (Type-safe providers)** | High |
| **Error Feedback** | Basic | **Rich (Includes line/column metadata)** | Variable |
| **Extensibility** | Middleware-based | **Custom Provider Trait** | Infinite |
| **Rust Idioms** | Veteran/Older | **Modern/Rocket-inspired** | Minimalist |

## Rationale

### 1. The "Provider" Pattern vs. Traditional Merging
Traditional libraries like `config-rs` use a linear builder pattern where each step overwrites or merges into a global state. This often makes it difficult to reason about "where" a specific value came from during debugging.

**Figment's Provider Pattern** treats every configuration source (file, env, flag) as a separate entity that "provides" data. Figment then performs a single, coherent merge operation. This aligns with the **ADR 001** principle of "Mechanical Sympathy"—it reduces intermediate allocations and allows for more efficient processing of the configuration tree.

### 2. Extensibility: Vault Discovery
Lithos needs to dynamically discover the "Vault Config" by walking up the directory tree from the Current Working Directory (CWD). Figment makes this trivial by allowing us to implement the `Provider` trait for a `VaultDiscovery` struct. This provider can find the `.lithos/` directory and return a `Toml` provider for that specific path, integrating seamlessly into the standard hierarchy.

### 3. Error Diagnostics
Since Lithos is a tool for developers and power users, "Invalid TOML" errors must be actionable. Figment provides excellent error metadata, including line and column information, which allows the application to point the user directly to the source of the configuration error.

### 4. Ecosystem Alignment
Figment is built by the Rocket team and is designed for high-performance, type-safe Rust applications. It is significantly more maintained and idiomatic than the older `config-rs` and provides a much better developer experience than manual `serde` merging, which would require significant boilerplate for every new configuration field.

## Consequences
-   **Custom Providers**: We must implement a `Clap` provider for Figment to bridge our `clap` CLI flags into the Figment hierarchy.
-   **Dependency Profile**: Figment is already present in our `Cargo.lock`, so this decision formalizes its use and sets the standard for how it should be configured (using the 6-layer chain).
-   **Boilerplate Reduction**: The `adapters/config` module will become significantly leaner, as Figment handles the complexity of hierarchy and environment variable mapping.
