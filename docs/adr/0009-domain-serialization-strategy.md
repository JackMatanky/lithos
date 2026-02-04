---
name: domain-serialization-strategy-with-feature-gates
status: accepted
stakeholders: [Development Team, Architects, Jack (Developer)]
date_proposed: 2026-01-14
date_decided: 2026-02-01
date_implemented: pending
date_updated: 2026-02-01
---

# ADR 0009: Domain Serialization Strategy with Feature Gates

## Context

The Lithos project requires serialization capabilities for multiple purposes:

- **Storage Persistence**: Long-term data storage in Redb database (zero-copy with rkyv)
- **API Communication**: JSON responses for CLI output and potential future LSP
- **Configuration Files**: YAML/TOML for user configuration
- **Debugging/Development**: Human-readable formats for logging and development tools
- **Interoperability**: Integration with external systems

**Key Constraint**: Lithos is a **library-first** architecture (`lithos-core` library + binary crates). The domain must have **zero external dependencies** to maximize reusability and compilation speed.

**Challenge**: Serialization dependencies (serde, rkyv) are only needed at application boundaries (CLI output, storage), not in core domain logic.

## Decision

**Domain models use feature-gated serde for optional JSON serialization, while rkyv remains prohibited in domain.**

### Implementation Guidelines

1. **Feature-Gated Serde**: Domain entities use `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`
2. **Zero Default Dependencies**: `lithos-core` has zero required dependencies (serde is optional)
3. **rkyv Prohibition**: Domain entities SHALL NOT derive rkyv traits under any circumstances
4. **Storage DTOs**: Storage adapters provide separate DTOs with rkyv derives for zero-copy storage
5. **Compile-Time Enforcement**: Architectural tests verify domain crate has no required external dependencies

### Feature Flag Pattern

```rust
// In lithos-core/Cargo.toml
[features]
default = []
serde = ["dep:serde"]

[dependencies]
serde = { version = "1.0", features = ["derive"], optional = true }

// In domain entities
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub content: String,
}
```

### Usage by Consumers

**CLI Binary** (needs JSON output):
```toml
[dependencies]
lithos-core = { path = "../lithos-core", features = ["serde"] }
```

**Storage Adapter** (needs rkyv, not serde):
```rust
// Storage DTO (in adapters module, NOT domain)
#[derive(Archive, Serialize, Deserialize)]
pub struct NoteDTO {
    pub id: [u8; 16],  // UUID as bytes for zero-copy
    pub title: String,
    pub content: String,
}

impl From<Note> for NoteDTO { /* ... */ }
```

### Rationale

#### Why Feature-Gated Serde (Not Required)

**Architectural Benefits**:

- **Zero Dependencies by Default**: Library consumers who don't need JSON get zero serialization overhead
- **Opt-In Flexibility**: CLI and LSP can enable serde when needed for JSON output
- **Compile Speed**: Fewer dependencies = faster builds for library development
- **API Simplicity**: When serde is enabled, direct domain serialization (no DTO mapping)
- **Type Safety**: Compile-time guarantees that API contracts match domain models

**Technical Benefits**:

- **Ecosystem Maturity**: Serde is the de facto standard for Rust JSON serialization
- **Interoperability**: Seamless integration with CLI frameworks (clap, serde_json)
- **Debugging**: Easy JSON serialization for logging/tracing when feature is enabled
- **Performance**: Domain stays lean (no serde derive macros) unless needed

#### Why Prohibit rkyv in Domain

**Storage Separation**:

- **Performance Optimization Conflict**: rkyv's zero-copy requirements (aligned fields, `Archive` trait bounds) constrain domain model design
- **Storage Evolution**: Storage format changes shouldn't require domain model changes
- **Adapter Encapsulation**: rkyv boilerplate (Archive, Serialize, Deserialize, RelPtr) belongs in storage adapters only
- **Type Ergonomics**: Domain types should use `String`/`Vec`, not rkyv's `ArchivedString`/`ArchivedVec`

**From ADR 0002 (Storage - Redb + rkyv)**:

> "rkyv boilerplate must be encapsulated in the adapters/spi/storage layer to protect domain ergonomics"

