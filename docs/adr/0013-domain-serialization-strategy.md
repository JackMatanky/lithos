# ADR 0013: Domain Serialization Strategy

*   **Status**: Proposed
*   **Date**: 2026-01-14
*   **Stakeholders**: Development Team, Architects

## Context

The Lithos project requires serialization capabilities for multiple purposes:
- **Storage Persistence**: Long-term data storage in Redb database
- **API Communication**: JSON responses for REST APIs and GraphQL
- **Configuration Files**: YAML/TOML for user configuration
- **Network Protocols**: Potential future binary protocols
- **Debugging/Development**: Human-readable formats for logging and development tools
- **Interoperability**: Integration with external systems

Domain models must remain pure and focused on business logic. However, serialization needs span multiple architectural layers, creating tension between convenience and architectural purity.

## Decision

**Domain models MUST derive serde traits for JSON/YAML serialization when they represent API resources, while rkyv remains prohibited in domain.**

### Implementation Guidelines

1. **Serde Requirement**: Domain entities that represent API resources (Note, Schema) MUST derive `serde::{Serialize, Deserialize}`
2. **rkyv Prohibition**: Domain entities SHALL NOT derive rkyv traits under any circumstances
3. **Storage DTOs**: Adapters SHALL provide separate DTOs with rkyv derives for storage
4. **Required Dependencies**: Serde is required for API-facing domain models
5. **Domain Purity Guardian**: Automated tests enforce that only API models have serde derives

### Rationale

#### Why Allow Serde in Domain (Controlled)

**Architectural Benefits:**
- **API Simplicity**: Direct domain model serialization reduces DTO mapping complexity
- **Type Safety**: Compile-time guarantees that API contracts match domain models
- **Developer Experience**: Less boilerplate for simple CRUD APIs
- **Evolutionary Safety**: Domain changes automatically reflected in APIs (with proper versioning)

**Technical Benefits:**
- **Performance**: Zero-copy for JSON serialization in many cases
- **Ecosystem Maturity**: Serde is the de facto standard for Rust serialization
- **Interoperability**: Seamless integration with web frameworks, OpenAPI generators
- **Debugging**: Easy JSON serialization for logging/tracing

#### Why Prohibit rkyv in Domain

**Storage Separation:**
- **Performance Optimization Conflict**: rkyv's zero-copy requirements may constrain domain model design
- **Storage Evolution**: Storage format changes shouldn't require domain model changes
- **Adapter Encapsulation**: rkyv boilerplate belongs in SPI storage adapters only

**From ADR 0002 (Storage - Redb + rkyv):**
> "rkyv boilerplate must be encapsulated in the adapters/spi/storage layer to protect domain ergonomics"

## Alternatives Considered

### Alternative 1: Complete Serialization Ban (Zero Dependencies)
- **Pros**: Maximum architectural purity, zero coupling
- **Cons**: Significant DTO mapping overhead, reduced developer experience
- **Verdict**: Too restrictive for practical API development

### Alternative 2: Allow Both serde and rkyv in Domain
- **Pros**: Maximum flexibility for all serialization needs
- **Cons**: Dependency bloat, architectural pollution, testing complexity
- **Verdict**: Violates domain purity principles

### Alternative 3: Custom Derives Only
- **Pros**: Domain controls serialization without external dependencies
- **Cons**: Reinventing the wheel, ecosystem isolation
- **Verdict**: Not practical for production systems

### Alternative 4: Application-Layer DTOs Only
- **Pros**: Clean separation, maximum flexibility
- **Cons**: Mapping boilerplate, maintenance overhead, potential for stale DTOs
- **Verdict**: Acceptable but verbose for simple cases

## Technical Validation

### Research Findings
- **rkyv Analysis**: Zero-copy deserialization framework suitable for storage, but creates domain coupling
- **Serde Analysis**: De facto standard for Rust serialization, excellent for APIs, minimal domain impact
- **Use Case Separation**: rkyv excels at storage performance, serde excels at API interoperability

