# Story 3.1: Create Note Bounded Context

Status: ready-for-dev

<!-- This story file contains COMPREHENSIVE context to prevent developer mistakes, omissions, and disasters -->

## Story

As a developer working with note data,
I want a comprehensive Note aggregate with all subentities,
So that the domain accurately represents the rich structure of notes in Obsidian vaults.

## Acceptance Criteria

**Given** I have researched Obsidian note structures and wiki-link patterns
**When** I review the Note bounded context
**Then** the Note aggregate includes these subentities:
- Note (main entity with identity and metadata)
- Frontmatter (YAML metadata with fields)
- Links (wiki-links, aliases, and references)
- Embeds (embedded content references)
- Tags (hierarchical tag system)
- Headings (document structure)
- Tasks (task management with status)
- Sections (content organization)

**Given** the Note aggregate is defined
**When** I validate entity relationships
**Then** Frontmatter is a subentity of Note (Note contains Frontmatter)

**Given** semantic validation is integrated
**When** I create a Note instance
**Then** internal consistency validation occurs (semantic validation per entity)

**Given** I have researched Obsidian vault patterns
**When** I check the Note entity design
**Then** it supports vault-relative paths and wiki-link resolution

## Tasks / Subtasks (TDD Framework: Red-Green-Refactor)

### Task 1: Define Domain Tests First (RED Phase - AC: All)
- [ ] **STRICT NAMING:** All tests MUST use verb-first behavioral naming (e.g., `returns_error_when_path_is_empty` NOT `test_empty_path`)
- [ ] Write failing unit tests for Frontmatter entity (test validation, construction, invariants)
- [ ] Write failing unit tests for FrontmatterValue enum (test type conversions, edge cases)
- [ ] Write failing unit tests for Link entity (test wiki-link parsing, position tracking)
- [ ] Write failing unit tests for Embed entity (test file type classification, validation)
- [ ] Write failing unit tests for Tag entity (test hierarchical parsing, regex validation)
- [ ] Write failing unit tests for Heading entity (test level validation 1-6, position tracking)
- [ ] Write failing unit tests for Task entity (test status enum, markdown parsing)
- [ ] Write failing unit tests for Section entity (test range calculation, heading association)
- [ ] Write failing integration tests for Note aggregate (test composition, validation pipeline)
- [ ] **VIRTUAL TIME:** Use `time_test!` macro for validating Note `created_at`/`updated_at` timestamps
- [ ] **Quality Assurance Subtask:** Run `mise run lint`, fix ALL linter errors/warnings, #[allow] MUST NOT be used unless all other options have been exhausted, in which case provide full justification of why it could not be fixed otherwise
- [ ] Write failing property-based tests for edge cases (empty strings, boundary values, invalid formats)
- [ ] **TDD REQUIREMENT:** All tests MUST fail initially (RED phase complete when tests fail as expected)

### Task 2: Implement Domain Entities (GREEN Phase - AC: 1-5)
- [ ] Implement Frontmatter entity with HashMap<String, FrontmatterValue> and validation logic
- [ ] Implement FrontmatterValue enum with proper type conversions and validation
- [ ] Implement Link entity with source/target references, alias handling, position tracking
- [ ] Implement Embed entity with file type classification and validation constraints
- [ ] Implement Tag entity with hierarchical path parsing and regex validation (^[a-zA-Z0-9_-]+)
- [ ] Implement Heading entity with level validation (1-6) and position tracking
- [ ] Implement Task entity with status enum (Incomplete, Complete, Cancelled) and position tracking
- [ ] Implement Section entity with content range calculation and optional heading reference
- [ ] Implement Note aggregate root with UUID v7 identity generation and vault-relative path validation
- [ ] **VIRTUAL CLOCK:** Integrate with virtual clock infrastructure for deterministic timestamp generation
- [ ] Implement semantic validation pipeline (Syntactic → Orchestration → Semantic)
- [ ] **TDD REQUIREMENT:** Make all previously failing tests pass (GREEN phase complete when all tests pass)

### Task 3: Implement Domain Error Types (GREEN Phase - AC: All)
- [ ] Implement comprehensive DomainError enum with thiserror::Error derives
- [ ] Add error variants for path validation (InvalidPath, EmptyPath, non-relative paths, missing .md extension)
- [ ] Add error variants for entity validation (InvalidTag, InvalidHeadingLevel, EmptyLinkTarget, etc.)
- [ ] Add error variants for business rules (ValidationFailed, semantic consistency errors)
- [ ] Implement error conversion traits (From/Into) for domain boundaries
- [ ] Write unit tests for error message clarity, accuracy, and proper error chaining
- [ ] **TDD REQUIREMENT:** All error-related tests must pass

