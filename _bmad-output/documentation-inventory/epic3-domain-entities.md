# Epic 3 Domain Entity Inventory

## Note Bounded Context

| Entity | File | Purpose | Key Methods | Validation Rules |
| --- | --- | --- | --- | --- |
| Note | [crates/domain/src/note/aggregate.rs](crates/domain/src/note/aggregate.rs) | Aggregate root for Obsidian notes with subentities and domain events. | `new`, `add_link`, `add_heading`, `add_tag`, `add_task`, `add_section`, `validate` | Vault-relative path (non-empty, relative, .md extension, no traversal), link/embed targets must be non-empty, embeds cannot have anchors, external links cannot use block anchors. |
| Frontmatter | [crates/domain/src/note/frontmatter.rs](crates/domain/src/note/frontmatter.rs) | YAML metadata container and typed accessors. | `new`, `get`, `get_as`, `get_string_array`, `title`, `file_class`, `aliases` | Field coercion respects type constraints; relies on config keys for lookup. |
| FieldValue | [crates/domain/src/note/frontmatter.rs](crates/domain/src/note/frontmatter.rs) | Typed runtime representation of frontmatter values. | `as_*`, `is_*` accessors | Variant type checks enforce correct conversions. |
| FromFieldValue | [crates/domain/src/note/frontmatter.rs](crates/domain/src/note/frontmatter.rs) | Trait for typed extraction from `FieldValue`. | `from_value` | Type-specific conversion (string/bool/number/date/array). |
| Link | [crates/domain/src/note/link.rs](crates/domain/src/note/link.rs) | Link/Embed value object with style, anchors, and embed metadata. | `new_wikilink`, `new_markdown_link`, `new_embed`, `target`, `alias`, `anchor`, `style`, `embed_type` | Target path must be non-empty; embeds cannot have anchors; external links cannot use block anchors. |
| LinkType | [crates/domain/src/note/link.rs](crates/domain/src/note/link.rs) | Enum for link styles. | N/A | N/A |
| EmbedType | [crates/domain/src/note/link.rs](crates/domain/src/note/link.rs) | Enum for embed media types. | N/A | N/A |
| Tag | [crates/domain/src/note/tag.rs](crates/domain/src/note/tag.rs) | Hierarchical tag with segments. | `parse`, `segments`, `as_str` | Must start with `#`, no empty segments, segments match `^[a-zA-Z0-9_-]+$`. |
| Heading | [crates/domain/src/note/structure.rs](crates/domain/src/note/structure.rs) | Heading value object for document structure. | `new`, `level`, `text`, `position` | Level 1-6; text must be non-empty. |
| Section | [crates/domain/src/note/structure.rs](crates/domain/src/note/structure.rs) | Section value object for content ranges. | `new`, `range`, `heading`, `content` | None beyond data integrity; relies on caller to supply valid ranges. |
| Task | [crates/domain/src/note/task.rs](crates/domain/src/note/task.rs) | Task item with completion status. | `new`, `status`, `text`, `position` | Text must be non-empty. |
| TaskStatus | [crates/domain/src/note/task.rs](crates/domain/src/note/task.rs) | Enum for task status. | N/A | N/A |
| NoteCreated | [crates/domain/src/note/events.rs](crates/domain/src/note/events.rs) | Domain event for note creation. | `new` | N/A |
| FrontmatterValidated | [crates/domain/src/note/events.rs](crates/domain/src/note/events.rs) | Event for validated frontmatter (app-level emission). | `new` | N/A |
| NoteEvents | [crates/domain/src/note/events.rs](crates/domain/src/note/events.rs) | Event enum for note aggregate. | N/A | N/A |

## Schema Bounded Context

