# Story 3.6: Create Epic 3 Documentation

Status: ready-for-dev

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer working with the domain models,
I want comprehensive documentation of the domain entities, their relationships, and evolution guidelines,
So that developers understand the domain language and can work effectively with the models.

## Acceptance Criteria

1. **Given** all Epic 3 domain models are implemented
   **When** I create documentation
   **Then** it includes developer-focused content:
   - Domain entity relationships and bounded contexts
   - Semantic validation rules for each entity
   - Domain entity relationship contracts (how bounded contexts interact)
   - Evolution guidelines for domain models (when to add vs modify entities)
   - Architecture diagrams showing entity relationships and contracts

2. **Given** documentation is created
   **When** I validate completeness
   **Then** it covers all entities and their contracts: Note aggregate, Schema domain, Config, Template

3. **Given** documentation exists
   **When** I check relationship contracts
   **Then** it defines how bounded contexts interact (e.g., Template references Schema, Note uses Config)

4. **Given** documentation exists
   **When** a developer reads it
   **Then** they understand domain evolution rules and inter-entity contracts without needing user-facing knowledge

## Tasks / Subtasks

### Task 1: Analyze Epic 3 Domain Models for Documentation Scope
- [ ] Read all files in `crates/domain/src/models/note/` directory to understand Note bounded context: Note aggregate, Frontmatter, Link, Embed, Tag, Heading, Task, Section entities and their validation rules, business logic
- [ ] Read all files in `crates/domain/src/models/schema/` directory to understand Schema bounded context: Schema aggregate, Property, PropertyBank, PropertySpec trait and implementations (StringSpec, NumberSpec, etc.), inheritance resolution, trait-based generic design
- [ ] Read all files in `crates/domain/src/models/config/` directory to understand Config bounded context: Config entity with phantom types, ConfigValue, ConfigPath, ValidationRule, hierarchical merging, encryption boundary
- [ ] Read all files in `crates/domain/src/models/template/` directory to understand Template bounded context: Template aggregate, VariableDefinition, TemplateComposition, MiniJinja compatibility, domain purity (no syntax validation)
- [ ] Analyze inter-entity relationships: Note ↔ Config (defaults), Note ↔ Schema (validation), Template ↔ Schema (variable constraints), Template ↔ Config (execution settings)
- [ ] Document evolution patterns: adding fields/subentities, modifying validation rules, trait evolution, phantom type changes, backward compatibility requirements
- [ ] Create inventory document `_bmad-output/documentation-inventory/epic3-domain-entities.md` listing all entities with file locations, purposes, key methods, validation requirements

### Task 2: Create Domain Entity Documentation Structure
- [ ] Create file `docs/domain-models.md` with title `# Epic 3 Domain Models`, 2-3 paragraph overview of all 4 bounded contexts, and table of contents
- [ ] In `docs/domain-models.md`, establish documentation hierarchy with sections for each bounded context using the template: Overview, Structure, Rust-Specific Patterns Used, Validation Rules, Business Logic, Relationships, Evolution Guidelines
- [ ] Add Documentation Standards section in `docs/domain-models.md` with naming conventions, documentation template description, and code example requirements
- [ ] Create file `docs/domain-entity-template.md` containing the complete documentation template with fill-in-the-blank sections for all required information and example filled-out template
- [ ] Add Splitting Criteria section in `docs/domain-models.md` with decision matrix: if total lines > 2000, bounded context section > 500 lines, readability affected, or specific context needs frequent updates, split to separate files
- [ ] Monitor line count during documentation writing and decide on splitting if criteria are met

### Task 3: Document Note Bounded Context
- [ ] Add `## Note Bounded Context` section to `docs/domain-models.md` with 2-3 paragraph overview explaining Note as main domain entity representing Obsidian notes
- [ ] Write Structure subsection listing complete Note aggregate structure: Note entity fields (id, path, frontmatter, links, embeds, tags, headings, tasks, sections), all subentity structures with their fields and validation requirements
- [ ] Write Rust-Specific Patterns Used subsection: UUID v7 identity, memory strategy (Box<str>, Arc<str>), immutability, type safety with enums, error handling with thiserror
- [ ] Write Validation Rules subsection: path validation (non-empty, relative, .md extension), tag validation (regex, no empty segments), heading level validation (1-6), frontmatter date validation (ISO 8601), link/embed target validation
- [ ] Write Business Logic subsection: semantic validation during construction, internal consistency checks, relationship invariants, vault context requirements
- [ ] Write Relationships subsection: Note ↔ Config contract (defaults), Note ↔ Schema contract (validation), internal composition relationships
- [ ] Write Evolution Guidelines subsection: adding new fields/subentities with defaults, modifying existing fields with migration paths, removing fields with deprecation periods
- [ ] Add ASCII architecture diagram showing Note aggregate and all subentities with relationships
- [ ] Cross-reference with `_bmad-output/documentation-inventory/epic3-domain-entities.md` to ensure all entities and validation rules are documented