### Task 4: Refactor for Quality (REFACTOR Phase - AC: All)
- [ ] Extract common validation logic into reusable functions (<25 cognitive complexity)
- [ ] Optimize memory usage (Box<str> for immutable strings, avoid unnecessary allocations)
- [ ] Ensure proper ownership patterns (immutable entities, no internal mutation)
- [ ] Add comprehensive documentation with invariants, examples, and error conditions
- [ ] Implement performance optimizations (pre-allocated collections, efficient string handling)
- [ ] Verify hexagonal architecture compliance (no external dependencies, proper boundary separation)
- [ ] **TDD REQUIREMENT:** All tests still pass after refactoring (no regressions)

### Task 5: Comprehensive Testing Coverage (RED-GREEN-REFACTOR - AC: All)
- [ ] Achieve 90%+ test coverage for all domain entities and validation logic
- [ ] **FACTORY MACROS:** Use `test_builder!` macro for constructing Note aggregate examples in fixtures
- [ ] Create test fixtures module with deterministic examples (fixed UUIDs, predictable data)
- [ ] Implement property-based testing with proptest for edge cases and boundary conditions
- [ ] Add integration tests for Note aggregate with realistic subentity combinations
- [ ] Add performance benchmarks (<100μs Note construction, <10μs validation)
- [ ] **TDD REQUIREMENT:** Coverage reports show 90%+ coverage, all property-based tests pass

### Task 6: Documentation and Integration (REFACTOR Phase - AC: All)
- [ ] Update domain crate lib.rs with proper public API surface and re-exports
- [ ] Add comprehensive doc comments following project standards (invariants, examples, errors)
- [ ] Ensure all entities derive required traits (Debug, Clone, PartialEq, serde optional)
- [ ] Verify integration points with future bounded contexts (storage adapters, application layer)
- [ ] Update Cargo.toml with required dependencies (uuid, thiserror, blake3, optional serde)
- [ ] **TDD REQUIREMENT:** All documentation examples compile and run successfully

### Task 7: Quality Assurance and Commit (MANDATORY FINAL TASK - TDD Validation)
- [ ] **TDD VALIDATION:** Confirm all tests pass and coverage meets 90%+ requirement
- [ ] **TDD VALIDATION:** Verify property-based tests catch edge cases appropriately
- [ ] **TDD VALIDATION:** Ensure performance benchmarks meet targets (<100μs Note construction)
- [ ] Run `mise run fmt` to format all code according to project standards
- [ ] Run `mise run lint` to check for all code quality issues and anti-patterns
- [ ] Run `mise run verify` for comprehensive verification (fmt + lint + tests + coverage)
- [ ] Run `pre-commit run --all-files` to execute all pre-commit hooks
- [ ] **CRITICAL:** Fix ALL linter warnings - NO EXCEPTIONS, NO BYPASSING (TDD requires clean code)
- [ ] **CRITICAL:** Ensure ALL pre-commit hooks pass - NO EXCEPTIONS, NO BYPASSING
- [ ] **MANDATORY:** If any warnings or hook failures exist, fix them immediately and re-run verification
- [ ] **MANDATORY:** Confirm all domain entities pass clippy cognitive complexity limits (<25)
- [ ] **MANDATORY:** Verify no `unwrap()`, `expect()`, `todo()`, `panic!()` remain in production code
- [ ] **MANDATORY:** Verify hexagonal architecture boundaries maintained (no external dependencies)
- [ ] Stage all files created or modified during story development
- [ ] Commit with conventional commit message: `feat: implement note bounded context with comprehensive subentities and TDD validation`

## Technical Requirements

### Domain Model Foundation

**Core Entity Structure:**
- **Note Entity**: Main aggregate root with UUID v7 identity (time-ordered, sortable)
- **Immutability**: All domain entities MUST be immutable following Rust ownership patterns
- **Validation**: Three-phase validation pipeline (Syntactic → Orchestration → Semantic)
- **Error Handling**: Use `thiserror::Error` for typed domain errors

**Identity Pattern - CRITICAL:**
- Use **UUID v7** for Note identity (NOT vault path as primary key)
- UUID v7 provides time-ordered, sortable identifiers that are stable during file renames
- Vault-relative path stored as separate field for filesystem correspondence
- This prevents the "directory trap" per Architecture ADR 0002

