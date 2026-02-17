# Schema Documentation

This document provides comprehensive guidance for creating and using schemas in Lithos to define note structure and validation rules.

## What are Schemas?

Schemas in Lithos define the structure and validation rules for your notes. They ensure consistent frontmatter properties, automatic validation during note creation, and property reuse across your vault.

### Purpose

- **Consistency**: Ensure all notes of a type have the same frontmatter structure
- **Validation**: Automatically validate property types and constraints before note creation
- **Reuse**: Define properties once in a property bank and reference them across schemas
- **Inheritance**: Build complex schemas by extending simpler base schemas

### Benefits

- **DRY Principle**: Define properties once, use everywhere
- **Type Safety**: Automatic validation of property types and values
- **Error Prevention**: Catch configuration errors before note creation
- **Maintainability**: Centralized property definitions make updates easy

## Property Bank

The property bank is a single source of truth for reusable property definitions, stored in `schemas/property_bank.json`.

### Purpose

- Centralize common property definitions
- Enable property reuse across multiple schemas
- Maintain consistency in property types and constraints
- Simplify schema maintenance

### Location

`schemas/property_bank.json` (relative to vault root)

### Structure

Each property in the bank has an ID and a complete property specification:

```json
{
  "property_id": {
    "type": "string|number|boolean|date|file",
    "required": true|false,
    "default": "optional default value",
    "metadata": {
      "description": "Human-readable description"
    },
    "spec": {
      // Type-specific constraints
    }
  }
}
```

### Property Types and Constraints

#### String Properties

```json
{
  "title": {
    "type": "string",
    "required": true,
    "spec": {
      "regex": "^[A-Z][a-zA-Z0-9 ]*$",
      "enum": ["value1", "value2", "value3"]
    }
  }
}
```

#### Number Properties

```json
{
  "priority": {
    "type": "number",
    "required": false,
    "default": 1,
    "spec": {
      "min": 1,
      "max": 5,
      "step": 1
    }
  }
}
```

#### Boolean Properties

```json
{
  "completed": {
    "type": "boolean",
    "required": false,
    "default": false
  }
}
```

#### Date Properties

```json
{
  "created": {
    "type": "date",
    "required": true,
    "spec": {
      "format": "2006-01-02"
    }
  }
}
```

#### File Properties

```json
{
  "attachment": {
    "type": "file",
    "required": false,
    "spec": {
      "directory": "attachments/",
      "fileClass": "image"
    }
  }
}
```

### Example Property Bank

```json
{
  "standard_title": {
    "type": "string",
    "required": true,
    "metadata": {"description": "Standard title property"}
  },
  "standard_created": {
    "type": "date",
    "required": true,
    "metadata": {"description": "Creation timestamp"}
  },
  "email_address": {
    "type": "string",
    "required": true,
    "spec": {
      "regex": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
    },
    "metadata": {"description": "Email address with validation"}
  },
  "priority_level": {
    "type": "number",
    "required": false,
    "default": 3,
    "spec": {
      "min": 1,
      "max": 5
    },
    "metadata": {"description": "Priority level from 1-5"}
  }
}
```

## Schema Structure

Schemas are JSON files that define the structure and validation rules for notes.

### File Format

- JSON format (`.schema.json` extension recommended)
- Located in `schemas/` directory
- Named descriptively (e.g., `contact.schema.json`, `meeting.schema.json`)

### Required Fields

- `name`: Unique identifier for the schema
- `properties`: Array of property definitions

### Optional Fields

- `extends`: Name of parent schema to inherit from
- `excludes`: Array of property IDs to exclude from inherited schema

### Property Definitions

Properties can be defined inline or reference the property bank:

#### Inline Property Definition

```json
{
  "id": "custom_property",
  "type": "string",
  "required": true,
  "spec": {
    "regex": "^[A-Z].*"
  }
}
```

#### Property Bank Reference

```json
{
  "$ref": "property_bank_property_id"
}
```

## Inheritance

Schemas can inherit properties from parent schemas using the `extends` field.

### Basic Inheritance

```json
{
  "name": "contact",
  "extends": "base_note",
  "properties": [
    {"$ref": "email_address"}
  ]
}
```

### Excluding Inherited Properties

```json
{
  "name": "minimal_contact",
  "extends": "contact",
  "excludes": ["phone", "address"],
  "properties": [
    {"$ref": "emergency_contact"}
  ]
}
```

### Multi-level Inheritance

Schemas can extend schemas that themselves extend other schemas:

```
base_note (title, created)
  ↓
person (extends base_note + name, email)
  ↓
contact (extends person + phone, address)
```

### Cycle Detection

Lithos detects circular inheritance and reports an error:

```
Error: Circular inheritance detected in schema chain: contact -> person -> contact
Hint: Remove the circular reference by changing the extends field
```

## Property References

Use `$ref` to reference properties defined in the property bank.

### Benefits

- **DRY Principle**: Define once, use everywhere
- **Consistency**: Same property definition across schemas
- **Maintenance**: Update property in one place

### Syntax

```json
{
  "$ref": "property_id"
}
```

### Example

