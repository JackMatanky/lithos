# Data Model Relationships Diagram

**Legend:**

- 🔵 Domain Core models
- 🟢 SPI Adapter models
- 🟡 Split models (domain + adapter)
- 🔶 Shared Internal Package types
- → Dependency/relationship

```
[SPI Adapter Layer]
Config 🟢
  └─> Contains vault paths, directory locations, log levels

[Domain Core Layer]
File 🔵
  ├─> Path: string (absolute path, primary key)
  ├─> Basename: string (computed from Path)
  ├─> Folder: string (computed from Path)
  └─> ModTime: time.Time (from os.Stat())

Frontmatter 🔵
  ├─> FileClass: string (schema reference)
  └─> Fields: map[string]interface{} (actual data values)

Note 🔵
  ├─> File: File (embedded struct)
  └─> Frontmatter: Frontmatter (embedded struct)

Schema 🔵
  ├─> Name: string (schema identifier)
  ├─> Extends: string (parent schema name, optional)
  ├─> Excludes: []string (properties to remove from parent)
  ├─> Properties: []Property (declared properties)
  └─> ResolvedProperties: []Property (computed after inheritance)

Property 🔵
  ├─> Name, Required, Array (metadata)
  ├─> May reference PropertyBank via $ref
  └─> Spec: PropertySpec (interface)
        ├─> StringPropertySpec (Enum, Pattern)
        ├─> NumberPropertySpec (Min, Max, Step)
        ├─> DatePropertySpec (Format)
        ├─> FilePropertySpec (FileClass, Directory)
        └─> BoolPropertySpec (no config)

PropertyBank 🔵
  ├─> Properties: map[string]Property (reusable property library)
  ├─> Location: string (path to _properties/ directory)
  └─> Loaded before schemas, referenced via JSON $ref

Template 🔵
  ├─> FilePath: string (identifier)
  ├─> Name: string (computed from FilePath)
  ├─> Content: string (raw template text)
  └─> Parsed: *template.Template (cached AST, optional)

[Shared Internal Package Types]
Pattern 🔶
  ├─> Discriminated union: string | *regexp.Regexp
  ├─> Constructors: NewPatternExact(), NewPatternRegex()
  ├─> Type guards: IsExact(), IsRegex()
  └─> Accessors: String(), Regex()

Relationships:
  Note composes File + Frontmatter (3-model pattern)
  Schema extends Schema (inheritance chain)
  Schema contains Properties (composition)
  Property references PropertyBank entries (via $ref)
  Property contains PropertySpec (interface polymorphism)
  FilePropertySpec references FileClass/Directory (filtering constraints)
  Note validated against Schema (via Frontmatter.FileClass lookup)
  Frontmatter.Fields contains actual data, Schema.Properties defines validation rules
```
