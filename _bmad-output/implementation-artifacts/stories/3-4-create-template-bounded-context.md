# Story 3.4: Create Template Bounded Context

Status: ready-for-dev

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer working with template definitions,
I want a Template domain model with validation,
So that template structure and business rules are properly validated at the domain level.

## Acceptance Criteria

**Given** I have researched template engine patterns
**When** I review the Template bounded context
**Then** Template entity includes structure validation and business rules

**Given** Template entity is defined
**When** I check semantic validation
**Then** template business rules and composition logic are validated internally

**Given** template patterns are established
**When** I validate the design
**Then** Template supports modular composition and variable definitions

**Given** the Template bounded context is defined
**When** I check domain events
**Then** TemplateCreated event is emitted for template lifecycle

**Given** CQRS separation is needed
**When** I define ports
**Then** TemplateCommand and TemplateQuery trait interfaces are provided for future implementation

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Template Domain Tests First (RED Phase - AC: All)
- [ ] Write failing unit tests for Template entity (structure validation, variable definitions)
- [ ] Write failing unit tests for VariableDefinition (type safety, default values, constraints)
- [ ] Write failing unit tests for TemplateComposition (modular assembly, dependency resolution)
- [ ] Write failing unit tests for domain business rules (variable naming, composition cycles)
- [ ] Write failing property-based tests for edge cases (invalid variable names, composition cycles)
- [ ] Write failing integration tests for template composition and variable resolution
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)

### Task 2: Implement Template Core Entities (GREEN Phase - AC: 1-3)
- [ ] Create file `crates/domain/src/models/template/template.rs` and define Template struct with `#[derive(Debug, Clone, PartialEq)] pub struct Template` and fields `pub id: Uuid`, `pub name: String`, `pub content: String`, `pub variables: HashMap<String, VariableDefinition>`, `pub extends: Option<String>`, `pub metadata: TemplateMetadata`
- [ ] In `crates/domain/src/models/template/template.rs`, implement `new()` constructor that validates name format (regex `^[a-zA-Z0-9_-]+$`), variable name conflicts, composition cycles, returns `Result<Self, TemplateError>`
- [ ] In `crates/domain/src/models/template/template.rs`, implement `validate_business_rules()` method checking variable definitions match HashMap, no circular references in extends, returns `Result<(), TemplateError>`
- [ ] Create file `crates/domain/src/models/template/metadata.rs` and define TemplateMetadata struct with `#[derive(Debug, Clone, PartialEq)] pub struct TemplateMetadata` and fields `pub description: Option<String>`, `pub version: Option<String>`, `pub tags: Vec<String>`, `pub created_at: DateTime<Utc>`, `pub updated_at: DateTime<Utc>`
- [ ] In `crates/domain/src/models/template/metadata.rs`, implement `Default` trait with sensible defaults for timestamps
- [ ] Create file `crates/domain/src/models/template/variable.rs` and define VariableDefinition enum with `#[derive(Debug, Clone, PartialEq)] #[non_exhaustive] pub enum VariableDefinition { String { default: Option<String>, min_length: Option<usize>, max_length: Option<usize>, pattern: Option<String> }, Number { default: Option<f64>, min: Option<f64>, max: Option<f64> }, Boolean { default: Option<bool> }, Date { default: Option<String>, format: Option<String> }, File { default: Option<String>, file_types: Option<Vec<String>> } }`
- [ ] In `crates/domain/src/models/template/variable.rs`, implement `validate_value()` method for VariableDefinition that performs type-safe validation based on variant, returns `Result<(), TemplateError>`
- [ ] In `crates/domain/src/models/template/variable.rs`, implement `has_default()` and `get_default_value()` helper methods
- [ ] Create file `crates/domain/src/models/template/position.rs` and define InsertionPosition enum with `#[derive(Debug, Clone, PartialEq)] pub enum InsertionPosition { BeforeVariable(String), AfterVariable(String), Beginning, End }`
- [ ] In `crates/domain/src/models/template/position.rs`, implement validation for variable names in BeforeVariable/AfterVariable variants
- [ ] Create file `crates/domain/src/models/template/section.rs` and define TemplateSection struct with `#[derive(Debug, Clone, PartialEq)] pub struct TemplateSection` and fields `pub name: String`, `pub content: String`, `pub position: InsertionPosition`
- [ ] In `crates/domain/src/models/template/section.rs`, implement validation for non-empty name and reasonable content length
- [ ] Create file `crates/domain/src/models/template/composition.rs` and define TemplateComposition struct with `#[derive(Debug, Clone, PartialEq)] pub struct TemplateComposition` and fields `pub base_template: String`, `pub variable_overrides: HashMap<String, serde_json::Value>`, `pub additional_sections: Vec<TemplateSection>`, `pub includes: Vec<String>`
- [ ] In `crates/domain/src/models/template/composition.rs`, implement `validate()` method that checks base_template exists in available_templates, no circular includes, variable_override types compatible, returns `Result<(), TemplateError>`
- [ ] In `crates/domain/src/models/template/composition.rs`, implement `detect_cycles()` method using depth-first search to detect circular references, max depth 5, returns `Result<(), TemplateError>`
- [ ] Update `crates/domain/src/models/template/mod.rs` to add module declarations and re-export all entities `pub use template::Template; pub use metadata::TemplateMetadata; pub use variable::VariableDefinition; pub use position::InsertionPosition; pub use section::TemplateSection; pub use composition::TemplateComposition;`
- [ ] **TDD REQUIREMENT:** Make all Template domain tests pass (GREEN phase complete when all tests pass)

