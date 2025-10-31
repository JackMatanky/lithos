# Lithos Schemas Directory

This directory contains JSON schema definitions and validation files for the Lithos note-taking system.

## Directory Structure

```
schemas/
├── README.md                     # This file - usage guide
├── lithos-domain-schema.json     # Main JSON schema for all domain models
├── property_bank.json            # PropertyBank instance (if used)
├── examples/                     # Example schema and data files
│   ├── contact-schema-example.json      # Example contact schema definition
│   ├── property-bank-example.json       # Example PropertyBank definition
│   └── note-example.json                # Example note data structure
└── user-schemas/                 # User-defined schemas (gitignored)
    ├── contact.json
    ├── project.json
    └── meeting_note.json
```

## Main Schema File

**`lithos-domain-schema.json`** - Comprehensive JSON schema defining all Lithos domain models:

- **Property** - DDD entity with validation constraints and deterministic identity
- **PropertySpec variants** - StringSpec, NumberSpec, BoolSpec, DateSpec, FileSpec
- **Schema** - Metadata structure with inheritance and property constraints
- **PropertyBank** - Singleton registry of reusable property definitions
- **Note** - Core business entity representing markdown notes
- **Template** - Executable template for note generation
- **Config** - Application configuration value object

## Usage Examples

### Validating Schema Files

Use the JSON schema to validate your schema definitions:

```bash
# Validate a schema file against the domain schema
npx ajv-cli validate -s schemas/lithos-domain-schema.json -d schemas/user-schemas/contact.json

# Validate PropertyBank file
npx ajv-cli validate -s schemas/lithos-domain-schema.json -d schemas/property_bank.json

# Validate note data
npx ajv-cli validate -s schemas/lithos-domain-schema.json -d examples/note-example.json
```

### IDE Integration

Configure your editor to use the JSON schema for validation and auto-completion:

**VS Code** - Add to `.vscode/settings.json`:
```json
{
  "json.schemas": [
    {
      "fileMatch": ["schemas/**/*.json", "!schemas/lithos-domain-schema.json"],
      "url": "./schemas/lithos-domain-schema.json"
    }
  ]
}
```

**JetBrains IDEs** - Configure in Settings > Languages & Frameworks > Schemas and DTDs > JSON Schema Mappings

### Creating Schema Files

When creating new schema definitions, follow these patterns:

#### 1. Basic Schema Structure
```json
{
  "name": "my_schema",
  "properties": [
    {
      "id": "generated_hash_id_here",
      "name": "title",
      "required": true,
      "array": false,
      "spec": {
        "type": "string",
        "pattern": "^.{1,200}$"
      }
    }
  ]
}
```

#### 2. Schema with Inheritance
```json
{
  "name": "specialized_note",
  "extends": "base_note",
  "excludes": ["unwanted_field"],
  "properties": [
    {"$ref": "#/properties/standard_title"},
    {
      "id": "custom_property_hash",
      "name": "custom_field",
      "required": false,
      "array": false,
      "spec": {
        "type": "string",
        "enum": ["option1", "option2", "option3"]
      }
    }
  ]
}
```

