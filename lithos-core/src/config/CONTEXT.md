# Config

The Config context defines how settings are discovered, merged, validated, and exposed to other contexts.

## Language

**Config Source**:
An origin of settings input (for example file, environment, or CLI override).
_Avoid_: input, payload

**Environment Config**:
System-wide configuration from environment variables or global config files.
_Avoid_: global settings, system config

**Local (Vault) Config**:
Vault-specific configuration that overrides environment settings for a specific vault.
_Avoid_: local settings, vault config, project config

**Precedence Chain**:
The ordered rule that decides which source wins when keys conflict.
Precedence order (lowest to highest): Environment Config < Local (Vault) Config.
_Avoid_: priority guess, merge magic

**Resolved Config**:
The final validated settings object consumed by other contexts.
_Avoid_: raw config, partial config

**Config Spec**:
A context-facing config contract that exposes only the values needed by a specific downstream context.
_Avoid_: raw settings map, generic config blob

## Invariants

- The precedence chain is deterministic: Environment Config < Local (Vault) Config.
- Invalid settings do not produce a resolved config.
- Downstream contexts consume resolved config, not raw source fragments.
- Contexts consume narrowed Config Specs rather than directly consuming full resolved config.
- Local (vault) config always overrides environment config for the same keys.
- Defines a unified `Repository` trait for all persistence operations.

## Examples

- `TaskConfigSpec` is a config spec for task-related behavior.
- `FrontmatterConfigSpec` is a config spec for frontmatter interpretation behavior.
- Environment Config: `RawGlobalConfig` from `~/.config/lithos/lithos.toml`
- Local Config: `RawVaultConfig` from `<vault-root>/lithos.toml`

## Not Owned Here

- Note parsing semantics and note-level extraction rules.
- Schema definition and schema graph resolution semantics.
- Template rendering behavior and template syntax semantics.