**Domain Purity Requirements - CRITICAL:**
- Domain crate has ZERO external dependencies (only std lib + optional serde for serialization)
- **PURITY GUARDIAN:** Compliance is enforced by the `Domain Purity Guardian` automated test
- NO I/O operations in domain layer
- NO `rkyv` in domain dependencies - persistence derives belong in storage adapter DTOs
- Use `pub(crate)` by default; `pub` only for crate's public interface
- All traits defined in `domain/src/ports/` directory

**Persistence Strategy:**
- Domain entities remain pure and dependency-free
- Storage adapters (`adapters/spi/storage`) create separate DTOs with `rkyv` derives
- Use `From/Into` traits to convert between domain entities and storage DTOs
- Consider `Arc<str>` for shared immutable strings (paths, tags) to reduce memory usage in large vaults
- This preserves hexagonal architecture while enabling zero-copy deserialization

### Subentity Specifications

**Frontmatter Subentity:**
```rust
// MUST be immutable, owned by Note
// Represents YAML metadata extracted from note headers
pub struct Frontmatter {
    fields: HashMap<String, FrontmatterValue>,
    // Validation state tracked internally
}

pub enum FrontmatterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(String),  // ISO 8601 string (e.g., "2024-01-15" or "2024-01-15T10:30:00Z")
                   // Parsed to chrono types in adapter layer; domain stores as validated string
    Array(Vec<FrontmatterValue>),
    Object(HashMap<String, FrontmatterValue>), // For nested YAML objects
}
```

**Links Subentity:**
```rust
// Track wiki-links [[target]], aliases [[target|alias]], and references
pub struct Link {
    source_note_id: Uuid,      // UUID v7 reference
    target_path: String,       // Vault-relative path
    alias: Option<String>,     // Display text if different from target
    link_type: LinkType,       // WikiLink only (Embed is separate)
    position: usize,           // Offset in document for miette diagnostics
}

pub enum LinkType {
    WikiLink,     // [[target]] or [[target|alias]]
    // Other link types as discovered
}
```

**Embeds Subentity:**
```rust
// Track embedded content ![[file]]
pub struct Embed {
    source_note_id: Uuid,      // UUID v7 reference
    target_path: String,       // Vault-relative path to embedded file
    embed_type: EmbedType,     // Image, PDF, Note, etc.
    position: usize,           // Offset in document for miette diagnostics
}

pub enum EmbedType {
    Image,        // ![[image.png]]
    Pdf,          // ![[document.pdf]]
    Note,         // ![[another-note]]
    Audio,        // ![[audio.mp3]]
    Video,        // ![[video.mp4]]
    // Extensible for other file types
}
```

**Tags Subentity:**
```rust
// Hierarchical tag system #parent/child/grandchild
pub struct Tag {
    full_path: String,         // e.g., "project/work/urgent"
    segments: Vec<String>,     // ["project", "work", "urgent"]
}
```

**Headings Subentity:**
```rust
// Document structure from markdown headings
pub struct Heading {
    level: u8,                 // 1-6 for # to ######
    text: String,
    position: usize,           // Offset in document for miette diagnostics
}
```

**Tasks Subentity:**
```rust
// Task items with status tracking
pub struct Task {
    text: String,
    status: TaskStatus,
    position: usize,           // For error reporting with miette
}

pub enum TaskStatus {
    Incomplete,
    Complete,
    Cancelled,
    // Extensible for custom statuses
}
```

**Sections Subentity:**
```rust
// Content organization between headings
pub struct Section {
    heading: Option<Heading>,  // None for content before first heading
    content: String,
    range: std::ops::Range<usize>,
}
```

**Note Aggregate Root:**
```rust
// Main aggregate containing all subentities
pub struct Note {
    id: Uuid,                  // UUID v7 primary identity
    path: String,              // Vault-relative path (e.g., "projects/lithos.md")
    frontmatter: Option<Frontmatter>,  // YAML metadata (optional)
    links: Vec<Link>,          // Wiki-links found in content
    embeds: Vec<Embed>,        // Embedded files
    tags: Vec<Tag>,            // Hierarchical tags
    headings: Vec<Heading>,    // Document structure
    tasks: Vec<Task>,          // Task items
    sections: Vec<Section>,    // Content sections
    // Note: Raw markdown content is NOT stored in domain model
    // Content is parsed to extract subentities, then discarded
    // Storage layer may persist content separately if needed
}
```

### Architecture Compliance - MANDATORY READING