### Task 4: Document Schema Bounded Context
- [ ] Add `## Schema Bounded Context` section to `docs/domain-models.md` with 2-3 paragraph overview explaining Schema for defining metadata structure and validation rules
- [ ] Write Structure subsection listing complete Schema structure: Schema aggregate (name, extends, excludes, properties), Property entity (id, name, required, array, spec), PropertyBank singleton, PropertySpec trait and implementations (StringSpec<MAX_LEN>, NumberSpec<MIN,MAX>, BoolSpec, DateSpec, FileSpec)
- [ ] Write Rust-Specific Patterns Used subsection: trait-based polymorphism with associated types, const generics for compile-time validation, trait objects for runtime flexibility, deterministic ID generation, zero-cost abstraction
- [ ] Write Validation Rules subsection: schema name validation, property name validation, PropertySpec validation (string length/bounds/patterns, number ranges/step, bool/date/file constraints), inheritance cycle detection, excludes validation
- [ ] Write Business Logic subsection: inheritance resolution algorithm, Property Bank lookup, override semantics, excludes processing, deterministic resolution, cycle detection with DFS
- [ ] Write Relationships subsection: Schema ↔ PropertyBank (reusable properties), Schema ↔ Schema (inheritance), PropertyBank ↔ PropertySpec (validation implementations), Template ↔ Schema (variable constraints)
- [ ] Write Evolution Guidelines subsection: adding PropertySpec variants, modifying constraints, inheritance changes, trait evolution, const generic changes
- [ ] Add ASCII architecture diagram showing Schema, PropertyBank, and PropertySpec hierarchy with relationships
- [ ] Cross-reference with `_bmad-output/documentation-inventory/epic3-domain-entities.md` to ensure all entities and validation rules are documented

