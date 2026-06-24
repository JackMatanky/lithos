# Schema
The Schema context defines metadata property rules for note frontmatter and resolves schema references into validated schema-domain state.

## Language
**Schema**:
A named set of note frontmatter metadata property constraints.
_Avoid_: class, model definition

**Extends List**:
The schema-declared list of parent schemas to extend.
_Avoid_: inheritance chain, parent copy list

**Exclude List**:
The child-declared set of parent properties to omit during inheritance resolution.
_Avoid_: delete hint, override blacklist

**Property Bank**:
A global schema-like registry of reusable properties that schemas can reference.
_Avoid_: property cache, field bag

### Properties
**Property**:
A named entry that can exist directly in a schema or in the Property Bank.
_Avoid_: attribute instance, field value

**Property Spec**:
The type-specific rule set that defines validation fields for a property.
_Avoid_: generic field list, loose options

**Property Bank Reference**:
A reference directive within a schema property entry that resolves to a property in the Property Bank.
_Avoid_: schema link, include path

### Resolution
**Resolved Schema**:
Schema state after Property Bank Reference expansion and property inheritance resolution.
_Avoid_: raw schema, partial schema

**Property Reference Expansion**:
The resolution step that replaces Property Bank References with their concrete Property Bank properties.
_Avoid_: import, include, copy-paste merge

**Property Inheritance**:
The resolution step that carries non-excluded parent schema properties into a child schema.
_Avoid_: copy, merge shortcut

## Invariants
- Resolution behavior is deterministic for the same schema inputs.
- The Property Bank is treated as global schema state for shared property definitions.
- Every Property Bank Reference resolves to exactly one concrete Property Spec before schema projection.
- Child schema inheritance resolves in parent-to-child order with explicit Exclude List entries applied.
- Excluded parent properties are omitted from the child Resolved Schema output.
- Semantic validation failures block projection as Resolved Schema state.
- A Resolved Schema contains no unresolved Property Bank References.

## Interfaces
- Defines segregated `Repository` interfaces (Read, Write, and Unified) for persistence operations.

## Not Owned Here
- Note content parsing and note structural extraction.
- Template rendering behavior and generation semantics.
- Filesystem path policy and storage transaction mechanics.

## Resources
- **[Metadata Menu](https://mdelobelle.github.io/metadatamenu/)** provides conceptual reference material for schema-driven note metadata workflows.
  - GitHub: <https://github.com/mdelobelle/metadatamenu>
  - Source Digest: `docs/refs/digests/obsidian_mdelobelle-metadatamenu-src-digest.txt`
  - Docs: `docs/refs/digests/obsidian_mdelobelle-metadatamenu-docs-digest.txt`
  - Internal Reference: `docs/refs/obsidian/metadata-menu-reference.md`
