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

## Invariants

- Discovery returns paths and source metadata only; it does not parse, merge, validate, or hash config contents.
- Config consumes discovered config paths and owns all behavior after file paths are selected.
- Indexer consumes resolved config specs and owns filesystem node indexing after Config is resolved.

## Not Owned Here

- Config content parsing, merge behavior, validation, hashing, and config specs.
- Filesystem node indexing, freshness classification, and file/directory identity persistence.