### Task 5: Document Config Bounded Context
- [ ] **Write Config Bounded Context section in `docs/domain-models.md`**:
  - Add `## Config Bounded Context` header
  - Write **Overview** section explaining Config's purpose for hierarchical application configuration with type safety
  - Write **Structure** section with complete entity list:
    ```
    ### Structure

    **Config Entity with Phantom Types** (`crates/domain/src/models/config/config.rs`)
    - `values: HashMap<ConfigPath<Level>, HashMap<String, ConfigValue>>` - Hierarchical configuration by level
    - `validation_rules: HashMap<String, ValidationRule>` - Validation constraints
    - `encrypted_fields: HashSet<String>` - Tracking of encrypted sensitive fields
    - Phantom type parameter: `<Level = Global>` - Compile-time context safety
    - Type-safe aliases: `GlobalConfig`, `UserConfig`, `ProjectConfig`, `VaultConfig`

    **ConfigPath Enum with Phantom Types** (`crates/domain/src/models/config/path.rs`)
    - `Global(PhantomData<Global>)`, `User(PhantomData<User>)`, `Project(PhantomData<Project>)`, `Vault(PhantomData<Vault>)`
    - Precedence order: Global → User → Project → Vault (Vault highest precedence)
    - Type-safe aliases: `GlobalPath`, `UserPath`, `ProjectPath`, `VaultPath`

    **Phantom Type Markers** (`crates/domain/src/models/config/types.rs`)
    - `pub struct Global`, `pub struct User`, `pub struct Project`, `pub struct Vault`
    - Zero-cost type markers (no runtime overhead)
    - Purpose: Compile-time prevention of mixing config contexts

    **ConfigValue Enum** (`crates/domain/src/models/config/value.rs`)
    - `String(String)`, `Number(f64)`, `Boolean(bool)`, `Encrypted(Vec<u8>)`, `Array(Vec<ConfigValue>)`, `Object(HashMap<String, ConfigValue>)`
    - Type-safe representation for all configuration value types
    - Encrypted variant: Domain stores encrypted blob, encryption/decryption in adapter

    **ValidationRule Enum** (`crates/domain/src/models/config/validation.rs`)
    - `Required` - Field must be present in final merged configuration
    - `Enum(Vec<String>)` - Field value must match one of these string values
    - `Range { min: Option<f64>, max: Option<f64> }` - Numeric field must be within range
    - `Pattern(String)` - String field must match regex pattern
    - `DependsOn(String)` - Field depends on another field being present/set
    ```
  - Write **Rust-Specific Patterns Used** section:
    ```
    ### Rust-Specific Patterns Used

    - **Phantom Types**: Compile-time context safety prevents mixing Global/User/Project/Vault configs
    - **Type-Safe Aliases**: `GlobalConfig = Config<Global>` enables compile-time guarantees
    - **Associated Types in Ports**: Repository ports use associated types for type-safe operations
    - **Zero-Cost Abstraction**: PhantomData has no runtime overhead, compile-time only
    - **Generic Algorithms**: Hierarchical merging works across all Config<Level> types
    - **Encryption Boundary**: Domain stores encrypted blobs, encryption/decryption isolated to adapter layer
    - **Compile-Time Precedence**: ConfigPath enum enforces hierarchical override order at type level
    ```
  - Write **Validation Rules** section:
    ```
    ### Validation Rules

    **Hierarchical Merging Validation**
    - Type compatibility: ConfigValue types must match across override levels
    - Override precedence: Vault > Project > User > Global (compile-time enforced)
    - Merge conflicts: Detected and surfaced as ConfigError::MergeConflict

    **ConfigValue Validation**
    - **String**: Non-empty if required, matches pattern if specified, within length constraints
    - **Number**: Within Range if specified, not NaN, finite value
    - **Boolean**: Always valid type (no constraints)
    - **Encrypted**: Non-empty if required (validation happens on decrypted value)
    - **Array**: Non-empty if required, all elements pass individual validation
    - **Object**: Non-empty if required, all keys present, values pass validation

    **Phantom Type Safety**
    - Compile-time: Can't pass UserConfig to function expecting GlobalConfig
    - Runtime: ConfigPath enum prevents invalid mixing
    - Type-safe APIs: Functions accept specific Config<Level> types, preventing context confusion
    ```
  - Write **Business Logic** section:
    ```
    ### Business Logic

    - **Hierarchical Merging Algorithm**:
      1. Start with empty HashMap
      2. Merge Global level (all key-value pairs)
      3. Merge User level (override same keys)
      4. Merge Project level (override same keys)
      5. Merge Vault level (override same keys, highest precedence)
      6. Apply validation rules to merged HashMap
      7. Return final merged configuration
    - **Encrypted Field Handling**: Domain tracks which fields are encrypted via HashSet, adapter handles encryption/decryption
    - **Validation Rule Application**: After merging, apply all ValidationRules from validation_rules HashMap
    - **Type Safety Guarantees**: Phantom types ensure config levels can't be mixed incorrectly at compile time
    - **Fallback Behavior**: get() method automatically tries Vault, then Project, then User, then Global levels
    ```
  - Write **Relationships** section:
    ```
    ### Relationships

    **Config Hierarchy** (Internal)
    - Global → User → Project → Vault (precedence order)
    - Each level can override keys from previous levels
    - Vault level has highest precedence for all keys

    **Config ↔ Note Contract**
    - Config provides default values for Note frontmatter fields
    - Config provides metadata display preferences (tag formatting, heading display)
    - Relationship: Note reads from Config, no bidirectional dependency

    **Config ↔ Schema Contract**
    - Config specifies schema file locations and validation rules
    - Config drives schema loading and validation behavior
    - Relationship: Config references Schema, Schema has no knowledge of Config

    **Config ↔ Template Contract**
    - Config provides template execution parameters and settings
    - Config provides template pack locations and rendering preferences
    - Relationship: Template reads from Config, no bidirectional dependency
    ```
  - Write **Evolution Guidelines** section:
    ```
    ### Evolution Guidelines

    **Adding New Config Levels**
    - Add new phantom type marker (e.g., pub struct Team)
    - Add ConfigPath variant (e.g., Team(PhantomData<Team>))
    - Add precedence order (update get() to check new level)
    - Create type alias: `pub type TeamConfig = Config<Team>`
    - Example: Adding "Team" config level requires phantom marker, ConfigPath variant, precedence update

    **Modifying Phantom Types**
    - Adding phantom type parameters is breaking change
    - Example: Config<Level, Context> requires updating all type aliases
    - Consider carefully if additional compile-time context safety is needed

    **Adding New ConfigValue Variants**
    - Add variant to ConfigValue enum
    - Implement validation logic for new variant
    - Update serialization/deserialization if needed
    - Example: Adding `Binary(Vec<u8>)` variant for binary config values

    **Validation Rule Evolution**
    - Adding new ValidationRule variants requires updating validation logic
    - Ensure backward compatibility (existing configs with old rules still validate)
    - Example: Adding `RegexMatch(String)` rule requires new validation code path
    ```
  - Add ASCII architecture diagram showing Config hierarchy and phantom types:
    ```
    ## Config Architecture Diagram

                        Config<Level> (Phantom Type Parameter)
                          |
            +-----------+-----------+-----------+-----------+
            |           |           |           |           |
      Global     User       Project     Vault
    (Level=Global) (Level=User) (Level=Project) (Level=Vault)
            |           |           |           |           |
          ConfigPath enum variants with phantom types
            |
      HashMap<ConfigPath<Level>, HashMap<String, ConfigValue>>
                          |
                 ConfigValue enum variants
    String  Number  Boolean  Encrypted  Array  Object
    ```
