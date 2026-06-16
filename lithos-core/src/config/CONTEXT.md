# Config

The Config context defines how settings are discovered, merged, validated, and exposed to other contexts. It is the single source of truth for resolved application settings.

## Language

**Config Source**:
An origin of settings input — for example a file, environment variable, or CLI override.
_Avoid_: input, payload

**Environment Config**:
System-wide configuration from environment variables or global config files, applied before vault-local settings.
_Avoid_: global settings, system config

**Local (Vault) Config**:
Vault-specific configuration that overrides environment settings for a particular vault.
_Avoid_: local settings, vault config, project config

**Precedence Chain**:
The ordered rule that decides which Config Source wins when the same key appears in multiple sources. Environment Config yields to Local Config.
_Avoid_: priority guess, merge magic

**Resolved Config**:
The final, validated settings object produced after merging and validation. This is what downstream contexts consume.
_Avoid_: raw config, partial config, merged config

**Config Spec**:
A narrowed view of Resolved Config exposing only the values needed by a specific downstream context.
_Avoid_: raw settings map, generic config blob, full config

**Declarative Path**:
A configuration value that records an intended file or directory location without asserting that the location exists on disk at configuration time.
_Avoid_: FS-validated path, resolved path, storage key in config

## Example Dialogue

> **Dev**: Config loads from two places — a global file and a vault file. How does it decide what wins?
>
> **Domain expert**: The Precedence Chain. Environment Config is always the base; Local Config overrides it. If the same key appears in both, the vault value wins.
>
> **Dev**: What if one of the files is missing?
>
> **Domain expert**: Config can build from whichever sources are present. If only global is found, it uses that. If both are present, it merges them through the Precedence Chain.
>
> **Dev**: And what does a downstream context like Note actually get?
>
> **Domain expert**: A Config Spec — a narrowed view of the Resolved Config containing only the keys Note needs. It never gets the whole config object.
>
> **Dev**: What about paths to vault directories in the config file?
>
> **Domain expert**: Those are Declarative Paths. Config records what the user intended, but it doesn't verify that the directories exist. Existence checks happen when those paths are actually used.
