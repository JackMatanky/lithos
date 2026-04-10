# Raw Schema DTO Refactor Plan (Per-Type DTOs + Strictness)

## Objective
Refactor `lithos-core/src/schema/raw/` ingestion DTOs to be strict about
`additionalProperties: false`, align with meta-schema terminology and
constraints, accept both `bool` and `boolean` on input (serialize as `bool`),
and keep semantic validation outside raw while emitting early warnings during
domain construction.

## Scope
- `lithos-core/src/schema/raw/`
- Property bank ingestion and conversion paths
- Domain construction path that consumes raw property bank entries
- Tests for raw parsing and early semantic warnings

## Non-Goals
- No changes to unrelated contexts (note, template, config, db, fs)
- No semantic validation in raw beyond syntactic regex checks
- No new dependencies
- No changes to schema JSON files unless required by code alignment

## Decisions
- Use per-type raw DTOs (`RawPropertyString`, `RawPropertyNumber`, etc.)
- Use internally tagged enums (`type`) for disambiguation
- Accept both `bool` and `boolean` on input; serialize as `bool`
- Reuse schema DTOs for property bank entries
- Property bank `required: true` => warning + override to `false`
- Syntactic regex validation for schema names in raw
- Options uniqueness/empties are semantic warnings in domain construction

## Constraints & Standards
- Obey context isolation rules
- Follow naming taxonomy (`get_` prefixes avoided, etc.)
- Avoid string allocation anti-patterns
- Use `Box<str>` for owned, immutable strings
- No `unwrap` in production code

## Current Pain Points
- `flatten` prevents `deny_unknown_fields` for strict schema alignment
- `RawPropertySpec` tag handling mismatched with schema (`bool` vs `boolean`)
- Property bank forbids `required`, but raw does not enforce
- Options constraints are specified in schema but not enforced (by design)

## Target Architecture

### New Raw DTO Types (schema-level)
Create explicit per-type DTOs with strict field sets:

- `RawPropertyBoolean`
- `RawPropertyString`
- `RawPropertyNumber`
- `RawPropertyDate`
- `RawPropertyFile`

Common fields:
- `required: bool` (schema only)
- `multi: bool`

Type-specific fields:
- String: `options: Option<RawOptions>`, `pattern: Option<RawStringPattern>`
- Number: `min: Option<f64>`, `max: Option<f64>`, `step: Option<f64>`
- Date: `format: Option<Box<str>>`
- File: `directory: Option<Box<str>>`, `file_class: Option<SchemaName>`

All per-type DTOs:
- `#[serde(deny_unknown_fields)]`
- `#[serde(default)]` for `required` and `multi`

### Inline Property Enum
Replace `RawPropertySpec` + `flatten` with an internally tagged enum:

```rust
#[serde(tag = "type")]
enum RawPropertyInline {
    Boolean(RawPropertyBoolean),
    String(RawPropertyString),
    Number(RawPropertyNumber),
    Date(RawPropertyDate),
    File(RawPropertyFile),
}
```

Boolean compatibility:
- Accept both `"bool"` and `"boolean"` during deserialization
- Serialize as `"bool"`

### Property Bank Entries
Reuse the schema-level per-type DTOs for bank entries.

Policy:
- During domain construction, if `required == true`, emit warning and set to
  `false`.

### RawProperty Wrapper
Keep `RawProperty` as:

```rust
#[serde(untagged)]
enum RawProperty {
    Ref(RawPropertyRef),
    Inline(RawPropertyInline),
}
```

`RawPropertyRef`:
- Explicit override fields only (no flatten)
- `#[serde(deny_unknown_fields)]`
- `$ref` field uses `RawPropertyRefPath`

### Syntactic Validation in Raw
- `SchemaName` uses `serde(try_from = "String")` or custom `Deserialize`
- `file_class` and `extends` use `SchemaName`
- Keep filename match validation outside raw

## Early Semantic Warnings
Emit warnings as early as possible after raw parsing, during domain
construction:

- Property bank entries: `required: true` => warning, override to `false`
- Options:
  - Duplicate values in `Plain`
  - Empty `Plain` or empty `Ordered`

Warning mechanism:
- Use existing logging facility (prefer `tracing::warn!` if available)
- Ensure warnings include context (property name + file path if available)

## Implementation Steps

### Step 1: Define per-type DTOs
- Add new structs in `lithos-core/src/schema/raw/property.rs` or a new module
  under `raw/`.
- Apply `deny_unknown_fields` and defaults for `required`/`multi`.
- Update documentation and tests for each DTO.

### Step 2: Replace RawPropertySpec
- Remove `RawPropertySpec` or convert it to the tagged inline enum.
- Update all call sites using `RawPropertySpec`.
- Update serialization tests.

### Step 3: Update RawPropertyInline + RawProperty
- Convert `RawPropertyInline` to tagged enum over per-type DTOs.
- Keep `RawProperty` as `Ref | Inline` (untagged).
- Ensure ref variant is tried first and fails fast on missing `$ref`.

### Step 4: Make RawPropertyRef strict
- Replace `flatten` overrides with explicit fields in `RawPropertyRef`.
- Add `deny_unknown_fields`.
- Confirm overrides align with schema fields.

### Step 5: Apply syntactic regex validation
- Convert `RawFileSpec.file_class` to `Option<SchemaName>`.
- Ensure `extends` uses `SchemaName` (already does).
- Update tests accordingly.

### Step 6: Update bank entry ingestion
- Reuse schema-level DTOs for property bank entries.
- Update conversion path to warn + override `required` to false.

### Step 7: Add early semantic warnings
- In domain construction path (or earliest semantic conversion stage):
  - Detect bank `required` truthy
  - Detect duplicate/empty options
- Emit warnings and normalize as needed.

### Step 8: Tests
- Add or update tests for:
  - Unknown fields rejected for each per-type DTO
  - `bool` and `boolean` accepted, serialize as `bool`
  - `RawPropertyRef` rejects unknown fields
  - Bank `required: true` produces warning and sets `required = false`
  - Options warnings for duplicates/empties

### Step 9: Docs
- Update `raw/mod.rs` docs to describe per-type DTOs and strictness
- Document early semantic warnings in domain construction

## File Touchpoints (Expected)
- `lithos-core/src/schema/raw/property.rs`
- `lithos-core/src/schema/raw/spec_string.rs` (if RawOptions types move)
- `lithos-core/src/schema/raw/spec_number.rs`
- `lithos-core/src/schema/raw/spec_date.rs`
- `lithos-core/src/schema/raw/spec_file.rs`
- `lithos-core/src/schema/raw/spec_bool.rs`
- `lithos-core/src/schema/raw/mod.rs`
- Domain construction path for schema/property bank ingestion
- Tests for raw and domain validation

## Risks & Mitigations
- **Risk:** Breaking compatibility for existing files with unknown fields
  - Mitigation: Add clear error messages; update docs
- **Risk:** Ambiguity around `bool` vs `boolean`
  - Mitigation: Accept both, serialize as `bool`
- **Risk:** Early warnings lack context
  - Mitigation: Thread file path/property name into warning calls

## Verification
- `mise run fmt`
- `mise run lint`
- `mise run test:unit:schema`
- `mise run verify` (if time)