#### 3. PropertyBank Definition
```json
{
  "properties": {
    "standard_title": {
      "id": "title_property_hash",
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

## PropertySpec Types

The schema supports five PropertySpec types for validation:

### StringSpec
```json
{
  "type": "string",
  "enum": ["option1", "option2"],        // Optional: allowed values
  "pattern": "^[A-Za-z0-9]+$"            // Optional: regex pattern
}
```

### NumberSpec
```json
{
  "type": "number",
  "min": 0,                              // Optional: minimum value
  "max": 100,                            // Optional: maximum value
  "step": 1                              // Optional: increment (1 = integers)
}
```

### BoolSpec
```json
{
  "type": "bool"                         // No additional properties
}
```

### DateSpec
```json
{
  "type": "date",
  "format": "2006-01-02"                 // Go time layout (defaults to RFC3339)
}
```

### FileSpec
```json
{
  "type": "file",
  "file_class": "contact",               // Optional: filter by fileClass
  "directory": "contacts/"               // Optional: filter by directory
}
```

## Validation Rules

### Property Validation
- Property IDs must be SHA-256 hashes (64-character hex strings)
- Property names must be non-empty and unique within schemas
- Exactly one PropertySpec variant required per property
- PropertySpec constraints must be structurally valid

### Schema Validation
- Schema names must be unique and non-empty
- `excludes` only allowed when `extends` is specified
- Property names must be unique within each schema
- All PropertyBank references must be valid

### Common Patterns

#### Email Validation
```json
{
  "type": "string",
  "pattern": "^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$"
}
```

#### Phone Number Validation
```json
{
  "type": "string",
  "pattern": "^\\+?[1-9]\\d{1,14}$"
}
```

#### Priority Levels
```json
{
  "type": "string",
  "enum": ["low", "medium", "high", "urgent"]
}
```

#### Percentage Values
```json
{
  "type": "number",
  "min": 0,
  "max": 100,
  "step": 1
}
```

#### ISO Dates
```json
{
  "type": "date",
  "format": "2006-01-02"
}
```

#### File References
```json
{
  "type": "file",
  "file_class": "contact",
  "directory": "people/"
}
```

## Development Workflow

### 1. Schema Creation
1. Create schema file in `schemas/user-schemas/`
2. Validate against `lithos-domain-schema.json`
3. Test with example note data
4. Add to version control

### 2. PropertyBank Updates
1. Add new common properties to `property_bank.json`
2. Generate deterministic IDs for new properties
3. Validate updated PropertyBank file
4. Update schemas to reference new properties

### 3. Testing Schema Changes
1. Create test note data matching new schema
2. Validate note data against domain schema
3. Test inheritance resolution
4. Verify PropertyBank references work correctly

## Troubleshooting

### Common Validation Errors

**"Property ID must be SHA-256 hash"**
- Ensure ID is exactly 64 characters of hex (0-9, a-f)
- Use consistent hash generation for same property definition

**"Pattern is not valid regex"**
- Test regex patterns in Go using `regexp.Compile()`
- Escape special characters properly (e.g., `\\w` not `\w`)

**"Property name cannot be empty"**
- Ensure all properties have non-empty `name` fields
- Property names are case-sensitive

**"Excludes can only be set when extends is not empty"**
- Only use `excludes` array when `extends` field is specified
- Remove `excludes` or add `extends` parent schema

**"Invalid PropertyBank reference"**
- Ensure `$ref` follows format: `#/properties/property_name`
- Verify referenced property exists in PropertyBank
- Check property name matches exactly (case-sensitive)

### Property ID Generation

Property IDs should be SHA-256 hashes of (name + spec content):

```go
// Example ID generation in Go
import (
    "crypto/sha256"
    "encoding/hex"
)

func generatePropertyID(name string, specType string) string {
    content := name + specType
    hash := sha256.Sum256([]byte(content))
    return hex.EncodeToString(hash[:])
}
```

For consistency, use the same generation method across all property definitions.

## Schema Evolution

When updating schemas:

1. **Backward Compatible Changes** - Add optional properties, extend enums
2. **Breaking Changes** - Remove properties, change types, make required
3. **Version Management** - Update schema version field
4. **Migration** - Provide migration path for existing notes

## Integration with Development Tools

### Code Generation
```bash
# Generate Go structs from JSON schema
go-jsonschema -p domain schemas/lithos-domain-schema.json > internal/domain/generated.go

# Generate TypeScript interfaces
json-schema-to-typescript schemas/lithos-domain-schema.json > types/domain.ts
```

### API Documentation
```yaml
# OpenAPI reference
components:
  schemas:
    Note:
      $ref: 'schemas/lithos-domain-schema.json#/definitions/Note'
```

### Testing
```go
// Go validation example
import "github.com/xeipuuv/gojsonschema"

func validateAgainstSchema(data []byte, schemaPath string) error {
    schemaLoader := gojsonschema.NewReferenceLoader("file://" + schemaPath)
    documentLoader := gojsonschema.NewBytesLoader(data)

    result, err := gojsonschema.Validate(schemaLoader, documentLoader)
    if err != nil {
        return err
    }

    if !result.Valid() {
        return fmt.Errorf("validation failed: %v", result.Errors())
    }

    return nil
}
```
