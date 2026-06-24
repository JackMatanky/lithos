# Note
The Note context models vault notes and extracts note-domain structures such as metadata, tasks, links, and tags.

## Language
**Note**:
A single vault document represented as domain state.
_Avoid_: record, row

**Frontmatter**:
The structured metadata block associated with a note.
_Avoid_: header blob, config block

**File Class**:
A frontmatter property whose value references the schema applied to the note.
_Avoid_: note type, category label

**Link Edge**:
A directed relationship extracted between notes.
_Avoid_: pointer, reference line

### Lists
**List Item**:
A single item extracted from a markdown list structure.
_Avoid_: row, collection entry

**Checklist**:
A list item with a checked or unchecked checkbox state.
_Avoid_: todo row, action row

**Task**:
A checklist item promoted to task semantics by configured promotion rules.
_Avoid_: todo row, action row

## Invariants
- Vault files are the source of truth for note state.
- Parsed note identity remains stable across persistence operations.
- Extracted structures are derived through explicit parsing boundaries.
- File Class values resolve to schema references during metadata validation.

## Interfaces
- Defines a unified `Repository` trait for all persistence operations.

## Not Owned Here
- Schema definition semantics and schema inheritance/resolution rules.
- Template rendering contracts and template asset lifecycle.
- Persistence engine mechanics and filesystem path safety policy.

## Resources
- **[Obsidian](https://obsidian.md)** provides the primary conceptual model for vault-oriented markdown workflows.
  - Obsidian API:
    - Source: <https://raw.githubusercontent.com/obsidianmd/obsidian-api/master/obsidian.d.ts>
    - Internal Reference: `docs/refs/obsidian/api-reference.md`
- **[Obsidian Dataview](https://blacksmithgu.github.io/obsidian-dataview/)** influences metadata extraction and note indexing semantics for queryable note data.
  - GitHub: <https://github.com/blacksmithgu/obsidian-dataview>
  - Source Digest: `docs/refs/digests/obsidian_blacksmithgu-obsidian-dataview-src-digest.txt`
  - Docs: `docs/refs/digests/obsidian_blacksmithgu-obsidian-dataview-docs-digest.txt`
  - Internal Reference: `docs/refs/obsidian/dataview-reference.md`
