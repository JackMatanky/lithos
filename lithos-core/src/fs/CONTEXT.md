# FS

The FS context provides safe file discovery, read, and write capabilities scoped to configured vault roots.

## Language

**Vault Root**:
The configured filesystem boundary within which operations are allowed.
_Avoid_: cwd, arbitrary root

**File Source**:
A discoverable set of files eligible for ingestion.
_Avoid_: random scan, unrestricted walk

**Path Validation**:
The safety check that enforces root scope and normalized paths.
_Avoid_: best-effort check, loose validation

**Normalized Path**:
A vault-relative path normalized to forward slashes for cross-platform storage keys.
Use [`PathKey`] for database keys and serialized path storage.
Use [`FsPath`], [`DirPath`], and [`FilePath`] for filesystem operations.
_Avoid_: platform-specific path, absolute storage key

## Invariants

- File operations remain constrained to validated vault roots.
- Path validation is required before filesystem access.
- File access contracts are deterministic and testable.

## Not Owned Here

- Note/schema/template business semantics and validation rules.
- Persistence transaction semantics and archived read strategy.
- CLI command intent and user-facing output behavior.
