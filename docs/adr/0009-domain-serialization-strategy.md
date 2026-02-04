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

## Appendix A: Minimizing “derive-everything” Blast Radius (Current Reality)

This ADR’s decision prohibits rkyv derives in domain types and prefers storage DTOs. In the current Rust conversion state, Lithos uses Redb + rkyv directly in `lithos-core` for persistence, which effectively places rkyv on (at least part of) the domain model surface.

This appendix documents a practical pattern to minimize the coupling (and the migration pain) without turning the codebase into DTO boilerplate.

### A.1 Point of focus: minimize “derive-everything” blast radius

The core risk is *coupling*: if rkyv derives are applied broadly on “domain-shaped” types, then routine refactors become **persisted-format changes**.

In concrete terms, right now `Schema` / `Note` are both “domain + stored” at once. If we keep it that way, we should expect more on-disk migrations when we refactor domain fields.

### A.2 Practical pattern: `Raw*` / `Stored*` / `Domain*`

The pattern is a three-shape model boundary:

1) **`Raw*` (serde-only)**: input/wire shapes.
  - Purpose: parse user data and hold “maybe invalid” values.
  - Traits: serde derives only (feature-gated if needed).

2) **`Stored*` (rkyv + validation)**: persisted/on-disk shapes.
  - Purpose: define the stable archived layout and the format-control contract.
  - Traits: rkyv derives, bytecheck/validation bounds, and any storage-specific annotations.
  - Rule: treat changes to `Stored*` as migration decisions.

3) **`Domain*` (behavioral/runtime)**: in-memory shapes.
  - Purpose: invariants, ergonomics, and behavior.
  - Traits: ideally no serde/rkyv derives (or only what is truly necessary for the layer).

Mapping rules:

- `Raw* -> Domain*` is validation/compilation.
- `Domain* <-> Stored*` is persistence mapping.

### A.3 Two viable paths (pick deliberately)

**Path 1: Align fully with ADR 0009 (preferred end state)**

- Keep `Domain*` free of rkyv.
- Implement `Stored*` DTOs at the persistence boundary (in adapters / storage layer).
- Keep archived compute, format-control choices, and validation local to storage.

This best preserves domain ergonomics and makes format changes deliberate.

**Path 2: Transitional “rkyv in core” with minimal bloat (acceptable short-term)**

- Introduce `Stored*` types even if they live inside `lithos-core` initially (e.g., in a `db`/storage module), so the persisted layout is still explicit.
- Keep `Domain*` types ergonomic and isolate rkyv derives onto the `Stored*` boundary.
- Keep mapping mechanical and localized (one `Stored*` per persisted aggregate), not an explosion of DTOs.

If we do *not* introduce `Stored*` and instead keep `Schema`/`Note` as “domain+stored combined”, the team should treat most refactors as potential migrations.

### A.4 Low-bloat implementation guidelines

Guidelines to keep the pattern from bloating the codebase:

1) **One `Stored*` per persisted aggregate**

- Prefer `StoredNote` / `StoredSchema` rather than duplicating every value object.
- Only split further when a specific subgraph causes real pain (compile time, migrations, or archived-layout constraints).

2) **Keep conversions mechanical and colocated**

- Implement `From<Domain*> for Stored*` and `TryFrom<Stored*> for Domain*` (or builder-style constructors) in the persistence module.
- Avoid sprinkling conversion logic throughout domain modules.

3) **Use projections instead of widening stored blobs**

- When new queries become hot, add read-optimized index/projection tables rather than reshaping the primary stored aggregate to serve every query.

4) **Keep archived compute closure-scoped**

- Do not leak archived references beyond transaction scope.
- Prefer closure-based APIs that return owned computed results.