### Task 3: Implement Template Validation Logic (GREEN Phase - AC: All)
- [ ] Implement syntax validation for MiniJinja template expressions in adapter layer ({{variable}}, {% if %}, {% for %})
- [ ] Add variable reference validation (undefined variables, type mismatches)
- [ ] Implement modular composition validation (dependency cycles, missing templates)
- [ ] Add template structure validation (proper nesting, balanced blocks)
- [ ] Implement performance validation (template size limits, complexity constraints)
- [ ] Create validation error types with detailed diagnostic information
- [ ] **TDD REQUIREMENT:** Make all validation tests pass

### Task 4: Refactor for Quality (REFACTOR Phase - AC: All)
- [ ] Optimize template parsing performance (<500ms for typical templates)
- [ ] Implement memory-efficient template storage (shared string interning)
- [ ] Add comprehensive error handling with thiserror::Error and proper error chaining
- [ ] Ensure hexagonal architecture compliance (domain purity, no MiniJinja direct usage)
- [ ] Add performance optimizations for variable resolution and composition
- [ ] Verify proper ownership patterns for template content and variables
- [ ] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 5: Comprehensive Testing Coverage (RED-GREEN-REFACTOR - AC: All)
- [ ] Achieve 90%+ test coverage for all Template domain entities
- [ ] **FACTORY MACROS:** Use `test_builder!` for modular template assembly and composition tests to maintain fixture readability
- [ ] Create test fixtures module with sample templates, variables, and compositions
- [ ] Implement property-based testing for template syntax variations and edge cases
- [ ] Add integration tests for template composition and variable resolution workflows
- [ ] Add performance benchmarks for template parsing and validation (<500ms target)
- [ ] **TDD REQUIREMENT:** Coverage reports show 90%+ coverage, all property-based tests pass

### Task 6: Documentation and Integration (REFACTOR Phase - AC: All)
- [ ] Update domain crate lib.rs with Template module public exports
- [ ] Add comprehensive doc comments with template examples and validation rules
- [ ] Ensure integration points with Epic 11 (template execution) and Epic 6 (schema integration)
- [ ] Update Cargo.toml with required dependencies (serde for serialization, optional validation crates)
- [ ] **TDD REQUIREMENT:** All documentation examples compile and run successfully

### Task 8: Implement Domain Events (GREEN Phase - AC: All)
- [ ] Define TemplateCreated domain event
- [ ] Add event emission in Template entity methods
- [ ] Ensure events capture template creation state
- [ ] **TDD REQUIREMENT:** Make all domain event tests pass

### Task 9: Define CQRS Ports (GREEN Phase - AC: All)
- [ ] Define TemplateCommand trait interface (shell for future implementation)
- [ ] Define TemplateQuery trait interface (shell for future implementation)
- [ ] Place ports in domain ports module
- [ ] **TDD REQUIREMENT:** Make all port interface tests pass

