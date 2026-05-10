# Schema

The Schema context defines metadata/property rules and resolves references into validated schema-domain state.

## Language

**Schema**:
A named set of metadata/property constraints.
_Avoid_: class, model definition

**Property**:
A named metadata field that can be attached to a schema.
_Avoid_: attribute instance, field value

**Property Spec**:
The type-specific rule set that defines validation fields for a property.
_Avoid_: generic field list, loose options

**Property Bank**:
A global schema-level registry of reusable property specs that schemas can reference.
_Avoid_: property cache, field bag

**Property Bank Reference**:
A pointer from a schema property entry to a reusable property definition in the Property Bank.
_Avoid_: schema link, include path

**Schema Inheritance**:
The parent-child relationship where a child schema extends a parent schema.
_Avoid_: schema copy, manual merge

**Exclude List**:
The child-declared set of inherited parent properties to omit.
_Avoid_: delete hint, override blacklist

**Resolved Schema**:
Schema state after references and inheritance are fully expanded.
_Avoid_: raw schema, partial schema

## Invariants

- A resolved schema contains no unresolved internal references.
- Resolution behavior is deterministic for the same schema inputs.
- Semantic validation failures block projection as resolved schema state.
- Every Property Bank Reference resolves to exactly one concrete Property Spec before schema projection.
- The Property Bank is treated as global schema state for shared property definitions.
- Child schema inheritance resolves in parent-to-child order with explicit Exclude List entries applied.
- Excluded parent properties are omitted from the child resolved schema output.
- Defines a unified `Repository` trait for all persistence operations.

## Not Owned Here

- Note content parsing and note structural extraction.
- Template rendering behavior and generation semantics.
- Filesystem path policy and storage transaction mechanics.