**Hexagonal Boundary Enforcement:**
- Domain crate in `crates/domain/src/` with ZERO external dependencies
- All ports (traits) in `domain/src/ports/` using `#[async_trait]` for async methods
- NO direct references to adapters, app, or infrastructure concerns
- Use `pub(crate)` for internal types, `pub` only for public API surface

**Standard Traits - REQUIRED:**
```rust
// ALWAYS derive these for domain entities:
#[derive(Debug, Clone, PartialEq)]
// Add Default where appropriate
// Use custom implementations for complex logic

// Advanced Rust Patterns:
// - Consider Arc<str> for shared immutable strings in large aggregates
// - Use associated types in port traits for repository operations
```

**Conversion Traits - MANDATORY:**
- Use `From/Into` for infallible conversions
- Use `TryFrom/TryInto` for fallible conversions
- NEVER create ad-hoc `to_x()` methods

**Exhaustive Matching:**
```rust
// Use #[non_exhaustive] on domain enums
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DomainError {
    // variants
}

// PROHIBIT catch-all patterns in domain logic:
match error {
    DomainError::Variant1 => { /* handle */ },
    DomainError::Variant2 => { /* handle */ },
    // NO: _ => {} catch-alls!
}
```

**Error Standards:**
- Use `thiserror` for domain error types
- Every error variant must have descriptive message
- Use `#[from]` attribute for error conversions
- NO `unwrap()`, `expect()`, `todo()`, or `unimplemented()` in domain code

**Required Error Variants:**
```rust
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum DomainError {
    #[error("Invalid note path: {0}")]
    InvalidPath(String),

    #[error("Path cannot be empty")]
    EmptyPath,

    #[error("Invalid frontmatter field type: expected {expected}, got {actual}")]
    InvalidFrontmatterType { expected: String, actual: String },

    #[error("Invalid tag format: {0}")]
    InvalidTag(String),

    #[error("Tag segment cannot be empty")]
    EmptyTagSegment,

    #[error("Invalid heading level: {0} (must be 1-6)")]
    InvalidHeadingLevel(u8),

    #[error("Invalid task status: {0}")]
    InvalidTaskStatus(String),

    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),

    #[error("Link target path cannot be empty")]
    EmptyLinkTarget,

    #[error("Embed target path cannot be empty")]
    EmptyEmbedTarget,

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}
```

**Serde Serialization (Optional):**
```rust
// Domain entities MAY derive serde traits for JSON/YAML serialization
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    // fields
}
// This is optional - add if needed for JSON APIs or config files
```

**Memory Strategy:**
- Use `Box<str>` for small immutable strings (paths, identifiers) to save heap allocation overhead
- Use `Arc<str>` for shared immutable strings (common tags, paths) across notes to reduce memory duplication
- Use `String` for all mutable or frequently modified text content
- Avoid `Cow<'a, str>` in pure domain models (adds lifetime complexity without clear benefit)
- Reserve advanced memory optimization for storage adapter layer if profiling shows need

### Validation Rules

**Path Validation:**
- MUST be non-empty
- MUST be valid UTF-8
- MUST be relative (no leading `/` or drive letters)
- MAX length: 4096 characters
- MUST end with `.md` extension
- Example valid: `"projects/lithos-development.md"`
- Example invalid: `"/absolute/path.md"`, `""`, `"no-extension"`

**Tag Validation:**
- Format: `#segment1/segment2/segment3`
- Each segment MUST match regex: `^[a-zA-Z0-9_-]+$`
- NO empty segments (e.g., `#project//urgent` is invalid)
- NO leading or trailing slashes
- MIN 1 segment, MAX 10 segments (practical limit)
- Example valid: `"work/project/urgent"`, `"personal"`, `"tech-stack/rust"`
- Example invalid: `"invalid segment"`, `"project//sub"`, `""`, `"/leading"`

**Heading Level Validation:**
- MUST be in range 1-6 inclusive (corresponding to `#` through `######`)
- Example valid: `1`, `6`
- Example invalid: `0`, `7`, `255`

**Frontmatter Date Validation:**
- MUST be ISO 8601 format string
- Date-only: `"YYYY-MM-DD"` (e.g., `"2024-01-15"`)
- DateTime: `"YYYY-MM-DDTHH:MM:SSZ"` (e.g., `"2024-01-15T10:30:00Z"`)
- Validation happens at string level in domain; chrono parsing in adapter layer

**Link/Embed Target Validation:**
- MUST be non-empty
- MUST be vault-relative path
- Example valid: `"projects/lithos.md"`, `"assets/diagram.png"`
- Example invalid: `""`, `"/absolute/path.md"`