### Task 10: Quality Assurance and Commit (MANDATORY FINAL TASK - TDD Validation)
- [ ] **TDD VALIDATION:** Confirm all tests pass and coverage meets 90%+ requirement
- [ ] **TDD VALIDATION:** Verify property-based tests catch template syntax edge cases
- [ ] **TDD VALIDATION:** Ensure performance benchmarks meet targets (<500ms template operations)
- [ ] **TDD VALIDATION:** Verify MiniJinja compatibility and variable resolution accuracy
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING (TDD requires clean code)
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all domain entities pass clippy cognitive complexity limits (<25)
- [ ] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [ ] **MANDATORY:** Verify hexagonal architecture boundaries maintained (template domain purity)
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `feat: implement template bounded context with validation, composition, domain events, and CQRS ports`

## Technical Requirements

### Domain Model Foundation

**Core Entity Structure:**
- **Template Entity**: Main aggregate root with UUID v7 identity and content storage
- **VariableDefinition Enum**: Type-safe variable definitions with constraints and defaults
- **TemplateComposition Struct**: Modular template assembly with dependency management
- **Immutability**: All template entities MUST be immutable following Rust ownership patterns
- **Validation**: Domain business rules only (no syntax validation - that's adapter layer)
- **Error Handling**: Use `thiserror::Error` for typed domain validation errors

**CRITICAL ARCHITECTURAL PRINCIPLE:**
- **DOMAIN LAYER**: Business rules, variable definitions, composition logic, semantic validation
- **ADAPTER LAYER**: MiniJinja syntax validation, template rendering, file I/O, external format parsing
- **NO SYNTAX VALIDATION IN DOMAIN**: Domain stores template content as opaque strings
- **NO IO CONCERNS IN DOMAIN**: All file operations happen in adapters
- **PURITY GUARDIAN:** Domain purity is enforced by the `Domain Purity Guardian` automated test suite

**Template Entity Specification:**
```rust
/// Template represents a reusable template with validation and composition capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    /// UUID v7 identity for template
    pub id: Uuid,

    /// Unique template name (e.g., "daily-note", "project-summary")
    pub name: String,

    /// Template content (MiniJinja-compatible syntax)
    pub content: String,

    /// Variable definitions with types and constraints
    pub variables: HashMap<String, VariableDefinition>,

    /// Optional parent template for composition
    pub extends: Option<String>,

    /// Metadata for template management
    pub metadata: TemplateMetadata,
}

/// Template metadata for management and validation
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateMetadata {
    /// Template description
    pub description: Option<String>,

    /// Template version for compatibility
    pub version: Option<String>,

    /// Tags for template categorization
    pub tags: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
}
```

**VariableDefinition Enum:**
```rust
/// Type-safe variable definition with validation constraints
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum VariableDefinition {
    /// String variable with optional constraints
    String {
        default: Option<String>,
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<String>,
    },

    /// Number variable with range constraints
    Number {
        default: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
    },

    /// Boolean variable (simple true/false)
    Boolean {
        default: Option<bool>,
    },

    /// Date variable with format specification
    Date {
        default: Option<String>,
        format: Option<String>, // ISO 8601 format string
    },

    /// File reference variable with type constraints
    File {
        default: Option<String>,
        file_types: Option<Vec<String>>, // ["image", "pdf", "note", etc.]
    },
}
```

**TemplateComposition for Modular Assembly:**
```rust
/// Template composition for modular template building
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateComposition {
    /// Base template name
    pub base_template: String,

    /// Variable overrides for base template
    pub variable_overrides: HashMap<String, serde_json::Value>,

    /// Additional content sections to append
    pub additional_sections: Vec<TemplateSection>,

    /// Child templates to include
    pub includes: Vec<String>,
}

/// Template section for composition
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateSection {
    /// Section name for reference
    pub name: String,

    /// Section content
    pub content: String,

    /// Insertion point (before/after variable)
    pub position: InsertionPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InsertionPosition {
    BeforeVariable(String),  // Insert before named variable
    AfterVariable(String),   // Insert after named variable
    Beginning,               // Insert at template start
    End,                     // Insert at template end
}
```

### MiniJinja Compatibility Requirements

**DOMAIN LAYER RESPONSIBILITIES:**
- **Variable Definitions**: Type-safe variable specifications with constraints
- **Composition Logic**: Modular template assembly and dependency resolution
- **Business Rules**: Variable naming, composition cycles, semantic consistency
- **Content Storage**: Template content stored as opaque strings (no parsing)

**ADAPTER LAYER RESPONSIBILITIES:**
- **Syntax Validation**: MiniJinja-compatible syntax checking (variable references, control structures, expressions)
- **Template Rendering**: Actual MiniJinja engine integration for content generation
- **Structure Validation**: Balanced blocks, variable scoping, expression validity
- **Filter Compatibility**: Supported MiniJinja filters and custom filter validation

**Semantic Validation Distribution:**
- **DOMAIN**: Variable references match defined variables, type consistency, composition cycles
- **ADAPTER**: Template syntax is valid MiniJinja, expressions parse correctly, filters are supported
- **INTEGRATION**: Combined validation ensures templates are both semantically correct and syntactically valid

### Architecture Compliance - MANDATORY READING

**Hexagonal Boundary Enforcement:**
- **DOMAIN LAYER**: Template entities, variable definitions, composition business rules, domain validation only
- **ADAPTER LAYER**: MiniJinja syntax validation, template rendering, file I/O, external format parsing
- **PORTS**: Clean interfaces between domain and adapters for template operations
- **ZERO IO CONCERNS**: Domain never touches files, network, or external systems
- **PURE BUSINESS LOGIC**: Domain focuses on template structure and composition rules
- **ADAPTER ISOLATION**: MiniJinja dependencies and syntax validation isolated in adapter layer

**Standard Traits - REQUIRED:**
```rust
// ALWAYS derive these for domain entities:
#[derive(Debug, Clone, PartialEq)]
// Add Serialize/Deserialize for template persistence
// Use custom implementations for complex validation

// Advanced Rust Patterns:
// - Use associated types in repository ports for type-safe APIs
// - Consider GATs for async iterators in future template streaming
```

**Conversion Traits - MANDATORY:**
- Use `From/Into` for converting between template types and composition
- Use `TryFrom/TryInto` for validation during template construction
- NEVER create ad-hoc `to_x()` methods

**Exhaustive Matching:**
```rust
// Use #[non_exhaustive] on domain enums
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum VariableDefinition {
    String { /* fields */ },
    Number { /* fields */ },
    Boolean { /* fields */ },
    Date { /* fields */ },
    File { /* fields */ },
}

// PROHIBIT catch-all patterns in domain logic:
match var_def {
    VariableDefinition::String { .. } => { /* handle */ },
    VariableDefinition::Number { .. } => { /* handle */ },
    // NO: _ => {} catch-alls!
}
```

### Validation Rules

**Template Name Validation:**
- MUST be non-empty and valid UTF-8
- MUST match regex: `^[a-zA-Z0-9_-]+$` (alphanumeric, underscore, dash)
- MAX length: 64 characters
- Example valid: `"daily-note"`, `"project_summary"`, `"meeting-notes-2024"`
- Example invalid: `"Daily Note"`, `"meeting notes"`, `""`, `"template--name"`

**Variable Name Validation:**
- MUST be non-empty and valid UTF-8
- MUST match regex: `^[a-zA-Z_][a-zA-Z0-9_]*$` (valid identifier)
- MAX length: 32 characters per variable
- Cannot be MiniJinja reserved words (`if`, `for`, `true`, `false`, etc.)
- Example valid: `"title"`, `"created_at"`, `"is_completed"`
- Example invalid: `"title-text"`, `"123invalid"`, `""`, `"if"`

**Content Size Limits:**
- Template content: MAX 1MB (reasonable for text templates)
- Variable count: MAX 50 variables per template
- Variable value size: MAX 10KB per variable
- Include depth: MAX 5 levels (prevent infinite recursion)

**Domain Validation Rules (Business Logic):**
- **Variable Naming**: `^[a-zA-Z_][a-zA-Z0-9_]*$` regex, no reserved words
- **Composition Cycles**: Prevent circular template dependencies
- **Type Consistency**: Variable definitions match usage patterns
- **Size Limits**: Reasonable bounds on template complexity

**Adapter Validation Rules (Technical Implementation):**
- **MiniJinja Syntax**: Variable references `{{variable}}`, control structures, expressions
- **Block Balancing**: Proper nesting of `{% if %}`, `{% for %}`, etc.
- **Expression Parsing**: Valid MiniJinja expressions with correct precedence
- **Filter Support**: Compatible filters with proper argument types

### Subentity Examples

**Template Creation Example:**
```rust
let template = Template::new(
    "daily-note".to_string(),
    r#"
# Daily Note - {{date}}

## Tasks
{% for task in tasks %}
- [ ] {{task}}
{% endfor %}

## Notes
{{notes}}
"#.to_string(),
    HashMap::from([
        ("date".to_string(), VariableDefinition::Date {
            default: Some("today".to_string()),
            format: Some("%Y-%m-%d".to_string()),
        }),
        ("tasks".to_string(), VariableDefinition::String {
            default: Some("".to_string()),
            min_length: Some(0),
            max_length: Some(1000),
            pattern: None,
        }),
        ("notes".to_string(), VariableDefinition::String {
            default: Some("".to_string()),
            min_length: Some(0),
            max_length: Some(5000),
            pattern: None,
        }),
    ]),
    TemplateMetadata {
        description: Some("Daily note template with tasks and notes".to_string()),
        version: Some("1.0.0".to_string()),
        tags: vec!["daily".to_string(), "productivity".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    },
)?;
```

**Template Composition Example:**
```rust
let composition = TemplateComposition {
    base_template: "daily-note".to_string(),
    variable_overrides: HashMap::from([
        ("date".to_string(), serde_json::Value::String("2024-01-15".to_string())),
    ]),
    additional_sections: vec![
        TemplateSection {
            name: "reflection".to_string(),
            content: "\n## Daily Reflection\n\nWhat went well today?\n{{reflection}}".to_string(),
            position: InsertionPosition::End,
        },
    ],
    includes: vec![],
};

// This creates a composed template with the base template + reflection section
```

**Variable Validation Examples:**
```rust
// String variable with constraints
let title_var = VariableDefinition::String {
    default: Some("Untitled".to_string()),
    min_length: Some(1),
    max_length: Some(200),
    pattern: Some(r"^[A-Z][^.!?]*[.!?]?$".to_string()), // Starts with capital, ends with punctuation
};

// Number variable with range
let priority_var = VariableDefinition::Number {
    default: Some(3.0),
    min: Some(1.0),
    max: Some(5.0),
};

// File variable with type constraints
let attachment_var = VariableDefinition::File {
    default: None,
    file_types: Some(vec!["image".to_string(), "pdf".to_string()]),
};
```

### Testing Requirements

**Domain Tests (Pure Unit Tests):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_business_rules() {
        let template = Template::new(
            "test".to_string(),
            "{{valid_syntax}}".to_string(),
            HashMap::new(),
            TemplateMetadata::default(),
        );

        // Domain validation should pass (syntax validation happens in adapter)
        assert!(template.is_ok());

        // Test business rule violations
        let invalid_name_template = Template::new(
            "invalid name with spaces".to_string(),
            "{{variable}}".to_string(),
            HashMap::new(),
            TemplateMetadata::default(),
        );

        // Should fail domain business rule validation
        assert!(invalid_name_template.is_err());
    }

    #[test]
    fn test_variable_reference_validation() {
        let content = "{{undefined_variable}}";
        let template = Template::new(
            "test".to_string(),
            content.to_string(),
            HashMap::new(), // No variables defined
            TemplateMetadata::default(),
        );

        // Should fail semantic validation
        assert!(template.is_err());
    }

    #[test]
    fn test_template_composition() {
        let base_template = create_test_template();
        let composition = TemplateComposition {
            base_template: "base".to_string(),
            variable_overrides: HashMap::new(),
            additional_sections: vec![create_test_section()],
            includes: vec![],
        };

        let result = Template::compose(&base_template, &composition);
        assert!(result.is_ok());
    }
}
```

**Test Coverage Target:**
- **90%+ coverage** for Template domain entities and validation logic
- Test both success and error cases for syntax and semantic validation
- Property-based testing for template syntax variations and edge cases
- Integration tests for template composition and variable resolution

**Test Fixtures Strategy:**
```rust
#[cfg(test)]
pub mod fixtures {
    use super::*;