```json
{
  "name": "meeting",
  "properties": [
    {"$ref": "standard_title"},
    {"$ref": "standard_created"},
    {
      "id": "attendees",
      "type": "string",
      "required": false,
      "spec": {
        "regex": "^([^,]+,)*[^,]+$"
      }
    }
  ]
}
```

## Validation

Lithos automatically validates notes against their schema during creation.

### When Validation Occurs

- After template rendering
- Before note file is written
- If validation fails, note creation is aborted

### Validation Types

#### Structural Validation

- Required properties present
- Property types match schema
- Referenced properties exist in property bank

#### Constraint Validation

- Regex patterns match
- Numbers within min/max range
- Values in enum lists
- Files exist and meet criteria

#### Cross-schema Validation

- Inheritance chains are valid
- No circular references
- Parent schemas exist

### Error Messages

Validation errors include actionable remediation hints:

```
Error: Required property 'email' missing from frontmatter
Hint: Add email field to the frontmatter or mark as optional in schema

Error: Property 'priority' value 10 exceeds maximum 5
Hint: Change priority value to be between 1 and 5

Error: Property reference '$ref: "missing_prop"' not found in property bank
Hint: Add the missing property to schemas/property_bank.json or fix the reference
```

## Examples

### Complete Example 1: Simple Contact Schema

**Property Bank** (`schemas/property_bank.json`):

```json
{
  "standard_title": {
    "type": "string",
    "required": true,
    "metadata": {"description": "Standard title property"}
  },
  "standard_created": {
    "type": "date",
    "required": true,
    "metadata": {"description": "Creation timestamp"}
  }
}
```

**Schema** (`schemas/contact.schema.json`):

```json
{
  "name": "contact",
  "properties": [
    {"$ref": "standard_title"},
    {"$ref": "standard_created"},
    {
      "id": "email",
      "type": "string",
      "required": true,
      "spec": {
        "regex": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
      }
    },
    {
      "id": "phone",
      "type": "string",
      "required": false
    }
  ]
}
```

**Template** (`templates/contact.md`):

```markdown
---
schema: contact
title: {{ prompt "name" "Contact Name" "" }}
created: {{ now "2006-01-02" }}
email: {{ prompt "email" "Email Address" "" }}
phone: {{ prompt "phone" "Phone Number" "" }}
---

# {{ .Frontmatter.title }}

- **Email:** {{ .Frontmatter.email }}
- **Phone:** {{ .Frontmatter.phone }}
- **Created:** {{ .Frontmatter.created }}
```

### Complete Example 2: Inheritance with Base Schema

**Base Schema** (`schemas/base_note.schema.json`):

```json
{
  "name": "base_note",
  "properties": [
    {"$ref": "standard_title"},
    {"$ref": "standard_created"}
  ]
}
```

**Extended Schema** (`schemas/meeting.schema.json`):

```json
{
  "name": "meeting",
  "extends": "base_note",
  "properties": [
    {
      "id": "date",
      "type": "date",
      "required": true
    },
    {
      "id": "attendees",
      "type": "string",
      "required": false
    }
  ]
}
```

### Complete Example 3: Complex Schema with All Constraint Types

```json
{
  "name": "project",
  "properties": [
    {"$ref": "standard_title"},
    {"$ref": "standard_created"},
    {"$ref": "priority_level"},
    {
      "id": "status",
      "type": "string",
      "required": true,
      "spec": {
        "enum": ["planning", "active", "on-hold", "completed"]
      }
    },
    {
      "id": "budget",
      "type": "number",
      "required": false,
      "spec": {
        "min": 0,
        "max": 1000000,
        "step": 100
      }
    },
    {
      "id": "is_confidential",
      "type": "boolean",
      "required": false,
      "default": false
    },
    {
      "id": "deadline",
      "type": "date",
      "required": false
    },
    {
      "id": "documentation",
      "type": "file",
      "required": false,
      "spec": {
        "directory": "docs/",
        "fileClass": "document"
      }
    }
  ]
}
```

### Complete Example 4: Full Workflow from Schema to Note

1. **Setup vault structure:**
   ```
   my-vault/
   ├── schemas/
   │   ├── property_bank.json
   │   └── contact.schema.json
   └── templates/
       └── contact.md
   ```

2. **Create property bank:**
   ```json
   {
     "standard_title": {"type": "string", "required": true},
     "standard_created": {"type": "date", "required": true}
   }
   ```

3. **Create schema:**
   ```json
   {
     "name": "contact",
     "properties": [
       {"$ref": "standard_title"},
       {"$ref": "standard_created"},
       {"id": "email", "type": "string", "required": true}
     ]
   }
   ```

4. **Create template:**
   ```markdown
   ---
   schema: contact
   title: {{ prompt "name" "Name" "" }}
   created: {{ now "2006-01-02" }}
   email: {{ prompt "email" "Email" "" }}
   ---

   # {{ .Frontmatter.title }}
   Email: {{ .Frontmatter.email }}
   ```

5. **Generate note:**
   ```bash
   cd my-vault
   lithos new contact
   ```

6. **Resulting note** (`contact.md`):
   ```markdown
   ---
   title: John Doe
   created: 2025-10-29
   email: john@example.com
   ---

   # John Doe
   Email: john@example.com
   ```

The frontmatter is automatically validated against the schema during creation.
