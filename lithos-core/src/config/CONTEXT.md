# Config

The Config context defines how settings are discovered, merged, validated, and exposed to other contexts.

## Language

**Config Source**:
An origin of settings input (for example file, environment, or CLI override).
_Avoid_: input, payload

**Precedence Chain**:
The ordered rule that decides which source wins when keys conflict.
_Avoid_: priority guess, merge magic

**Resolved Config**:
The final validated settings object consumed by other contexts.
_Avoid_: raw config, partial config

**Config Spec**:
A context-facing config contract that exposes only the values needed by a specific downstream context.
_Avoid_: raw settings map, generic config blob

## Invariants

- The precedence chain is deterministic for the same source set.
- Invalid settings do not produce a resolved config.
- Downstream contexts consume resolved config, not raw source fragments.
- Contexts consume narrowed Config Specs rather than directly consuming full resolved config.

## Examples

- `TaskConfigSpec` is a config spec for task-related behavior.
- `FrontmatterConfigSpec` is a config spec for frontmatter interpretation behavior.

## Not Owned Here

- Note parsing semantics and note-level extraction rules.
- Schema definition and schema graph resolution semantics.
- Template rendering behavior and template syntax semantics.
