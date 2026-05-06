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
A frontmatter key whose value is a schema-name reference to the schema applied to the note.
_Avoid_: note type, category label

**Task**:
A checklist item extracted from note content.
_Avoid_: todo row, action row

**Link Edge**:
A directed relationship extracted between notes.
_Avoid_: pointer, reference line

## Invariants

- Vault files are the source of truth for note state.
- Parsed note identity remains stable across persistence operations.
- Extracted structures are derived through explicit parsing boundaries.
- File Class values are schema-name references used for note metadata validation.

## Not Owned Here

- Schema definition semantics and schema inheritance/resolution rules.
- Template rendering contracts and template asset lifecycle.
- Persistence engine mechanics and filesystem path safety policy.
