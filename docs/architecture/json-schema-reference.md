# JSON Schema Reference

## Overview

The Lithos JSON Schema provides formal validation and documentation for all domain models in the system. It serves as:

1. **Reference for development** - Precise specification of data structures
2. **User-facing documentation** - Clear description of all models and their constraints
3. **Validation reference** - Formal rules for schema file validation
4. **Integration with development tooling** - Support for IDE validation, code generation, and API documentation

## Schema Location

The comprehensive domain schema is located at:
```
schemas/lithos-domain-schema.json
```

This file contains definitions for all core domain models including Property, PropertySpec variants, Schema, PropertyBank, Note, Template, and Config entities.

## Domain Model Definitions

### Core Entities

#### Note
The central business entity representing a markdown note. Combines identity (NoteID) with content metadata (Frontmatter).

```json
{
  "id": "contact-john-doe",
  "frontmatter": {
    "fileClass": "contact",
    "fields": {
      "title": "John Doe",
      "email": "john@example.com",
      "tags": ["colleague", "developer"]
    }
  }
}
```

#### Schema
Defines metadata structure with property constraints and inheritance for validating notes of a given fileClass.

```json
{
  "name": "contact",
  "extends": "base-note",
  "excludes": ["internal_id"],
  "properties": [
    {"$ref": "#/properties/standard_title"},
    {
      "id": "abc123...",
      "name": "email",
      "required": true,
      "array": false,
      "spec": {
        "type": "string",
        "pattern": "^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$"
      }
    }
  ]
}
```

#### Property
DDD entity defining a single metadata field with validation constraints and deterministic identity.

Properties have:
- **ID**: SHA-256 hash of (name + spec content) for deterministic identity
- **Name**: Field identifier matching frontmatter key
- **Required**: Whether field must be present
- **Array**: Whether field accepts multiple values
- **Spec**: Type-specific validation constraints

#### PropertyBank
Singleton registry of reusable Property definitions that schemas can reference via `$ref` syntax.

```json
{
  "properties": {
    "standard_title": {
      "id": "abc123...",
      "name": "title",
      "required": true,
      "array": false,
      "spec": {
        "type": "string",
        "pattern": "^.{1,200}$"
      }
    }
  }
}
```

### PropertySpec Variants

The schema defines five PropertySpec types for validation constraints:

#### StringSpec
String validation with enum lists and regex patterns:
```json
{
  "type": "string",
  "enum": ["red", "green", "blue"],
  "pattern": "^[A-Z][a-z]+$"
}
```

#### NumberSpec
Numeric validation with min/max bounds and step increments:
```json
{
  "type": "number",
  "min": 0,
  "max": 100,
  "step": 1
}
```

#### BoolSpec
Boolean validation (marker type with no additional constraints):
```json
{
  "type": "bool"
}
```

#### DateSpec
Date/time format validation using Go time layout strings:
```json
{
  "type": "date",
  "format": "2006-01-02"
}
```

#### FileSpec
File reference validation with fileClass and directory filters:
```json
{
  "type": "file",
  "file_class": "project",
  "directory": "work/"
}
```

## Schema Validation Rules

### Property Validation
- Property names must be non-empty and unique within schemas
- Property IDs must be SHA-256 hashes (64-character hex strings)
- Exactly one PropertySpec variant must be provided
- PropertySpec constraints must be structurally valid (e.g., valid regex patterns)

### Schema Inheritance
- Schema names must be unique and non-empty
- `excludes` can only be set when `extends` is not empty
- Property names must be unique within each schema
- Inheritance resolution happens at startup (not runtime)

### PropertyBank References
- `$ref` must follow JSON pointer format: `#/properties/{property-name}`
- Referenced properties must exist in PropertyBank
- Property names in PropertyBank must match pattern `^[a-zA-Z0-9_-]+$`

### Type-Specific Constraints
- **StringSpec**: Pattern must be valid regex
- **NumberSpec**: min ≤ max (if both specified), step > 0 (if specified)
- **DateSpec**: Format must contain valid Go time layout tokens
- **FileSpec**: fileClass and directory patterns must be valid regex

## Usage Examples

### Schema Definition File
```json
{
  "name": "meeting_note",
  "extends": "base-note",
  "excludes": ["tags"],
  "properties": [
    {"$ref": "#/properties/standard_title"},
    {"$ref": "#/properties/standard_created"},
    {
      "id": "def456...",
      "name": "attendees",
      "required": true,
      "array": true,
      "spec": {
        "type": "file",
        "file_class": "contact"
      }
    }
  ]
}
```