### Subentity Example Instances

**Example: Tag Parsing**
```rust
let tag = Tag::parse("#work/project/urgent")?;
assert_eq!(tag.full_path, "work/project/urgent");
assert_eq!(tag.segments, vec!["work", "project", "urgent"]);

let simple_tag = Tag::parse("#personal")?;
assert_eq!(simple_tag.full_path, "personal");
assert_eq!(simple_tag.segments, vec!["personal"]);

// Invalid examples that should return errors:
assert!(Tag::parse("#invalid segment").is_err());  // Space in segment
assert!(Tag::parse("#project//sub").is_err());     // Empty segment
assert!(Tag::parse("").is_err());                  // Empty string
```

**Example: Frontmatter Construction**
```rust
let mut fields = HashMap::new();
fields.insert("title".to_string(), FrontmatterValue::String("My Note".to_string()));
fields.insert("created".to_string(), FrontmatterValue::Date("2024-01-15".to_string()));
fields.insert("tags".to_string(), FrontmatterValue::Array(vec![
    FrontmatterValue::String("rust".to_string()),
    FrontmatterValue::String("programming".to_string()),
]));

let frontmatter = Frontmatter::new(fields)?;
```

**Example: Link Construction**
```rust
let link = Link {
    source_note_id: Uuid::now_v7(),
    target_path: "projects/lithos.md".to_string(),
    alias: Some("Lithos Project".to_string()),
    link_type: LinkType::WikiLink,
    position: 150,  // Character offset in source document
};

// Corresponds to markdown: [[projects/lithos.md|Lithos Project]]
```

**Example: Embed Construction**
```rust
let embed = Embed {
    source_note_id: Uuid::now_v7(),
    target_path: "assets/architecture-diagram.png".to_string(),
    embed_type: EmbedType::Image,
    position: 320,
};

// Corresponds to markdown: ![[assets/architecture-diagram.png]]
```

**Example: Heading and Section**
```rust
let heading = Heading {
    level: 2,  // ## Second-level heading
    text: "Implementation Details".to_string(),
    position: 0,
};

let section = Section {
    heading: Some(heading),
    content: "This section contains implementation notes...".to_string(),
    range: 0..500,
};
```

### Frontmatter YAML Parsing Strategy

**CRITICAL: Parsing is Adapter Responsibility**
- YAML parsing happens in `adapters/spi/markdown` layer (NOT domain)
- Domain receives pre-parsed `HashMap<String, FrontmatterValue>`
- Use `serde_yaml` crate in adapter for parsing raw YAML text
- YAML syntax errors surfaced as adapter-layer errors, converted to domain errors

**Parsing Flow:**
```
Raw Markdown (adapter)
  → Extract YAML between --- delimiters (adapter)
    → Parse YAML to serde_yaml::Value (adapter)
      → Convert to HashMap<String, FrontmatterValue> (adapter)
        → Validate and construct Frontmatter (domain)
```

### Testing Requirements

**Hexagonal Testing Hierarchy:**

**Domain Tests (Pure Unit Tests):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation_with_valid_data() {
        // Pure logic tests, no dependencies
        // Focus on validation, conversions, business rules
    }

    #[test]
    fn test_frontmatter_value_parsing() {
        // Test type conversions
    }

    #[test]
    fn test_hierarchical_tag_parsing() {
        // Test tag segmentation logic
    }
}
```

**Test Coverage Target:**
- **90%+ coverage** for domain entities and validation logic (per Epic 3 AC)
- Test both success and error cases
- Property-based testing with `proptest` for edge cases
- Deterministic testing with fixed UUIDs and timestamps

**Test Fixtures Strategy:**
```rust
#[cfg(test)]
pub mod fixtures {
    use super::*;
    use uuid::Uuid;

    /// Fixed UUID for deterministic tests (valid UUID v7 format)
    /// Uses timestamp 2024-01-01 00:00:00 UTC for consistency
    pub const TEST_NOTE_ID: Uuid = Uuid::from_u128(0x0184_0000_0000_0000_0000_0000_0000_0001);
    pub const TEST_NOTE_ID_2: Uuid = Uuid::from_u128(0x0184_0000_0000_0000_0000_0000_0000_0002);