- [ ] **Review against inventory**: Cross-reference with `_bmad-output/documentation-inventory/epic3-domain-entities.md`
  - Ensure all entities from inventory are documented
  - Ensure all phantom type markers are explained
  - Ensure hierarchical merging algorithm is documented
  - Ensure encryption boundary separation is clear
  - Add any missing information before proceeding to next bounded context

### Task 6: Document Template Bounded Context
- [ ] Document Template entity with modular composition and variable definitions
- [ ] Detail VariableDefinition enum variants with type constraints and defaults
- [ ] Document TemplateComposition for inheritance and modular assembly
- [ ] Include MiniJinja compatibility requirements and domain layer boundaries
- [ ] Document Template relationships with Schema bounded context (variable validation)
- [ ] If splitting occurs, create `docs/domain/template.md` for in-depth details

### Task 7: Create Bounded Context Interaction Contracts
- [ ] Create Bounded Context Contracts section in `docs/domain-models.md`
- [ ] Document Note ↔ Config Contract with sequence diagram
- [ ] Document Note ↔ Schema Contract with sequence diagram
- [ ] Document Template ↔ Schema Contract with sequence diagram
- [ ] Document Template ↔ Config Contract with sequence diagram
- [ ] Add contract evolution rules section

### Task 8: Create Evolution Guidelines and Architecture Diagrams
- [ ] Create Epic 3 Architecture Diagrams section in `docs/domain-models.md`
- [ ] Create Domain Model Evolution Guidelines section
- [ ] Create dedicated evolution guidelines document if needed
- [ ] Review all diagrams and guidelines for completeness

### Task 9: Validate Documentation Completeness and Quality
- [ ] Cross-reference documentation against actual domain model implementations
- [ ] Ensure all validation rules and business logic are documented
- [ ] Validate that relationship contracts are clearly defined and actionable
- [ ] Test documentation by having another developer use it to understand domain models
- [ ] Incorporate feedback and iterate on documentation clarity

### Task 10: Quality Assurance and Commit (MANDATORY FINAL TASK)
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all documentation meets quality standards and covers all requirements
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `docs: create comprehensive epic 3 domain model documentation with entity relationships and evolution guidelines`

## Dev Notes

### Epic 3 Domain Model Scope
- **Target Entities**: Note aggregate (with 7 subentities), Schema domain, Config hierarchy, Template composition system
- **Documentation Focus**: Developer understanding of domain language, relationships, and evolution rules
- **Quality Standard**: Enable developers to work effectively with domain models without user-facing knowledge
- **Architecture Context**: Hexagonal boundaries, bounded context interactions, domain purity requirements

### Previous Story Intelligence
**Story 3.4 Critical Architectural Lesson:**
- **MAJOR FIX**: MiniJinja syntax validation moved from domain to adapter layer
- **Domain Purity**: Domain entities store content as opaque strings, validation in adapters
- **Testing Implication**: Domain tests validate business rules only, no syntax concerns
- **Documentation Need**: Clear hexagonal boundaries and responsibility separation