    pub fn sample_template() -> Template {
        Template::new(
            "daily-note".to_string(),
            "# {{title}}\n\n{{content}}".to_string(),
            HashMap::from([
                ("title".to_string(), VariableDefinition::String {
                    default: Some("Daily Note".to_string()),
                    min_length: Some(1),
                    max_length: Some(100),
                    pattern: None,
                }),
                ("content".to_string(), VariableDefinition::String {
                    default: Some("".to_string()),
                    min_length: Some(0),
                    max_length: Some(5000),
                    pattern: None,
                }),
            ]),
            TemplateMetadata {
                description: Some("Sample daily note template".to_string()),
                version: Some("1.0.0".to_string()),
                tags: vec!["sample".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ).expect("Valid template")
    }

    pub fn sample_composition() -> TemplateComposition {
        TemplateComposition {
            base_template: "daily-note".to_string(),
            variable_overrides: HashMap::from([
                ("title".to_string(), serde_json::Value::String("My Daily Note".to_string())),
            ]),
            additional_sections: vec![],
            includes: vec![],
        }
    }
}
```

**Performance Testing:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_template_validation(c: &mut Criterion) {
    let template = fixtures::sample_template();

    c.bench_function("template_business_rules_validation", |b| {
        b.iter(|| {
            let result = template.validate_business_rules();
            black_box(result);
        });
    });
    // Target: <100ms for typical domain validation
}

fn bench_template_rendering(c: &mut Criterion) {
    let template_content = include_str!("../fixtures/large_template.md");
    let adapter = TemplateRendererAdapter::new();

    c.bench_function("template_syntax_validation_adapter", |b| {
        b.iter(|| {
            let result = adapter.validate_minijinja_syntax(black_box(template_content));
            black_box(result);
        });
    });
    // Target: <500ms for typical template syntax validation (adapter layer)
}

fn bench_template_composition(c: &mut Criterion) {
    let base = fixtures::sample_template();
    let composition = fixtures::sample_composition();

    c.bench_function("template_composition", |b| {
        b.iter(|| {
            let result = Template::compose(black_box(&base), black_box(&composition));
            black_box(result);
        });
    });
    // Target: <100ms for template composition
}
```

### File Structure Requirements

**Single File Structure (Split at 1000+ Lines):**
```
crates/domain/src/
├── lib.rs                    # Public API surface, re-exports
├── models/
│   ├── mod.rs               # Module declarations
│   └── template.rs          # All Template entities, variables, composition logic
├── ports/
│   ├── mod.rs               # Port trait declarations
│   └── template.rs          # TemplateCommand/TemplateQuery traits (shells)
└── errors.rs                # Domain errors (EXTENDED with template errors)
```

**Splitting Guideline:** Start with single file. Split when >1000 lines into template_core.rs, template_variables.rs, template_composition.rs.

**Implementation Decision:**
Use **subfolder organization** for Template bounded context due to complexity of validation logic, composition system, and MiniJinja compatibility requirements.

**Naming Conventions - STRICT:**
- Files: `snake_case.rs`
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions/Variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Traits: `PascalCase` with `Port` suffix for ports

### Code Quality Standards

**Clippy Complexity Limits - ENFORCED:**
- Cognitive complexity: **max 25 (deny)**
- Function length: **max 100 lines (deny)**
- Keep template validation and composition logic composable

**MANDATORY Quality Gates (Task 7):**
- **NO EXCEPTIONS:** All clippy warnings MUST be fixed (no bypassing)
- **NO EXCEPTIONS:** All pre-commit hooks MUST pass (no bypassing)
- **MANDATORY:** Run `mise run verify` for comprehensive quality assurance
- **MANDATORY:** Commit blocked until all quality gates pass
- **MANDATORY:** Conventional commit message required for final commit

**Formatting:**
- Run `mise run verify` before committing
- Pre-commit hooks enforce formatting
- Import grouping: `StdExternalCrate`

## Dev Notes

### Project Context Integration

**Current Codebase State:**
- Workspace structure exists at `crates/domain/`, `crates/app/`, `crates/adapters/`, `crates/cli/`
- Domain crate has basic error types and ports structure
- Stories 3.1 (Note), 3.2 (Schema), 3.3 (Config) completed - Template is the final domain model
- Epic 11 will implement template execution using this domain model

**Technology Stack (from project-context.md):**
- **Rust 1.92+**: Memory safety, zero-cost abstractions
- **MiniJinja 1.0**: Template engine for rendering (adapter layer usage)
- **serde 1.0**: Template serialization/deserialization
- **thiserror 2.0**: Structured domain error definitions
- **chrono 0.4**: Date/time handling for template variables

**Critical Anti-Patterns to AVOID:**
- ❌ Using `unwrap()`, `expect()`, `todo()`, `panic!()` in production code
- ❌ Using `as` casting (use `.try_into().expect("...")` or proper error handling)
- ❌ Leaking MiniJinja logic into domain (no syntax validation or rendering in domain)
- ❌ Creating ad-hoc conversion methods instead of From/TryFrom traits
- ❌ Using catch-all `_ => {}` patterns in exhaustive domain logic matches

### Architecture Intelligence

**MiniJinja Template Engine Integration:**
- **Domain Layer**: Syntax validation and semantic checking (compatible with MiniJinja)
- **Adapter Layer**: Actual template rendering using MiniJinja engine
- **Validation Scope**: Domain validates template structure, adapter handles rendering
- **Performance Targets**: <500ms template validation, <100ms composition
- **Compatibility**: Must support MiniJinja 1.0 syntax and features

**Template Bounded Context Requirements:**
- **Modular Composition**: Templates can extend and include other templates
- **Variable System**: Type-safe variables with validation constraints
- **Syntax Validation**: Ensure templates are valid MiniJinja syntax
- **Semantic Validation**: Variable references match defined variables
- **Performance Bounds**: Reasonable limits on template size and complexity

### Implementation Strategy

**TDD Domain Validation (Business Rules Only):**
```rust
impl Template {
    /// Validate domain business rules (no syntax validation)
    pub fn validate_domain_rules(&self) -> Result<(), TemplateError> {
        // 1. Validate variable names follow naming conventions
        // 2. Check for composition cycles
        // 3. Validate variable type consistency
        // 4. Check size limits and complexity constraints
        // 5. Return domain validation errors
        unimplemented!("TDD implementation required")
    }
}

impl TemplateComposition {
    /// Validate composition business rules
    pub fn validate_composition(&self, available_templates: &HashSet<String>) -> Result<(), TemplateError> {
        // 1. Check all referenced templates exist
        // 2. Validate composition cycles
        // 3. Check variable override compatibility
        // 4. Return composition validation errors
        unimplemented!("TDD implementation required")
    }
}
```

**TDD Adapter Validation (MiniJinja Integration):**
```rust
// In adapter layer (crates/adapters/src/)
pub struct TemplateAdapter;

impl TemplateAdapter {
    /// Validate MiniJinja syntax (adapter responsibility)
    pub fn validate_syntax(content: &str) -> Result<(), TemplateAdapterError> {
        // 1. Parse with MiniJinja engine
        // 2. Check for syntax errors
        // 3. Validate supported features
        // 4. Return syntax validation errors
        unimplemented!("Adapter implementation required")
    }

    /// Render template with MiniJinja (adapter responsibility)
    pub async fn render_template(&self, template: &Template, variables: &HashMap<String, serde_json::Value>) -> Result<String, TemplateAdapterError> {
        // 1. Create MiniJinja environment
        // 2. Compile template
        // 3. Render with provided variables
        // 4. Return rendered content
        unimplemented!("Adapter implementation required")
    }
}
```

**Composition Algorithm:**
```rust
impl Template {
    /// Compose template from base template and composition
    pub fn compose(base: &Template, composition: &TemplateComposition) -> Result<Template, TemplateError> {
        // 1. Start with base template content
        // 2. Apply variable overrides
        // 3. Insert additional sections at specified positions
        // 4. Include child templates
        // 5. Validate final composition
        // 6. Return composed template
        unimplemented!("TDD implementation required")
    }
}
```

### Cross-Story Dependencies

**Prerequisites:**
- ✅ Epic 1 completed (workspace, tooling, quality gates)
- ✅ Stories 3.1-3.3 completed (Note, Schema, Config bounded contexts established)
- ✅ Epic 2 completed (test patterns established)

**Enables Future Stories:**
- **Epic 11**: Template execution and rendering (uses this domain model)
- **Epic 6**: Schema integration (templates reference schemas for variable types)
- **Epic 13**: CLI template commands (uses template domain for validation)
- **Epic 9**: Vault operations (templates used for note generation)

**Integration Points:**
- **Template Execution (Epic 11)**: Adapters render templates using MiniJinja
- **Schema Integration (Epic 6)**: Template variables can reference schema types
- **CLI Commands (Epic 13)**: Template validation before execution
- **Vault Operations (Epic 9)**: Templates used to generate new notes

### Epic 2 Test Infrastructure Integration
**Planned Integration with Epic 2 Test Utils:**
This story will leverage the test utilities being developed in Epic 2:
- **Story 2-4**: Centralized test utilities and infrastructure (artifact management, isolation)
- **Story 2-6**: Integration testing patterns and infrastructure (cross-crate testing, external service mocking)
- **Story 2-7**: Benchmarking infrastructure and performance testing patterns (criterion integration, regression detection)
- **Dependency**: Epic 2 completion required before implementing comprehensive testing in this story
- **Integration Points**: Use shared test fixtures for template entities, mock MiniJinja environments, and performance benchmarking utilities

### References

**Architecture Documents:**
- [Source: _bmad-output/planning-artifacts/architecture.md#Template Engine]
  - MiniJinja integration with adapter layer syntax validation
  - Performance targets (<500ms operations, <2s for complex rendering)
  - Hexagonal architecture with template ports and adapters
- [Source: _bmad-output/planning-artifacts/architecture.md#Domain Layer]
  - Template bounded context in domain layer with validation
  - CQRS separation and event-driven template lifecycle
  - Error handling and validation layer separation

**Epic Context:**
- [Source: _bmad-output/planning-artifacts/epics/epic-3-core-domain-models-value-objects-phase-15.md#Story 3.4]
  - Complete acceptance criteria for Template bounded context
  - Template entity with structure validation and business rule requirements
  - Semantic validation for template syntax and variable references
  - Modular composition and variable definition support

**PRD Requirements:**
- [Source: _bmad-output/planning-artifacts/prd.md#Template Management FR1-FR7]
  - Template creation, execution, composition, and date function requirements
  - Modular templates with variable definitions and custom functions
  - Interactive template execution with CLI-first design

**Previous Story Learnings:**
- [Source: _bmad-output/implementation-artifacts/stories/3-1-create-note-bounded-context.md]
  - Domain entity patterns with validation and error handling
  - TDD framework implementation with red-green-refactor phases
  - File structure conventions and naming standards
  - Quality assurance task structure and requirements

**Project Context:**
- [Source: _bmad-output/project-context.md#Critical Implementation Rules]
  - Hexagonal architecture boundary enforcement
  - MiniJinja template engine integration patterns
  - Error handling with thiserror and proper error chaining
  - Quality gates (clippy cognitive complexity <25, no unwrap/expect)

## Dev Agent Record

### Agent Model Used

<!-- Dev agent will fill this in during implementation -->

### Debug Log References

<!-- Dev agent will add references to logs if debugging is needed -->

### Completion Notes List

<!-- Dev agent will document completion status and any deviations -->

### File List

<!-- Dev agent will list all files created/modified during implementation -->
```
Expected files to be created (7 TDD tasks for 3-4):
- crates/domain/src/errors.rs (EXTENDED with template error variants)
- crates/domain/src/models/mod.rs (UPDATED with template module declaration)
- crates/domain/src/models/template/mod.rs (re-exports Template, VariableDefinition, TemplateComposition)
- crates/domain/src/models/template/template.rs (Template aggregate root with validation)
- crates/domain/src/models/template/variable.rs (VariableDefinition enum with type constraints)
- crates/domain/src/models/template/composition.rs (TemplateComposition for modular assembly)
- crates/domain/src/models/template/validation.rs (Domain business rule validation)
- crates/domain/src/ports/template.rs (TemplatePort trait - future adapter integration)
- crates/domain/src/lib.rs (UPDATED with public template re-exports)
- crates/domain/Cargo.toml (UPDATED with serde, chrono dependencies)
- benches/template_benchmarks.rs (performance benchmarks - optional)

Comprehensive tests in each file with #[cfg(test)] modules (90%+ coverage target)
```
