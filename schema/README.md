# Lithos Schema Format

This directory contains the JSON Schema meta-schema files used by Lithos to validate vault schema files at ingestion time.

## Meta-Schema Files

| File | Validates |
|------|-----------|
| `property-bank.schema.json` | `property_bank.{json,toml,yaml}` in a vault's `.lithos/schemas/` directory |
| `note-metadata.schema.json` | Every other schema file in a vault's `.lithos/schemas/` directory |

---

## Vault Schema Files

In a vault, schemas live at `.lithos/schemas/` and can be written in **JSON**, **TOML**, or **YAML**. All three formats are equivalent — use whichever you prefer.

```
my-vault/
└── .lithos/
    └── schemas/
        ├── property_bank.json    ← reusable property definitions
        ├── task.toml             ← note schema (TOML)
        ├── task_project.yaml     ← note schema (YAML, extends task)
        └── ...
```

---

## Property Bank

The property bank is a dictionary of reusable property definitions. Schemas reference these with `$ref` instead of duplicating definitions everywhere.

The property bank has **no** `name`, `extends`, or `excludes` fields — it is not itself a schema.

### Structure

**JSON**
```json
{
  "properties": {
    "<property_name>": { }
  }
}
```

**TOML**
```toml
[properties.<property_name>]
```

**YAML**
```yaml
properties:
  <property_name>:
```

### Example

**JSON**
```json
{
  "properties": {
    "date_iso_8601": {
      "required": false,
      "multi": false,
      "type": "date",
      "format": "%Y-%m-%d"
    },
    "task_status": {
      "required": false,
      "multi": false,
      "type": "string",
      "options": {
        "1": "to_do",
        "2": "in_progress",
        "3": "done",
        "4": "on_hold",
        "5": "discarded"
      }
    },
    "contact": {
      "required": false,
      "multi": true,
      "type": "file",
      "directory": "51_contacts/"
    }
  }
}
```

**TOML**
```toml
[properties.date_iso_8601]
required = false
multi = false
type = "date"
format = "%Y-%m-%d"

[properties.task_status]
required = false
multi = false
type = "string"

[properties.task_status.options]
"1" = "to_do"
"2" = "in_progress"
"3" = "done"
"4" = "on_hold"
"5" = "discarded"

[properties.contact]
required = false
multi = true
type = "file"
directory = "51_contacts/"
```

**YAML**
```yaml
properties:
  date_iso_8601:
    required: false
    multi: false
    type: date
    format: "%Y-%m-%d"
  task_status:
    required: false
    multi: false
    type: string
    options:
      "1": to_do
      "2": in_progress
      "3": done
      "4": on_hold
      "5": discarded
  contact:
    required: false
    multi: true
    type: file
    directory: "51_contacts/"
```

---

## Note Schemas

Each note schema file defines the frontmatter properties expected on a type of markdown note.

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Unique identifier, lowercase with underscores |
| `properties` | yes | The properties this schema defines or overrides |
| `extends` | no | Name of a parent schema to inherit from |
| `excludes` | no | Inherited property names to drop. Only valid with `extends` |

### Structure

**JSON**
```json
{
  "name": "<schema_name>",
  "extends": "<parent_schema_name>",
  "excludes": ["<field_to_remove>"],
  "properties": {
    "<field_name>": { }
  }
}
```

**TOML**
```toml
name = "<schema_name>"
extends = "<parent_schema_name>"
excludes = ["<field_to_remove>"]

[properties.<field_name>]
```

**YAML**
```yaml
name: <schema_name>
extends: <parent_schema_name>
excludes:
  - <field_to_remove>
properties:
  <field_name>:
```

### Example

**JSON**
```json
{
  "name": "task_project",
  "extends": "task",
  "excludes": ["date", "project", "parent_task"],
  "properties": {
    "type": {
      "required": false,
      "multi": false,
      "type": "string",
      "options": ["project"]
    }
  }
}
```

**TOML**
```toml
name = "task_project"
extends = "task"
excludes = ["date", "project", "parent_task"]

[properties.type]
required = false
multi = false
type = "string"
options = ["project"]
```

**YAML**
```yaml
name: task_project
extends: task
excludes:
  - date
  - project
  - parent_task
properties:
  type:
    required: false
    multi: false
    type: string
    options:
      - project
```

---

## Property Definitions

Every property is either an **inline definition** or a **property bank reference**.

### Property Bank Reference

The format is `property_bank#/<name>` where `<name>` is a key in the vault's property bank. A `$ref` entry must have no other fields.

**JSON**
```json
"status": { "$ref": "property_bank#/task_status" }
```

**TOML**
```toml
[properties.status]
"$ref" = "property_bank#/task_status"
```

**YAML**
```yaml
properties:
  status:
    $ref: "property_bank#/task_status"
```

### Inline Definition

All inline definitions share two common fields plus a required `type`:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `type` | string | — | **Required.** One of `string`, `number`, `boolean`, `date`, `file` |
| `required` | boolean | `false` | Whether the field must be present in every note |
| `multi` | boolean | `false` | Whether the field holds multiple values |

Additional fields depend on `type`.