    pub fn example_frontmatter() -> Frontmatter {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), FrontmatterValue::String("Test Note".to_string()));
        fields.insert("created".to_string(), FrontmatterValue::Date("2024-01-15".to_string()));
        Frontmatter::new(fields).expect("Valid frontmatter")
    }

    pub fn example_tag() -> Tag {
        Tag::parse("#work/project").expect("Valid tag")
    }

    pub fn example_note() -> Note {
        Note {
            id: TEST_NOTE_ID,
            path: "test/example.md".to_string(),
            frontmatter: Some(example_frontmatter()),
            links: vec![],
            embeds: vec![],
            tags: vec![example_tag()],
            headings: vec![],
            tasks: vec![],
            sections: vec![],
        }
    }
}
```

**Performance Testing:**
```rust
// Add to benches/domain_models.rs when using criterion
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_note_creation(c: &mut Criterion) {
    c.bench_function("note_creation", |b| {
        b.iter(|| {
            // Create note with typical subentity counts
            let note = Note {
                id: Uuid::now_v7(),
                path: "projects/test.md".to_string(),
                frontmatter: Some(example_frontmatter()),
                links: vec![example_link(); 10],
                embeds: vec![],
                tags: vec![example_tag(); 5],
                headings: vec![example_heading(); 3],
                tasks: vec![],
                sections: vec![example_section(); 3],
            };
            black_box(note);
        });
    });
}

// Target: <100μs per Note creation with moderate subentity counts
```

**Performance Considerations:**
- Domain tests should execute in milliseconds
- No heavy I/O or async operations in pure domain tests
- Integration tests in separate `tests/` directory
- Benchmark Note construction with various subentity counts to ensure linear scaling

### File Structure Requirements

**Directory Layout Options:**

**Option 1: Subfolder Organization (Recommended for Large Bounded Contexts):**
```
crates/domain/src/
├── lib.rs                    # Public API surface, re-exports
├── models/
│   ├── mod.rs               # Module declarations
│   └── note/                # Note bounded context subfolder
│       ├── mod.rs           # Re-exports Note aggregate and subentities
│       ├── note.rs          # Note aggregate root
│       ├── frontmatter.rs   # Frontmatter subentity
│       ├── link.rs          # Links subentity
│       ├── tag.rs           # Tags subentity
│       ├── heading.rs       # Headings subentity
│       ├── task.rs          # Tasks subentity
│       └── section.rs       # Sections subentity
├── ports/
│   ├── mod.rs               # Port trait declarations
│   └── repository.rs        # Future NoteRepositoryPort trait (not in this story)
└── errors.rs                # Domain error types
```

**Option 2: Single File (For Simpler Implementations):**
```
crates/domain/src/
├── lib.rs                    # Public API surface, re-exports
├── models/
│   ├── mod.rs               # Module declarations
│   └── note.rs              # Note aggregate + all subentities in one file
├── ports/
│   ├── mod.rs               # Port trait declarations
│   └── repository.rs        # Future NoteRepositoryPort trait (not in this story)
└── errors.rs                # Domain error types
```

**Implementation Decision:**
- Use **Option 1 (subfolder)** if the Note bounded context exceeds ~300 lines or has complex subentities
- Use **Option 2 (single file)** if all entities can fit cleanly in one file with good organization
- Either approach is acceptable; prioritize readability and maintainability
- Future bounded contexts (Schema, Config, Template) will follow the same pattern for consistency

**Naming Conventions - STRICT:**
- Files: `snake_case.rs`
- Modules: `snake_case`
- Structs/Enums: `PascalCase`
- Functions/Variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Traits: `PascalCase` with `Port` suffix for ports

**Import Organization Example:**
```rust
// Standard library imports (grouped together)
use std::collections::HashMap;
use std::ops::Range;

// External crate imports (grouped together)
use uuid::Uuid;

// Internal crate imports (grouped together)
use crate::errors::DomainError;