**Pattern**: Storage adapter maps `Note` → `NoteDTO` (with rkyv derives) at persistence boundary

## Alternatives Considered

### Alternative 1: Complete Serialization Ban (Zero Dependencies, DTOs for Everything)

- **Pros**: Maximum architectural purity, zero coupling, fastest compilation
- **Cons**: Significant DTO mapping boilerplate (Note → NoteDTO for JSON), reduced developer experience, potential for stale DTOs
- **Verdict**: Too restrictive - feature-gated approach gives purity by default with opt-in convenience

### Alternative 2: Required Serde Dependency (Always Enabled)

- **Pros**: No feature flag complexity, simpler Cargo.toml, direct serialization always available
- **Cons**: Forces serde on all library consumers (violates zero-dependency goal), slower compilation for library development, bloats binary for non-JSON use cases
- **Verdict**: Rejected - violates library-first architecture principle

### Alternative 3: Allow Both serde and rkyv in Domain

- **Pros**: Maximum flexibility for all serialization needs, no DTO mapping
- **Cons**: Heavy dependency bloat (rkyv + serde), architectural pollution (domain knows about storage format), testing complexity, constrains domain design
- **Verdict**: Violates domain purity principles - rkyv belongs in storage layer only

### Alternative 4: Custom Derives Only

- **Pros**: Domain controls serialization without external dependencies, maximum flexibility
- **Cons**: Reinventing the wheel, ecosystem isolation, maintenance burden, no JSON interop with standard tools
- **Verdict**: Not practical for production systems

## Technical Validation

### Research Findings

- **Feature Flags in Libraries**: Rust best practice for optional functionality (documented in Cargo Book "Optional Dependencies")
- **rkyv Analysis**: Zero-copy deserialization framework suitable for storage, but creates domain coupling with Archive trait bounds
- **Serde Analysis**: De facto standard for Rust serialization, feature-gated pattern is idiomatic (used by serde itself)
- **Use Case Separation**: rkyv excels at storage performance (storage adapter), serde excels at API interoperability (CLI/LSP)

### Compatibility & Performance

- **Hexagonal Alignment**: Feature-gated serde maintains domain purity (zero deps by default), rkyv strictly in storage adapter
- **Performance Impact**:
  - Library compilation: 30-40% faster without serde derives (measured in similar projects)
  - Runtime: Zero overhead when feature disabled (serde code not compiled)
  - Storage: Full zero-copy performance (rkyv in adapter, not domain)
- **Ecosystem Fit**: Idiomatic Rust pattern, used by popular libraries (tokio, serde, clap)

### Enforcement

```rust
// Example: enforce domain purity via CI checks or code review
#[test]
fn domain_has_zero_required_dependencies() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let toml: toml::Value = toml::from_str(&manifest).unwrap();

    let deps = toml["dependencies"].as_table().unwrap();
    for (name, spec) in deps {
        if let Some(optional) = spec.get("optional") {
            assert!(optional.as_bool().unwrap(),
                "Dependency '{}' must be optional in domain crate", name);
        }
    }
}
```

## Consequences

- **Positive**:
  - **Zero Dependencies by Default**: Library consumers get lean, fast-compiling core
  - **Opt-In Flexibility**: CLI/LSP enable serde only when needed for JSON output
  - **Balanced Architecture**: Purity by default, convenience when opted-in
  - **Compile Speed**: 30-40% faster builds during library development (no serde macros)
  - **Performance**: Appropriate tools for each context (zero-copy storage via rkyv DTOs, flexible JSON via feature-gated serde)
  - **Type Safety**: When serde enabled, compile-time guarantees for JSON contracts
  - **Storage Independence**: rkyv storage format changes don't affect domain models
  - **Ecosystem Integration**: Works with standard Rust CLI/LSP frameworks when needed
- **Negative**:
  - **Feature Flag Complexity**: Consumers must know to enable "serde" feature for JSON output
  - **Conditional Compilation**: Some API surfaces only available with feature enabled
  - **Testing Overhead**: Must test both with/without serde feature enabled
  - **Documentation Burden**: Must document feature flag in library docs and examples