---

## Property Types

### `string`

Holds a text value.

| Field | Description |
|-------|-------------|
| `options` | Restrict to a fixed set of values — see [Options Modes](#options-modes) |
| `pattern` | Regular expression the value must match |

**JSON**
```json
"context": {
  "required": false,
  "multi": false,
  "type": "string",
  "options": ["education", "personal", "professional", "work"]
}
```

**TOML**
```toml
[properties.context]
required = false
multi = false
type = "string"
options = ["education", "personal", "professional", "work"]
```

**YAML**
```yaml
properties:
  context:
    required: false
    multi: false
    type: string
    options:
      - education
      - personal
      - professional
      - work
```

---

### `number`

Holds an integer or float value.

| Field | Description |
|-------|-------------|
| `step` | Increment/decrement step for UI controls (e.g. `1.0` for whole numbers) |
| `min` | Minimum allowed value (inclusive) |
| `max` | Maximum allowed value (inclusive) |

**JSON**
```json
"edition": {
  "required": false,
  "multi": false,
  "type": "number",
  "min": 1,
  "step": 1.0
}
```

**TOML**
```toml
[properties.edition]
required = false
multi = false
type = "number"
min = 1
step = 1.0
```

**YAML**
```yaml
properties:
  edition:
    required: false
    multi: false
    type: number
    min: 1
    step: 1.0
```

---

### `boolean`

Holds a `true` or `false` value. No additional fields.

**JSON**
```json
"is_confidential": {
  "required": false,
  "multi": false,
  "type": "boolean"
}
```

**TOML**
```toml
[properties.is_confidential]
required = false
multi = false
type = "boolean"
```

**YAML**
```yaml
properties:
  is_confidential:
    required: false
    multi: false
    type: boolean
```

---

### `date`

Holds a date, time, or datetime value. Format strings use **strftime notation** (standard across most programming languages).

| Field | Description |
|-------|-------------|
| `format` | Display and parsing format using strftime tokens |

| Format | Example | Use |
|--------|---------|-----|
| `%Y-%m-%d` | `2025-10-29` | ISO date |
| `%Y-%m-%dT%H:%M` | `2025-10-29T14:30` | Local datetime |
| `%Y` | `2025` | Year only |
| `%H:%M` | `14:30` | Time only |

**JSON**
```json
"date_published": {
  "required": false,
  "multi": false,
  "type": "date",
  "format": "%Y-%m-%d"
}
```

**TOML**
```toml
[properties.date_published]
required = false
multi = false
type = "date"
format = "%Y-%m-%d"
```

**YAML**
```yaml
properties:
  date_published:
    required: false
    multi: false
    type: date
    format: "%Y-%m-%d"
```

---

### `file`

Links to one or more other notes in the vault.

| Field | Description |
|-------|-------------|
| `directory` | Vault-relative path where link targets must reside. Supports alternation: `(folder_a\|folder_b)/` |
| `file_class` | Schema name the linked note must use |

**JSON**
```json
"parent_task": {
  "required": false,
  "multi": true,
  "type": "file",
  "directory": "(41_personal|42_education|43_professional)/",
  "file_class": "task_parent"
}
```

**TOML**
```toml
[properties.parent_task]
required = false
multi = true
type = "file"
directory = "(41_personal|42_education|43_professional)/"
file_class = "task_parent"
```

**YAML**
```yaml
properties:
  parent_task:
    required: false
    multi: true
    type: file
    directory: "(41_personal|42_education|43_professional)/"
    file_class: task_parent
```

---

## Options Modes

The `options` field on `string` properties supports three modes for different use cases.

### Mode 1 — Plain Value List

Use when display order is unimportant or alphabetical is acceptable.

**JSON**
```json
"options": ["education", "personal", "professional", "work"]
```

**TOML**
```toml
options = ["education", "personal", "professional", "work"]
```

**YAML**
```yaml
options:
  - education
  - personal
  - professional
  - work
```

---

### Mode 2 — Ordered Value Map

Use when the order of values has semantic meaning (workflow states, priority levels, steps in a cycle). Keys are **1-indexed** position integers. The order of keys in the file does not matter — the integer keys define display order.

**JSON**
```json
"options": {
  "1": "to_do",
  "2": "in_progress",
  "3": "done",
  "4": "on_hold",
  "5": "discarded"
}
```

**TOML**
```toml
[properties.status.options]
"1" = "to_do"
"2" = "in_progress"
"3" = "done"
"4" = "on_hold"
"5" = "discarded"
```

**YAML**
```yaml
options:
  "1": to_do
  "2": in_progress
  "3": done
  "4": on_hold
  "5": discarded
```

---

### Mode 3 — Value with Labels

Use when the stored value differs from the display label. Each item has:
- `value` (required) — the value stored in frontmatter
- `label` (optional) — human-readable display text
- `order` (optional) — explicit display position (useful for sparse numbering like 10, 20, 30)

If `order` is omitted, array position defines display order. If `label` is omitted, `value` is used for display.

**JSON**
```json
"options": [
  {"value": "january", "label": "January"},
  {"value": "february", "label": "February"},
  {"value": "march", "label": "March"}
]
```

**TOML**
```toml
[[properties.month_name.options]]
value = "january"
label = "January"

[[properties.month_name.options]]
value = "february"
label = "February"

[[properties.month_name.options]]
value = "march"
label = "March"
```

**YAML**
```yaml
options:
  - value: january
    label: January
  - value: february
    label: February
  - value: march
    label: March
```

With explicit order:

**JSON**
```json
"options": [
  {"value": "low", "label": "Low Priority", "order": 10},
  {"value": "medium", "label": "Medium Priority", "order": 20},
  {"value": "high", "label": "High Priority", "order": 30}
]
```

**TOML**
```toml
[[properties.priority.options]]
value = "low"
label = "Low Priority"
order = 10

[[properties.priority.options]]
value = "medium"
label = "Medium Priority"
order = 20

[[properties.priority.options]]
value = "high"
label = "High Priority"
order = 30
```

**YAML**
```yaml
options:
  - value: low
    label: Low Priority
    order: 10
  - value: medium
    label: Medium Priority
    order: 20
  - value: high
    label: High Priority
    order: 30
```

---

## Inheritance

Schemas can inherit from a parent schema using `extends`. The resolved property set of a child schema is:

```
parent properties − excludes + child properties
```

Child properties override parent properties of the same name.

```
task
  ├── task_project   (extends task, excludes: date, project, parent_task)
  ├── task_parent    (extends task, excludes: date, parent_task)
  ├── task_meeting   (extends task)
  └── task_child     (extends task, excludes: date_start, date_end)
```

Lithos detects circular inheritance and reports an error at ingestion time.

---

## Complete Example

A contact schema with a property bank, base schema, and extended schema.

### Property Bank

**JSON (`property_bank.json`)**
```json
{
  "properties": {
    "title": {
      "required": false,
      "multi": false,
      "type": "string"
    },
    "date_iso_8601": {
      "required": false,
      "multi": false,
      "type": "date",
      "format": "%Y-%m-%d"
    },
    "organization": {
      "required": false,
      "multi": true,
      "type": "file",
      "directory": "(52_organizations)/"
    }
  }
}
```

**TOML (`property_bank.toml`)**
```toml
[properties.title]
required = false
multi = false
type = "string"

[properties.date_iso_8601]
required = false
multi = false
type = "date"
format = "%Y-%m-%d"

[properties.organization]
required = false
multi = true
type = "file"
directory = "(52_organizations)/"
```

**YAML (`property_bank.yaml`)**
```yaml
properties:
  title:
    required: false
    multi: false
    type: string
  date_iso_8601:
    required: false
    multi: false
    type: date
    format: "%Y-%m-%d"
  organization:
    required: false
    multi: true
    type: file
    directory: "(52_organizations)/"
```

---

### Base Schema: `dir`

**JSON (`dir.json`)**
```json
{
  "name": "dir",
  "properties": {
    "title": { "$ref": "property_bank#/title" },
    "country": {
      "required": false,
      "multi": true,
      "type": "string"
    },
    "url": {
      "required": false,
      "multi": false,
      "type": "string"
    }
  }
}
```

**TOML (`dir.toml`)**
```toml
name = "dir"

[properties.title]
"$ref" = "property_bank#/title"

[properties.country]
required = false
multi = true
type = "string"

[properties.url]
required = false
multi = false
type = "string"
```

**YAML (`dir.yaml`)**
```yaml
name: dir
properties:
  title:
    $ref: "property_bank#/title"
  country:
    required: false
    multi: true
    type: string
  url:
    required: false
    multi: false
    type: string
```

---

### Extended Schema: `dir_contact`

**JSON (`dir_contact.json`)**
```json
{
  "name": "dir_contact",
  "extends": "dir",
  "properties": {
    "name_last": {
      "required": false,
      "multi": false,
      "type": "string"
    },
    "name_first": {
      "required": false,
      "multi": false,
      "type": "string"
    },
    "date_birth": { "$ref": "property_bank#/date_iso_8601" },
    "organization": { "$ref": "property_bank#/organization" },
    "gender": {
      "required": false,
      "multi": false,
      "type": "string",
      "options": ["female", "male", "other"]
    }
  }
}
```

**TOML (`dir_contact.toml`)**
```toml
name = "dir_contact"
extends = "dir"

[properties.name_last]
required = false
multi = false
type = "string"

[properties.name_first]
required = false
multi = false
type = "string"

[properties.date_birth]
"$ref" = "property_bank#/date_iso_8601"

[properties.organization]
"$ref" = "property_bank#/organization"

[properties.gender]
required = false
multi = false
type = "string"
options = ["female", "male", "other"]
```

**YAML (`dir_contact.yaml`)**
```yaml
name: dir_contact
extends: dir
properties:
  name_last:
    required: false
    multi: false
    type: string
  name_first:
    required: false
    multi: false
    type: string
  date_birth:
    $ref: "property_bank#/date_iso_8601"
  organization:
    $ref: "property_bank#/organization"
  gender:
    required: false
    multi: false
    type: string
    options:
      - female
      - male
      - other
```