// Module contents below
```

Per `rustfmt.toml` configuration, imports are automatically grouped as `StdExternalCrate`.

### Code Quality Standards

**Clippy Complexity Limits - ENFORCED:**
- Cognitive complexity: **max 25 (deny)**
- Function length: **max 100 lines (deny)**
- Keep functions focused and composable

**Formatting:**
- Run `mise run verify` before committing (enforces rustfmt + clippy)
- Pre-commit hooks will enforce formatting
- Import grouping: `StdExternalCrate` (per rustfmt.toml)

**Documentation Standards:**
```rust
/// Brief description of what this does.
///
/// # Invariants
/// - List key invariants and constraints
/// - Explain ownership or threading requirements
///
/// # Examples
/// ```
/// // Runnable example
/// let note = Note::new(...);
/// ```
///
/// # Errors
/// Returns `DomainError::ValidationFailed` if input is invalid.
pub fn create_note(...) -> Result<Note, DomainError> {
    // implementation
}
```

**The "Why" Mandate:**
- Doc comments focus on **Invariants** and **Architectural Context**
- Explain non-obvious design decisions
- Example-driven specs for all public traits

## Dev Notes

### Project Context Integration

**Current Codebase State:**
- Workspace structure exists at `crates/domain/`, `crates/app/`, `crates/adapters/`, `crates/cli/`
- Domain crate has placeholder `DomainError` enum
- NO existing domain models yet - this is the FIRST domain implementation
- Epic 1 (tooling) completed, Epic 2 (test patterns) ready, Epic 3 starting now

**Technology Stack (from project-context.md):**
- **Rust 1.92+**: Memory safety, zero-cost abstractions
- **UUID 1.19 (v7)**: Time-ordered identifiers for Note identity
- **rkyv 0.8**: Zero-copy deserialization for storage (derive Archive, Serialize, Deserialize, CheckBytes)
- **thiserror 2.0**: Structured domain error definitions
- **Tokio 1.49**: Async runtime (use `#[async_trait]` for async trait methods)

**Critical Anti-Patterns to AVOID:**
- ❌ Using `String` for file paths (use `PathBuf` or `&Path`)
- ❌ Using `unwrap()`, `expect()`, `todo()`, `panic!()` in production code
- ❌ Using `as` casting (use `.try_into().expect("...")` or `.context("...")`)
- ❌ Holding `std::sync::MutexGuard` across `.await` points
- ❌ Leaking adapter concerns into domain (no Redb, no file I/O)
- ❌ Creating ad-hoc conversion methods instead of From/TryFrom traits
- ❌ Using catch-all `_ => {}` patterns in exhaustive domain logic matches

### Obsidian Research - Note Structure

**From PRD and Architecture Analysis:**

**Obsidian Note Anatomy:**
1. **Frontmatter (YAML)**: Metadata block at file start between `---` delimiters
2. **Wiki-links**: `[[Target]]` or `[[Target|Alias]]` for internal links
3. **Embeds**: `![[Image.png]]` for embedded content
4. **Tags**: `#tag` or `#parent/child/grandchild` for hierarchical tags
5. **Headings**: `#` through `######` for document structure
6. **Tasks**: `- [ ]` unchecked, `- [x]` checked, `- [-]` cancelled
7. **Sections**: Content blocks between headings

**Key Patterns from Architecture:**
- Vault-relative paths for filesystem correspondence
- UUID v7 for stable identity (survives file renames)
- Schema-driven validation for frontmatter fields
- Link resolution across vault for consistency
- Alias resolution for wiki-links

**Obsidian Data Structures (Go Implementation Reference):**
- `TFile`: File metadata and path information
- `CachedMetadata`: Parsed note structure with links, embeds, headings, tags
- These inform our domain model but Rust implementation differs

### Implementation Strategy

**Step-by-Step Approach:**

1. **Start with Error Types** (`errors.rs`):
   - Define `DomainError` enum for Note bounded context
   - Use `thiserror::Error` with descriptive messages
   - Cover validation failures, parsing errors, invariant violations

2. **Create Value Objects** (`value_objects.rs`):
   - Small, immutable types like `NoteId(Uuid)` if needed for type safety
   - Implement `From/Into` traits for conversions
   - Add validation in constructors

3. **Build Subentities** (one file per subentity):
   - Start simple: Frontmatter, Tag, Heading, Task
   - Add Links, Embeds, Sections
   - Each with own validation logic and tests

4. **Construct Note Aggregate** (`note.rs`):
   - Bring subentities together
   - Implement creation methods with validation
   - Ensure immutability and ownership clarity

5. **Write Comprehensive Tests**:
   - Unit tests for each subentity
   - Integration tests for Note aggregate
   - Property-based tests for edge cases
   - Aim for 90%+ coverage

**Validation Pipeline (Three-Phase):**
1. **Syntactic**: Type correctness, basic format validation
2. **Orchestration**: Cross-entity consistency (happens in app layer, not here)
3. **Semantic**: Business rule validation within entity

**Example Note Creation:**
```rust
// Direct struct construction (recommended for domain models)
let note = Note {
    id: Uuid::now_v7(),  // Time-ordered identity
    path: "projects/lithos-development.md".to_string(),
    frontmatter: Some(Frontmatter::new(fields)?),
    links: vec![],
    embeds: vec![],
    tags: vec![Tag::parse("#project/lithos")?],
    headings: vec![],
    tasks: vec![],
    sections: vec![],
};

// Validation happens in constructors of subentities (Tag::parse, Frontmatter::new, etc.)
// Note struct itself can have a validation method if cross-entity validation is needed
```

