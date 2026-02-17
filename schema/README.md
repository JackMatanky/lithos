# Lithos Schema Format

This directory contains the JSON Schema meta-schema files used by Lithos to validate vault schema files at ingestion time.

## Meta-Schema Files

| File                        | Validates                                                           |
| --------------------------- | ------------------------------------------------------------------- |
| `property-bank.schema.json` | `property_bank.json` in a vault's `.lithos/schemas/` directory      |
| `note-metadata.schema.json` | Every other `*.json` file in a vault's `.lithos/schemas/` directory |

---

## Vault Schema Files

In a vault, schemas live at `.lithos/schemas/` and consist of two kinds of files:

```
my-vault/
└── .lithos/
    └── schemas/
        ├── property_bank.json   ← reusable property definitions
        ├── task.json            ← note schema
        ├── task_project.json    ← note schema (extends task)
        └── ...
```

---

## Property Bank

`property_bank.json` is a dictionary of reusable property definitions. Schemas reference these with `$ref` instead of duplicating the definition everywhere.

### Structure

```json
{
  "properties": {
    "<property_name>": <PropertyDefinition>
  }
}
```

The property bank has **no** `name`, `extends`, or `excludes` fields — it is not itself a schema.

### Example

```json
{
  "properties": {
    "date_iso_8601": {
      "required": false,
      "array": false,
      "type": "date",
      "format": "2006-01-02"
    },
    "task_status": {
      "required": false,
      "array": false,
      "type": "string",
      "enum": [
        { "1": "to_do" },
        { "2": "in_progress" },
        { "3": "done" },
        { "4": "on_hold" },
        { "5": "schedule" },
        { "6": "discarded" }
      ]
    },
    "contact": {
      "required": false,
      "array": true,
      "type": "file",
      "directory": "51_contacts/"
    }
  }
}
```

---

## Note Schemas

Each note schema file defines the frontmatter properties expected on a type of markdown note.

### Structure

```json
{
  "name": "<schema_name>",
  "extends": "<parent_schema_name>",
  "excludes": ["<inherited_field_to_remove>"],
  "properties": {
    "<field_name>": <Property>
  }
}
```

- `name` — required. Unique identifier, lowercase with underscores.
- `properties` — required. The properties this schema defines or overrides.
- `extends` — optional. Inherit all properties from another schema.
- `excludes` — optional. List of inherited property names to drop. Only valid with `extends`.

### Example

```json
{
  "name": "task_project",
  "properties": {
    "type": {
      "required": false,
      "array": false,
      "type": "string",
      "enum": ["project"]
    }
  },
  "extends": "task",
  "excludes": ["date", "project", "parent_task"]
}
```

---

## Property Definitions

Every property is either an **inline definition** or a **property bank reference**.

### Property Bank Reference

```json
"status": { "$ref": "property_bank#/task_status" }
```

The format is `property_bank#/<name>` where `<name>` is a key in the vault's `property_bank.json`. A `$ref` entry must have no other fields.

### Inline Definition

All inline definitions share two common fields:

| Field      | Type    | Default | Description                                                        |
| ---------- | ------- | ------- | ------------------------------------------------------------------ |
| `type`     | string  | —       | **Required.** One of `string`, `number`, `boolean`, `date`, `file` |
| `required` | boolean | `false` | Whether the field must be present in every note                    |
| `array`    | boolean | `false` | Whether the field holds multiple values                            |

Additional fields depend on `type`.

---

## Property Types

### `string`

Holds a text value. Optional constraints:

| Field     | Description                                                             |
| --------- | ----------------------------------------------------------------------- |
| `enum`    | Restrict to a fixed set of values — see [Enum Modes](#enum-modes) below |
| `pattern` | Regular expression the value must match                                 |

```json
"context": {
  "required": false,
  "array": false,
  "type": "string",
  "enum": ["education", "personal", "professional", "work"]
}
```

### `number`

Holds an integer or float value. Optional constraints:

| Field  | Description                                                             |
| ------ | ----------------------------------------------------------------------- |
| `step` | Increment/decrement step for UI controls (e.g. `1.0` for whole numbers) |
| `min`  | Minimum allowed value (inclusive)                                       |
| `max`  | Maximum allowed value (inclusive)                                       |

```json
"edition": {
  "required": false,
  "array": false,
  "type": "number",
  "min": 1,
  "step": 1.0
}
```

### `boolean`

Holds a `true` or `false` value. No additional fields.

```json
"is_confidential": {
  "required": false,
  "array": false,
  "type": "boolean"
}
```

### `date`

Holds a date, time, or datetime value. Optional constraints:

| Field    | Description                                                 |
| -------- | ----------------------------------------------------------- |
| `format` | Display and parsing format using Go reference time notation |

Common formats:

| Format             | Example            | Use            |
| ------------------ | ------------------ | -------------- |
| `2006-01-02`       | `2025-10-29`       | ISO date       |
| `2006-01-02T15:04` | `2025-10-29T14:30` | Local datetime |
| `2006`             | `2025`             | Year only      |
| `15:04`            | `14:30`            | Time only      |

```json
"date_published": {
  "required": false,
  "array": false,
  "type": "date",
  "format": "2006-01-02"
}
```

### `file`

Links to one or more other notes in the vault. Optional constraints:

| Field        | Description                                                                                                    |
| ------------ | -------------------------------------------------------------------------------------------------------------- |
| `directory`  | Vault-relative path where valid link targets must reside. Supports alternation: `(41_personal\|42_education)/` |
| `file_class` | The schema name the linked note must use                                                                       |

```json
"parent_task": {
  "required": false,
  "array": true,
  "type": "file",
  "directory": "(41_personal|42_education|43_professional)/",
  "file_class": "task_parent"
}
```

---

## Enum Modes

The `enum` field on `string` properties supports three modes. All three use arrays, preserving order regardless of formatters.

### Mode 1 — Plain Value List

Use when display order is unimportant or alphabetical is acceptable.

```json
"enum": ["education", "personal", "professional", "work"]
```

### Mode 2 — Ordered Numeric Map

Use when the order of values has semantic meaning (workflow states, priority levels, etc.). Each entry is a single-key object where the key is a positive integer encoding display position.

```json
"enum": [
  {"1": "to_do"},
  {"2": "in_progress"},
  {"3": "done"},
  {"4": "on_hold"},
  {"5": "discarded"}
]
```

### Mode 3 — Value-to-Label Map

Use when the stored value (snake_case) differs from what should be shown to the user. Each entry is a single-key object where the key is the stored value and the value is the display label.

```json
"enum": [
  {"january": "January"},
  {"february": "February"},
  {"march": "March"},
  {"april": "April"},
  {"may": "May"},
  {"june": "June"},
  {"july": "July"},
  {"august": "August"},
  {"september": "September"},
  {"october": "October"},
  {"november": "November"},
  {"december": "December"}
]
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

**`property_bank.json`:**

```json
{
  "properties": {
    "title": {
      "required": false,
      "array": false,
      "type": "string"
    },
    "date_iso_8601": {
      "required": false,
      "array": false,
      "type": "date",
      "format": "2006-01-02"
    },
    "organization": {
      "required": false,
      "array": true,
      "type": "file",
      "directory": "(52_organizations)/"
    }
  }
}
```

**`contact.json`:**

```json
{
  "name": "contact",
  "properties": {
    "name_last": {
      "required": false,
      "array": false,
      "type": "string"
    },
    "name_first": {
      "required": false,
      "array": false,
      "type": "string"
    },
    "date_birth": {
      "$ref": "property_bank#/date_iso_8601"
    },
    "organization": {
      "$ref": "property_bank#/organization"
    },
    "gender": {
      "required": false,
      "array": false,
      "type": "string",
      "enum": ["female", "male", "other"]
    }
  }
}
```
