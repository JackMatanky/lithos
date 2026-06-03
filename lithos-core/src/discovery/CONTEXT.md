# Discovery

The Discovery context locates the runtime filesystem context needed before configuration can be loaded.

## Language

**Vault Root**:
The directory that bounds a Lithos vault and serves as the base for local config path resolution.
_Avoid_: project root, workspace root

**Root Marker**:
A conventional config filename whose presence can establish a Vault Root during ascending discovery.
_Avoid_: config location, index marker

**Discovered Config Path**:
A selected config file path with source, format, and base-path metadata, before config contents are parsed.
_Avoid_: loaded config, resolved config

## Module Architecture

Discovery is split into focused modules:

- **`engine`**: [`DiscoveryEngine`] orchestrates vault/global discovery using policy-driven precedence; returns [`VaultDiscoveryResult`] or [`GlobalDiscoveryResult`].
- **`error`**: [`DiscoveryError`] for fallible operations, [`VaultDiscoveryWarning`] for non-fatal diagnostics.
- **`policy`**: [`DiscoveryPolicy`] defines source precedence; [`VaultSourceType`], [`GlobalSourceType`] enumerate origins.
- **`selector`**: [`select_candidate`] picks the highest-precedence marker; [`promote_alternative`] prefers a specific format.
- **`walk`**: [`AscendingWalker`] iterates parent directories with symlink cycle detection; [`DiscoveryBoundaries`] holds start/ceiling context.
- **`probe`**: [`DiscoveryProbe`] trait for directory probing; [`VaultRootProbe`] detects root marker files using [`MarkerPattern`] patterns.
- **`marker`**: [`FoundRootMarker`] is the typed handoff to Config.
- **`diagnostics`**: [`DiscoveryWarning`] enum for structured non-fatal diagnostics.

## Resolution Precedence

1. **Explicit Flag**: A path provided directly via CLI (highest precedence).
2. **Environment Variable**: A path from `LITHOS_VAULT`.
3. **Ascending Walk**: Search upward from cwd for a marker file, bounded by ceilings.

## Invariants

- Discovery returns paths and source metadata only; it does not parse, merge, validate, or hash config contents.
- `discovery/` must NOT import from `config/` — ever.
- Config consumes discovered config paths and owns all behavior after file paths are selected.
- Indexer consumes resolved config specs and owns filesystem node indexing after Config is resolved.

## Not Owned Here

- Config content parsing, merge behavior, validation, hashing, and config specs.
- Filesystem node indexing, freshness classification, and file/directory identity persistence.