**Async Trait Pattern (For Future Stories):**
```rust
// This story doesn't define async traits, but future repository ports will use this pattern
use async_trait::async_trait;

#[async_trait]
pub trait NoteRepositoryPort: Send + Sync {
    type NoteId; // Associated type for note identity
    type Error: std::error::Error; // Associated error type

    /// Persist a note to storage.
    ///
    /// # Errors
    /// Returns associated error type if persistence fails.
    async fn save(&self, note: &Note) -> Result<Self::NoteId, Self::Error>;

    /// Retrieve a note by ID.
    async fn find_by_id(&self, id: &Self::NoteId) -> Result<Option<Note>, Self::Error>;
}

// Import pattern:
// use async_trait::async_trait;
```

### Cross-Story Dependencies

**Prerequisites:**
- ✅ Epic 1 completed (workspace, tooling, quality gates)
- ✅ Epic 2 ready (test patterns for domain testing)
- ✅ Architecture established (hexagonal boundaries, ADRs)

**Enables Future Stories:**
- Story 3.2: Schema Bounded Context (will reference Note for validation)
- Story 3.3: Config Bounded Context (configuration for note operations)
- Story 3.4: Template Bounded Context (templates generate notes)
- Epic 4: File Loading (loads notes into domain models)
- Epic 8: Storage Layer (persists Note entities with rkyv)
- Epic 9: Vault Indexing (indexes Note aggregates)

### Epic 2 Test Infrastructure Integration
**Planned Integration with Epic 2 Test Utils:**
This story will leverage the test utilities being developed in Epic 2:
- **Story 2-4**: Centralized test utilities and infrastructure (artifact management, isolation)
- **Story 2-6**: Integration testing patterns and infrastructure (cross-crate testing, external service mocking)
- **Story 2-7**: Benchmarking infrastructure and performance testing patterns (criterion integration, regression detection)
- **Dependency**: Epic 2 completion required before implementing comprehensive testing in this story
- **Integration Points**: Use shared test fixtures for domain entities, mock repositories, and performance benchmarking utilities

### References

**Architecture Documents:**
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
  - UUID v7 identity decision
  - Redb + rkyv storage strategy
  - Hexagonal boundary enforcement

- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns]
  - Naming conventions
  - Structure patterns
  - Error handling standards

- [Source: _bmad-output/project-context.md#Critical Implementation Rules]
  - Architectural integrity requirements
  - Language-specific Rust patterns
  - AI pitfall protections

**Epic Context:**
- [Source: _bmad-output/planning-artifacts/epics/epic-3-core-domain-models-value-objects-phase-15.md#Story 3.1]
  - Complete acceptance criteria
  - Note aggregate specification
  - Subentity requirements

**PRD Requirements:**
- [Source: _bmad-output/planning-artifacts/prd.md#Vault Operations FR20-FR25]
  - Note indexing and search requirements
  - Wiki-link resolution needs
  - Large vault performance targets

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
Expected files to be created (Option 1 - Subfolder):
- crates/domain/src/errors.rs (comprehensive DomainError enum with all variants)
- crates/domain/src/models/mod.rs (updated with note module declaration)
- crates/domain/src/models/note/mod.rs (re-exports all subentities)
- crates/domain/src/models/note/note.rs (Note aggregate root)
- crates/domain/src/models/note/frontmatter.rs (Frontmatter + FrontmatterValue)
- crates/domain/src/models/note/link.rs (Link + LinkType)
- crates/domain/src/models/note/embed.rs (Embed + EmbedType)
- crates/domain/src/models/note/tag.rs (Tag with parsing logic)
- crates/domain/src/models/note/heading.rs (Heading)
- crates/domain/src/models/note/task.rs (Task + TaskStatus)
- crates/domain/src/models/note/section.rs (Section)
- crates/domain/src/lib.rs (updated with public re-exports)
- benches/domain_models.rs (performance benchmarks - optional)

OR (Option 2 - Single File):
- crates/domain/src/errors.rs (comprehensive DomainError enum)
- crates/domain/src/models/mod.rs (updated with note module)
- crates/domain/src/models/note.rs (all entities in one well-organized file)
- crates/domain/src/lib.rs (updated with public re-exports)
- benches/domain_models.rs (performance benchmarks - optional)

Note: Option 1 recommended given the number of subentities (8) and validation logic. TDD approach ensures comprehensive test coverage before implementation.
```