### Compatibility & Performance
- **Hexagonal Alignment**: Serde derives maintain separation, rkyv in domain violates it
- **Performance Impact**: Appropriate tools for each context (zero-copy storage, flexible APIs)
- **Ecosystem Fit**: Serde integrates with web frameworks, rkyv optimizes storage operations

## Consequences

### rkyv Capabilities (Storage-Focused)
- **Zero-copy deserialization** from any byte source
- **Archive trait** for in-memory representations
- **Validation** during deserialization
- **Versioning support** for schema evolution
- **Streaming** for large datasets
- **Custom serializers** for complex types

### Serde Capabilities (API-Focused)
- **Human-readable formats**: JSON, YAML, TOML, XML
- **Binary formats**: Bincode, MessagePack, CBOR
- **Streaming** for large datasets
- **Custom serializers** via traits
- **Schema validation** via external crates
- **Interoperability** with web standards

### Comparative Analysis

| Aspect | rkyv | serde |
|--------|------|-------|
| **Primary Use Case** | Storage persistence | API communication |
| **Performance** | Zero-copy optimal | Format-dependent |
| **Ecosystem** | Storage-focused | Universal |
| **Domain Coupling** | High (affects model design) | Low (just derives) |
| **Human Readability** | No (binary) | Yes (JSON/YAML) |
| **Versioning** | Built-in | External crates |
| **Validation** | Built-in | External crates |

### Decision Factors

1. **Architectural Purity**: rkyv creates stronger coupling than serde derives
2. **Use Case Separation**: rkyv = storage, serde = APIs (different concerns)
3. **Practicality**: JSON APIs are common, DTO mapping creates maintenance burden
4. **Evolution**: Storage changes more frequent than API changes
5. **Ecosystem**: Serde is ubiquitous, rkyv is specialized

## Implementation Requirements

### Domain Layer
- **Optional Serde**: `serde = { version = "1.0", features = ["derive"], optional = true }`
- **No rkyv**: Explicitly prohibited in domain Cargo.toml
- **Conditional Compilation**: `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`

### Application Layer
- **API DTOs**: Use domain models directly where possible, DTOs where transformation needed
- **Versioning**: Semantic versioning for API changes
- **Documentation**: OpenAPI/Swagger generation from serde schemas

### Adapter Layer
- **Storage DTOs**: Separate structs with rkyv derives
- **Conversion**: `From/Into` traits between domain models and storage DTOs
- **Zero-Copy**: Leverage rkyv's performance advantages

### Testing
- **Domain Purity**: Tests verify zero required dependencies
- **Serialization Tests**: Optional serde features tested separately
- **Integration Tests**: Full serialization round-trips validated

## Consequences

### Positive
- **Balanced Architecture**: Practical API development without sacrificing purity
- **Performance**: Appropriate tools for each context (zero-copy storage, flexible APIs)
- **Date Format Flexibility**: Supports any date format (ISO 8601, Moment.js, custom) without domain coupling
- **Schema-Driven Typing**: Type classification happens at application layer with schema context
- **Developer Experience**: Reduced DTO mapping for simple CRUD APIs
- **Ecosystem Integration**: Works with standard Rust web frameworks

### Negative
- **Dependency Management**: Required serde dependency increases domain crate size
- **Type Safety Trade-off**: Frontmatter values are strings until schema validation
- **Application Complexity**: Type validation logic moves to application layer

### Risks
- **Scope Creep**: "Optional" serde could become required over time
- **Misuse**: Developers might use serde for storage concerns
- **Dependency Updates**: Serde ecosystem changes could affect domain

## Mitigation Strategies

1. **Strict Code Reviews**: Ensure serde use is justified and API-focused
2. **Feature Flags**: Make serde truly optional with clear feature documentation
3. **Domain Purity Guardian**: Automated enforcement of architectural rules
4. **Documentation**: Clear guidelines on when/where to use each serialization approach

## Status Tracking

*   **Proposed**: 2026-01-14

## References

- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [Domain-Driven Design](https://domainlanguage.com/ddd/)
- [Serde Documentation](https://serde.rs/)
- [rkyv Documentation](https://docs.rs/rkyv/)
- ADR 0002: Storage - Redb + rkyv
- ADR 0005: Configuration Management (Figment)