### PropertyBank Definition
```json
{
  "properties": {
    "standard_title": {
      "id": "abc123...",
      "name": "title",
      "required": true,
      "array": false,
      "spec": {
        "type": "string",
        "pattern": "^.{1,200}$"
      }
    },
    "iso_date": {
      "id": "def456...",
      "name": "created",
      "required": false,
      "array": false,
      "spec": {
        "type": "date",
        "format": "2006-01-02"
      }
    }
  }
}
```

### Note Frontmatter Validation
```yaml
# Valid note frontmatter for meeting_note schema
---
fileClass: meeting_note
title: "Weekly Team Sync"
created: "2025-01-12"
attendees:
  - "[[John Doe]]"
  - "[[Jane Smith]]"
---
```

## Development Workflow Integration

### IDE Integration
The JSON schema enables:
- **Syntax highlighting** and validation in JSON/YAML editors
- **Auto-completion** for property names and values
- **Inline documentation** showing property descriptions
- **Error detection** for invalid schema definitions

### Validation Commands
```bash
# Validate schema file against JSON schema
lithos schema validate schemas/contact.json

# Validate property bank against schema
lithos schema validate schemas/property_bank.json

# Validate all schemas in directory
lithos schema validate-all schemas/
```

### Code Generation
The schema supports:
- **Go struct generation** from schema definitions
- **TypeScript interface generation** for web interfaces
- **API documentation generation** using OpenAPI integration
- **Test data generation** for automated testing

## Schema Evolution

### Versioning Strategy
- Schema file includes `version` field for tracking changes
- Breaking changes require version increment
- Backward compatibility maintained within major versions

### Migration Support
```bash
# Migrate schemas to new version
lithos schema migrate --from=1.0.0 --to=1.1.0 schemas/

# Validate migration compatibility
lithos schema check-migration schemas/
```

### Extension Points
The schema supports:
- **Custom PropertySpec types** via additional oneOf variants
- **Schema metadata** via additional properties
- **Validation hooks** for complex cross-field validation
- **Custom formatters** for specialized data types

## Integration with External Tools

### JSON Schema Ecosystem
The schema is compatible with:
- [JSON Schema specification](http://json-schema.org/draft-07/schema#) (Draft 7)
- [Ajv validator](https://ajv.js.org/) for JavaScript/Node.js
- [jsonschema library](https://python-jsonschema.readthedocs.io/) for Python
- [Everit JSON Schema](https://github.com/everit-org/json-schema) for Java

### API Documentation
Integration with OpenAPI:
```yaml
# OpenAPI schema reference
components:
  schemas:
    LithosNote:
      $ref: 'schemas/lithos-domain-schema.json#/definitions/Note'
    LithosSchema:
      $ref: 'schemas/lithos-domain-schema.json#/definitions/Schema'
```

### Testing Integration
```go
// Go validation example
import "github.com/xeipuuv/gojsonschema"

func validateSchema(schemaData []byte) error {
    schemaLoader := gojsonschema.NewStringLoader(string(schemaData))
    documentLoader := gojsonschema.NewReferenceLoader("file://schemas/lithos-domain-schema.json")

    result, err := gojsonschema.Validate(documentLoader, schemaLoader)
    if err != nil {
        return err
    }

    if !result.Valid() {
        return fmt.Errorf("validation errors: %v", result.Errors())
    }

    return nil
}
```

## Best Practices

### Schema Design
1. **Use PropertyBank** for common properties to reduce duplication
2. **Design inheritance hierarchies** carefully to avoid deep nesting
3. **Validate patterns** thoroughly before deployment
4. **Document custom properties** with clear descriptions
5. **Test edge cases** with comprehensive validation suites

### Performance Considerations
1. **Pre-compile schemas** at application startup for faster validation
2. **Cache resolved schemas** to avoid inheritance resolution overhead
3. **Use indexes** for large PropertyBank collections
4. **Batch validations** for bulk operations

### Error Handling
1. **Provide clear error messages** with remediation suggestions
2. **Include context** about which schema/property failed validation
3. **Log validation errors** for debugging and monitoring
4. **Fail fast** on structural schema errors during startup

## Future Enhancements

### Planned Features
- **Schema composition** beyond simple inheritance
- **Conditional validation** based on property values
- **Custom validation functions** for complex business rules
- **Schema versioning** with automatic migration support
- **Performance profiling** for validation bottlenecks

### Integration Roadmap
- **LSP support** for real-time validation in editors
- **Web UI** for visual schema editing
- **CLI scaffolding** for new schema creation
- **Export formats** (TypeScript, GraphQL, Protobuf)
- **Documentation generation** from schema annotations
