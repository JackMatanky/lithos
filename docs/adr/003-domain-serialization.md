---
name: domain-serialization-strategy-with-feature-gates
status: accepted
stakeholders: [Development Team, Architects, Jack (Developer)]
date_proposed: 2026-01-14
date_decided: 2026-02-01
date_implemented: pending
date_updated: 2026-03-10
---

# ADR 003: Domain Serialization Strategy with Feature Gates

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

**Domain models have rkyv derives for zero-copy database operations and feature-gated serde for optional JSON serialization. View types (`*View`) are an optional optimization layer, only created when domain shape is inefficient for storage.**

### Three-Shape Serialization Model

1. **Raw\* (serde only, optional per context)**: Unvalidated input from filesystem
   - Purpose: Tolerant parsing from YAML/JSON files with nullable fields for better error messages
   - Location: `<context>/raw.rs`
   - Example: `RawSchema { name: Option<String>, properties: Option<Vec<...>> }`

2. **Domain (rkyv + serde feature-gated)**: Validated entities used throughout application
   - Purpose: Invariant-preserving domain models, zero-copy database operations
   - Location: `<context>/aggregate.rs`
   - Derives: `rkyv::Archive + Serialize + Deserialize`, optionally `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`
   - **Has rkyv derives by default** for zero-copy database reads via storage implementations
   - Example: `Schema { name: SchemaName, properties: Vec<Property> }`

3. **\*View (rkyv only, optional optimization)**: Storage-optimized projection representation
   - Purpose: Represents a read-optimized projection in the expendable database cache. Used only when domain shape causes storage inefficiency (wrapper newtypes complicate indexing, deep nesting, Arc sharing issues)
   - Location: `db/view/<context>.rs`
   - Mechanical conversions at storage boundary
   - Example: `SchemaView { name: String, properties: Vec<PropertyView> }` (flattened newtypes)

### Implementation Guidelines

1. **Feature-Gated Serde**: Domain entities use `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` for CLI/LSP JSON output
2. **rkyv on Domain**: Domain entities derive rkyv traits (`Archive`, `Serialize`, `Deserialize`) for zero-copy database operations
3. **Default Strategy**: Store domain types directly in database (they have rkyv derives); only create `*View` when profiling reveals performance issues
4. **Storage DTOs (Optional)**: Create `*View` types only when:
   - Wrapper newtypes (SchemaName, NotePath) complicate database indexing
   - Deep nesting causes excessive alignment copy overhead
   - Arc<T> sharing doesn't serialize efficiently with rkyv
5. **Compile-Time Enforcement**: Architectural tests verify serde remains feature-gated (optional dependency)

### Feature Flag Pattern

```rust
// In lithos-core/Cargo.toml
[features]
default = []
serde = ["dep:serde"]

[dependencies]
serde = { version = "1.0", features = ["derive"], optional = true }
rkyv = { version = "0.8", features = ["validation"] }  # Required for zero-copy DB

// In domain entities (<context>/aggregate.rs)
use rkyv::{Archive, Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Schema {
    name: SchemaName,  // Private field, validated on construction
    properties: Vec<Property>,
}
```

### Usage by Consumers

**CLI Binary** (needs JSON output):
```toml
[dependencies]
lithos-core = { path = "../lithos-core", features = ["serde"] }
```

**Storage Layer** (uses domain type directly or optional *View if needed):
```rust
// Default: Store domain type directly (in db/storage/schema.rs)
impl schema::Storage for RedbSchemaStorage<'_> {
    type Archived<'a> = &'a ArchivedSchema;  // Domain type's archived form

    fn get_by_name(&self, name: &SchemaName)
        -> Result<Option<Schema>, DbError>
    {
        self.db.get_owned::<Schema>("schemas", name.as_ref())
    }
}

// Optional: *View only when domain shape inefficient (in db/view/schema.rs)
#[derive(Archive, Serialize, Deserialize)]
#[repr(C)]
pub struct SchemaView {
    pub name: String,  // Flattened from SchemaName newtype
    pub properties: Vec<PropertyView>,
}

impl From<&Schema> for SchemaView { /* mechanical conversion */ }
impl From<SchemaView> for Schema { /* mechanical conversion */ }
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

#### Why rkyv Derives on Domain (Revised Decision)

**Zero-Copy Performance**:

- **Storage Abstraction**: Unified `Storage` traits use GATs (Generic Associated Types) to expose `Archived<'a>` views without leaking transaction lifetimes
- **No Ergonomic Cost**: Domain types still use `String`/`Vec` (not `ArchivedString`/`ArchivedVec`); rkyv derives don't change the API surface
- **Closure-Scoped Access**: Archived views accessed via `with_archived_*` methods that take closures, preventing lifetime leaks
- **Type Safety**: Compiler enforces that archived references cannot escape transaction scope via HRTBs (`for<'a> FnOnce`)

**Storage Separation Preserved**:

- **Unified Traits**: Implementations in `db/` implement storage capabilities; domain never directly touches database
- **Optional `*View`**: Only create when domain shape causes storage inefficiency (decision tree in Appendix A)
- **Migration Control**: Treat domain type changes as potential migrations; use `*View` layer when migration risk is high

**Practical Benefits**:

- **90% Case**: Domain types stored directly without extra conversion layer (simpler, faster)
- **10% Case**: `*View` optimization available when profiling reveals need (flattened newtypes, optimized layouts)
- **Zero Unsafe**: GATs + HRTBs provide compile-time safety without `unsafe` blocks

## Alternatives Considered

### Alternative 1: Complete Serialization Ban (Zero Dependencies, DTOs for Everything)

- **Pros**: Maximum architectural purity, zero coupling, fastest compilation
- **Cons**: Significant DTO mapping boilerplate (Note → NoteDTO for JSON), reduced developer experience, potential for stale DTOs
- **Verdict**: Too restrictive - feature-gated approach gives purity by default with opt-in convenience

### Alternative 2: Required Serde Dependency (Always Enabled)

- **Pros**: No feature flag complexity, simpler Cargo.toml, direct serialization always available
- **Cons**: Forces serde on all library consumers (violates zero-dependency goal), slower compilation for library development, bloats binary for non-JSON use cases
- **Verdict**: Rejected - violates library-first architecture principle

### Alternative 3: Allow Both serde and rkyv in Domain (SELECTED)

- **Pros**: Maximum flexibility, zero-copy database operations, no DTO mapping for 90% of cases, type safety via GATs
- **Cons**: rkyv is required dependency (not optional), domain refactors can trigger storage migrations, need to be mindful of format stability
- **Verdict**: **ACCEPTED** - Storage abstraction preserves separation of concerns while enabling zero-copy performance. `*View` layer available as escape hatch when needed.

### Alternative 4: Custom Derives Only

- **Pros**: Domain controls serialization without external dependencies, maximum flexibility
- **Cons**: Reinventing the wheel, ecosystem isolation, maintenance burden, no JSON interop with standard tools
- **Verdict**: Not practical for production systems

## Technical Validation

### Research Findings

- **Feature Flags in Libraries**: Rust best practice for optional functionality (documented in Cargo Book "Optional Dependencies")
- **rkyv Analysis**: Zero-copy deserialization framework suitable for storage, but creates domain coupling with Archive trait bounds
- **Serde Analysis**: De facto standard for Rust serialization, feature-gated pattern is idiomatic (used by serde itself)
- **Use Case Separation**: rkyv excels at storage performance (storage implementation), serde excels at API interoperability (CLI/LSP)

### Compatibility & Performance

- **Module Boundary Alignment**: Feature-gated serde maintains domain purity (zero deps by default), rkyv strictly in storage implementations
- **Performance Impact**:
  - Library compilation: 30-40% faster without serde derives (measured in similar projects)
  - Runtime: Zero overhead when feature disabled (serde code not compiled)
  - Storage: Full zero-copy performance (rkyv in implementation, not domain)
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

## Appendix A: Minimizing "derive-everything" Blast Radius (Historic Context)

*Note: This appendix was written when rkyv was prohibited in domain. With the acceptance of Alternative 3 (rkyv allowed), this section serves as historical context for why the three-shape model was adopted.*

### A.1 The Coupling Risk

The core risk is *coupling*: if rkyv derives are applied broadly on "domain-shaped" types, then routine refactors become **persisted-format changes**.

In concrete terms, if `Schema` / `Note` are both "domain + stored" at once, we should expect more on-disk migrations when we refactor domain fields.

### A.2 Mitigation Strategy: Three-Shape Model

To mitigate this while allowing rkyv in domain, we adopt the three-shape model:

1. **`Raw*` (serde-only)**: input/wire shapes.
   - Purpose: parse user data and hold "maybe invalid" values.
   - Traits: serde derives only (feature-gated if needed).

2. **`Domain*` (rkyv + behavior)**: runtime shapes.
   - Purpose: invariants, ergonomics, and behavior.
   - Traits: rkyv derives for zero-copy access.
   - **Rule**: Changes here ARE potential migration events. Treat them with care.

3. **`*View` (rkyv + validation)**: persisted/on-disk projection shapes (Optional).
   - Purpose: define the stable archived layout for read models when domain shape evolves incompatibly.
   - Traits: rkyv derives, bytecheck/validation bounds.
   - **Rule**: If domain changes become too frequent/breaking, introduce this layer to decouple.

### A.3 Decision Tree for View Types

We do not introduce `*View` types speculatively. We only introduce them when:

1. **Wrapper Types Complicate Indexing**: `SchemaName` (newtype) vs `String` keys.
2. **Deep Nesting**: Domain hierarchy causes excessive alignment copy overhead.
3. **Shared State**: `Arc<T>` or cyclic references don't serialize efficiently.
4. **Migration Pressure**: Domain refactors are causing too many storage migrations.

Until then, we stick to **Path 3**: Domain types have rkyv derives and are stored directly.
