# Config

The Config context defines how settings are discovered, merged, validated, and exposed to other contexts. It is the single source of truth for resolved application settings.

## Language

**Config Source**:
An origin of settings input — for example a file, environment variable, or CLI override.
_Avoid_: input, payload

**Global Config**:
System-wide configuration from environment variables or global config files. Applied as the base layer before vault-local overrides.
_Avoid_: environment config, system config

**Local (Vault) Config**:
Vault-specific configuration that overrides environment settings for a particular vault.
_Avoid_: local settings, vault config, project config

**Precedence Chain**:
The ordered rule that decides which Config Source wins when the same key appears in multiple sources. Global Config yields to Local Config.
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
> **Domain expert**: The Precedence Chain. Global Config is always the base; Local Config overrides it. If the same key appears in both, the vault value wins.
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

---

# Discovery

The Discovery context locates the runtime filesystem context needed before configuration can be loaded. It answers: where is the vault, and where are its config files?

## Language

**Vault Root**:
The directory that bounds a Traces vault and serves as the base for local config path resolution.
_Avoid_: project root, workspace root

**Root Marker**:
A conventional config filename whose presence in a directory establishes that directory as the Vault Root.
_Avoid_: config location, index marker, marker file

**Candidate Marker**:
A config file found during traversal or global resolution, ordered by source precedence but not yet confirmed as the config input.
_Avoid_: discovered config, selected config, result file

**Ascending Walk**:
The traversal strategy that searches parent directories upward from a starting anchor until a Root Marker is found or a boundary stops the walk.
_Avoid_: upward scan, directory crawl, recursive search

**Ceiling**:
A directory boundary that stops the Ascending Walk, preventing discovery from searching above a declared limit.
_Avoid_: stop path, upper bound, limit dir

**Override**:
An explicit Vault Root path supplied via CLI flag or environment variable that preempts traversal entirely.
_Avoid_: forced path, hardcoded path, explicit config

**Discovery Result**:
The output of a discovery run: the located Vault Root and all ranked Candidate Markers, ready for consumption by Config.
_Avoid_: discovery output, found configs, resolved paths

**Discovery Report**:
Process metadata produced alongside the Discovery Result, capturing skipped Overrides, skipped Ceilings, and why the Ascending Walk stopped. Consumed by the Bootstrapper for diagnostics only; downstream contexts never see it.
_Avoid_: discovery log, diagnostics, discovery warnings

## Example Dialogue

> **Dev**: The user ran `traces sync` from inside their vault. How does Discovery find the vault root?
>
> **Domain expert**: Discovery starts an Ascending Walk from the current directory. It checks each parent directory for a Root Marker — a file like `traces.toml`. The first directory where it finds one becomes the Vault Root.
>
> **Dev**: What if the user also set `TRACES_VAULT`?
>
> **Domain expert**: That's an Override. It preempts the Ascending Walk entirely. Discovery validates the path and returns it as the Vault Root without walking at all.
>
> **Dev**: And if the walk goes too far up?
>
> **Domain expert**: A Ceiling stops it. If the walk reaches a Ceiling directory — or a project boundary like `.git` — it stops, even if no Root Marker was found.
>
> **Dev**: What does Discovery hand off to Config?
>
> **Domain expert**: A Discovery Result: the Vault Root and the ordered list of Candidate Markers. Config picks the winner and parses it. The Discovery Report goes to the Bootstrapper for surfacing diagnostics — Config never sees it.
