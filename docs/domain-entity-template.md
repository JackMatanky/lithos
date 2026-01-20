# Domain Entity Documentation Template

## Overview
[Describe the entity purpose and business context.]

## Structure
[List fields, types, and ownership boundaries.]

## Rust-Specific Patterns Used
- **Const Generics:** [If used for compile-time validation.]
- **Phantom Types:** [If used for context safety.]
- **Associated Types:** [If used in ports/interfaces.]
- **Memory Optimization:** [Box<str>, Arc<str> usage.]
- **Domain Purity:** [Enforcement via domain-only dependencies.]

## Validation Rules
[Semantic validation requirements and invariants.]

## Business Logic
[Key business rules and invariants.]

## Relationships
[Connections to other entities/contexts.]

## Evolution Guidelines
[When/how to modify this entity.]

---

## Example Filled-Out Template

# Note (Aggregate)

## Overview
The Note aggregate is the core domain entity representing a vault note. It owns subentities that describe metadata, structure, and embedded references while enforcing vault-relative invariants.

## Structure
- `id: Uuid` — UUID v7 identity.
- `path: Box<str>` — vault-relative path.
- `frontmatter: Option<Frontmatter>` — YAML metadata.
- `links: Vec<Link>`, `embeds: Vec<Link>` — outbound relationships.
- `tags: Vec<Tag>`, `headings: Vec<Heading>`, `tasks: Vec<Task>`, `sections: Vec<Section>` — structured subentities.
- `pending_events: Vec<NoteEvents>` — internal domain events.

## Rust-Specific Patterns Used
- **Memory Optimization:** `Box<str>` for immutable path storage.
- **Domain Purity:** Aggregate is domain-only with no I/O dependencies.

## Validation Rules
- Path must be non-empty, relative, and end with `.md`.
- Link and embed source IDs must match the aggregate ID.

## Business Logic
- Emits `NoteCreated` on construction.
- Enforces internal consistency through `validate`.

## Relationships
- References Config for frontmatter keys (adapter/app layer usage).
- Validated against Schema in application layer.

## Evolution Guidelines
- Add new subentities with defaults and migration paths.
- Deprecate fields before removal.