**Story 3.3 Config Implementation Patterns:**
- **Hierarchical Structure**: Global → User → Project → Vault configuration layers
- **Validation Integration**: Semantic validation ensures configuration integrity
- **Error Handling**: thiserror::Error for typed configuration validation errors

**Story 3.2 Schema Complexity:**
- **PropertyBank Registry**: Singleton for reusable property definitions
- **Inheritance System**: Extends/Excludes for schema composition
- **Type Safety**: PropertySpec variants with specific validation rules

**Story 3.1 Note Aggregate Foundation:**
- **Rich Subentities**: Frontmatter, Links, Embeds, Tags, Headings, Tasks, Sections
- **Semantic Validation**: Internal consistency validation per entity
- **Vault Integration**: Path resolution and wiki-link handling

### Architecture Documentation Requirements
**Progressive Documentation Strategy:**
- **Power Users**: Get API docs + advanced guides with implementation details
- **New Users**: Get guided tutorials with concrete examples
- **Documentation as Code**: mdBook format with concrete outcomes focus
- **Migration Guides**: Critical for adoption and understanding evolution

**Hexagonal Architecture Documentation:**
- **Domain Layer**: Pure business logic, no external dependencies, entity validation
- **Adapter Layer**: External integrations, I/O operations, framework-specific code
- **Ports**: Clean interfaces between domain and adapters
- **Contracts**: Clear interaction patterns between bounded contexts

### Git Intelligence Summary
**Documentation Patterns Established:**
- Comprehensive README.md created (Story 1.8)
- ADR process established (Story 1.7)
- Project roadmap with milestones (Story 1.9)
- Consistent commit messages with conventional format

**Code Quality Standards Applied:**
- Clippy cognitive complexity limits (<25)
- No unwrap/expect/panic in production code
- Comprehensive error handling with thiserror
- Proper trait implementations (From/TryFrom, not ad-hoc methods)

## Technical Requirements

### Documentation Structure and Standards

**Documentation Structure (Single File Primary):**
- **Primary Approach**: Single comprehensive `docs/domain-models.md` file
- **Splitting Criteria**: Only split if file exceeds 2000 lines OR becomes hard to maintain/read
- **Split Structure**: If split occurs, overview in `docs/domain/overview.md` with essential information
- **Specific Files**: In-depth technical details in `docs/domain/` directory with clear cross-references
- **Leverage Existing**: Use existing `architecture.md` for hexagonal boundaries (no separate file needed)

**Single File Organization:**
```markdown
# Epic 3 Domain Models

## Overview
[Essential information about all 4 bounded contexts]

## Note Bounded Context
[Complete Note documentation with all entities and relationships]

## Schema Bounded Context
[Complete Schema documentation with inheritance and validation]

## Config Bounded Context
[Complete Config documentation with hierarchy and contracts]

## Template Bounded Context
[Complete Template documentation with composition and MiniJinja integration]

## Bounded Context Contracts
[All inter-context interaction contracts and evolution guidelines]

## Architecture Diagrams
[Visual representations of relationships and boundaries]
```

**Splitting Strategy (If Needed):**
```
docs/
├── domain-models.md               # Single comprehensive file (primary approach)
└── domain/
    ├── overview.md                # Essential information about all bounded contexts
    ├── note.md                    # Note bounded context in-depth technical details
    ├── schema.md                  # Schema bounded context in-depth technical details
    ├── config.md                  # Config bounded context in-depth technical details
    ├── template.md                # Template bounded context in-depth technical details
    └── evolution-guidelines.md    # Domain model evolution patterns

# Existing file (minimal update):
_bmad-output/planning-artifacts/architecture.md  # [MINIMAL UPDATE] Hexagonal boundaries documentation
```

**Splitting Strategy (If Needed):**
```
docs/
├── domain-models.md               # Single comprehensive file (primary approach)
├── domain/
│   ├── overview.md                # Essential information about all bounded contexts
│   ├── note.md                    # Note bounded context in-depth technical details
│   ├── schema.md                  # Schema bounded context in-depth technical details
│   ├── config.md                  # Config bounded context in-depth technical details
│   └── template.md                # Template bounded context in-depth technical details
└── domain/
    └── evolution-guidelines.md    # Domain model evolution patterns
```