| Entity | File | Purpose | Key Methods | Validation Rules |
| --- | --- | --- | --- | --- |
| SchemaName | [crates/domain/src/schema/aggregate.rs](crates/domain/src/schema/aggregate.rs) | Validated schema name value object. | `new`, `as_str` | Non-empty, <= 64 chars, alphanumeric/underscore/dash. |
| Schema | [crates/domain/src/schema/aggregate.rs](crates/domain/src/schema/aggregate.rs) | Resolved schema aggregate with properties and events. | `new`, `get`, `has`, `properties`, `pending_events` | Schema name validated by `SchemaName`. |
| PropertyBank | [crates/domain/src/schema/aggregate.rs](crates/domain/src/schema/aggregate.rs) | Registry for reusable properties with dual indexing. | `register`, `get`, `decode`, `get_by_id`, `get_by_name` | Prevents duplicate names; property spec validation. |
| PropertyName | [crates/domain/src/schema/property.rs](crates/domain/src/schema/property.rs) | Validated property name value object. | `new`, `as_str` | Non-empty, <= 64 chars, alphanumeric/underscore/dash. |
| Property | [crates/domain/src/schema/property.rs](crates/domain/src/schema/property.rs) | Property definition with spec and array/required flags. | `new`, `validate`, `validate_value`, accessors | Spec validation via `PropertySpec::validate_spec`. |
| PropertySpecTrait | [crates/domain/src/schema/property_spec.rs](crates/domain/src/schema/property_spec.rs) | Trait for spec validation and typing. | `spec_type`, `validate`, `validate_spec` | Spec-specific validation. |
| PropertySpec | [crates/domain/src/schema/property_spec.rs](crates/domain/src/schema/property_spec.rs) | Sum type for property constraints. | `validate`, `validate_spec`, `spec_type` | Dispatch to spec variants. |
| StringSpec | [crates/domain/src/schema/property_spec.rs](crates/domain/src/schema/property_spec.rs) | String constraints (length, enum, regex). | `validate`, `validate_spec` | min/max length, enum membership, regex pattern. |
| NumberSpec | [crates/domain/src/schema/property_spec.rs](crates/domain/src/schema/property_spec.rs) | Number constraints (range, step). | `validate`, `validate_spec` | min/max bounds, step alignment. |
| BoolSpec | [crates/domain/src/schema/property_spec.rs](crates/domain/src/schema/property_spec.rs) | Boolean marker spec. | `validate`, `validate_spec` | Type enforcement only. |
| DateSpec | [crates/domain/src/schema/property_spec.rs](crates/domain/src/schema/property_spec.rs) | Date formatting constraints. | `validate`, `validate_spec` | Format string must be non-empty; values must match format. |
| FileSpec | [crates/domain/src/schema/property_spec.rs](crates/domain/src/schema/property_spec.rs) | File constraints. | `validate`, `validate_spec` | Directory restriction and non-empty file class. |
| RawSchema | [crates/domain/src/schema/raw.rs](crates/domain/src/schema/raw.rs) | Input schema definition (extends/excludes, raw props). | `new` | None (adapter responsibility). |
| RawProperty | [crates/domain/src/schema/raw.rs](crates/domain/src/schema/raw.rs) | Inline or ref property definition. | N/A | None (adapter responsibility). |
| RawPropertyInline | [crates/domain/src/schema/raw.rs](crates/domain/src/schema/raw.rs) | Inline property definition payload. | N/A | None (adapter responsibility). |
| RawPropertyRef | [crates/domain/src/schema/raw.rs](crates/domain/src/schema/raw.rs) | `$ref` property reference. | N/A | None (adapter responsibility). |
| Resolver | [crates/domain/src/schema/resolver.rs](crates/domain/src/schema/resolver.rs) | Domain service resolving raw schemas. | `resolve` | Missing ref detection, excludes applied, deterministic ordering. |
| Graph | [crates/domain/src/schema/graph.rs](crates/domain/src/schema/graph.rs) | Domain service for inheritance DAG. | `resolve_order`, `add_node` | Cycle detection, missing parent detection. |
| SchemaCreated | [crates/domain/src/schema/events.rs](crates/domain/src/schema/events.rs) | Domain event for schema creation. | `new` | N/A |
| PropertyBankUpdated | [crates/domain/src/schema/events.rs](crates/domain/src/schema/events.rs) | Domain event for property bank updates. | `new` | N/A |
| SchemaEvents | [crates/domain/src/schema/events.rs](crates/domain/src/schema/events.rs) | Event enum for schema context. | N/A | N/A |

## Config Bounded Context