**Entity Documentation Template:**
```markdown
# [Entity Name] Entity

## Overview
[Purpose and business context]

## Structure
[Fields, types, relationships]

## Rust-Specific Patterns Used
- **Const Generics**: [If used for compile-time validation]
- **Phantom Types**: [If used for context safety]
- **Associated Types**: [If used in ports/interfaces]
- **Memory Optimization**: [Box<str>, Arc<str> usage]
- **Virtual Clock**: [Use of time_test! for deterministic time logic]
- **Domain Purity**: [Enforcement via Domain Purity Guardian]
- **Factory Macros**: [Use of test_builder! for domain fixtures]

## Validation Rules
[Semantic validation requirements]

## Business Logic
[Key business rules and invariants]

## Relationships
[Connections to other entities/contexts]

## Evolution Guidelines
[When/how to modify this entity]
```

**Advanced Rust Patterns Documentation:**
- **Const Generics**: Used in PropertySpec for compile-time constraint validation (e.g., StringSpec<200>)
- **Phantom Types**: Employed in Config hierarchies to prevent context mixing at compile-time
- **Associated Types**: Used in repository ports for type-safe query results and error handling
- **Memory Optimization**: Arc<str> for shared strings, Box<str> for heap-efficient storage

### Domain Entity Coverage Requirements

**Note Bounded Context:**
- **Note Aggregate**: Main entity with identity, metadata, and content
- **Subentities**: Frontmatter (YAML), Links (wiki-links), Embeds, Tags, Headings, Tasks, Sections
- **Validation**: Semantic consistency, vault-relative paths, wiki-link resolution
- **Contracts**: Uses Config for metadata, Schema for validation, Template for rendering

**Schema Bounded Context:**
- **Schema Entity**: Name, Extends, Excludes, Properties arrays
- **PropertyBank**: Singleton registry for reusable Property definitions
- **Property Entity**: ID, Name, Required, Array, Spec (deterministic hashing)
- **PropertySpec Variants**: String (enum/pattern), Number (range), Bool (simple), Date (format), File (types)
- **Contracts**: Provides validation rules for Template variables and Note frontmatter

**Config Bounded Context:**
- **Hierarchical Structure**: Global → User → Project → Vault precedence
- **Validation**: Type safety, integrity checks, merge conflict resolution
- **Contracts**: Provides configuration for Note metadata, Template rendering, Schema defaults

**Template Bounded Context:**
- **Template Entity**: UUID identity, content storage, variable definitions
- **VariableDefinition Enum**: String/Number/Bool/Date/File with constraints
- **TemplateComposition**: Extends/includes with dependency resolution
- **MiniJinja Compatibility**: Domain stores opaque content, adapter handles syntax
- **Contracts**: References Schema for variable validation, uses Config for rendering parameters

### Bounded Context Interaction Contracts

**Note ↔ Config Contract:**
- Note uses Config for frontmatter defaults and metadata validation
- Config provides hierarchical fallback for missing note properties
- Contract: Config changes don't break existing Note validation

**Note ↔ Schema Contract:**
- Note frontmatter validated against linked Schema
- Schema provides property definitions and validation rules
- Contract: Schema evolution maintains backward compatibility

**Template ↔ Schema Contract:**
- Template variables validated against Schema property specs
- Schema provides type information and constraints for template variables
- Contract: Schema changes reflected in template validation

**Template ↔ Config Contract:**
- Config provides template execution parameters and defaults
- Template uses Config for rendering context and environment settings
- Contract: Config changes don't break template rendering

### Evolution Guidelines

**Entity Modification Rules:**
- **Add New Fields/Properties**: Always backward compatible, optional with defaults
- **Modify Existing Fields**: Requires migration path and backward compatibility
- **Remove Fields/Properties**: Only after deprecation period, with clear migration guide
- **Change Validation Rules**: Must maintain existing valid data validity

**Bounded Context Evolution:**
- **Contract Changes**: Require coordinated updates across interacting contexts
- **Breaking Changes**: Document migration path and timeline
- **New Contexts**: Define clear interaction contracts with existing contexts
- **Versioning**: Use semantic versioning for domain model changes

**Hexagonal Architecture Evolution:**
- **Domain Changes**: Can be breaking if contracts change
- **Adapter Changes**: Should be non-breaking for domain consumers
- **Port Changes**: Require coordinated updates across domain and adapters
- **New Ports**: Define clear responsibilities and interaction patterns

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