| Entity | File | Purpose | Key Methods | Validation Rules |
| --- | --- | --- | --- | --- |
| Config | [crates/domain/src/config/aggregate.rs](crates/domain/src/config/aggregate.rs) | Final merged configuration aggregate. | `build`, `validate`, accessors | Vault path non-empty, logging enum constraints, key non-empty. |
| ConfigEvents | [crates/domain/src/config/events.rs](crates/domain/src/config/events.rs) | Event enum for config context. | N/A | N/A |
| ConfigUpdated | [crates/domain/src/config/events.rs](crates/domain/src/config/events.rs) | Event for config update. | `new` | N/A |
| Global | [crates/domain/src/config/global.rs](crates/domain/src/config/global.rs) | Global configuration defaults. | `default`, `validate` | Schema/template validation, trusted vaults format. |
| Global Filesystem | [crates/domain/src/config/global.rs](crates/domain/src/config/global.rs) | Global filesystem configuration. | `validate` | Schema/template validation. |
| TrustedVaults | [crates/domain/src/config/global.rs](crates/domain/src/config/global.rs) | Trusted vault allowlist. | `validate` | Must choose list or map, not both/neither. |
| Vault | [crates/domain/src/config/vault.rs](crates/domain/src/config/vault.rs) | Vault-specific overrides. | `default` | Optional validation via `Filesystem`/`Metadata`. |
| Vault Filesystem | [crates/domain/src/config/vault.rs](crates/domain/src/config/vault.rs) | Vault filesystem configuration. | `validate` | Cache dir non-empty, schema/template validation. |
| Metadata | [crates/domain/src/config/vault.rs](crates/domain/src/config/vault.rs) | Vault metadata with schema version/name. | `new`, `validate`, `validate_vault_path` | Vault path non-empty. |
| Frontmatter | [crates/domain/src/config/types.rs](crates/domain/src/config/types.rs) | Frontmatter key mapping. | `validate` | Keys must be non-empty. |
| Logging | [crates/domain/src/config/types.rs](crates/domain/src/config/types.rs) | Logging config. | `validate` | Log level must be debug/info/warn/error. |
| Schema | [crates/domain/src/config/types.rs](crates/domain/src/config/types.rs) | Schema filesystem config. | `validate`, `property_bank_path` | Paths non-empty. |
| Template | [crates/domain/src/config/types.rs](crates/domain/src/config/types.rs) | Template filesystem config. | `validate` | templates_dir non-empty. |
| SettingValue | [crates/domain/src/config/types.rs](crates/domain/src/config/types.rs) | Polymorphic config value container. | Conversions, `Debug` | Serialization required; encrypted bytes opaque. |

## Template Bounded Context

| Entity | File | Purpose | Key Methods | Validation Rules |
| --- | --- | --- | --- | --- |
| Template | [crates/domain/src/template/aggregate.rs](crates/domain/src/template/aggregate.rs) | Template aggregate with content and variables. | `new`, `compose`, `validate`, accessors | Name format, content size, placeholder balance, variable name rules. |
| Metadata | [crates/domain/src/template/aggregate.rs](crates/domain/src/template/aggregate.rs) | Template metadata. | `default` | N/A |
| VariableDefinition | [crates/domain/src/template/variable.rs](crates/domain/src/template/variable.rs) | Typed variable constraints. | `validate_value`, `has_default`, `get_default_value` | Type validation, length/range/pattern/file type constraints. |
| PlaceholderSyntax | [crates/domain/src/template/syntax.rs](crates/domain/src/template/syntax.rs) | Placeholder delimiter configuration. | `new`, `wrap` | None (structural). |
| Composition | [crates/domain/src/template/composition.rs](crates/domain/src/template/composition.rs) | Template composition definition. | `validate`, `detect_cycles` | Depth <= 5, no cycles, overrides match variable definitions. |
| Section | [crates/domain/src/template/composition.rs](crates/domain/src/template/composition.rs) | Template section insertion payload. | N/A | N/A |
| InsertionPosition | [crates/domain/src/template/composition.rs](crates/domain/src/template/composition.rs) | Enum for insertion placement. | N/A | N/A |
| TemplateCreated | [crates/domain/src/template/events.rs](crates/domain/src/template/events.rs) | Domain event for template creation. | `new` | N/A |
| TemplateEvents | [crates/domain/src/template/events.rs](crates/domain/src/template/events.rs) | Event enum for template context. | N/A | N/A |
| validate_content | [crates/domain/src/template/validation.rs](crates/domain/src/template/validation.rs) | Content size validator. | N/A | <= 1MB. |
| validate_structure | [crates/domain/src/template/validation.rs](crates/domain/src/template/validation.rs) | Placeholder balance validator. | N/A | Balanced open/close delimiters. |

## Shared Domain Utilities

| Entity | File | Purpose | Key Methods | Validation Rules |
| --- | --- | --- | --- | --- |
| Validation Utilities | [crates/domain/src/validation.rs](crates/domain/src/validation.rs) | Shared validation helpers. | `validate_vault_path`, `validate_numeric_range`, `validate_string_length`, `validate_numeric_step` | Path relative, no traversal, extension check, range/step checks. |
| Patterns | [crates/domain/src/patterns.rs](crates/domain/src/patterns.rs) | Regex pattern constants. | N/A | N/A |
| DomainError | [crates/domain/src/errors.rs](crates/domain/src/errors.rs) | Shared domain error types. | N/A | N/A |
| ConfigError | [crates/domain/src/errors.rs](crates/domain/src/errors.rs) | Config-specific error types. | N/A | N/A |
